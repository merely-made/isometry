// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Addressed voxel bodies in the same camera and depth target as the terrain.

use mesocosm_core::{Organism, OrganismId, World};
use mesocosm_lens::{BodyLensProjection, BodyPlacement, CritterPose, MAX_ROSTER};
use mesocosm_mesh::{LiveBodyProjection, LiveBodyProjector, VolumeMap};
use mesocosm_render::live_body::{LiveBody, LiveBodyRenderer};
use serde::Serialize;

use super::{BodySelection, SlabWindow};
#[cfg(test)]
use super::{CameraMode, SLAB_DEPTH};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BodyMode {
    Capsules,
    #[default]
    Voxels,
}

impl BodyMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "capsules" => Some(Self::Capsules),
            "voxels" => Some(Self::Voxels),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Capsules => "capsules",
            Self::Voxels => "voxels",
        }
    }
}

pub const DEFAULT_BODY_BUDGET: usize = MAX_ROSTER + 1;

#[derive(Clone, Debug, Default, Serialize)]
pub struct BodyFrameStats {
    pub material_parts: usize,
    pub secretory_parts: usize,
    pub candidates: usize,
    pub body_scale: f32,
    pub voxel_bodies: usize,
    pub voxel_parts: usize,
    pub carcasses: usize,
    pub fallback_bodies: usize,
    pub fallback_parts_dropped: usize,
    pub omitted_bodies: usize,
    pub missing_volumes: usize,
    pub projection_failures: usize,
    pub last_error: Option<String>,
    pub controlled_drawn: bool,
    pub mesh_builds: usize,
    pub mesh_upload_bytes: u64,
    pub instance_upload_bytes: u64,
    pub frame_upload_bytes: u64,
    pub draw_parts: usize,
}

struct PlacedBody {
    projection: LiveBodyProjection,
    materials: Vec<mesocosm_render::PartMaterial>,
    origin: [f32; 3],
    tint: [f32; 3],
}

pub(super) struct BodyLayer {
    projector: LiveBodyProjector,
    renderer: LiveBodyRenderer,
    placed: Vec<PlacedBody>,
    pub fallback: Vec<CritterPose>,
    pub played_fallback: Option<CritterPose>,
    pub stats: BodyFrameStats,
    pub budget: usize,
    pub scale: f32,
    pub ground_anatomy: bool,
    pub isolated: bool,
    focus_subject: Option<OrganismId>,
    selected: Option<BodySelection>,
    depth: wgpu::Texture,
    pub depth_view: wgpu::TextureView,
}

impl BodyLayer {
    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let (depth, depth_view) = depth_target(device, width, height);
        Self {
            projector: LiveBodyProjector::default(),
            renderer: LiveBodyRenderer::new(device, mesocosm_lens::FRAME_FORMAT, 256),
            placed: Vec::new(),
            fallback: Vec::new(),
            played_fallback: None,
            stats: BodyFrameStats::default(),
            budget: DEFAULT_BODY_BUDGET,
            scale: 1.0,
            ground_anatomy: false,
            isolated: false,
            focus_subject: None,
            selected: None,
            depth,
            depth_view,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        (self.depth, self.depth_view) = depth_target(device, width, height);
    }

    pub fn clear_inspection(&mut self) {
        self.placed.clear();
        self.selected = None;
    }

    pub fn select_part(
        &self,
        subject: OrganismId,
        current: Option<BodySelection>,
        backwards: bool,
    ) -> Option<BodySelection> {
        let body = self
            .placed
            .iter()
            .find(|body| body.projection.organism == subject)?;
        let parts: Vec<_> = body
            .projection
            .mesh
            .placements
            .iter()
            .filter(|part| drawable(&body.projection, part.part))
            .collect();
        if parts.is_empty() {
            return None;
        }
        let current = current.filter(|selection| selection.organism == subject);
        let index = current
            .and_then(|selection| parts.iter().position(|part| part.part == selection.part))
            .map(|index| {
                if backwards {
                    index.checked_sub(1).unwrap_or(parts.len() - 1)
                } else {
                    (index + 1) % parts.len()
                }
            })
            .unwrap_or_else(|| if backwards { parts.len() - 1 } else { 0 });
        Some(BodySelection {
            organism: subject,
            part: parts[index].part,
            revision: body.projection.revision,
        })
    }

