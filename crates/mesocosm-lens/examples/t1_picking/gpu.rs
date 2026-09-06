// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! T1's headed shell: the G2 tracer frame plus a per-frame overlay showing
//! the cursor and the tactile answer it received.

use std::sync::Arc;

use mesocosm_lens::{BrickTracer, FRAME_FORMAT};
use mesocosm_render::composite::Composite;
use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, NetrenderOptions,
    PresentedFrame, Scene, WgpuHandles, create_netrender_instance,
};
use winit::window::Window;

use crate::receipt::Receipt;
use crate::scenario::{Answer, Scenario};

const MASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const INITIAL_SIZE: [u32; 2] = crate::scenario::INITIAL_SIZE;
pub const MIN_FRAMES: u32 = crate::scenario::MIN_FRAMES;
pub const WINDOW_TITLE: &str = crate::scenario::WINDOW_TITLE;

pub struct Gpu {
    pub surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    handles: WgpuHandles,
    tracer: BrickTracer,
    _trace_target: wgpu::Texture,
    trace_view: wgpu::TextureView,
    net: netrender::Renderer,
    composite: Composite,
    scenario: Scenario,
}

impl Gpu {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let surface = instance
            .create_surface(window)
            .map_err(|error| error.to_string())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                apply_limit_buckets: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|error| error.to_string())?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Mesocosm T1 picking host"),
                required_features: netrender::REQUIRED_FEATURES,
                required_limits: wgpu::Limits {
                    max_inter_stage_shader_variables: 28,
                    ..Default::default()
                },
                ..Default::default()
            })
            .await
            .map_err(|error| error.to_string())?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|format| format.is_srgb())
            .unwrap_or(caps.formats[0]);
        let size = INITIAL_SIZE;
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size[0],
            height: size[1],
            present_mode: caps.present_modes[0],
            color_space: wgpu::SurfaceColorSpace::Auto,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);
        let handles = WgpuHandles {
            instance,
            adapter,
            device,
            queue,
        };
        let net = create_netrender_instance(
            handles.clone(),
            NetrenderOptions {
                tile_cache_size: Some(64),
                enable_vello: true,
                ..Default::default()
            },
        )
        .map_err(|error| format!("netrender init failed: {error:?}"))?;
        let scenario = Scenario::new()?;
        let tracer = BrickTracer::with_format(
            handles.device.clone(),
            handles.queue.clone(),
            size[0],
            size[1],
            FRAME_FORMAT,
        );
        let (trace_target, trace_view) = trace_target(&handles.device, size);
        Ok(Self {
            surface,
            surface_config,
            handles: handles.clone(),
            tracer,
            _trace_target: trace_target,
            trace_view,
            net,
            composite: Composite::new(&handles.device, format),
            scenario,
        })
    }

    fn configure(&self) {
        self.surface
            .configure(&self.handles.device, &self.surface_config);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        // The judged frame stays at the fixed receipt size; only the
        // on-screen surface follows the window, stretching the master.
        self.surface_config.width = width.max(1);
        self.surface_config.height = height.max(1);
        self.configure();
    }

    pub fn draw(&mut self, frame: u32) -> Result<Option<(Receipt, wgpu::Texture)>, String> {
        let surface = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(texture)
            | wgpu::CurrentSurfaceTexture::Suboptimal(texture) => texture,
            wgpu::CurrentSurfaceTexture::Lost | wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure();
                return Ok(None);
            },
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(None),
            _ => return Err("surface acquisition failed".into()),
        };
        let view = surface.texture.create_view(&Default::default());
        let rendered = self.render(&view, frame);
        self.handles.queue.present(surface);
        rendered.map(Some)
    }

    fn render(
        &mut self,
        surface_view: &wgpu::TextureView,
        frame: u32,
    ) -> Result<(Receipt, wgpu::Texture), String> {
        let mut encoder =
            self.handles
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Mesocosm T1 trace"),
                });
        let trace =
            self.scenario
                .encode(&mut self.tracer, &mut encoder, &self.trace_view, frame)?;
        self.handles.queue.submit([encoder.finish()]);

        let size = INITIAL_SIZE;
        let chrome = overlay(size, &self.scenario, frame);
        let external = [ExternalTextureComposite::new(
            &self.trace_view,
            ExternalTexturePlacement::new([0.0, 0.0, size[0] as f32, size[1] as f32]),
        )
        .with_scene_op_boundary(0)];
        let mut present = FramePresenter {
            target: surface_view,
            composite: &self.composite,
            size: [self.surface_config.width, self.surface_config.height],
            master: None,
        };
        self.net.render_with_compositor_and_external_textures(
            &chrome,
            MASTER_FORMAT,
            &mut present,
            netrender::peniko::Color::new([0.0, 0.0, 0.0, 0.0]),
            &external,
        );
        let master = present
            .master
            .ok_or("netrender did not present a master texture")?;
        let timings = self
            .net
            .last_frame_timings()
            .ok_or("netrender did not report frame timings")?;
        Ok((
            Receipt::new(
                frame,
                size,
                self.surface_config.format,
                &self.scenario,
                &self.handles.adapter,
                trace,
                timings,
            ),
            master,
        ))
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_native_receipts(
        &self,
        receipt_path: Option<&std::path::Path>,
        capture_path: Option<&std::path::Path>,
        json: &str,
        master: &wgpu::Texture,
    ) -> Result<(), String> {
        if let Some(path) = receipt_path {
            ensure_parent(path)?;
            std::fs::write(path, json).map_err(|error| error.to_string())?;
        }
        let Some(path) = capture_path else {
            return Ok(());
        };
        ensure_parent(path)?;
        let pixels =
            self.net
                .wgpu_device
                .read_rgba8_texture(master, INITIAL_SIZE[0], INITIAL_SIZE[1]);
        let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
        let mut png = png::Encoder::new(file, INITIAL_SIZE[0], INITIAL_SIZE[1]);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        png.write_header()
            .and_then(|mut writer| writer.write_image_data(&pixels))
            .map_err(|error| error.to_string())
    }
}

