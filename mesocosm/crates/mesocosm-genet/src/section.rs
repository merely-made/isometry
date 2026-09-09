// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The main view: the ruled terrarium section, brick-traced.
//!
//! `mesocosm-lens` owns the tracer; this module owns the vessel's policy over
//! it — which Ground the map binds, where the slab sits, which body is posed,
//! and how the traced texture reaches the surface the HUD then composites on.
//!
//! Which way it looks is [`camera`]'s, and stays a host flag: the shallow
//! oblique Mark ruled on 2026-09-04 is the default, and the two level arms the
//! measured slice compared it against are still there behind `--camera` (DC4,
//! Q9). None of the three moves the hash.

mod bodies;
mod camera;
mod capture;
mod materials;
mod terrain;
mod terrarium;
pub use terrain::TerrainStyle;
mod view;
pub use terrarium::{
    BODY_SCALE as TERRARIUM_BODY_SCALE, Cutaway, framed_habitat, occupied as terrarium_occupied,
};
pub use view::camera_basis;
mod inspection;

pub use bodies::{BodyFrameStats, BodyMode, DEFAULT_BODY_BUDGET};
pub use inspection::BodySelection;

pub use camera::{CameraMode, Framing, OBLIQUE_DEGREES, SLAB_DEPTH, SlabWindow, TERRARIUM_DEGREES};

use mesocosm_core::places::Ground;
use mesocosm_core::{BodyDocument, Organism, World};
use mesocosm_lens::{
    BodyLensProjection, BodyPlacement, BrickChange, BrickFrameInput, BrickMap, BrickRevision,
    BrickTracer, CritterPose, FRAME_FORMAT, Grade, MAX_ROSTER, TraceCamera,
};
use mesocosm_render::composite::Composite;

/// The G2 slab: section depth and the palette depth of the retro grade, plus
/// the half-height Mark ruled on 2026-08-29. Kept as defaults, not as constants
/// a camera policy may not vary.
///
/// **28, framing the content's height and letting the width follow.** The world
/// is 5.2:1 (129 voxels against a 25-voxel band) inside a 1.78:1 window, so no
/// half-height both frames the width and fills the height; 28 shows 77% of the
/// world and reads a 9-voxel limb at 9% of frame. A host may still carry
/// another ([`HostConfig::slab_half_height`](crate::HostConfig)), and every
/// framing replays to one hash. See the scale plan, "The framing".
pub const SLAB_HALF_HEIGHT: f32 = 28.0;
const PALETTE: u32 = 3;

/// Voxels one arrow-key press shifts the section by. Presentation only: this
/// number never reaches an intent.
pub const PAN_STEP: f32 = 2.0;

/// Captures are written in this format regardless of the surface's, so the
/// PNG encoder never has to swizzle a BGRA frame.
const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// A presentation-only offset added to the follow centre.
#[derive(Clone, Copy, Debug, Default)]
pub struct Pan {
    pub x: f32,
    pub y: f32,
}

/// The host's presentation reads for one frame, taken off the stepped world
/// before the device is borrowed. None of it is world state.
#[derive(Clone, Copy)]
pub struct SectionFrame<'a> {
    pub world: &'a World,
    pub volumes: &'a mesocosm_mesh::VolumeMap,
    pub ground: &'a Ground,
    /// The host's drain of the world's changed bricks. The slots they map to
    /// are the only region the tracer re-uploads, so a carve costs its own
    /// bricks and not the enclosure.
    pub dirty: &'a [[i16; 3]],
    pub centre: [f32; 3],
    /// The controlled critter, at full capsule fidelity.
    pub pose: Option<&'a CritterPose>,
    /// Everything else alive in the slab.
    pub roster: &'a [CritterPose],
}

pub struct Section {
    device: wgpu::Device,
    queue: wgpu::Queue,
    tracer: BrickTracer,
    map: BrickMap,
    grade: Grade,
    terrain_appearance: Option<mesocosm_lens::TerrainAppearance>,
    width: u32,
    height: u32,
    /// How much world this section frames. Presentation, so it lives beside the
    /// device rather than in the world.
    half_height: f32,
    /// Which way it looks. Presentation, beside the half-height and for the
    /// same reason: it frames the world and decides nothing in it.
    mode: CameraMode,
    body_mode: BodyMode,
    bodies: bodies::BodyLayer,
    terrarium: Option<terrarium::TerrariumView>,
    /// What the tracer writes: display-encoded values in a linear-tagged
    /// format, exactly as the lens's own captures read them back.
    traced: wgpu::Texture,
    traced_view: wgpu::TextureView,
    /// The same texels in an sRGB-tagged twin. Sampling the raw bytes into an
    /// sRGB target would encode a second time and wash the section out; the
    /// twin's decode-on-sample cancels the target's encode. Same remedy, same
    /// reason, as the HUD's vello raster.
    display: wgpu::Texture,
    display_view: wgpu::TextureView,
    composite: Composite,
}