    pub fn validate_selection(
        &mut self,
        selection: BodySelection,
        world: &World,
        volumes: &VolumeMap,
    ) -> bool {
        let Some(revision) = self
            .placed
            .iter()
            .find(|body| {
                body.projection.organism == selection.organism
                    && body.projection.revision == selection.revision
                    && body.projection.mesh.placements.iter().any(|part| {
                        part.part == selection.part && drawable(&body.projection, part.part)
                    })
            })
            .map(|body| body.projection.revision)
        else {
            return false;
        };
        let Some(organism) = world
            .organisms
            .iter()
            .find(|organism| organism.id == selection.organism)
        else {
            return false;
        };
        self.projector
            .project(organism.id, organism.body(), volumes)
            .is_ok_and(|current| {
                current.revision == revision
                    && current
                        .mesh
                        .placements
                        .iter()
                        .any(|part| part.part == selection.part && drawable(&current, part.part))
            })
    }

    pub fn set_focus(&mut self, subject: Option<OrganismId>, selected: Option<BodySelection>) {
        self.focus_subject = subject;
        self.selected = selected;
    }

    pub fn prepare(&mut self, world: &World, volumes: &VolumeMap, window: SlabWindow) {
        self.placed.clear();
        self.fallback.clear();
        self.played_fallback = None;
        self.stats = BodyFrameStats {
            body_scale: self.scale,
            ..Default::default()
        };
        let controlled = world.controlled_id();
        let mut candidates: Vec<_> = world
            .organisms
            .iter()
            .filter(|o| !self.isolated || Some(o.id) == controlled)
            .filter(|o| o.body().living().next().is_some())
            .filter(|o| {
                Some(o.id) == controlled || intersects(o, window, self.scale, self.ground_anatomy)
            })
            .collect();
        candidates.sort_by(|a, b| {
            let priority = |o: &Organism| Some(o.id) != controlled;
            priority(a)
                .cmp(&priority(b))
                .then_with(|| {
                    distance(a.position, window.centre)
                        .total_cmp(&distance(b.position, window.centre))
                })
                .then(a.id.cmp(&b.id))
        });
        self.stats.candidates = candidates.len();
        self.stats.omitted_bodies = candidates.len().saturating_sub(self.budget);
        for organism in candidates.into_iter().take(self.budget) {
            let tint = crate::app::look_of(organism).0;
            match self
                .projector
                .project(organism.id, organism.body(), volumes)
            {
                Ok(projection) => {
                    let materials = super::materials::project(&organism.phenotype, world.ruleset());
                    self.stats.material_parts += materials
                        .iter()
                        .map(|m| m.part)
                        .collect::<std::collections::BTreeSet<_>>()
                        .len();
                    self.stats.secretory_parts += materials
                        .iter()
                        .filter(|m| m.process == mesocosm_core::process::Process::Secrete)
                        .count();
                    self.stats.controlled_drawn |= Some(organism.id) == controlled;
                    self.stats.voxel_parts += projection.mesh.placement_count();
                    self.stats.voxel_bodies += 1;
                    self.stats.carcasses += usize::from(!organism.is_alive());
                    self.placed.push(PlacedBody {
                        projection,
                        materials,
                        origin: body_origin(organism, self.scale, self.ground_anatomy),
                        tint,
                    });
                },
                Err(error) => {
                    self.stats.last_error = Some(format!("critter {}: {error:?}", organism.id.0));
                    self.stats.projection_failures += 1;
                    self.stats.missing_volumes += usize::from(matches!(
                        error,
                        mesocosm_mesh::MeshError::MissingVolume { .. }
                    ));
                    self.add_fallback(organism, controlled == Some(organism.id), tint);
                },
            }
        }
    }

