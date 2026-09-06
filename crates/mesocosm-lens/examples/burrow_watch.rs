// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G4's headed judgment harness: the burrow run, watched from inside.
//!
//! ```text
//! cargo run -p mesocosm-lens --example burrow_watch --release
//! ```
//!
//! Every other G4 receipt measures the run. This one puts a person in
//! it, because §2's last done-condition is not a number: *does hiding
//! from a hunter have tension*. Wave 2.1's founding condition outranks
//! every receipt in the gate, and it cannot be asserted, only judged.
//!
//! So the camera sits at the hidden organism's eye rather than the
//! hunter's. `burrow_run` frames the scenario from `from` (the hunter's
//! stance) because a receipt wants to see the doorway open; a judgment
//! wants to be the one behind the wall. Same world, same seed, same
//! two organisms, opposite end of the sight line.
//!
//! What you do: nothing, at first. The hunter cannot see you and does
//! not come. Press **space** to carve the wall between you, which is
//! the scenario's recorded second intent, and watch what a working
//! sight line does to a thing that was ignoring you. Arrow keys look
//! around. **R** restores the world to the moment before the carve, so
//! the beat can be replayed as many times as judging it takes.
//!
//! The chrome bar is the instrument panel, since a judgment made
//! against an invisible simulation is a judgment about graphics: the
//! left lamp is sight (calm while the hunter cannot see you, alarm
//! when it can), the bar under it is the hunter's distance closing,
//! and the right lamp lights when it crosses the threshold.

use std::sync::Arc;

use mesocosm_core::places::spot;
use mesocosm_core::{Intent, OrganismId, World, state_hash};
use mesocosm_lens::{
    BodyLensProjection, BodyPlacement, BrickChange, BrickFrameInput, BrickMap, BrickRevision,
    BrickTracer, CritterPose, Flight, Grade,
};
use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, NetrenderOptions,
    PresentedFrame, Renderer, Scene, SurfaceKey, WgpuHandles, create_netrender_instance,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

#[path = "g4_frame/doorway_fixture.rs"]
mod burrow_scenario;

const SIZE: [u32; 2] = [1280, 720];
const MASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
const TRACE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
/// Ticks per frame. One keeps the world's clock and the eye's together,
/// which is what makes an approach feel like an approach.
const TICKS_PER_FRAME: u32 = 1;
/// The sight horizon the scenario's `spot` calls use.
const SIGHT: i32 = 8;

fn main() {
    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = Watch {
        live: None,
        state: Scenario::new(),
        yaw_input: 0.0,
        pitch_input: 0.0,
        frames: u64::MIN,
    };
    event_loop.run_app(&mut app).expect("winit run");
}

/// The world and the two stances the scenario is about.
struct Scenario {
    world: World,
    /// Where the hunter starts, and where it must come through.
    hunter_start: [i32; 3],
    doorway: [i32; 3],
    /// The eye: the hidden organism.
    player: [i32; 3],
    carved: bool,
    yaw: f32,
    pitch: f32,
    ticks: u64,
    /// The world as it stood before the carve, so the beat replays.
    before_carve: Option<World>,
}

impl Scenario {
    fn new() -> Self {
        let fixture = burrow_scenario::setup();
        // Face the doorway: the thing worth watching.
        let dx = (fixture.doorway[0] - fixture.player[0]) as f32;
        let dz = (fixture.doorway[2] - fixture.player[2]) as f32;
        let horizontal = (dx * dx + dz * dz).sqrt();
        let eye_y = fixture.player[1] as f32 + 1.3;
        let doorway_eye_y = fixture.doorway[1] as f32 + 1.0;
        Self {
            world: fixture.world,
            hunter_start: fixture.hunter_start,
            doorway: fixture.doorway,
            player: fixture.player,
            carved: false,
            yaw: f32::atan2(dx, dz),
            pitch: f32::atan2(doorway_eye_y - eye_y, horizontal),
            ticks: 0,
            before_carve: None,
        }
    }

    fn flight(&self) -> Flight {
        Flight {
            eye: [
                self.player[0] as f32 + 0.5,
                self.player[1] as f32 + 1.3,
                self.player[2] as f32 + 0.5,
            ],
            yaw: self.yaw,
            pitch: self.pitch,
            fov: 0.55,
            far: 24.0,
        }
    }