impl Section {
    /// Binds the live Ground at genesis and builds the tracer on the host's
    /// own device. `format` is the surface's, for the composite that lands
    /// the traced frame under the HUD.
    pub fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        ground: &Ground,
        framing: Framing,
    ) -> Result<Self, String> {
        let map = BrickMap::from_ground(ground).map_err(|error| error.to_string())?;
        let tracer =
            BrickTracer::with_format(device.clone(), queue.clone(), width, height, FRAME_FORMAT);
        let composite = Composite::new(&device, format);
        let bodies = bodies::BodyLayer::new(&device, width, height);
        let (traced, traced_view) = target(&device, width, height, FRAME_FORMAT, "traced section");
        let (display, display_view) = target(
            &device,
            width,
            height,
            CAPTURE_FORMAT,
            "traced section srgb",
        );
        Ok(Self {
            device,
            queue,
            tracer,
            map,
            grade: Grade::retro(PALETTE),
            terrain_appearance: None,
            width,
            height,
            half_height: half_height_or_default(framing.half_height),
            mode: framing.mode,
            body_mode: BodyMode::default(),
            bodies,
            terrarium: None,
            traced,
            traced_view,
            display,
            display_view,
            composite,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.tracer.resize(self.width, self.height);
        self.bodies.resize(&self.device, self.width, self.height);
        (self.traced, self.traced_view) = target(
            &self.device,
            self.width,
            self.height,
            FRAME_FORMAT,
            "traced section",
        );
        (self.display, self.display_view) = target(
            &self.device,
            self.width,
            self.height,
            CAPTURE_FORMAT,
            "traced section srgb",
        );
    }

    /// Aspect comes from the window rather than a fixed 16:9, so a resized
    /// section stretches nothing.
    fn aspect(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }

    /// Scene depth stays the G2 number or the terrarium's bounds; an isolated
    /// preview fits the complete body. The tracer orthonormalizes its basis, so an
    /// off-axis section costs three unit vectors and nothing else.
    fn camera(&self, centre: [f32; 3]) -> Option<TraceCamera> {
        self.view(centre).trace()
    }

    /// Which way this section is looking. The receipt names it, so a capture
    /// can be told apart from the next one.
    pub fn mode(&self) -> CameraMode {
        self.mode
    }

    pub fn configure_bodies(&mut self, mode: BodyMode, budget: usize) {
        self.body_mode = mode;
        self.bodies.budget = budget.max(1);
        if mode == BodyMode::Capsules {
            self.bodies.clear_inspection();
        }
    }

    pub fn body_stats(&self) -> BodyFrameStats {
        self.bodies.stats.clone()
    }

    /// How much world the section frames, in voxels of half-height.
    pub fn half_height(&self) -> f32 {
        self.half_height
    }

    /// Host-owned framing, shared by terrain rays, body depth and culling.
    pub fn set_half_height(&mut self, half: f32) {
        self.half_height = half_height_or_default(half);
    }

    /// The world box this camera actually shows, from the camera's own
    /// numbers. What falls outside cannot reach a pixel, so it is what the
    /// roster culls against.
    pub fn slab_window(&self, centre: [f32; 3]) -> SlabWindow {
        self.view(centre).window()
    }

    /// Roster members the last traced frame drew. The receipt's evidence that
    /// the section shows the ecology rather than one body.
    pub fn last_roster_members(&self) -> u32 {
        if self.body_mode == BodyMode::Voxels {
            return (self.bodies.stats.voxel_bodies + self.bodies.stats.fallback_bodies)
                .saturating_sub(usize::from(self.bodies.stats.controlled_drawn))
                as u32;
        }
        self.tracer
            .last_diagnostics()
            .map_or(0, |diagnostics| diagnostics.roster_members)
    }

    /// Capsules the last frame's roster members carried past their budget. The
    /// widest are kept, so this is detail spent rather than bodies lost.
    pub fn last_roster_capsules_dropped(&self) -> u32 {
        self.tracer
            .last_diagnostics()
            .map_or(0, |diagnostics| diagnostics.roster_capsules_dropped)
    }

    /// Traces one frame into the tenant-owned display texture.
    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame: SectionFrame<'_>,
    ) -> Result<(), String> {
        let mut full = false;
        let slots = if let Some(view) = &mut self.terrarium {
            if let Some(map) = view.refresh(frame.ground, self.mode)? {
                self.map = map;
                full = true;
            }
            Vec::new()
        } else if frame.dirty.is_empty() {
            Vec::new()
        } else {
            self.map
                .refresh(frame.ground, frame.dirty.iter().copied())
                .map_err(|error| error.to_string())?
        };
        let change = if full {
            BrickChange::Full
        } else {
            BrickChange::Slots(&slots)
        };
        let camera = self
            .camera(frame.centre)
            .ok_or("invalid terrarium camera")?;
        if self.body_mode == BodyMode::Voxels {
            let window = self.slab_window(frame.centre);
            self.bodies.prepare(frame.world, frame.volumes, window);
            {
                let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("clear shared section attachments"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.traced_view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.bodies.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
            }
            let camera_view = self.view(frame.centre);
            let matrix = camera_view.matrix();
            if let Err(error) = self.bodies.draw(
                &self.device,
                &self.queue,
                encoder,
                &self.traced_view,
                camera_view,
            ) {
                eprintln!("{error}; using capsule fallback");
                self.bodies.stats.last_error = Some(error);
                self.bodies.fallback_all(frame.world);
            }
            if self.bodies.isolated && self.bodies.stats.fallback_bodies == 0 {
                self.copy_to_display(encoder);
                return Ok(());
            }
            let mut input = BrickFrameInput::for_camera(
                &self.map,
                BrickRevision(frame.ground.revision()),
                camera,
                &self.grade,
            )
            .changed(change)
            .with_clip_from_world(matrix)
            .with_roster(&self.bodies.fallback);
            input.terrain_appearance = self.terrain_appearance;
            if let Some(pose) = self.bodies.played_fallback.as_ref() {
                input = input.with_pose(pose);
            }
            self.tracer
                .encode_with_depth(encoder, &self.traced_view, &self.bodies.depth_view, input)
                .map_err(|error| error.to_string())?;
        } else {
            let mut input = BrickFrameInput::for_camera(
                &self.map,
                BrickRevision(frame.ground.revision()),
                camera,
                &self.grade,
            )
            .changed(change)
            .with_roster(frame.roster);
            input.terrain_appearance = self.terrain_appearance;
            if let Some(pose) = frame.pose {
                input = input.with_pose(pose);
            }
            self.tracer
                .encode(encoder, &self.traced_view, input)
                .map_err(|error| error.to_string())?;
        }
        self.copy_to_display(encoder);
        Ok(())
    }

    /// The completed same-device texture imported by Netrender's frame graph.
    pub fn display_texture(&self) -> &wgpu::Texture {
        &self.display
    }

    /// Chromeless fallback: trace and composite directly into the surface.
    pub fn draw(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        surface: &wgpu::TextureView,
        frame: SectionFrame<'_>,
    ) -> Result<(), String> {
        self.render(encoder, frame)?;
        self.composite.draw(
            &self.device,
            &self.queue,
            encoder,
            surface,
            &self.display_view,
            (0.0, 0.0, self.width as f32, self.height as f32),
            (self.width, self.height),
        );
        Ok(())
    }

    fn copy_to_display(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_texture(
            self.traced.as_image_copy(),
            self.display.as_image_copy(),
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }
}

fn target(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
    label: &str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

/// The half-height a host actually frames with: its own, or the default when
/// it named none. One reading, so the camera and the follow centre's clamp
/// cannot disagree about how tall the frame is.
pub fn half_height_or_default(configured: f32) -> f32 {
    if configured > 0.0 {
        configured
    } else {
        SLAB_HALF_HEIGHT
    }
}

/// Where the slab sits: on the controlled critter, plus the pan, **clamped so
/// the frame never dips below bedrock**.
///
/// The z component takes no pan, because the section's whole claim is that it
/// cuts through whoever is being played.
///
/// The clamp is the companion ruled with the half-height (2026-08-29): the
/// world's lowest brick is y = 0, so a frame centred below `half_height` shows
/// void along the bottom, which reads as a slab floating rather than a
/// terrarium. Past roughly 24 the shipped follow centre does exactly that.
/// It is presentation, like the framing — it moves no intent, so it cannot
/// reach the trace.
///
/// The floor is the camera's own vertical reach rather than the half-height
/// (DC4): a tilted section leans its slab depth into the vertical and frames
/// about a voxel more than a level one, so reading the half-height would let
/// `oblique` show exactly the strip of void the clamp exists to refuse. The
/// two level modes are unchanged, to the bit.
pub fn centre_on(at: [i32; 3], pan: Pan, half_height: f32, mode: CameraMode) -> [f32; 3] {
    let floor = mode.vertical_half(half_height_or_default(half_height));
    [
        at[0] as f32 + pan.x,
        (at[1] as f32 + pan.y).max(floor),
        at[2] as f32,
    ]
}

/// The controlled critter's pose, through the landed V2 projection.
///
/// It stays the tracer's single pose rather than a roster member, because a
/// member's capsule budget is smaller than the played body's: see
/// [`mesocosm_lens::MAX_ROSTER`].
/// The pose comes back with the count of parts the capsule budget dropped, so
/// a truncated player is reported rather than merely smaller.
pub fn pose_of(world: &World, tint: [f32; 3]) -> Option<(CritterPose, u32)> {
    pose_of_scaled(world, tint, 1.0, false)
}

pub fn pose_of_scaled(
    world: &World,
    tint: [f32; 3],
    scale: f32,
    grounded: bool,
) -> Option<(CritterPose, u32)> {
    let body = world.body()?;
    let mut at = world.position()?.map(|v| v as f32);
    if grounded {
        at[1] -= body.aabb().min[1] as f32 * scale;
    }
    pose_at_origin(body, at, tint, scale)
}

/// Every other living organism the window holds, posed and tinted.
///
/// The scan is a bounds test per organism and a projection only for those
/// inside, so the frame's cost tracks organisms in the slab rather than
/// organisms in the world. The lens truncates whatever exceeds its own cap;
/// the take here just stops projecting once the cap is met.
pub fn roster_of(
    world: &World,
    window: SlabWindow,
    tint: impl Fn(&Organism) -> [f32; 3],
) -> Vec<CritterPose> {
    roster_of_scaled(world, window, tint, 1.0, false)
}

pub fn roster_of_scaled(
    world: &World,
    window: SlabWindow,
    tint: impl Fn(&Organism) -> [f32; 3],
    scale: f32,
    grounded: bool,
) -> Vec<CritterPose> {
    let controlled = world.controlled_id();
    world
        .organisms
        .iter()
        .filter(|organism| {
            organism.is_alive()
                && Some(organism.id) != controlled
                && window.holds(organism.position)
        })
        .filter_map(|organism| {
            let body = organism.body();
            let mut at = organism.position.map(|v| v as f32);
            if grounded {
                at[1] -= body.aabb().min[1] as f32 * scale;
            }
            pose_at_origin(body, at, tint(organism), scale).map(|(pose, _)| pose)
        })
        .take(MAX_ROSTER)
        .collect()
}

/// One body placed where it stands, and the parts its capsule budget could
/// not carry.
///
/// Body space is world voxels — the raster lane draws a part's voxels at
/// `position + v` — so the scale is 1 and the projection's floor subtraction
/// is undone, or the two views would disagree about where the same voxel is.
///
/// **A body past the lens's capsule limit is drawn truncated, never dropped.**
/// Until DC3 this swallowed the refusal with `.ok()` and the frame simply had
/// no body in it, which is how a played critter could disappear while alive.
/// The overflow is now a number the host puts in its receipt.
#[cfg(test)]
fn pose_at(
    body: &BodyDocument,
    at: [i32; 3],
    tint: [f32; 3],
    scale: f32,
) -> Option<(CritterPose, u32)> {
    pose_at_origin(body, at.map(|v| v as f32), tint, scale)
}

fn pose_at_origin(
    body: &BodyDocument,
    at: [f32; 3],
    tint: [f32; 3],
    scale: f32,
) -> Option<(CritterPose, u32)> {
    let floor = body.aabb().min[1] as f32 * scale;
    let placement = BodyPlacement {
        ground: [at[0], at[1] + floor, at[2]],
        scale,
        tint,
    };
    BodyLensProjection::project_truncated(body, placement)
        .ok()
        .map(|(projected, dropped)| (projected.pose, dropped as u32))
}

#[cfg(test)]
mod depth_tests;
#[cfg(test)]
mod inspection_tests;
#[cfg(test)]
mod tests;
