// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Headed first-encounter host for the damaged crossing.
//!
//! The fixture owns the world and its consequences. This binary only translates
//! winit input, advances the fixed clock, and presents the shared GPU frame.

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use paredros_room::crossing::{self, BodyId, BodyKind, HeldInput, Input, TriggeredInput};
use paredros_room::gpu::{self, Composer, SIZE, Tenant};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::KeyEvent;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

const STEP: f64 = 1.0 / 60.0;
const MAX_CATCHUP: usize = 5;

fn main() {
    let smoke = env::var_os("PAREDROS_CROSSING_SMOKE").is_some();
    let body = if matches!(env::var("PAREDROS_CROSSING_BODY").as_deref(), Ok("climber")) {
        BodyKind::Climber
    } else {
        BodyKind::Crawler
    };
    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(smoke, body);
    event_loop.run_app(&mut app).expect("crossing host");
}

struct App {
    instance: wgpu::Instance,
    world: Option<crossing::ContactWorld>,
    player: BodyId,
    body: BodyKind,
    live: Option<Live>,
    input: InputState,
    yaw: f32,
    distance: f32,
    ambient: f32,
    last: Option<Instant>,
    accumulator: f64,
    ticks: u64,
    smoke: bool,
    smoke_done: bool,
    initial_position: [f32; 3],
    last_cursor: Option<winit::dpi::PhysicalPosition<f64>>,
    dragging: bool,
    simulation_error: Option<String>,
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tenant: Tenant,
    composer: Composer,
    hud: crossing::Hud,
}

#[derive(Default)]
struct InputState {
    held: HeldInput,
    triggered: TriggeredInput,
    keys: HashSet<KeyCode>,
    practice_strike: bool,
    previous_held: HeldInput,
    release_latch: bool,
}

