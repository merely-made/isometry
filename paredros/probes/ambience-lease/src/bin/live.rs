// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The windowed run: the resident lane, presented.
//!
//! Closes the spatial compute plan's last P2 open. Everything the
//! offscreen probe proved is here in a window at vsync: the ambience
//! kernels advance a resident padded-3D buffer, an adapter dispatch
//! publishes into renderling's slab, renderling draws 20,000 lit motes,
//! netrender composes a chrome bar over them, and the swapchain
//! presents. The CPU's per-frame view of the cloud stays four bytes
//! wide, and the epoch check runs every frame exactly as it does
//! offscreen.
//!
//! `AMBIENCE_CAPTURE=1` drives a fixed number of frames, writes the
//! receipt, and exits; without it the window stays up so a human can
//! watch the cloud turn.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use ambience_lease::compose::{Composer, MASTER_FORMAT};
use ambience_lease::lease::Ambience;
use ambience_lease::tenant::{TRANSFORM_WORDS, Tenant};
use netrender::{Scene, TenantNeeds, WgpuHandles, boot_on};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const SIZE: [u32; 2] = [1280, 720];
const MOTES: u32 = 20_000;
const EXTENT: f32 = 120.0;
const CAPTURE_AFTER: u64 = 240;
/// The chrome bar's height, so the receipt can measure the scene under
/// it rather than the bar.
const CHROME_HEIGHT: u32 = 40;
const CAPTURE: &str = r"C:\Users\mark_\Code\testing\paredros\p2_live_ambience.png";

fn main() {
    let capture_mode = std::env::var("AMBIENCE_CAPTURE").is_ok_and(|v| v != "0");

    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = LiveApp {
        instance: wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        ),
        capture_mode,
        live: None,
        frames: 0,
        regrows: 0,
        worst: 0.0,
        captured: false,
    };
    event_loop.run_app(&mut app).expect("winit run");
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    handles: WgpuHandles,
    ambience: Ambience,
    tenant: Tenant,
    composer: Composer,
    base: u32,
}

struct LiveApp {
    instance: wgpu::Instance,
    capture_mode: bool,
    live: Option<Live>,
    frames: u64,
    regrows: u32,
    worst: f64,
    captured: bool,
}

impl ApplicationHandler for LiveApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("Conatus: the resident lane, live")
            .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1]));
        let window = Arc::new(event_loop.create_window(attributes).expect("window"));
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("surface");

        // One device for the kernels, renderling, and netrender, chosen
        // against the surface the window actually got: the picture and
        // its presentation cannot end up on different adapters.
        let handles = boot_on(
            self.instance.clone(),
            Some(&surface),
            &TenantNeeds {
                optional_features: wgpu::Features::INDIRECT_FIRST_INSTANCE
                    | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
                    | wgpu::Features::VERTEX_WRITABLE_STORAGE
                    | wgpu::Features::CLEAR_TEXTURE,
                label: Some("ambience live"),
                ..Default::default()
            },
        )
        .expect("one device for kernels, renderling, and netrender");
        println!("adapter: {}", handles.adapter.get_info().name);

        let capabilities = surface.get_capabilities(&handles.adapter);
        // Prefer a linear format: the master is already encoded, and a
        // second srgb transfer at present washes the cloud out.
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let mut ambience = Ambience::new(&handles.device, &handles.queue, MOTES, EXTENT);
        let lease = ambience.lease();
        let tenant = Tenant::new(&handles, SIZE, lease.count, lease.extent);
        let (slab, base) = tenant.slab_and_transform_base();
        ambience.attach_slab(&slab);
        println!(
            "{} motes resident; renderling transforms at word {base}",
            lease.count
        );
        let composer = Composer::new(handles.clone(), SIZE);

        let mut live = Live {
            window,
            surface,
            format,
            handles,
            ambience,
            tenant,
            composer,
            base,
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
                    configure(live);
                }
            }
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {}
        }
    }
}

impl LiveApp {
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let began = Instant::now();

        // The epoch, every frame, before publishing: the same check the
        // offscreen run proved load-bearing.
        let (slab, grew) = live.tenant.commit();
        if grew {
            live.ambience.attach_slab(&slab);
            self.regrows += 1;
        }

