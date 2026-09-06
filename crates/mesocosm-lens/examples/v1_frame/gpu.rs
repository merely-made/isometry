// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use mesocosm_lens::{
    CritterPose, FRAME_FORMAT, Flight, FrameInput, Grade, Lens, LensScene, MapRevision, critter,
    maps,
};
use mesocosm_render::composite::Composite;
use netrender::{
    Compositor, ExternalTextureComposite, ExternalTexturePlacement, NetrenderOptions,
    PresentedFrame, Scene, WgpuHandles, create_netrender_instance,
};
use winit::window::Window;

use crate::receipt::Receipt;

const MASTER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub const INITIAL_SIZE: [u32; 2] = [960, 540];
pub const MIN_FRAMES: u32 = 2;
pub const WINDOW_TITLE: &str = "Mesocosm V1";

pub struct Gpu {
    pub surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    handles: WgpuHandles,
    lens: Lens,
    lens_target: wgpu::Texture,
    lens_view: wgpu::TextureView,
    net: netrender::Renderer,
    composite: Composite,
    document: LensScene,
    document_bytes: Vec<u8>,
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
                label: Some("mesocosm V1"),
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
        let document = probe_document();
        let document_bytes = document.to_postcard().map_err(|error| error.to_string())?;
        let document =
            LensScene::from_postcard(&document_bytes).map_err(|error| error.to_string())?;
        let lens = Lens::with_format(
            handles.device.clone(),
            handles.queue.clone(),
            size[0],
            size[1],
            FRAME_FORMAT,
        );
        let (lens_target, lens_view) = make_lens_target(&handles.device, size);
        Ok(Self {
            surface,
            surface_config,
            lens,
            lens_target,
            lens_view,
            net,
            composite: Composite::new(&handles.device, format),
            handles,
            document,
            document_bytes,
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
        self.lens.resize(size[0], size[1]);
        (self.lens_target, self.lens_view) = make_lens_target(&self.handles.device, size);
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
        let surface_view = surface.texture.create_view(&Default::default());
        let rendered = self.render(&surface_view, frame);
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
                    label: Some("mesocosm V1 lens"),
                });
        let mut input = FrameInput::new(
            &self.document.maps,
            MapRevision(1),
            &self.document.flight,
            &self.document.grade,
        );
        input.pose = self.document.pose.as_ref();
        let lens = self
            .lens
            .encode(&mut encoder, &self.lens_view, input)
            .map_err(|error| error.to_string())?;
        self.handles.queue.submit([encoder.finish()]);

        let size = [self.surface_config.width, self.surface_config.height];
        let external = [ExternalTextureComposite::new(
            &self.lens_view,
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
        let dirty = self.net.vello_last_dirty_count().unwrap_or_default();
        Ok((
            Receipt::new(
                frame,
                size,
                self.surface_config.format,
                &self.document_bytes,
                &self.handles.adapter,
                lens,
                timings,
                dirty,
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
                    label: Some("mesocosm V1 present"),
                });
        {
            let _clear = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("mesocosm V1 clear"),
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

fn make_lens_target(device: &wgpu::Device, size: [u32; 2]) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("mesocosm V1 lens target"),
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
    scene.push_rect(0.0, 0.0, 2.0, height, [0.50, 0.95, 0.72, 1.0]);
    scene.push_rect(width - 2.0, 0.0, width, height, [0.50, 0.95, 0.72, 1.0]);
    scene
}

fn probe_document() -> LensScene {
    let maps = maps::synthesize(0x00A7_7AC4, 256);
    let ground = |x: f32, z: f32| {
        let side = maps.side;
        let index = (z.max(0.0) as u32 % side) * side + (x.max(0.0) as u32 % side);
        maps.height[index as usize] as f32
    };
    let mut body = critter::Body::caterpillar(9, 1.35);
    for step in 0..18 {
        let x = 122.0 + step as f32 * 0.9;
        let z = 126.0 + (step as f32 * 0.42).sin() * 5.0;
        body.step([x, ground(x, z) + 2.2, z], ground);
    }
    let pose = CritterPose::from_body(&body, ground, [0.36, 0.72, 0.46]);
    let head = body.chain.segments[0].at;
    let tail = body.chain.segments.last().expect("body has segments").at;
    let mid = [
        (head[0] + tail[0]) * 0.5,
        (head[1] + tail[1]) * 0.5,
        (head[2] + tail[2]) * 0.5,
    ];
    let heading = f32::atan2(head[0] - tail[0], head[2] - tail[2]);
    let flank = heading + std::f32::consts::FRAC_PI_2;
    let mut eye = [
        mid[0] + flank.sin() * 39.0,
        mid[1] + 13.0,
        mid[2] + flank.cos() * 39.0,
    ];
    eye[1] = eye[1].max(ground(eye[0], eye[2]) + 4.0);
    let distance = ((mid[0] - eye[0]).powi(2) + (mid[2] - eye[2]).powi(2)).sqrt();
    LensScene {
        grade: Grade::retro(maps.palette.len() as u32),
        maps,
        flight: Flight {
            eye,
            yaw: f32::atan2(mid[0] - eye[0], mid[2] - eye[2]),
            pitch: f32::atan2(mid[1] - 2.0 - eye[1], distance),
            fov: 0.9,
            far: 500.0,
        },
        pose: Some(pose),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn ensure_parent(path: &std::path::Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    Ok(())
}