    /// Where the hunter is, or `None` if there is no living hunter.
    ///
    /// Deliberately not defaulted to its start. An earlier version fell
    /// back to `hunter_start` when the organism was missing, and the
    /// panel then reported a confident "2.8 voxels away" for twenty
    /// thousand ticks after the hunter had starved and been reaped. An
    /// instrument that cannot say "gone" will say something false
    /// instead, and a judgment taken against it is worthless.
    fn hunter(&self) -> Option<[i32; 3]> {
        self.world
            .organisms
            .iter()
            .find(|organism| organism.id == OrganismId(900) && organism.is_alive())
            .map(|organism| organism.position)
    }

    /// Whether the hunter can see the hiding organism right now. The
    /// same call the simulation makes, asked from the same end.
    fn seen(&self) -> bool {
        self.hunter()
            .is_some_and(|hunter| spot(self.world.ground(), hunter, self.player, SIGHT))
    }

    fn hunter_distance(&self) -> Option<f32> {
        self.hunter().map(|h| {
            let dx = (h[0] - self.player[0]) as f32;
            let dy = (h[1] - self.player[1]) as f32;
            let dz = (h[2] - self.player[2]) as f32;
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
    }

    fn tick(&mut self) {
        for _ in 0..TICKS_PER_FRAME {
            self.world.apply(Intent::Idle);
            self.ticks += 1;
        }
    }

    /// The scenario's recorded second intent, on a keypress instead of
    /// in a trace.
    fn carve(&mut self) {
        if self.carved {
            return;
        }
        self.before_carve = Some(self.world.clone());
        self.world.apply(Intent::Carve {
            at: [self.doorway[0], self.doorway[1] + 1, self.doorway[2]],
            radius: 1,
        });
        self.carved = true;
    }

    /// Back to the moment before the wall opened.
    fn rewind(&mut self) {
        if let Some(world) = self.before_carve.take() {
            self.world = world;
            self.carved = false;
        }
    }
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    handles: WgpuHandles,
    net: Renderer,
    tracer: BrickTracer,
    map: BrickMap,
    _trace: wgpu::Texture,
    trace_view: wgpu::TextureView,
    grade: Grade,
}

struct Watch {
    live: Option<Live>,
    state: Scenario,
    yaw_input: f32,
    pitch_input: f32,
    frames: u64,
}

impl ApplicationHandler for Watch {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Mesocosm: the burrow run, watched from inside")
            .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1]));
        let window = Arc::new(event_loop.create_window(attributes).expect("window"));
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window.clone())
            .expect("window surface");
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
            apply_limit_buckets: false,
        }))
        .expect("an adapter for the burrow watch");
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).expect("device");
        println!("adapter: {}", adapter.get_info().name);

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(capabilities.formats[0]);
        let handles = WgpuHandles {
            instance,
            adapter,
            device: device.clone(),
            queue: queue.clone(),
        };
        let net = create_netrender_instance(
            handles.clone(),
            NetrenderOptions {
                tile_cache_size: Some(SIZE[0]),
                enable_vello: true,
                ..Default::default()
            },
        )
        .expect("netrender for the burrow watch");

        let map = BrickMap::from_ground(self.state.world.ground()).expect("atlas capacity");
        let tracer = BrickTracer::with_format(
            device.clone(),
            queue.clone(),
            SIZE[0],
            SIZE[1],
            TRACE_FORMAT,
        );
        let trace = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("burrow watch trace"),
            size: wgpu::Extent3d {
                width: SIZE[0],
                height: SIZE[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TRACE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let trace_view = trace.create_view(&Default::default());

        let mut live = Live {
            window,
            surface,
            format,
            handles,
            net,
            tracer,
            map,
            _trace: trace,
            trace_view,
            grade: Grade::retro(3),
        };
        configure(&mut live);
        println!(
            "hidden at {:?}, doorway {:?}, hunter starts {:?}",
            self.state.player, self.state.doorway, self.state.hunter_start
        );
        println!("space carves the wall; R rewinds to just before it; arrows look; Esc quits");
        live.window.request_redraw();
        self.live = Some(live);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(live) = self.live.as_mut() {
                    configure(live);
                }
            },
            WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == ElementState::Pressed;
                match event.logical_key {
                    Key::Named(NamedKey::Escape) if down => event_loop.exit(),
                    Key::Named(NamedKey::Space) if down => {
                        self.state.carve();
                        println!(
                            "carved at tick {}: the wall is open, sight is {}",
                            self.state.ticks,
                            if self.state.seen() {
                                "through"
                            } else {
                                "still blocked"
                            }
                        );
                    },
                    Key::Character(ref c) if down && (c == "r" || c == "R") => {
                        self.state.rewind();
                        println!("rewound to the moment before the carve");
                    },
                    Key::Named(NamedKey::ArrowLeft) => {
                        self.yaw_input = if down { -1.0 } else { 0.0 }
                    },
                    Key::Named(NamedKey::ArrowRight) => {
                        self.yaw_input = if down { 1.0 } else { 0.0 }
                    },
                    Key::Named(NamedKey::ArrowUp) => {
                        self.pitch_input = if down { 1.0 } else { 0.0 }
                    },
                    Key::Named(NamedKey::ArrowDown) => {
                        self.pitch_input = if down { -1.0 } else { 0.0 }
                    },
                    _ => {},
                }
            },
            WindowEvent::RedrawRequested => self.frame(),
            _ => {},
        }
    }
}