        live.ambience.step(1.0 / 60.0, live.base, TRANSFORM_WORDS);
        live.tenant.draw();

        let chrome = chrome_bar(self.frames);
        let master = live.composer.compose(&chrome, &live.tenant.view);

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
                frame.present();
            }
            Acquired::Outdated | Acquired::Lost => configure(live),
            Acquired::Timeout | Acquired::Occluded => {}
            Acquired::Validation => panic!("surface acquisition failed validation"),
        }

        let span = began.elapsed().as_secs_f64() * 1e3;
        // The first frames compile pipelines and upload 20,000
        // transforms; a worst-frame number that includes them measures
        // startup rather than the steady state.
        if self.frames > 8 {
            self.worst = self.worst.max(span);
        }
        self.frames += 1;

        if self.capture_mode && self.frames >= CAPTURE_AFTER && !self.captured {
            self.captured = true;
            let capture = live.composer.capture(&master);
            let path = PathBuf::from(CAPTURE);
            capture.write_png(&path).expect("write the receipt");
            // Measure the tenant's region only. The whole-frame guards
            // the offscreen probe uses take pixel zero as background,
            // and pixel zero here is the chrome bar, which made 99.7%
            // of the frame read as "lit". A guard calibrated for one
            // composition misreads another; this one is told where the
            // scene is.
            let (colours, lit) = scene_region(&capture.pixels, SIZE, CHROME_HEIGHT);
            println!(
                "live: {} frames, worst {:.2} ms (first frame compiles pipelines), {} regrows;                  scene region {colours} colours, {:.1}% lit -> {}",
                self.frames,
                self.worst,
                self.regrows,
                lit * 100.0,
                path.display()
            );
            assert!(
                colours > 32,
                "the windowed run presented a flat scene: {colours} colours"
            );
            assert!(
                (0.01..0.75).contains(&lit),
                "scene coverage {lit:.3}: nothing, or everything, was drawn"
            );
            // The device is quiesced before exit so the capture's
            // submissions are not abandoned mid-flight.
            live.handles
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .expect("final poll");
            event_loop.exit();
            return;
        }

        if self.frames.is_multiple_of(120) {
            println!("frame {}: {:.2} ms", self.frames, span);
        }
        live.window.request_redraw();
    }
}

/// A chrome bar over the cloud, so the frame proves composition rather
/// than just a renderling window: vello paints this, netrender composes
/// it above the tenant at scene-op boundary zero.
fn chrome_bar(frame: u64) -> Scene {
    let width = SIZE[0] as f32;
    let mut scene = Scene::new(SIZE[0], SIZE[1]);
    scene.push_rect(0.0, 0.0, width, 34.0, [0.05, 0.06, 0.09, 0.86]);
    // A sweep that moves with the frame, so a frozen window is visibly
    // frozen rather than merely still.
    let sweep = (frame % 240) as f32 / 240.0 * (width - 32.0);
    scene.push_rect(16.0, 12.0, 16.0 + sweep, 24.0, [0.45, 0.62, 0.95, 0.95]);
    scene.push_rect(0.0, 0.0, width, 2.0, [0.45, 0.62, 0.95, 1.0]);
    scene
}

/// Colours and coverage in the tenant's region, taking the region's
/// own corner as its background.
fn scene_region(pixels: &[u8], size: [u32; 2], skip_rows: u32) -> (usize, f32) {
    let row = size[0] as usize * 4;
    let start = skip_rows as usize * row;
    let region = &pixels[start..];
    let background = [region[0], region[1], region[2]];
    let mut seen = std::collections::HashSet::new();
    let mut lit = 0usize;
    for pixel in region.chunks_exact(4) {
        seen.insert([pixel[0], pixel[1], pixel[2]]);
        if pixel[..3] != background {
            lit += 1;
        }
    }
    let total = region.len() / 4;
    (seen.len(), lit as f32 / total as f32)
}

fn configure(live: &mut Live) {
    let size = live.window.inner_size();
    live.surface.configure(
        &live.handles.device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: live.format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
    let _ = MASTER_FORMAT;
}