impl App {
    fn new(smoke: bool, body: BodyKind) -> Self {
        let (world, player) = crossing::new_world(body);
        let initial_position = world.body(player).expect("player body").position;
        Self {
            instance: wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            ),
            world: Some(world),
            player,
            body,
            live: None,
            input: InputState::default(),
            yaw: 0.0,
            distance: 8.0,
            ambient: env::var("PAREDROS_CROSSING_AMBIENT")
                .ok()
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|n| n.is_finite())
                .unwrap_or(0.72)
                .clamp(0., 1.),
            last: None,
            accumulator: 0.0,
            ticks: 0,
            smoke,
            smoke_done: false,
            initial_position,
            last_cursor: None,
            dragging: false,
            simulation_error: None,
        }
    }

    fn reset(&mut self, body: BodyKind) {
        let (world, player) = crossing::new_world(body);
        self.initial_position = world.body(player).expect("player body").position;
        self.world = Some(world);
        self.player = player;
        self.body = body;
        self.ticks = 0;
        self.accumulator = 0.0;
        self.input = InputState::default();
        self.simulation_error = None;
        if let Some(live) = self.live.as_ref() {
            live.window.set_title("Paredros: Damaged Crossing");
        }
    }

    fn step(&mut self) {
        let Some(world) = self.world.as_mut() else {
            return;
        };
        let mut input = self.input.take();
        let practice_strike = self.input.practice_strike;
        self.input.practice_strike = false;
        // Keyboard movement is camera-relative; the fixture receives world
        // axes so replay remains independent of this presentation policy.
        let (x, z) = rotate_axes(input.held.move_x, input.held.move_z, self.yaw);
        input.held.move_x = x;
        input.held.move_z = z;
        if self.smoke {
            // A short, deterministic walk, then the fixture verbs in order.
            input.held.move_x = if self.ticks < 75 { 1.0 } else { 0.0 };
            input.held.move_z = if (75..120).contains(&self.ticks) {
                1.0
            } else {
                0.0
            };
            input.held.look_x = 0.0;
            input.held.look_z = 0.0;
            input.triggered.interact = self.ticks == 120;
            input.triggered.anchor = self.ticks == 135;
            input.triggered.attack = self.ticks == 150;
            input.triggered.recover = self.ticks == 165;
        }
        let mut inputs = vec![(self.player, input)];
        if practice_strike {
            if let Some(target) = world.body(BodyId(1)) {
                let player = world.body(self.player).expect("player body");
                let dx = player.position[0] - target.position[0];
                let dz = player.position[2] - target.position[2];
                let length = (dx * dx + dz * dz).sqrt().max(f32::EPSILON);
                inputs.push((
                    BodyId(1),
                    Input {
                        held: HeldInput {
                            look_x: dx / length,
                            look_z: dz / length,
                            ..HeldInput::default()
                        },
                        triggered: TriggeredInput {
                            attack: true,
                            ..TriggeredInput::default()
                        },
                    },
                ));
            }
        }
        if let Err(error) = world.step(&inputs) {
            self.simulation_error = Some(format!("{error:?}"));
            return;
        }
        self.ticks += 1;
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_none() {
            return;
        }
        let now = Instant::now();
        let elapsed = self
            .last
            .replace(now)
            .map_or(0.0, |then| now.duration_since(then).as_secs_f64());
        self.accumulator = (self.accumulator + elapsed).min(STEP * MAX_CATCHUP as f64);
        while self.accumulator >= STEP
            && self.simulation_error.is_none()
            && (!self.smoke || self.ticks < 210)
        {
            self.step();
            self.accumulator -= STEP;
        }
        let Some(live) = self.live.as_mut() else {
            return;
        };
        if let Some(error) = self.simulation_error.as_deref() {
            live.window.set_title(&format!(
                "Paredros crossing | simulation stopped, press 1/2 to restart ({error})"
            ));
        }
        let world = self.world.as_ref().expect("crossing world");
        let camera = crossing::camera(world, self.player, self.yaw, self.distance);
        live.tenant.look(camera.projection, camera.view);
        live.tenant.set_geometry_with_ambient(
            &crossing::geometry(world),
            &crossing::body_geometry(world),
            camera.eye,
            self.ambient,
        );
        let validation_scope = live.device.push_error_scope(wgpu::ErrorFilter::Validation);
        let report = live.tenant.draw();
        let internal_queue_submissions = report.internal_queue_submissions;
        assert_eq!(internal_queue_submissions, 0, "caller-owned tenant encoder");
        let chrome = live.hud.scene(world, self.player);
        let (master, composition) =
            live.composer
                .compose_opaque_tenant(&chrome, &live.tenant, report);
        let size = live.window.inner_size();
        use wgpu::CurrentSurfaceTexture as Acquired;
        match live.surface.get_current_texture() {
            Acquired::Success(frame) | Acquired::Suboptimal(frame) => {
                let target = frame.texture.create_view(&Default::default());
                live.composer.present(
                    &master,
                    &target,
                    live.format,
                    [size.width.max(1), size.height.max(1)],
                );
                live.window.pre_present_notify();
                live.queue.present(frame);
            },
            Acquired::Outdated | Acquired::Lost => configure(live),
            Acquired::Timeout | Acquired::Occluded => {},
            Acquired::Validation => eprintln!("crossing surface acquisition validation error"),
        }
        let validation_error = pollster::block_on(validation_scope.pop());
        if self.smoke {
            assert!(
                validation_error.is_none(),
                "crossing smoke validation error: {validation_error:?}"
            );
        }
        if self.smoke && self.ticks >= 210 && !self.smoke_done {
            self.smoke_done = true;
            let final_position = world.body(self.player).expect("player body").position;
            assert_ne!(
                final_position, self.initial_position,
                "smoke walk did not move the body"
            );
            let saved = world.save().expect("crossing smoke save");
            let restored = crossing::ContactWorld::restore(&saved).expect("crossing smoke restore");
            assert_eq!(restored, *world, "replay must recover the resolved world");
            assert_eq!(restored.save().expect("crossing restored save"), saved);
            let root = env::var_os("PAREDROS_CROSSING_OUTPUT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from(r"C:\Users\mark_\Code\testing\paredros\crossing"));
            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_millis();
            let output = root.join(format!("run-{stamp}"));
            let capture = live.composer.capture(&master);
            assert!(
                !capture.is_trivial(),
                "crossing smoke rendered a trivial frame"
            );
            capture
                .write_png(&output.join("crossing.png"))
                .expect("crossing capture");
            let receipt = serde_json::json!({"fixture":"damaged-crossing","body":format!("{:?}",self.body),"ticks":self.ticks,"moved":true,"start_position":self.initial_position,"end_position":final_position,"recorded_inputs":world.recorded_inputs().len(),"graph_encoder_batches":composition.graph_encoder_batches,"graph_submission_boundaries":composition.graph_submission_boundaries,"internal_queue_submissions":internal_queue_submissions,"replay_equal":true,"validation_error":false,"capture":output.join("crossing.png"),"distinct_colours":capture.distinct,"manual_play":false});
            std::fs::create_dir_all(&output).expect("crossing output directory");
            std::fs::write(
                output.join("crossing.json"),
                serde_json::to_vec_pretty(&receipt).unwrap(),
            )
            .expect("crossing receipt");
            event_loop.exit();
        } else {
            live.window.request_redraw();
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Paredros: Damaged Crossing")
                        .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1])),
                )
                .expect("window"),
        );
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("surface");
        let handles = gpu::boot(&self.instance, Some(&surface));
        let caps = surface.get_capabilities(&handles.adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let mut live = Live {
            window,
            surface,
            format,
            device: handles.device.clone(),
            queue: handles.queue.clone(),
            tenant: Tenant::new(&handles, SIZE),
            composer: Composer::new(handles, SIZE),
            hud: crossing::Hud::new().expect("crossing font"),
        };
        configure(&mut live);
        live.window.request_redraw();
        self.live = Some(live);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(live) = self.live.as_mut() {
                    configure(live)
                }
            },
            WindowEvent::Focused(false) => {
                self.input.clear();
                self.last_cursor = None;
                self.dragging = false;
            },
            WindowEvent::KeyboardInput { event, .. } => self.key(event, event_loop),
            WindowEvent::MouseWheel { delta, .. } => {
                self.distance = (self.distance - delta_y(delta)).clamp(3.0, 18.0);
            },
            WindowEvent::CursorMoved { position, .. } => {
                if self.dragging {
                    if let Some(previous) = self.last_cursor {
                        self.yaw += (position.x - previous.x) as f32 * 0.005;
                    }
                    self.last_cursor = Some(position);
                }
            },
            WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => self.input.triggered.interact = true,
                (ElementState::Pressed, MouseButton::Right) => {
                    self.dragging = true;
                    self.last_cursor = None;
                },
                (ElementState::Released, MouseButton::Right) => {
                    self.dragging = false;
                    self.last_cursor = None;
                },
                _ => {},
            },
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {},
        }
    }
}

