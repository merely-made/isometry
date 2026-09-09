// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native live equipment host with a separate authored comparison mode.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use netrender::{Compositor, PresentedFrame, Scene, SurfaceKey};
use paredros_room::body_sheet::{EquipmentCommand, EquipmentSession, EquipmentView};
use paredros_room::body_sheet::{Hud, LOGICAL_SIZE, LifeSheet, SheetKey, SheetView};
use paredros_room::gpu::{self, Composer, MASTER_FORMAT};
use paredros_world::fixtures::three_lives as fixture;
use paredros_world::{SubjectSheet, SubjectSheetInput};
use winit::application::ApplicationHandler;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyEvent, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};
#[path = "body_sheet/persistence_smoke.rs"]
mod persistence_smoke;

fn main() {
    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    event_loop
        .run_app(&mut App::new(
            env::var_os("PAREDROS_BODY_SHEET_SMOKE").is_some(),
        ))
        .expect("body sheet host");
}

struct App {
    instance: wgpu::Instance,
    lives: Vec<LifeSheet>,
    view: SheetView,
    live: Option<Live>,
    smoke: bool,
    captured: usize,
    cursor: Option<PhysicalPosition<f64>>,
    capture_root: PathBuf,
    equipment: EquipmentSession,
    equipment_view: EquipmentView,
    comparison: bool,
    equipment_smoke: bool,
    save_directory: PathBuf,
    persistence_smoke: bool,
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
    fn new(smoke: bool) -> Self {
        let mut view = SheetView::default();
        if smoke {
            view.focus = paredros_room::body_sheet::Focus::Action;
            view.action = 2;
        }
        let mut app = Self {
            instance: wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            ),
            lives: scenario(),
            view,
            live: None,
            smoke,
            captured: 0,
            equipment: EquipmentSession::new().expect("equipment session"),
            equipment_view: EquipmentView::default(),
            comparison: smoke && env::var_os("PAREDROS_EQUIPMENT_SMOKE").is_none(),
            equipment_smoke: env::var_os("PAREDROS_EQUIPMENT_SMOKE").is_some(),
            persistence_smoke: env::var_os("PAREDROS_EQUIPMENT_PERSIST_SMOKE").is_some(),
            save_directory: env::var_os("PAREDROS_EQUIPMENT_SAVES")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    env::var_os("LOCALAPPDATA")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("saves"))
                        .join("Merely/Paredros/equipment")
                }),
            cursor: None,
            capture_root: env::var_os("PAREDROS_BODY_SHEET_OUTPUT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("testing/body_sheet"))
                .join(format!(
                    "run-{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("clock")
                        .as_millis()
                )),
        };
        if app.persistence_smoke {
            persistence_smoke::prepare(&mut app);
        }
        app
    }
    fn redraw(&self) {
        if let Some(live) = &self.live {
            live.window.request_redraw();
        }
    }
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let scene = if self.comparison {
            live.hud.scene(&self.lives, &self.view)
        } else {
            live.hud.equipment_scene(
                &self.equipment.sheet(),
                &self.equipment.items(),
                &self.equipment_view,
                self.equipment.status(),
            )
        };
        let master = render(&live.composer, &scene);
        match live.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame)
            | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
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
            },
            wgpu::CurrentSurfaceTexture::Outdated | wgpu::CurrentSurfaceTexture::Lost => {
                configure(live)
            },
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {},
            wgpu::CurrentSurfaceTexture::Validation => {
                eprintln!("body sheet surface acquisition validation error")
            },
        }
        if self.persistence_smoke {
            persistence_smoke::capture(live, &master, &self.capture_root);
            event_loop.exit();
        } else if self.equipment_smoke && !self.comparison {
            let capture = live.composer.capture(&master);
            assert!(
                !capture.is_trivial(),
                "equipment smoke rendered a trivial frame"
            );
            let name = [
                "equipment_carried",
                "equipment_attached",
                "equipment_detached",
            ][self.captured];
            let path = self.capture_root.join(format!("{name}.png"));
            capture.write_png(&path).expect("equipment capture");
            println!("Equipment capture: {}", path.display());
            self.captured += 1;
            if self.captured == 1 {
                self.equipment_view.part = 1;
                self.equipment_command(EquipmentCommand::Attach);
                assert_eq!(
                    self.equipment
                        .game()
                        .attachments(self.equipment.subject())
                        .len(),
                    1
                );
            } else if self.captured == 2 {
                self.equipment_command(EquipmentCommand::Detach);
                assert!(
                    self.equipment
                        .game()
                        .attachments(self.equipment.subject())
                        .is_empty()
                );
            } else {
                let game = self.equipment.game();
                assert_eq!(
                    paredros_world::GameState::restore(&game.save().unwrap()).unwrap(),
                    *game
                );
                event_loop.exit();
            }
            self.redraw();
        } else if self.smoke {
            let capture = live.composer.capture(&master);
            assert!(
                !capture.is_trivial(),
                "body sheet smoke rendered a trivial frame"
            );
            let name = [
                "sedge_harness",
                "tremor_untrained",
                "mend_lost_limb",
                "mend_symbiont",
                "mend_adhesion",
                "mend_parts_list",
            ][self.captured];
            let path = self.capture_root.join(format!("{name}.png"));
            capture.write_png(&path).expect("body sheet capture");
            println!("Body sheet capture: {}", path.display());
            self.captured += 1;
            if self.captured < 6 {
                self.view.select_life(&self.lives, self.captured.min(2));
                if self.captured == 1 {
                    self.view.focus = paredros_room::body_sheet::Focus::Action;
                    self.view.action = 0;
                } else if self.captured == 2 {
                    self.view.part = 1;
                } else if self.captured == 3 {
                    self.view.part = self.lives[2]
                        .sheet
                        .parts
                        .iter()
                        .position(|part| part.id.0 == 7)
                        .expect("fixture symbiont");
                } else if self.captured == 4 {
                    self.view.focus = paredros_room::body_sheet::Focus::Action;
                    self.view.action = 1;
                } else {
                    self.view.key(&self.lives, SheetKey::TogglePartsView);
                }
                self.redraw();
            } else {
                event_loop.exit();
            }
        }
    }
    fn key(&mut self, event: KeyEvent, loop_: &ActiveEventLoop) {
        if event.state != ElementState::Pressed || event.repeat {
            return;
        }
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        if code == KeyCode::KeyC {
            self.comparison = !self.comparison;
            self.redraw();
            return;
        }
        if !self.comparison {
            match code {
                KeyCode::Escape => loop_.exit(),
                KeyCode::KeyE => self.equipment_command(EquipmentCommand::Attach),
                KeyCode::KeyD => self.equipment_command(EquipmentCommand::Detach),
                KeyCode::F5 => self.equipment_command(EquipmentCommand::Save),
                KeyCode::F9 => self.equipment_command(EquipmentCommand::Load),
                KeyCode::KeyL => self.equipment_view.parts_view = !self.equipment_view.parts_view,
                KeyCode::ArrowUp => self.equipment_view.step_part(&self.equipment.sheet(), -1),
                KeyCode::ArrowDown => self.equipment_view.step_part(&self.equipment.sheet(), 1),
                KeyCode::ArrowLeft => self.equipment_view.step_item(&self.equipment.items(), -1),
                KeyCode::ArrowRight => self.equipment_view.step_item(&self.equipment.items(), 1),
                _ => return,
            }
            self.redraw();
            return;
        }
        match code {
            KeyCode::Escape => loop_.exit(),
            KeyCode::Digit1 => self.view.select_life(&self.lives, 0),
            KeyCode::Digit2 => self.view.select_life(&self.lives, 1),
            KeyCode::Digit3 => self.view.select_life(&self.lives, 2),
            KeyCode::Tab => self.view.key(&self.lives, SheetKey::SwitchFocus),
            KeyCode::KeyL => self.view.key(&self.lives, SheetKey::TogglePartsView),
            KeyCode::ArrowUp => self.view.key(&self.lives, SheetKey::Up),
            KeyCode::ArrowDown => self.view.key(&self.lives, SheetKey::Down),
            KeyCode::PageUp => self.view.scroll_detail_by(&self.lives, -12),
            KeyCode::PageDown => self.view.scroll_detail_by(&self.lives, 12),
            _ => return,
        };
        self.redraw();
    }
    fn equipment_command(&mut self, command: EquipmentCommand) {
        if matches!(command, EquipmentCommand::Save | EquipmentCommand::Load) {
            let result = match command {
                EquipmentCommand::Save => self
                    .equipment
                    .save_bytes()
                    .and_then(|bytes| {
                        paredros_room::body_sheet::save_equipment(&self.save_directory, &bytes)
                    })
                    .map(|path| format!("Saved {}", path.display())),
                EquipmentCommand::Load => paredros_room::body_sheet::load_equipment(
                    &self.save_directory,
                )
                .and_then(|(path, bytes)| {
                    self.equipment.load_bytes(&bytes)?;
                    self.equipment_view = EquipmentView::default();
                    Ok(format!("Loaded {}", path.display()))
                }),
                _ => unreachable!(),
            };
            self.equipment
                .set_status(result.unwrap_or_else(|error| format!("{command:?} failed: {error}")));
            return;
        }
        let items = self.equipment.items();
        let Some(item) = items.get(self.equipment_view.item) else {
            return;
        };
        match command {
            EquipmentCommand::Attach => {
                let sheet = self.equipment.sheet();
                if let Some(part) = sheet.parts.get(self.equipment_view.part) {
                    self.equipment.attach(item.id, part.id);
                }
            },
            EquipmentCommand::Detach => self.equipment.detach(item.id),
            EquipmentCommand::Save | EquipmentCommand::Load => unreachable!(),
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, loop_: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let window = Arc::new(
            loop_
                .create_window(
                    Window::default_attributes()
                        .with_title("Paredros: Body + Actions")
                        .with_inner_size(PhysicalSize::new(LOGICAL_SIZE[0], LOGICAL_SIZE[1])),
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
            composer: Composer::new(handles, LOGICAL_SIZE),
            hud: Hud::new().expect("body sheet font"),
        };
        configure(&mut live);
        live.window.request_redraw();
        self.live = Some(live);
    }
    fn window_event(&mut self, loop_: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => loop_.exit(),
            WindowEvent::Resized(_) => {
                if let Some(live) = self.live.as_mut() {
                    configure(live);
                }
                self.redraw();
            },
            WindowEvent::KeyboardInput { event, .. } => self.key(event, loop_),
            WindowEvent::CursorMoved { position, .. } => self.cursor = Some(position),
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                if let (Some(cursor), Some(live)) = (self.cursor, self.live.as_ref()) {
                    let point = logical_point(cursor, live.window.inner_size());
                    if self.comparison {
                        self.view.click(&self.lives, point);
                    } else if let Some(command) = self.equipment_view.click(
                        &self.equipment.sheet(),
                        &self.equipment.items(),
                        point,
                    ) {
                        self.equipment_command(command);
                    }
                    self.redraw();
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
                if !self.comparison {
                    return;
                }
                let detail = self
                    .cursor
                    .zip(self.live.as_ref())
                    .is_some_and(|(point, live)| {
                        logical_point(point, live.window.inner_size())[0] >= 708.
                    });
                if detail {
                    self.view
                        .scroll_detail_by(&self.lives, wheel(delta).round() as i32);
                } else {
                    self.view.scroll_by(&self.lives, wheel(delta) * 34.);
                }
                self.redraw();
            },
            WindowEvent::RedrawRequested => self.frame(loop_),
            _ => {},
        }
    }
}