    fn add_fallback(&mut self, organism: &Organism, controlled: bool, tint: [f32; 3]) {
        let at = body_origin(organism, self.scale, self.ground_anatomy);
        let placement = BodyPlacement {
            ground: [
                at[0],
                at[1] + organism.body().aabb().min[1] as f32 * self.scale,
                at[2],
            ],
            scale: self.scale,
            tint,
        };
        match BodyLensProjection::project_truncated(organism.body(), placement) {
            Ok((body, dropped)) if controlled || self.fallback.len() < MAX_ROSTER => {
                self.stats.fallback_bodies += 1;
                self.stats.controlled_drawn |= controlled;
                self.stats.fallback_parts_dropped += dropped;
                if !controlled {
                    self.stats.fallback_parts_dropped += body
                        .pose
                        .capsules
                        .len()
                        .saturating_sub(mesocosm_lens::MAX_ROSTER_CAPSULES);
                }
                if controlled {
                    self.played_fallback = Some(body.pose);
                } else {
                    self.fallback.push(body.pose);
                }
            },
            _ => self.stats.omitted_bodies += 1,
        }
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        camera: super::view::View,
    ) -> Result<[[f32; 4]; 4], String> {
        let matrix = camera.matrix();
        let bodies: Vec<_> = self
            .placed
            .iter()
            .map(|body| LiveBody {
                mesh: &body.projection.mesh,
                materials: &body.materials,
                origin: body.origin,
                scale: self.scale,
                tint: body.tint,
                focused: self.focus_subject == Some(body.projection.organism),
                selected_part: self.selected.and_then(|selection| {
                    (selection.organism == body.projection.organism
                        && selection.revision == body.projection.revision)
                        .then_some(selection.part)
                }),
            })
            .collect();
        let stats = self
            .renderer
            .draw(
                device,
                queue,
                encoder,
                colour,
                &self.depth_view,
                matrix,
                Some(camera.clip()),
                &bodies,
            )
            .map_err(|error| format!("voxel bodies: {error:?}"))?;
        self.stats.mesh_builds = stats.mesh_builds;
        self.stats.mesh_upload_bytes = stats.mesh_upload_bytes as u64;
        self.stats.instance_upload_bytes = stats.instance_upload_bytes as u64;
        self.stats.frame_upload_bytes = stats.frame_upload_bytes as u64;
        self.stats.draw_parts = stats.draw_parts;
        Ok(matrix)
    }

    pub fn fallback_all(&mut self, world: &World) {
        let subjects: Vec<_> = self
            .placed
            .iter()
            .map(|body| body.projection.organism)
            .collect();
        for subject in subjects {
            if let Some(organism) = world.organisms.iter().find(|o| o.id == subject) {
                self.add_fallback(
                    organism,
                    Some(subject) == world.controlled_id(),
                    crate::app::look_of(organism).0,
                );
            }
        }
        self.stats.voxel_bodies = 0;
        self.stats.voxel_parts = 0;
        self.stats.material_parts = 0;
        self.stats.secretory_parts = 0;
        self.stats.projection_failures += 1;
        self.placed.clear();
    }
}

fn drawable(projection: &LiveBodyProjection, part: mesocosm_core::PartId) -> bool {
    projection
        .mesh
        .placements
        .iter()
        .find(|placement| placement.part == part)
        .and_then(|placement| projection.mesh.mesh_for(placement.volume))
        .is_some_and(|mesh| !mesh.quads.is_empty())
}

fn distance(at: [i32; 3], centre: [f32; 3]) -> f32 {
    (0..3).map(|i| (at[i] as f32 - centre[i]).powi(2)).sum()
}

fn body_origin(organism: &Organism, scale: f32, grounded: bool) -> [f32; 3] {
    let mut origin = organism.position.map(|v| v as f32);
    if grounded {
        origin[1] -= organism.body().aabb().min[1] as f32 * scale;
    }
    origin
}