impl Watch {
    fn frame(&mut self) {
        let Some(live) = self.live.as_mut() else {
            return;
        };

        self.state.yaw += self.yaw_input * 0.03;
        self.state.pitch = (self.state.pitch + self.pitch_input * 0.02).clamp(-1.2, 1.2);
        let was_seen = self.state.seen();
        self.state.tick();
        if self.state.seen() != was_seen {
            println!(
                "tick {}: sight {} (hunter {})",
                self.state.ticks,
                if self.state.seen() {
                    "OPENED"
                } else {
                    "closed"
                },
                match self.state.hunter_distance() {
                    Some(distance) => format!("{distance:.1} voxels away"),
                    None => "gone".into(),
                }
            );
        }

        // Ground changes only on a carve, so the atlas refresh is the
        // narrow one the receipts measure rather than a per-frame
        // upload.
        let mut projection = self.state.world.ground().clone();
        let dirty = projection.drain_dirty();
        let slots = if dirty.is_empty() {
            Vec::new()
        } else {
            live.map
                .refresh(self.state.world.ground(), dirty)
                .expect("atlas refresh")
        };

        let pose = hunter_pose(&self.state.world);
        let flight = self.state.flight();
        let revision = BrickRevision(self.state.world.ground().revision());
        let mut input = BrickFrameInput::new(&live.map, revision, &flight, &live.grade);
        if !slots.is_empty() {
            input = input.changed(BrickChange::Slots(&slots));
        }
        if let Some(pose) = pose.as_ref() {
            input = input.with_pose(pose);
        }

        let mut encoder =
            live.handles
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("burrow watch trace"),
                });
        live.tracer
            .encode(&mut encoder, &live.trace_view, input)
            .expect("trace");
        live.handles.queue.submit([encoder.finish()]);

        let chrome = instrument(
            self.state.seen(),
            self.state.hunter_distance(),
            self.state.carved,
            self.state.hunter() == Some(self.state.doorway),
        );
        let external = [ExternalTextureComposite::new(
            &live.trace_view,
            ExternalTexturePlacement::new([0.0, 0.0, SIZE[0] as f32, SIZE[1] as f32]),
        )
        .with_scene_op_boundary(0)];
        let mut grab = MasterGrab { master: None };
        live.net.render_with_compositor_and_external_textures(
            &chrome,
            MASTER_FORMAT,
            &mut grab,
            netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
            &external,
        );
        let Some(master) = grab.master else {
            return;
        };

        let size = live.window.inner_size();
        use wgpu::CurrentSurfaceTexture as Acquired;
        match live.surface.get_current_texture() {
            Acquired::Success(frame) | Acquired::Suboptimal(frame) => {
                let target = frame.texture.create_view(&Default::default());
                let master_view = master.create_view(&Default::default());
                live.net.compose_external_texture(
                    &master_view,
                    &target,
                    live.format,
                    size.width.max(1),
                    size.height.max(1),
                    ExternalTexturePlacement::new([
                        0.0,
                        0.0,
                        size.width.max(1) as f32,
                        size.height.max(1) as f32,
                    ]),
                );
                live.window.pre_present_notify();
                live.handles.queue.present(frame);
            },
            Acquired::Outdated | Acquired::Lost => configure(live),
            Acquired::Timeout | Acquired::Occluded => {},
            Acquired::Validation => panic!("surface acquisition failed validation"),
        }

        self.frames += 1;
        if self.frames.is_multiple_of(600) {
            match (self.state.hunter(), self.state.hunter_distance()) {
                (Some(at), Some(distance)) => println!(
                    "tick {}: hunter at {at:?}, {distance:.1} voxels away, sight {}, world hash {}",
                    self.state.ticks,
                    if self.state.seen() { "open" } else { "blocked" },
                    state_hash(&self.state.world)
                ),
                _ => println!(
                    "tick {}: NO LIVING HUNTER (it starved and was reaped); nothing to judge until the scenario keeps one alive",
                    self.state.ticks
                ),
            }
        }
        live.window.request_redraw();
    }
}