impl App {
    fn key(&mut self, event: KeyEvent, event_loop: &ActiveEventLoop) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        if event.state == ElementState::Pressed && code == KeyCode::Escape {
            event_loop.exit();
            return;
        }
        if event.state == ElementState::Pressed {
            self.input.keys.insert(code);
        } else {
            self.input.keys.remove(&code);
            if matches!(
                code,
                KeyCode::KeyW | KeyCode::KeyA | KeyCode::KeyS | KeyCode::KeyD
            ) {
                self.input.previous_held = self.input.held;
                self.input.release_latch = true;
            }
        }
        self.input.held.move_x = axis(&self.input.keys, KeyCode::KeyW, KeyCode::KeyS);
        self.input.held.move_z = axis(&self.input.keys, KeyCode::KeyD, KeyCode::KeyA);
        self.input.held.brace = self.input.keys.contains(&KeyCode::ShiftLeft)
            || self.input.keys.contains(&KeyCode::ShiftRight);
        if event.state == ElementState::Pressed && !event.repeat {
            match code {
                KeyCode::KeyE => self.input.triggered.interact = true,
                KeyCode::KeyF => self.input.practice_strike = true,
                KeyCode::KeyQ | KeyCode::KeyT => self.input.triggered.anchor = true,
                KeyCode::Space => self.input.triggered.attack = true,
                KeyCode::KeyR => self.input.triggered.recover = true,
                KeyCode::Digit1 => self.reset(BodyKind::Crawler),
                KeyCode::Digit2 => self.reset(BodyKind::Climber),
                KeyCode::ArrowLeft => self.yaw -= 0.12,
                KeyCode::ArrowRight => self.yaw += 0.12,
                _ => {},
            }
        }
    }
}

