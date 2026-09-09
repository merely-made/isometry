// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's body renderer.
//!
//! Small on purpose. The geometry is flat-shaded palette quads with no
//! textures, no interpolated normals, and no skinning, because parts are rigid
//! by ruling. That is why this game does not want an engine: an engine's
//! rendering value is in the parts this problem does not have.
//!
//! # Headless first
//!
//! A [`Renderer`] draws to a texture and can read it back, so "the new part is
//! visible" is an assertion rather than an opinion. A window is a thin wrapper
//! over the same path. What stays a judgment is whether the result looks
//! *good*, which is the right thing to leave to a human at a screen.
//!
//! Frames are compared by **coverage**, never by byte equality. Two GPUs with
//! different drivers may rasterise the same scene to slightly different pixels,
//! so the tests here assert what is drawn and where, not which exact bytes came
//! back. Gameplay identity lives in the core's state hash, never in a raster.

pub mod camera;
pub mod composite;
pub mod geometry;
pub mod live_body;

use mesocosm_mesh::BodyMesh;
use wgpu::util::DeviceExt;

pub use camera::Camera;
pub use geometry::{
    SceneItem, Vertex, build_scene_vertices, build_vertices, deadened, face_shade, kingdom_colour,
    material_colour, warning_colour,
};
pub use live_body::{
    BodyDrawStats, ClipSlab, LiveBody, LiveBodyError, LiveBodyRenderer, PartMaterial,
};

/// Colour the frame is cleared to. Distinct from every material colour, so
/// coverage can be measured by "not this".
pub const BACKGROUND: [f64; 4] = [0.06, 0.07, 0.09, 1.0];

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const COLOUR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// No GPU adapter at all. A machine without one cannot run the visual
    /// tests, which is reported rather than silently passing them.
    NoAdapter,
    DeviceLost,
    Readback,
}

/// A rendered frame, as RGBA8 rows without padding.
#[derive(Clone, Debug)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Frame {
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let i = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[i],
            self.pixels[i + 1],
            self.pixels[i + 2],
            self.pixels[i + 3],
        ]
    }

    fn background() -> [u8; 3] {
        // The clear colour is given in linear space and the target is sRGB, so
        // compare against the encoded value rather than the literal.
        BACKGROUND[..3]
            .iter()
            .map(|c| {
                let c = *c as f32;
                let encoded = if c <= 0.0031308 {
                    c * 12.92
                } else {
                    1.055 * c.powf(1.0 / 2.4) - 0.055
                };
                (encoded * 255.0).round() as u8
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("three channels")
    }

    /// Pixels that are not the background, within a tolerance for rounding.
    pub fn covered(&self) -> u32 {
        let bg = Self::background();
        let mut count = 0;
        for chunk in self.pixels.chunks_exact(4) {
            let differs = (0..3).any(|c| chunk[c].abs_diff(bg[c]) > 4);
            if differs {
                count += 1;
            }
        }
        count
    }

    pub fn is_blank(&self) -> bool {
        self.covered() == 0
    }

    /// Bounding box of drawn pixels as `(min_x, min_y, max_x, max_y)`.
    pub fn covered_bounds(&self) -> Option<(u32, u32, u32, u32)> {
        let bg = Self::background();
        let mut bounds: Option<(u32, u32, u32, u32)> = None;
        for y in 0..self.height {
            for x in 0..self.width {
                let p = self.pixel(x, y);
                if (0..3).any(|c| p[c].abs_diff(bg[c]) > 4) {
                    bounds = Some(match bounds {
                        None => (x, y, x, y),
                        Some((x0, y0, x1, y1)) => (x0.min(x), y0.min(y), x1.max(x), y1.max(y)),
                    });
                }
            }
        }
        bounds
    }
}

/// Draws meshed bodies. Owns a device only in headless use; a windowed host
/// hands its own device in.
pub struct Renderer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
}

impl Renderer {
    /// Builds a renderer with its own headless device.
    pub fn headless(width: u32, height: u32) -> Result<Self, RenderError> {
        pollster::block_on(Self::headless_async(width, height))
    }

