// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! GPU evidence using the same material adapter and cached renderer as play.

use crate::section::{CameraMode, bodies::clip_from_world};
use mesocosm_core::{Founding, OrganismId, World};
use mesocosm_mesh::{LiveBodyProjector, VolumeMap};
use mesocosm_render::{LiveBody, LiveBodyRenderer, Renderer};

struct Capture {
    host: Renderer,
    renderer: LiveBodyRenderer,
    projector: LiveBodyProjector,
    volumes: VolumeMap,
    colour: wgpu::Texture,
    depth: wgpu::Texture,
}

impl Capture {
    fn new(world: &World) -> Option<Self> {
        let host = match Renderer::headless(512, 512) {
            Ok(host) => host,
            Err(mesocosm_render::RenderError::NoAdapter) => {
                eprintln!("no adapter; skipping material pixel receipt");
                return None;
            },
            Err(error) => panic!("material receipt renderer failed: {error:?}"),
        };
        let texture = |format, usage| {
            host.device().create_texture(&wgpu::TextureDescriptor {
                label: Some("material comparison"),
                size: wgpu::Extent3d {
                    width: 512,
                    height: 512,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };
        let colour = texture(
            wgpu::TextureFormat::Rgba8Unorm,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        );
        let depth = texture(
            wgpu::TextureFormat::Depth32Float,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        );
        let renderer = LiveBodyRenderer::new(host.device(), wgpu::TextureFormat::Rgba8Unorm, 256);
        Some(Self {
            host,
            renderer,
            projector: LiveBodyProjector::new(),
            // Match host admission: legacy volume content is fixed at origin.
            // Growing envelopes must not redefine an existing volume address.
            volumes: crate::fixture::volumes_for(world),
            colour,
            depth,
        })
    }

    fn body(
        &mut self,
        world: &World,
        subject: OrganismId,
        hide_secretion: bool,
    ) -> (Vec<u8>, usize) {
        let organism = world
            .organisms
            .iter()
            .find(|o| o.id == subject)
            .expect("recorded body");
        let projection = self
            .projector
            .project(subject, organism.body(), &self.volumes)
            .unwrap();
        let mut materials = super::project(&organism.phenotype, world.ruleset());
        if hide_secretion {
            materials.retain(|m| m.process != mesocosm_core::process::Process::Secrete);
        }
        let bounds = organism.body().aabb();
        let centre = [0, 1, 2].map(|i| (bounds.min[i] + bounds.max[i]) as f32 * 0.5);
        let extent = (0..3)
            .map(|i| (bounds.max[i] - bounds.min[i]) as f32)
            .fold(1.0, f32::max);
        let matrix = clip_from_world(CameraMode::Oblique, centre, extent * 0.75, 1.0);
        let device = self.host.device();
        let queue = self.host.queue();
        let colour = self.colour.create_view(&Default::default());
        let depth = self.depth.create_view(&Default::default());
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("material comparison clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &colour,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
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
        let stats = self
            .renderer
            .draw(
                device,
                queue,
                &mut encoder,
                &colour,
                &depth,
                matrix,
                None,
                &[LiveBody {
                    materials: &materials,
                    ..LiveBody::new(&projection.mesh, [0.0; 3])
                }],
            )
            .unwrap();
        queue.submit([encoder.finish()]);
        let pixels = read(device, queue, &self.colour);
        assert!(pixels.chunks_exact(4).any(|pixel| pixel[..3] != [0, 0, 0]));
        (pixels, stats.mesh_upload_bytes)
    }
}

fn read(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture) -> Vec<u8> {
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("material readback"),
        size: 512 * 512 * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(512 * 4),
                rows_per_image: Some(512),
            },
        },
        wgpu::Extent3d {
            width: 512,
            height: 512,
            depth_or_array_layers: 1,
        },
    );
    queue.submit([encoder.finish()]);
    staging.slice(..).map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let pixels = staging.slice(..).get_mapped_range().unwrap().to_vec();
    staging.unmap();
    pixels
}

fn save(name: &str, pixels: &[u8]) {
    let Ok(directory) = std::env::var("MESOCOSM_MATERIAL_RECEIPTS") else {
        return;
    };
    let path = std::path::Path::new(&directory).join(format!("gpu-{name}.png"));
    let mut png = png::Encoder::new(std::fs::File::create(path).unwrap(), 512, 512);
    png.set_color(png::ColorType::Rgba);
    png.set_depth(png::BitDepth::Eight);
    png.write_header()
        .unwrap()
        .write_image_data(pixels)
        .unwrap();
}

#[test]
fn accepted_expression_changes_pixels_without_rebuilding_geometry() {
    let founding = Founding::SpacedRoster;
    let mut world = World::expression_practice(7, founding, founding.palette()).unwrap();
    let initial_hash = mesocosm_core::state_hash(&world);
    let condition = world.discoveries()[0].condition;
    let subject = world.controlled_id().unwrap();
    let Some(mut capture) = Capture::new(&world) else {
        return;
    };
    let (before, initial_uploads) = capture.body(&world, subject, false);
    assert!(initial_uploads > 0);
    let preview = world.preview_expression(condition).unwrap();
    let mut candidate = world.clone();
    candidate
        .organisms
        .iter_mut()
        .find(|o| o.id == subject)
        .unwrap()
        .phenotype = preview.phenotype;
    let (expected, uploads) = capture.body(&candidate, subject, false);
    assert_eq!(uploads, 0);
    assert_ne!(
        before, expected,
        "expression has a visible material consequence"
    );
    assert_eq!(mesocosm_core::state_hash(&world), initial_hash);
    assert!(matches!(
        world.apply(mesocosm_core::Intent::Express { condition }),
        mesocosm_core::Outcome::Expressed { .. }
    ));
    let (actual, uploads) = capture.body(&world, subject, false);
    assert_eq!(uploads, 0);
    assert_eq!(expected, actual, "checked preview matches accepted tissue");
    save("before", &before);
    save("preview", &expected);
    save("accepted", &actual);
}

#[test]
fn descendant_comparison_renders_recorded_bodies() {
    let founding = Founding::SpacedRoster;
    let proof = World::descendant_practice(7, founding, founding.palette()).unwrap();
    let Some(mut capture) = Capture::new(&proof.stages[0].world) else {
        return;
    };
    let mut receipts = Vec::new();
    let mut rendered = std::collections::BTreeMap::new();
    for stage in &proof.stages {
        let hash = mesocosm_core::state_hash(&stage.world);
        let (pixels, _) = capture.body(&stage.world, stage.subject, false);
        let name = stage.label.replace(' ', "-");
        save(&name, &pixels);
        if stage.label == "after revision child" {
            let (unmarked, uploads) = capture.body(&stage.world, stage.subject, true);
            assert_eq!(uploads, 0);
            assert_ne!(
                pixels, unmarked,
                "inherited secretion itself changes the child's pixels"
            );
            save("after-revision-child-without-secretion-mark", &unmarked);
        }
        rendered.insert(stage.label, pixels);
        assert_eq!(mesocosm_core::state_hash(&stage.world), hash);
        let organism = stage
            .world
            .organisms
            .iter()
            .find(|o| o.id == stage.subject)
            .unwrap();
        let materials = super::project(&organism.phenotype, stage.world.ruleset());
        receipts.push(serde_json::json!({ "stage": stage.label, "subject": stage.subject.0,
            "tick": stage.world.tick, "state_hash": format!("{hash:016x}"),
            "program": stage.world.lineages().get(organism.species).unwrap().program().digest(),
            "secretory_parts": materials.iter().filter(|m| m.process == mesocosm_core::process::Process::Secrete).count(),
            "assisted_setup": proof.assisted_setup }));
    }
    assert_eq!(
        rendered["initial parent"],
        rendered["initial control parent"]
    );
    assert_ne!(rendered["initial parent"], rendered["expressed parent"]);
    for row in &receipts {
        let expected = matches!(
            row["stage"].as_str().unwrap(),
            "expressed parent" | "after revision child"
        );
        assert_eq!(row["secretory_parts"].as_u64().unwrap() > 0, expected);
    }
    if let Ok(directory) = std::env::var("MESOCOSM_MATERIAL_RECEIPTS") {
        let evidence = serde_json::json!({
            "assisted_setup": proof.assisted_setup,
            "control": "separate counterfactual world",
            "condition": proof.condition, "revision": proof.revision,
            "expression": proof.expression, "revision_commit": proof.revision_commit,
            "before_revision_birth": proof.before_revision_birth,
            "after_revision_birth": proof.after_revision_birth, "control_birth": proof.control_birth,
            "intervention_events": proof.intervention_events, "control_events": proof.control_events,
        });
        std::fs::write(
            std::path::Path::new(&directory).join("descendant-events.json"),
            serde_json::to_vec_pretty(&evidence).unwrap(),
        )
        .unwrap();
        std::fs::write(
            std::path::Path::new(&directory).join("descendants.json"),
            serde_json::to_vec_pretty(&receipts).unwrap(),
        )
        .unwrap();
    }
}
