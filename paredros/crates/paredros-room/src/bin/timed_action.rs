// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native input receipt for Paredros's bounded timed limb action.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use netrender::{Compositor, PresentedFrame, Scene, SurfaceKey};
use paredros_identity::Tick;
use paredros_room::body_sheet::{Hud, LOGICAL_SIZE};
use paredros_room::gpu::{self, Composer};
use paredros_world::fixtures::three_lives;
use paredros_world::timed_action::{Direction, TimedActionRules, TimedActionSession};
use paredros_world::{GameIntent, GameState, Session, World, WorldConfig};
use wing_functions::{
    Edge, FunctionalNetwork, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

fn main() {
    let event_loop = EventLoop::new().expect("event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new();
    event_loop.run_app(&mut app).expect("timed action host");
    if app.smoke {
        assert!(app.presented, "smoke ended without a presented frame");
        println!("timed action: lifecycle assertions and presented frame passed");
    }
}

struct App {
    instance: wgpu::Instance,
    live: Option<Live>,
    action: TimedActionSession,
    held: bool,
    last_charge: Instant,
    status: Vec<String>,
    smoke: bool,
    presented: bool,
    opened_at: Instant,
    save_path: PathBuf,
}
struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    composer: Composer,
    hud: Hud,
}

impl App {
    fn new() -> Self {
        let subject = three_lives::KEEPER;
        let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
        let at = game
            .items()
            .all()
            .find_map(|item| match item.location {
                paredros_world::ItemLocation::At(at) => Some(at),
                _ => None,
            })
            .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::Generate {
            tick,
            subject,
            body_seed: 1,
            at,
        })
        .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::Name {
            tick,
            subject,
            name: paredros_world::Name::new("Keeper").unwrap(),
        })
        .unwrap();
        let tick = game.next_tick();
        game.apply(GameIntent::AdmitAnatomy {
            tick,
            subject,
            revision: paredros_identity::BodyRevisionId(0),
            document: Box::new(three_lives::three_lives()[0].body.clone()),
        })
        .unwrap();
        let session = Session::begin(game, subject).unwrap();
        let part = PartRef {
            subject: subject.0,
            part: 1,
        };
        let part_two = PartRef {
            subject: subject.0,
            part: 2,
        };
        let network = FunctionalNetwork::new(
            vec![
                Node {
                    id: NodeId(1),
                    kind: NodeKind::Source {
                        part,
                        capacity: 24,
                        charge: 24,
                    },
                },
                Node {
                    id: NodeId(2),
                    kind: NodeKind::Effect { part },
                },
                Node {
                    id: NodeId(3),
                    kind: NodeKind::Source {
                        part: part_two,
                        capacity: 24,
                        charge: 24,
                    },
                },
                Node {
                    id: NodeId(4),
                    kind: NodeKind::Effect { part: part_two },
                },
            ],
            vec![
                Edge {
                    from: NodeId(1),
                    to: NodeId(2),
                    capacity: 24,
                },
                Edge {
                    from: NodeId(3),
                    to: NodeId(4),
                    capacity: 24,
                },
            ],
        )
        .unwrap();
        let rules = TimedActionRules {
            max_contributors: 8,
            max_elapsed_ticks: 60,
            charge_per_tick: 2,
            max_charge_per_limb: 12,
            evaluation: WorldRules {
                allowed_operators: [Operator::Strengthen].into_iter().collect(),
                allowed_costs: [1, 2, 3, 5, 8, 12].into_iter().collect(),
                max_range: 0,
                max_hops: 4,
            },
        };
        let path = env::var_os("PAREDROS_TIMED_ACTION_SAVE")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("timed_action.save"));
        let mut app = Self {
            instance: wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            ),
            live: None,
            action: TimedActionSession::begin(session, network, rules).unwrap(),
            held: false,
            last_charge: Instant::now(),
            status: vec!["Ready: choose a direction, then hold Space or left mouse.".into()],
            smoke: env::var_os("PAREDROS_TIMED_ACTION_SMOKE").is_some(),
            presented: false,
            opened_at: Instant::now(),
            save_path: path,
        };
        if app.smoke {
            app.smoke_sequence();
        }
        app
    }
    fn smoke_sequence(&mut self) {
        assert!(self.action.prepare(Direction::Forward).is_ok());
        assert!(self.action.join(mesocosm_core::PartId(2)).is_ok());
        let start = self.action.action().unwrap().last_tick;
        assert_eq!(self.action.charge(Tick(start.0 + 1)).unwrap().len(), 2);
        assert_eq!(self.action.charge(Tick(start.0 + 2)).unwrap().len(), 2);
        self.injure();
        let action = self.action.action().expect("injury preserves action");
        assert_eq!(
            action.contributors[&mesocosm_core::PartId(1)].state,
            paredros_world::timed_action::ContributionState::Cancelled
        );
        assert!(action.contributors[&mesocosm_core::PartId(2)].charge > 0);
        let before = self.action.save().unwrap();
        self.save();
        assert!(
            self.status
                .first()
                .is_some_and(|line| line.starts_with("Saved "))
        );
        self.held = true;
        self.load();
        assert!(
            self.status
                .first()
                .is_some_and(|line| line.starts_with("Loaded "))
        );
        assert!(!self.held, "load clears held native input");
        assert_eq!(
            self.action.save().unwrap(),
            before,
            "load restores exact timed state"
        );
        let tick = self.action.action().unwrap().last_tick;
        let receipts = self.action.release(tick).unwrap();
        assert_eq!(receipts.len(), 1, "surviving limb releases one receipt");
        self.status = vec![format!(
            "Smoke: released {} surviving typed strike receipt.",
            receipts.len()
        )];
    }
    fn tick(&mut self) {
        if !self.held || self.last_charge.elapsed() < Duration::from_millis(100) {
            return;
        }
        let tick = self
            .action
            .action()
            .map(|a| Tick(a.last_tick.0 + 1))
            .unwrap_or_else(|| self.action.session().game().next_tick());
        match self.action.charge(tick) {
            Ok(outcomes) => {
                self.status = vec![
                    format!("Charging tick {}: {:?}", tick.0, outcomes),
                    format!(
                        "Contributors: {}",
                        self.action.action().map_or(0, |a| a.contributors.len())
                    ),
                ]
            },
            Err(e) => self.status = vec![format!("Charge paused: {e:?}")],
        }
        self.last_charge = Instant::now();
        self.redraw();
    }
    fn dispatch(&mut self, code: KeyCode, pressed: bool, loop_: &ActiveEventLoop) {
        if !pressed {
            if code == KeyCode::Space {
                self.release();
            }
            return;
        }
        match code {
            KeyCode::Escape => loop_.exit(),
            KeyCode::ArrowUp => self.prepare(Direction::Forward),
            KeyCode::ArrowDown => self.prepare(Direction::Backward),
            KeyCode::ArrowLeft => self.prepare(Direction::Left),
            KeyCode::ArrowRight => self.prepare(Direction::Right),
            KeyCode::Space => {
                if self.action.action().is_none() {
                    self.prepare(Direction::Forward);
                }
                self.held = true;
                self.last_charge = Instant::now();
            },
            KeyCode::KeyJ => self.join_limb(),
            KeyCode::KeyI => self.injure(),
            KeyCode::F5 => self.save(),
            KeyCode::F9 => self.load(),
            _ => return,
        }
        self.redraw();
    }
    fn prepare(&mut self, direction: Direction) {
        self.status = match self.action.prepare(direction) {
            Ok(action) => vec![
                format!(
                    "Prepared {:?} at tick {}",
                    action.direction, action.started_at.0
                ),
                format!("Joined limbs: {}", action.contributors.len()),
            ],
            Err(e) => vec![format!("Prepare failed: {e:?}")],
        };
    }
    fn join_limb(&mut self) {
        self.status = match self.action.join(mesocosm_core::PartId(2)) {
            Ok(()) => vec!["Joined limb part 2 into authoritative action.".into()],
            Err(e) => vec![format!("Limb join failed: {e:?}")],
        };
    }
    fn release(&mut self) {
        self.held = false;
        if self.action.action().is_none() {
            return;
        }
        let tick = self.action.action().map(|a| a.last_tick).unwrap();
        self.status = match self.action.release(tick) {
            Ok(receipts) => vec![
                format!("Released {} typed strike receipt(s)", receipts.len()),
                format!("{receipts:?}"),
            ],
            Err(e) => vec![format!("Release failed: {e:?}")],
        };
        self.redraw();
    }
    fn injure(&mut self) {
        let subject = self.action.session().control().played();
        let old = self
            .action
            .session()
            .game()
            .bodies()
            .get(subject)
            .unwrap()
            .revision;
        let first = self.action.session().game().next_tick();
        let second = Tick(first.0 + 1);
        let next = paredros_identity::BodyRevisionId(old.0 + 1);
        let result = self.action.apply_game_batch(&[
            GameIntent::Fall {
                tick: first,
                subject,
                distance: 5,
            },
            GameIntent::ReconcileAnatomy {
                tick: second,
                subject,
                from_revision: old,
                revision: next,
                severed_parts: vec![mesocosm_core::PartId(1)],
            },
        ]);
        self.status = vec![format!(
            "Injury cut: {}",
            result
                .map(|events| format!(
                    "{} events; part 1 removed, surviving contributors retained",
                    events.len()
                ))
                .unwrap_or_else(|e| format!("{e:?}"))
        )];
        self.redraw();
    }
    fn save(&mut self) {
        self.status = match self
            .action
            .save()
            .map_err(|e| format!("{e:?}"))
            .and_then(|bytes| {
                let pending = self.save_path.with_extension("pending");
                std::fs::write(&pending, bytes).map_err(|e| e.to_string())?;
                std::fs::rename(&pending, &self.save_path).map_err(|e| e.to_string())
            }) {
            Ok(()) => vec![format!("Saved {}", self.save_path.display())],
            Err(e) => vec![format!("Save failed: {e}")],
        };
        self.redraw();
    }
    fn load(&mut self) {
        self.status = match std::fs::read(&self.save_path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| TimedActionSession::restore(&bytes).map_err(|e| format!("{e:?}")))
        {
            Ok(action) => {
                self.action = action;
                self.held = false;
                self.last_charge = Instant::now();
                vec![format!("Loaded {}", self.save_path.display())]
            },
            Err(e) => vec![format!("Load failed: {e}")],
        };
        self.redraw();
    }
    fn redraw(&self) {
        if let Some(live) = &self.live {
            live.window.request_redraw();
        }
    }
    fn display_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "Arrows: prepare direction | Hold Space or left mouse: charge | Release: strike".into(),
            "J: join second limb | I: injure first limb | F5: save | F9: load | Esc: close".into(),
        ];
        lines.extend(self.status.iter().cloned());
        if let Some(action) = self.action.action() {
            lines.push(format!(
                "Action: {:?}  started={}  last_tick={}",
                action.direction, action.started_at.0, action.last_tick.0
            ));
            for (part, contribution) in &action.contributors {
                lines.push(format!(
                    "Part {}: {:?}, charge={}, node={}",
                    part.0, contribution.state, contribution.charge, contribution.node.0
                ));
            }
        } else {
            lines.push("Action: idle".into());
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_handler_lifecycle_keeps_authoritative_receipt() {
        let mut app = App::new();
        app.save_path = std::env::temp_dir().join("paredros-timed-action-handler-test.save");
        app.smoke_sequence();
        assert!(app.status[0].contains("surviving typed strike receipt"));
        let _ = std::fs::remove_file(app.save_path);
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let window = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title("Paredros: Timed Action")
                    .with_inner_size(PhysicalSize::new(LOGICAL_SIZE[0], LOGICAL_SIZE[1])),
            )
            .unwrap(),
        );
        let surface = self.instance.create_surface(window.clone()).unwrap();
        let handles = gpu::boot(&self.instance, Some(&surface));
        let caps = surface.get_capabilities(&handles.adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let mut live = Live {
            window,
            surface,
            format,
            device,
            queue,
            composer: Composer::new(handles, LOGICAL_SIZE),
            hud: Hud::new().unwrap(),
        };
        configure(&mut live);
        live.window.request_redraw();
        self.live = Some(live);
    }
    fn window_event(&mut self, el: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state,
                        repeat,
                        ..
                    },
                ..
            } if !repeat => self.dispatch(code, state == ElementState::Pressed, el),
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if state == ElementState::Pressed {
                    if self.action.action().is_none() {
                        self.prepare(Direction::Forward);
                    }
                    self.held = true;
                    self.last_charge = Instant::now();
                } else {
                    self.release();
                }
            },
            WindowEvent::Focused(false) => self.held = false,
            WindowEvent::Resized(_) => {
                if let Some(live) = self.live.as_mut() {
                    configure(live);
                }
            },
            WindowEvent::RedrawRequested => self.frame(el),
            WindowEvent::CloseRequested => el.exit(),
            _ => {},
        }
    }
    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        el.set_control_flow(if self.held || self.smoke {
            ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(100))
        } else {
            ControlFlow::Wait
        });
        if self.smoke && self.opened_at.elapsed() > Duration::from_secs(30) {
            panic!("native smoke did not present within 30 seconds");
        }
        self.tick();
    }
}
fn configure(live: &mut Live) {
    let size = live.window.inner_size();
    live.surface.configure(
        &live.device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: live.format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            color_space: wgpu::SurfaceColorSpace::Auto,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
}
struct Grab {
    master: Option<wgpu::Texture>,
}
impl Compositor for Grab {
    fn declare_surface(&mut self, _: SurfaceKey, _: [f32; 4]) {}
    fn destroy_surface(&mut self, _: SurfaceKey) {}
    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        self.master = Some(frame.master.clone());
    }
}
fn render(composer: &Composer, scene: &Scene) -> wgpu::Texture {
    let mut grab = Grab { master: None };
    composer.net.render_with_compositor(
        scene,
        paredros_room::gpu::MASTER_FORMAT,
        &mut grab,
        netrender::peniko::Color::new([0., 0., 0., 1.]),
    );
    grab.master.expect("netrender presented no master")
}
impl App {
    fn frame(&mut self, el: &ActiveEventLoop) {
        let lines = self.display_lines();
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let scene = live.hud.timed_scene("PAREDROS / TIMED ACTION", &lines);
        let master = render(&live.composer, &scene);
        if self.smoke {
            let capture = live.composer.capture(&master);
            assert!(!capture.is_trivial(), "timed action frame is trivial");
            let path = self.save_path.with_extension("png");
            capture.write_png(&path).expect("timed action capture");
            println!("timed action frame: {}", path.display());
        }
        if let wgpu::CurrentSurfaceTexture::Success(frame)
        | wgpu::CurrentSurfaceTexture::Suboptimal(frame) = live.surface.get_current_texture()
        {
            let target = frame.texture.create_view(&Default::default());
            let size = live.window.inner_size();
            live.composer.present(
                &master,
                &target,
                live.format,
                [size.width.max(1), size.height.max(1)],
            );
            live.window.pre_present_notify();
            live.queue.present(frame);
            self.presented = true;
            if self.smoke {
                el.exit();
            }
        }
    }
}
