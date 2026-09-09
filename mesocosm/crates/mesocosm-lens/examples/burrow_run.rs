// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G4's first composed receipt: one generated `World`, its ordered carve
//! trace, the autonomous pursuit consequence, replay hash, and the narrow
//! Ground-to-DDA upload and external netrender frame that make the opening
//! visible.

use std::fs::File;

use mesocosm_core::places::{WALKER_HEIGHT, spot};
use mesocosm_core::{Intent, OrganismId, Outcome, World, state_hash};
use mesocosm_lens::{
    BodyLensProjection, BodyPlacement, BrickChange, BrickDiagnostics, BrickFrameInput, BrickMap,
    BrickRevision, BrickTracer, Flight, Grade,
};
use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, PresentedFrame, Scene,
    WgpuHandles, create_netrender_instance,
};

#[path = "g4_frame/doorway_fixture.rs"]
mod burrow_scenario;

const SEED: u64 = burrow_scenario::SEED;
const FRAME_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

struct Run {
    world: World,
    from: [i32; 3],
    doorway: [i32; 3],
    player: [i32; 3],
    trace: [Intent; 2],
}

fn main() -> Result<(), String> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "g4_burrow_run.png".into());
    let mut run = setup();
    let mut twin = setup();
    assert!(!spot(run.world.ground(), run.from, run.player, 8));

    let mut map = BrickMap::from_ground(run.world.ground()).map_err(|error| error.to_string())?;
    let mut frame = G4Frame::new(480, 270)?;
    let flight = flight(&run);
    let body = player_body(&run.world)?;
    assert_eq!(
        body.parts.len(),
        run.world
            .controlled()
            .expect("the G4 run has a played organism")
            .body()
            .living()
            .count(),
        "the SDF projection omitted a living player part"
    );
    let grade = Grade::retro(3);
    let before = frame.capture(
        BrickFrameInput::new(
            &map,
            BrickRevision(run.world.ground().revision()),
            &flight,
            &grade,
        )
        .with_pose(&body.pose),
    )?;

    let idle = run.world.apply(run.trace[0].clone());
    twin.world.apply(run.trace[0].clone());
    assert!(matches!(idle, Outcome::Idled));
    assert_eq!(
        hunter(&run.world),
        run.from,
        "hunter escaped cover before sight opened"
    );
    let carved = run.world.apply(run.trace[1].clone());
    twin.world.apply(run.trace[1].clone());
    let Outcome::Carved { removed, .. } = carved else {
        return Err("G4 carve was rejected".into());
    };
    assert!(removed > 0, "G4 carve removed no Ground");
    assert!(spot(run.world.ground(), run.from, run.player, 8));
    assert_eq!(
        hunter(&run.world),
        run.doorway,
        "hunter did not enter the opening"
    );
    assert!(run.world.ground().stands(run.doorway, WALKER_HEIGHT));
    assert_eq!(state_hash(&run.world), state_hash(&twin.world));

    let mut projection = run.world.ground().clone();
    let dirty = projection.drain_dirty();
    let slots = map
        .refresh(run.world.ground(), dirty)
        .map_err(|error| error.to_string())?;
    let after = frame.capture(
        BrickFrameInput::new(
            &map,
            BrickRevision(run.world.ground().revision()),
            &flight,
            &grade,
        )
        .changed(BrickChange::Slots(&slots))
        .with_pose(&body.pose),
    )?;
    assert!(!slots.is_empty());
    assert_ne!(
        before.pixels, after.pixels,
        "carve did not reach DDA pixels"
    );
    write_png(&output, frame.width, frame.height, &after.pixels)?;
    println!(
        "G4 composed run: seed={SEED}, doorway={:?}, removed={removed}, dirty_slots={}, \
         upload_bytes={}, netrender_total_us={}, netrender_spans={}, replay_hash={:?}, \
         player_body_revision={}, player_capsules={}, before_pixels_changed=true, capture={output}",
        run.doorway,
        slots.len(),
        after.diagnostics.brick_upload_bytes,
        after.netrender_total_us,
        after.netrender_spans,
        state_hash(&run.world),
        body.revision.0,
        body.pose.capsules.len(),
    );
    Ok(())
}

struct PresentedMaster {
    texture: Option<wgpu::Texture>,
}

impl Compositor for PresentedMaster {
    fn declare_surface(&mut self, _key: netrender::SurfaceKey, _world_bounds: [f32; 4]) {}

    fn destroy_surface(&mut self, _key: netrender::SurfaceKey) {}

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        self.texture = Some(frame.master.clone());
    }
}

struct CapturedFrame {
    pixels: Vec<u8>,
    diagnostics: BrickDiagnostics,
    netrender_total_us: u64,
    netrender_spans: usize,
}