fn axis(keys: &HashSet<KeyCode>, positive: KeyCode, negative: KeyCode) -> f32 {
    keys.contains(&positive) as i8 as f32 - keys.contains(&negative) as i8 as f32
}
fn rotate_axes(forward: f32, right: f32, yaw: f32) -> (f32, f32) {
    let (sin, cos) = yaw.sin_cos();
    let scale = forward.hypot(right).max(1.0);
    (
        (forward * cos - right * sin) / scale,
        (forward * sin + right * cos) / scale,
    )
}
fn delta_y(delta: winit::event::MouseScrollDelta) -> f32 {
    match delta {
        winit::event::MouseScrollDelta::LineDelta(_, y) => y,
        winit::event::MouseScrollDelta::PixelDelta(p) => p.y as f32 / 40.0,
    }
}
fn configure(live: &mut Live) {
    let s = live.window.inner_size();
    live.surface.configure(
        &live.device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: live.format,
            width: s.width.max(1),
            height: s.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            color_space: wgpu::SurfaceColorSpace::Auto,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
}

trait InputTake {
    fn take(&mut self) -> Input;
    fn clear(&mut self);
}
impl InputTake for InputState {
    fn take(&mut self) -> Input {
        let held = if self.held.move_x == 0. && self.held.move_z == 0. && self.release_latch {
            HeldInput {
                move_x: self.previous_held.move_x,
                move_z: self.previous_held.move_z,
                ..self.held
            }
        } else {
            self.held
        };
        self.previous_held = held;
        self.release_latch = false;
        Input {
            held,
            triggered: std::mem::take(&mut self.triggered),
        }
    }
    fn clear(&mut self) {
        self.keys.clear();
        self.held = HeldInput::default();
        self.triggered = TriggeredInput::default();
        self.practice_strike = false;
        self.previous_held = HeldInput::default();
        self.release_latch = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_axes_at_zero_yaw() {
        assert_eq!(rotate_axes(1.0, 0.0, 0.0), (1.0, 0.0));
        assert_eq!(rotate_axes(0.0, 1.0, 0.0), (0.0, 1.0));
    }

    #[test]
    fn camera_axes_at_quarter_turn() {
        let (x, z) = rotate_axes(1.0, 0.0, std::f32::consts::FRAC_PI_2);
        assert!(x.abs() < 1e-6 && (z - 1.0).abs() < 1e-6);
        let (x, z) = rotate_axes(0.0, 1.0, std::f32::consts::FRAC_PI_2);
        assert!((x + 1.0).abs() < 1e-6 && z.abs() < 1e-6);
    }

    #[test]
    fn taking_input_clears_triggers_but_preserves_held() {
        let mut state = InputState::default();
        state.held.move_x = 1.0;
        state.triggered.attack = true;
        let first = state.take();
        let second = state.take();
        assert!(first.triggered.attack && !second.triggered.attack);
        assert_eq!(second.held.move_x, 1.0);
    }

    #[test]
    fn quick_release_latches_one_movement_tick() {
        let mut state = InputState::default();
        state.held.move_x = 1.0;
        state.previous_held = state.held;
        state.held = HeldInput::default();
        state.release_latch = true;
        assert_eq!(state.take().held.move_x, 1.0);
        assert_eq!(state.take().held.move_x, 0.0);
    }

    #[test]
    fn focus_clear_releases_everything() {
        let mut state = InputState::default();
        state.keys.insert(KeyCode::KeyW);
        state.held.brace = true;
        state.triggered.interact = true;
        state.clear();
        assert!(state.keys.is_empty());
        assert_eq!(state.held, HeldInput::default());
        assert_eq!(state.triggered, TriggeredInput::default());
    }
}