    pub async fn headless_async(width: u32, height: u32) -> Result<Self, RenderError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
                compatible_surface: None,
            })
            .await
            .map_err(|_| RenderError::NoAdapter)?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("mesocosm headless"),
                ..Default::default()
            })
            .await
            .map_err(|_| RenderError::DeviceLost)?;

        Ok(Self::with_device(device, queue, width, height))
    }

    /// Builds a renderer over a device someone else owns, which is the shipped
    /// arrangement: netrender owns the device and composites the result.
    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue, width: u32, height: u32) -> Self {
        Self::with_format(device, queue, width, height, COLOUR_FORMAT)
    }

    /// As [`Self::with_device`], for a target whose format is not ours. A
    /// window surface is usually BGRA, so a windowed host passes its own.
    pub fn with_format(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesocosm body"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::cast_slice(&[[[0f32; 4]; 4]]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera"),
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("body"),
            bind_group_layouts: &[Some(&camera_layout)],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("body"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::LAYOUT)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                // Faces are emitted outward-wound, but a body can be viewed
                // from any side and a mis-wound quad should be visible rather
                // than invisible. Culling is a later optimisation.
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        Self {
            device,
            queue,
            pipeline,
            camera_buffer,
            camera_bind_group,
            format,
            width,
            height,
        }
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
    }

    /// Records the body pass into an encoder, against a caller-owned target.
    ///
    /// This is the one drawing path. Headless rendering and a window differ
    /// only in where the target view comes from and what happens afterwards,
    /// which is what keeps the tested path and the shipped path identical.
    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        mesh: &BodyMesh,
        camera: &Camera,
    ) {
        self.draw_scene(encoder, target, &[SceneItem::new(mesh, [0, 0, 0])], camera);
    }

    /// Records a whole scene: several bodies, each placed in world space.
    ///
    /// One pass and one buffer for everything, which is what the depth buffer
    /// wants and what keeps a world of loose matter cheap to draw.
    pub fn draw_scene(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        items: &[SceneItem],
        camera: &Camera,
    ) {
        let vertices = build_scene_vertices(items);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[camera.view_proj_array()]),
        );

        let depth = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth.create_view(&Default::default());

        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertices"),
                contents: if vertices.is_empty() {
                    bytemuck::cast_slice(&[Vertex {
                        position: [0.0; 3],
                        color: [0.0; 3],
                    }])
                } else {
                    bytemuck::cast_slice(&vertices)
                },
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("body"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: BACKGROUND[0],
                        g: BACKGROUND[1],
                        b: BACKGROUND[2],
                        a: BACKGROUND[3],
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if !vertices.is_empty() {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            pass.draw(0..vertices.len() as u32, 0..1);
        }
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Renders a body offscreen and reads the frame back.
    ///
    /// Goes through the same [`Self::draw`] a window uses, so what the tests
    /// assert is what a host displays.
    pub fn render(&self, mesh: &BodyMesh, camera: &Camera) -> Result<Frame, RenderError> {
        self.render_scene(&[SceneItem::new(mesh, [0, 0, 0])], camera)
    }

    /// Renders a whole scene offscreen and reads the frame back.
    pub fn render_scene(&self, items: &[SceneItem], camera: &Camera) -> Result<Frame, RenderError> {
        self.render_scene_with(items, camera, |_, _| {})
    }

    /// Like [`Self::render_scene`], with a caller pass between the scene and
    /// the readback. The capture path composites chrome through this, so a
    /// capture shows exactly what a window shows.
    pub fn render_scene_with(
        &self,
        items: &[SceneItem],
        camera: &Camera,
        after: impl FnOnce(&mut wgpu::CommandEncoder, &wgpu::TextureView),
    ) -> Result<Frame, RenderError> {
        let colour = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("frame"),
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let colour_view = colour.create_view(&Default::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("body"),
            });
        self.draw_scene(&mut encoder, &colour_view, items, camera);
        after(&mut encoder, &colour_view);

        self.read_back(encoder, &colour)
    }

    fn read_back(
        &self,
        mut encoder: wgpu::CommandEncoder,
        colour: &wgpu::Texture,
    ) -> Result<Frame, RenderError> {
        // Copy rows must be aligned, so the staging buffer is padded and the
        // padding is stripped after mapping.
        let unpadded = self.width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;

        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (padded * self.height) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: colour,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &staging,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));

        let slice = staging.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|_| RenderError::Readback)?;

        let mapped = slice.get_mapped_range().expect("map range");
        let mut pixels = Vec::with_capacity((unpadded * self.height) as usize);
        for row in 0..self.height {
            let start = (row * padded) as usize;
            pixels.extend_from_slice(&mapped[start..start + unpadded as usize]);
        }
        drop(mapped);
        staging.unmap();

        Ok(Frame {
            width: self.width,
            height: self.height,
            pixels,
        })
    }
}