/// The picking overlay: the cursor cross, the answer's marker, and a class
/// chip in the top bar, all placed by projecting the judged stop through
/// the same slab basis the rays used.
fn overlay(size: [u32; 2], scenario: &Scenario, frame: u32) -> Scene {
    let stop = scenario.stop_for_frame(frame);
    let mut scene = Scene::new(size[0], size[1]);
    let (width, _height) = (size[0] as f32, size[1] as f32);

    // Top bar with a chip coloured by the answer's class.
    scene.push_rect(0.0, 0.0, width, 26.0, [0.025, 0.035, 0.055, 0.88]);
    let chip = class_colour(&stop.answer);
    scene.push_rect(14.0, 7.0, 74.0, 19.0, chip);
    scene.push_rect(0.0, 0.0, width, 2.0, [0.50, 0.95, 0.72, 1.0]);

    // The cursor cross.
    let cursor = scenario.pixel(stop.world, size);
    cross(&mut scene, cursor, [0.96, 0.96, 0.98, 0.95]);

    // The answer's marker.
    match stop.answer {
        Answer::Ground { cell, .. } => {
            let centre = scenario.pixel([cell[0] as f32 + 0.5, cell[1] as f32 + 0.5], size);
            outline(&mut scene, centre, 14.0, chip);
        },
        Answer::Critter { .. } => {
            let body = scenario.body_centre();
            let centre = scenario.pixel([body[0], body[1]], size);
            outline(&mut scene, centre, 22.0, chip);
        },
        Answer::Nothing => outline(&mut scene, cursor, 10.0, chip),
    }
    scene
}

fn class_colour(answer: &Answer) -> [f32; 4] {
    match answer {
        Answer::Ground { .. } => [0.95, 0.68, 0.28, 0.95],
        Answer::Critter { .. } => [0.27, 0.85, 0.60, 0.95],
        Answer::Nothing => [0.62, 0.66, 0.72, 0.95],
    }
}

fn cross(scene: &mut Scene, at: [f32; 2], colour: [f32; 4]) {
    scene.push_rect(at[0] - 9.0, at[1] - 1.5, at[0] + 9.0, at[1] + 1.5, colour);
    scene.push_rect(at[0] - 1.5, at[1] - 9.0, at[0] + 1.5, at[1] + 9.0, colour);
}

fn outline(scene: &mut Scene, at: [f32; 2], half: f32, colour: [f32; 4]) {
    let (l, t, r, b) = (at[0] - half, at[1] - half, at[0] + half, at[1] + half);
    scene.push_rect(l, t, r, t + 2.0, colour);
    scene.push_rect(l, b - 2.0, r, b, colour);
    scene.push_rect(l, t, l + 2.0, b, colour);
    scene.push_rect(r - 2.0, t, r, b, colour);
}

struct FramePresenter<'a> {
    target: &'a wgpu::TextureView,
    composite: &'a Composite,
    size: [u32; 2],
    master: Option<wgpu::Texture>,
}

impl Compositor for FramePresenter<'_> {
    fn declare_surface(&mut self, _key: netrender::SurfaceKey, _world_bounds: [f32; 4]) {}

    fn destroy_surface(&mut self, _key: netrender::SurfaceKey) {}

    fn present_frame(&mut self, frame: PresentedFrame<'_>) {
        let master_view = frame.master.create_view(&Default::default());
        let mut encoder =
            frame
                .handles
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Mesocosm T1 present"),
                });
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesocosm T1 clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: self.target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        self.composite.draw(
            &frame.handles.device,
            &frame.handles.queue,
            &mut encoder,
            self.target,
            &master_view,
            (0.0, 0.0, self.size[0] as f32, self.size[1] as f32),
            (self.size[0], self.size[1]),
        );
        frame.handles.queue.submit([encoder.finish()]);
        self.master = Some(frame.master.clone());
    }
}

fn trace_target(device: &wgpu::Device, size: [u32; 2]) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Mesocosm T1 trace target"),
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FRAME_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    (texture, view)
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_parent(path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    Ok(())
}
