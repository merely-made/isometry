// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native, read-only host for the authored three-lives Body/Actions sheet.

use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use netrender::{Compositor, PresentedFrame, Scene, SurfaceKey};
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
        Self {
            instance: wgpu::Instance::new(
                wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
            ),
            lives: scenario(),
            view,
            live: None,
            smoke,
            captured: 0,
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
        }
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
        let scene = live.hud.scene(&self.lives, &self.view);
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
        if self.smoke {
            let capture = live.composer.capture(&master);
            assert!(
                !capture.is_trivial(),
                "body sheet smoke rendered a trivial frame"
            );
            let name = self.lives[self.view.life]
                .label
                .replace(' ', "_")
                .to_lowercase();
            let path = self.capture_root.join(format!("{name}.png"));
            capture.write_png(&path).expect("body sheet capture");
            println!("Body sheet capture: {}", path.display());
            self.captured += 1;
            if self.captured < self.lives.len() {
                self.view.select_life(&self.lives, self.captured);
                if self.captured == 1 {
                    self.view.focus = paredros_room::body_sheet::Focus::Action;
                    self.view.action = 0;
                } else {
                    self.view.part = 1;
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
        match code {
            KeyCode::Escape => loop_.exit(),
            KeyCode::Digit1 => self.view.select_life(&self.lives, 0),
            KeyCode::Digit2 => self.view.select_life(&self.lives, 1),
            KeyCode::Digit3 => self.view.select_life(&self.lives, 2),
            KeyCode::Tab => self.view.key(&self.lives, SheetKey::SwitchFocus),
            KeyCode::ArrowUp => self.view.key(&self.lives, SheetKey::Up),
            KeyCode::ArrowDown => self.view.key(&self.lives, SheetKey::Down),
            KeyCode::PageUp => self.view.scroll_detail_by(&self.lives, -12),
            KeyCode::PageDown => self.view.scroll_detail_by(&self.lives, 12),
            _ => return,
        };
        self.redraw();
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
                    self.view
                        .click(&self.lives, logical_point(cursor, live.window.inner_size()));
                    self.redraw();
                }
            },
            WindowEvent::MouseWheel { delta, .. } => {
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
    fn logical_points_scale_and_guard_zero() {
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
