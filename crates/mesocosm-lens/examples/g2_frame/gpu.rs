// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use mesocosm_lens::{BrickTracer, FRAME_FORMAT};
use mesocosm_render::composite::Composite;
use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, NetrenderOptions,
    PresentedFrame, Scene, WgpuHandles, create_netrender_instance,
};
use winit::window::Window;

use crate::receipt::Receipt;
use crate::scenario::Scenario;

const MASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const INITIAL_SIZE: [u32; 2] = crate::scenario::INITIAL_SIZE;
pub const MIN_FRAMES: u32 = crate::scenario::MIN_FRAMES;
pub const WINDOW_TITLE: &str = crate::scenario::WINDOW_TITLE;

pub struct Gpu {
    pub surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    handles: WgpuHandles,
    tracer: BrickTracer,
    trace_target: wgpu::Texture,
    trace_view: wgpu::TextureView,
    net: netrender::Renderer,
    composite: Composite,
    scenario: Scenario,
    chrome: Scene,
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
                label: Some("Mesocosm headed frame host"),
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
            // wgpu 30 made surface color space explicit; Auto keeps the pre-30
            // platform-chosen behavior.
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
            trace_target,
            trace_view,
            net,
            composite: Composite::new(&handles.device, format),
            scenario,
            chrome: chrome_scene(size),
        })
    }

    fn configure(&self) {
        self.surface
            .configure(&self.handles.device, &self.surface_config);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let size = [width.max(1), height.max(1)];
        self.surface_config.width = size[0];
        self.surface_config.height = size[1];
        self.configure();
        self.tracer.resize(size[0], size[1]);
        (self.trace_target, self.trace_view) = trace_target(&self.handles.device, size);
        self.chrome = chrome_scene(size);
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
        // wgpu 30 moved presentation from SurfaceTexture to Queue.
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
                    label: Some("Mesocosm headed trace"),
                });
        let trace =
            self.scenario
                .encode(&mut self.tracer, &mut encoder, &self.trace_view, frame)?;
        self.handles.queue.submit([encoder.finish()]);

        let size = [self.surface_config.width, self.surface_config.height];
        let external = [ExternalTextureComposite::new(
            &self.trace_view,
            ExternalTexturePlacement::new([0.0, 0.0, size[0] as f32, size[1] as f32]),
        )
        .with_scene_op_boundary(0)];
        let mut present = FramePresenter {
            target: surface_view,
            composite: &self.composite,
            size,
            master: None,
        };
        self.net.render_with_compositor_and_external_textures(
            &self.chrome,
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
                self.net.vello_last_dirty_count().unwrap_or_default(),
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
        let pixels = self.net.wgpu_device.read_rgba8_texture(
            master,
            self.surface_config.width,
            self.surface_config.height,
        );
        let file = std::fs::File::create(path).map_err(|error| error.to_string())?;
        let mut png =
            png::Encoder::new(file, self.surface_config.width, self.surface_config.height);
        png.set_color(png::ColorType::Rgba);
        png.set_depth(png::BitDepth::Eight);
        png.write_header()
            .and_then(|mut writer| writer.write_image_data(&pixels))
            .map_err(|error| error.to_string())
    }
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
                    label: Some("Mesocosm headed present"),
                });
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Mesocosm headed clear"),
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
        label: Some("Mesocosm headed trace target"),
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

fn chrome_scene(size: [u32; 2]) -> Scene {
    let (width, height) = (size[0] as f32, size[1] as f32);
    let mut scene = Scene::new(size[0], size[1]);
    scene.push_rect(0.0, 0.0, width, 42.0, [0.025, 0.035, 0.055, 0.88]);
    scene.push_rect(18.0, 13.0, 168.0, 29.0, [0.27, 0.85, 0.60, 0.92]);
    scene.push_rect(
        width - 142.0,
        13.0,
        width - 18.0,
        29.0,
        [0.95, 0.68, 0.28, 0.92],
    );
    scene.push_rect(0.0, 0.0, width, 2.0, [0.50, 0.95, 0.72, 1.0]);
    scene.push_rect(0.0, height - 2.0, width, height, [0.50, 0.95, 0.72, 1.0]);
    scene
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_parent(path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    Ok(())
}