fn intersects(organism: &Organism, window: SlabWindow, scale: f32, grounded: bool) -> bool {
    let bounds = organism.body().aabb();
    let origin = body_origin(organism, scale, grounded);
    let middle = [0, 1, 2].map(|i| {
        origin[i] + (bounds.min[i] as f32 + bounds.max[i] as f32) * 0.5 * scale - window.centre[i]
    });
    let half = [0, 1, 2].map(|i| (bounds.max[i] as f32 - bounds.min[i] as f32) * 0.5 * scale);
    (0..3).all(|axis| {
        let extent: f32 = (0..3).map(|i| half[i] * window.axes[axis][i].abs()).sum();
        dot(middle, window.axes[axis]).abs() <= window.half[axis] + extent
    })
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

#[cfg(test)]
pub(super) fn clip_from_world(
    mode: CameraMode,
    centre: [f32; 3],
    half: f32,
    aspect: f32,
) -> [[f32; 4]; 4] {
    super::view::View {
        mode,
        centre,
        half,
        aspect,
        depth: SLAB_DEPTH,
        pitch: None,
        bounds: None,
    }
    .matrix()
}

fn depth_target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("section body and terrain depth"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_lens::TraceCamera;

    #[test]
    fn grounded_terrarium_anatomy_fits_the_fixed_volume() {
        let founding = mesocosm_core::Founding::SpacedRoster;
        let world = World::terrarium(7, founding, founding.palette()).unwrap();
        let bounds = super::super::framed_habitat(&world).bounds;
        let scale = super::super::TERRARIUM_BODY_SCALE;
        for organism in &world.organisms {
            let origin = body_origin(organism, scale, true);
            let body = organism.body().aabb();
            assert_eq!(
                origin[1] + body.min[1] as f32 * scale,
                organism.position[1] as f32
            );
            for axis in 0..3 {
                assert!(origin[axis] + body.min[axis] as f32 * scale >= bounds.min[axis] as f32);
                assert!(
                    origin[axis] + body.max[axis] as f32 * scale <= bounds.max[axis] as f32 + 1.0
                );
            }
        }
    }

    fn transform(matrix: [[f32; 4]; 4], point: [f32; 3]) -> [f32; 3] {
        [0, 1, 2].map(|row| {
            matrix[3][row] + (0..3).map(|col| matrix[col][row] * point[col]).sum::<f32>()
        })
    }

    #[test]
    fn mesh_projection_matches_traced_rays_in_all_camera_modes() {
        for mode in CameraMode::ALL {
            let centre = [7.0, 31.0, -4.0];
            let [_, up, forward] = mode.basis();
            let camera =
                TraceCamera::orthographic_slab(centre, forward, up, 28.0, 16.0 / 9.0, SLAB_DEPTH)
                    .unwrap();
            let camera = serde_json::to_value(camera).unwrap();
            let vector = |name: &str| [0, 1, 2].map(|i| camera[name][i].as_f64().unwrap() as f32);
            let ray_origin = vector("origin");
            let right = vector("right");
            let up = vector("up");
            let direction = vector("forward");
            let wall = vector("wall");
            let matrix = clip_from_world(mode, centre, 28.0, 16.0 / 9.0);
            for uv in [[0.0, 0.0], [-0.75, 0.5], [0.75, -0.5]] {
                // Read the actual uploaded ray parameters; this is the WGSL
                // origin construction, compared against the raster matrix.
                let advance = wall[0] * uv[0] + wall[1] * uv[1] + wall[2];
                let origin = [0, 1, 2].map(|i| {
                    ray_origin[i] + right[i] * uv[0] + up[i] * uv[1] + direction[i] * advance
                });
                let a = transform(matrix, origin);
                let b = transform(matrix, [0, 1, 2].map(|i| origin[i] + direction[i] * 2.0));
                assert!((a[0] - uv[0]).abs() < 1e-5);
                assert!((a[1] - uv[1]).abs() < 1e-5);
                assert!((a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5);
                assert!(b[2] > a[2], "standard-z must order the same ray forward");
            }
        }
    }

    #[test]
    fn vertical_cut_plane_is_independent_of_height() {
        for mode in CameraMode::ALL {
            let slab = super::super::view::View {
                mode,
                centre: [4.0, 20.0, 7.0],
                half: 28.0,
                aspect: 1.0,
                depth: SLAB_DEPTH,
                pitch: None,
                bounds: None,
            }
            .clip();
            assert_eq!(slab.normal[1], 0.0);
            assert!((slab.max - slab.min - SLAB_DEPTH).abs() < 1e-5);
        }
    }
}