/// The same-device projection join: Ground DDA encodes into an external
/// texture, then netrender owns the master frame around it. This keeps the
/// receipt at the actual frame boundary rather than treating a tracer capture
/// as an interchangeable presentation path.
struct G4Frame {
    width: u32,
    height: u32,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tracer: BrickTracer,
    // The view below borrows this allocation by device identity rather than
    // Rust lifetime; retain it for the complete external-texture frame.
    _source: wgpu::Texture,
    source_view: wgpu::TextureView,
    net: netrender::Renderer,
    chrome: Scene,
}

impl G4Frame {
    fn new(width: u32, height: u32) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .map_err(|error| format!("no adapter for the G4 frame receipt: {error}"))?;
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))
            .map_err(|error| format!("adapter declined a G4 frame device: {error}"))?;
        let net = create_netrender_instance(
            WgpuHandles {
                instance,
                adapter,
                device: device.clone(),
                queue: queue.clone(),
            },
            netrender::NetrenderOptions {
                tile_cache_size: Some(width),
                enable_vello: true,
                ..Default::default()
            },
        )
        .map_err(|error| format!("netrender rejected the G4 tracer device: {error:?}"))?;
        let source = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("G4 trace external texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FRAME_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let source_view = source.create_view(&Default::default());
        let mut chrome = Scene::new(width, height);
        chrome.push_rect(0.0, 0.0, width as f32, 6.0, [0.10, 0.76, 0.42, 0.88]);
        Ok(Self {
            width,
            height,
            tracer: BrickTracer::with_format(
                device.clone(),
                queue.clone(),
                width,
                height,
                FRAME_FORMAT,
            ),
            device,
            queue,
            _source: source,
            source_view,
            net,
            chrome,
        })
    }

    fn capture(&mut self, input: BrickFrameInput<'_>) -> Result<CapturedFrame, String> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("G4 trace encode"),
            });
        let diagnostics = self
            .tracer
            .encode(&mut encoder, &self.source_view, input)
            .map_err(|error| error.to_string())?;
        self.queue.submit([encoder.finish()]);

        let external = [ExternalTextureComposite::new(
            &self.source_view,
            ExternalTexturePlacement::new([0.0, 0.0, self.width as f32, self.height as f32]),
        )
        .with_scene_op_boundary(0)];
        let mut master = PresentedMaster { texture: None };
        self.net.render_with_compositor_and_external_textures(
            &self.chrome,
            FRAME_FORMAT,
            &mut master,
            netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
            &external,
        );
        let master = master
            .texture
            .ok_or("netrender did not present a G4 master frame")?;
        let timings = self
            .net
            .last_frame_timings()
            .ok_or("netrender did not report G4 frame timings")?;
        Ok(CapturedFrame {
            pixels: self
                .net
                .wgpu_device
                .read_rgba8_texture(&master, self.width, self.height),
            diagnostics,
            netrender_total_us: timings.total.as_micros() as u64,
            netrender_spans: timings.spans.len(),
        })
    }
}

fn hunter(world: &World) -> [i32; 3] {
    world
        .organisms
        .iter()
        .find(|organism| organism.id == OrganismId(900))
        .map(|organism| organism.position)
        .expect("the G4 hunter survives its short receipt")
}

fn flight(run: &Run) -> Flight {
    let dx = (run.doorway[0] - run.from[0]) as f32;
    let dz = (run.doorway[2] - run.from[2]) as f32;
    Flight {
        eye: [
            run.from[0] as f32 + 0.5,
            run.from[1] as f32 + 1.3,
            run.from[2] as f32 + 0.5,
        ],
        yaw: f32::atan2(dx, dz),
        pitch: 0.0,
        fov: 0.55,
        far: 16.0,
    }
}

fn player_body(world: &World) -> Result<BodyLensProjection, String> {
    let player = world
        .controlled()
        .ok_or("the G4 run lost its played organism before projection")?;
    BodyLensProjection::project(
        player.body(),
        BodyPlacement {
            ground: [
                player.position[0] as f32 + 0.5,
                player.position[1] as f32,
                player.position[2] as f32 + 0.5,
            ],
            scale: 0.35,
            tint: [0.18, 0.82, 0.38],
        },
    )
    .map_err(|error| format!("could not project the played G4 body: {error:?}"))
}

fn setup() -> Run {
    let fixture = burrow_scenario::setup();
    let world = fixture.world;
    let from = fixture.hunter_start;
    let doorway = fixture.doorway;
    let player = fixture.player;
    Run {
        world,
        from,
        doorway,
        player,
        trace: [
            Intent::Idle,
            Intent::Carve {
                at: [doorway[0], doorway[1] + 1, doorway[2]],
                radius: 1,
            },
        ],
    }
}

fn write_png(path: &str, width: u32, height: u32, pixels: &[u8]) -> Result<(), String> {
    let file = File::create(path).map_err(|error| error.to_string())?;
    let mut png = png::Encoder::new(file, width, height);
    png.set_color(png::ColorType::Rgba);
    png.set_depth(png::BitDepth::Eight);
    png.write_header()
        .and_then(|mut writer| writer.write_image_data(pixels))
        .map_err(|error| error.to_string())
}