/// The instrument panel, in rects: sight lamp, closing distance, and a
/// threshold lamp. A judgment about tension needs the simulation's own
/// state legible, or it becomes a judgment about the picture.
fn instrument(seen: bool, distance: Option<f32>, carved: bool, at_doorway: bool) -> Scene {
    let width = SIZE[0] as f32;
    let mut scene = Scene::new(SIZE[0], SIZE[1]);
    scene.push_rect(0.0, 0.0, width, 44.0, [0.02, 0.03, 0.05, 0.86]);

    // Sight: green while it cannot see you, red when it can, and a
    // dead grey when there is no hunter left to see anything. The third
    // state exists because the scenario reaches it.
    let sight = match (distance, seen) {
        (None, _) => [0.35, 0.35, 0.38, 0.9],
        (Some(_), true) => [0.94, 0.28, 0.24, 0.95],
        (Some(_), false) => [0.22, 0.78, 0.48, 0.92],
    };
    scene.push_rect(18.0, 12.0, 150.0, 32.0, sight);

    // Distance, closing left to right: the bar grows as it nears.
    let near = distance.map_or(0.0, |d| (1.0 - (d / 12.0).clamp(0.0, 1.0)).clamp(0.0, 1.0));
    let bar_left = 170.0;
    let bar_right = width - 200.0;
    scene.push_rect(bar_left, 18.0, bar_right, 26.0, [0.10, 0.12, 0.16, 0.9]);
    scene.push_rect(
        bar_left,
        18.0,
        bar_left + (bar_right - bar_left) * near,
        26.0,
        if seen {
            [0.95, 0.55, 0.25, 0.95]
        } else {
            [0.35, 0.45, 0.62, 0.95]
        },
    );

    // Carved lamp, then the threshold lamp when it actually steps in.
    if carved {
        scene.push_rect(
            width - 180.0,
            12.0,
            width - 110.0,
            32.0,
            [0.85, 0.75, 0.30, 0.9],
        );
    }
    if at_doorway {
        scene.push_rect(
            width - 96.0,
            12.0,
            width - 18.0,
            32.0,
            [0.95, 0.25, 0.30, 0.98],
        );
    }
    scene.push_rect(
        0.0,
        0.0,
        width,
        2.0,
        if seen {
            [0.94, 0.28, 0.24, 1.0]
        } else {
            [0.22, 0.78, 0.48, 1.0]
        },
    );
    scene
}

/// The hunter's body, projected so it can be seen coming.
///
/// The camera is the hidden organism's eye, so the played body is not
/// what belongs on screen: in first person you do not watch yourself
/// hide, you watch the thing that is looking for you. This projects the
/// consumer through the same `BodyLensProjection` the composed receipt
/// uses for the player, so what approaches is the authoritative body
/// rather than a marker standing in for one.
fn hunter_pose(world: &World) -> Option<CritterPose> {
    let hunter = world
        .organisms
        .iter()
        .find(|organism| organism.id == OrganismId(900) && organism.is_alive())?;
    BodyLensProjection::project(
        hunter.body(),
        BodyPlacement {
            ground: [
                hunter.position[0] as f32 + 0.5,
                hunter.position[1] as f32,
                hunter.position[2] as f32 + 0.5,
            ],
            scale: 0.35,
            // Warm against the cold ground, so an approach reads at a
            // glance rather than needing to be hunted for in the voxels.
            tint: [0.92, 0.44, 0.24],
        },
    )
    .ok()
    .map(|projected| projected.pose)
}

struct MasterGrab {
    master: Option<wgpu::Texture>,
}

impl Compositor for MasterGrab {
    fn declare_surface(&mut self, _key: SurfaceKey, _bounds: [f32; 4]) {}
    fn destroy_surface(&mut self, _key: SurfaceKey) {}
    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        self.master = Some(frame.master.clone());
    }
}

fn configure(live: &mut Live) {
    let size = live.window.inner_size();
    live.surface.configure(
        &live.handles.device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: live.format,
            color_space: wgpu::SurfaceColorSpace::default(),
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
}