fn scenario() -> Vec<LifeSheet> {
    fixture::three_lives()
        .into_iter()
        .map(|life| {
            let knowledge = fixture::knowledge(&life);
            let inputs = fixture::inputs(&life);
            LifeSheet {
                label: life.name,
                summary: life.ordinary_task,
                sheet: SubjectSheet::from_input(SubjectSheetInput {
                    subject: life.subject,
                    revision: life.revision,
                    current_revision: life.revision,
                    body: &life.body,
                    knowledge: &knowledge,
                    inputs: &inputs,
                    part_names: fixture::PART_NAMES,
                    selected_part: None,
                }),
            }
        })
        .collect()
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
fn wheel(delta: MouseScrollDelta) -> f32 {
    match delta {
        MouseScrollDelta::LineDelta(_, y) => -y,
        MouseScrollDelta::PixelDelta(point) => -(point.y as f32 / 30.),
    }
}
pub fn logical_point(point: PhysicalPosition<f64>, size: PhysicalSize<u32>) -> [f32; 2] {
    [
        point.x as f32 * LOGICAL_SIZE[0] as f32 / size.width.max(1) as f32,
        point.y as f32 * LOGICAL_SIZE[1] as f32 / size.height.max(1) as f32,
    ]
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
        MASTER_FORMAT,
        &mut grab,
        netrender::peniko::Color::new([0., 0., 0., 1.]),
    );
    grab.master.expect("netrender presented no master")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_load_restores_saved_items_and_failed_load_preserves_selection() {
        let mut app = App::new(false);
        app.save_directory = env::temp_dir().join(format!(
            "paredros-host-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        app.equipment_view.part = 2;
        app.equipment_command(EquipmentCommand::Attach);
        let saved = app.equipment.game().clone();
        app.equipment_command(EquipmentCommand::Save);
        assert!(app.equipment.status().starts_with("Saved "));
        app.equipment_command(EquipmentCommand::Detach);
        app.equipment_command(EquipmentCommand::Load);
        assert_eq!(app.equipment.game(), &saved);
        assert_eq!(app.equipment_view, EquipmentView::default());
        app.equipment_view.part = 3;
        let selected = app.equipment_view.clone();
        let corrupt =
            paredros_room::body_sheet::save_equipment(&app.save_directory, b"bad").unwrap();
        app.equipment_command(EquipmentCommand::Load);
        assert_eq!(app.equipment.game(), &saved);
        assert_eq!(app.equipment_view, selected);
        assert!(app.equipment.status().starts_with("Load failed:"));
        std::fs::remove_file(corrupt).unwrap();
        for entry in std::fs::read_dir(&app.save_directory).unwrap() {
            std::fs::remove_file(entry.unwrap().path()).unwrap();
        }
        std::fs::remove_dir(&app.save_directory).unwrap();
    }

    #[test]
    fn host_dispatches_selected_ids_and_comparison_does_not_change_the_live_subject() {
        let mut app = App::new(false);
        app.equipment_view.part = 2;
        app.equipment_view.item = 1;
        let subject = app.equipment.subject();
        let part = app.equipment.sheet().parts[2].id;
        let item = app.equipment.items()[1].id;
        app.equipment_command(EquipmentCommand::Attach);
        let attachment = app.equipment.game().attachments(subject)[0];
        assert_eq!(attachment.item, item);
        assert_eq!(attachment.part, part);
        let before = app.equipment.game().clone();
        app.comparison = true;
        app.view.select_life(&app.lives, 2);
        assert_eq!(app.equipment.game(), &before);
        app.comparison = false;
        app.equipment_command(EquipmentCommand::Detach);
        assert!(app.equipment.game().attachments(subject).is_empty());
        assert_eq!(app.equipment.subject(), subject);
        let game = app.equipment.game();
        assert_eq!(
            paredros_world::GameState::restore(&game.save().unwrap()).unwrap(),
            *game
        );
    }

    #[test]
    fn logical_points_scale_and_guard_zero() {
        assert_eq!(
            logical_point(
                PhysicalPosition::new(320., 180.),
                PhysicalSize::new(640, 360)
            ),
            [640., 360.]
        );
        assert_eq!(
            logical_point(
                PhysicalPosition::new(640., 360.),
                PhysicalSize::new(1280, 720)
            ),
            [640., 360.]
        );
        assert!(
            logical_point(PhysicalPosition::new(1., 1.), PhysicalSize::new(0, 0))[0].is_finite()
        );
    }
}
