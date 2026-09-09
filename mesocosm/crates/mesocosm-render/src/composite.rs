// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Compositing a layer over a rendered frame.
//!
//! Vello overwrites its entire target (`ColorLoad::Load` is documented as
//! clear-to-transparent), so HUD content cannot be rasterized straight onto
//! the game's surface. It renders into its own transparent texture, and this
//! pass blends that texture over the frame at a destination rectangle.
//!
//! Premultiplied-alpha blending, because that is what vello emits.
//!
//! **Named for the operation, not a position.** An earlier cut called this
//! `Overlay`, which is cambium's word for anchored floating UI (`overlay_at`,
//! `OverlaySurface`) and means a *thing*, not an act. `composite` is the
//! stack's existing word for blending layers into a frame, matching
//! netrender's `Compositor` and `paint_list_render::composite`.

use wgpu::util::DeviceExt;

/// The blit-with-blend pipeline. Built once per (device, surface format).
pub struct Composite {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

const SHADER: &str = r#"
@group(0) @binding(0) var content: texture_2d<f32>;
@group(0) @binding(1) var content_sampler: sampler;
// Destination corners in NDC: (left, top, right, bottom).
@group(0) @binding(2) var<uniform> dest: vec4<f32>;

struct VsOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs(@builtin(vertex_index) index: u32) -> VsOut {
    var corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
        vec2(0.0, 0.0), vec2(1.0, 1.0), vec2(0.0, 1.0),
    );
    let corner = corners[index];
    var out: VsOut;
    out.pos = vec4(
        mix(dest.x, dest.z, corner.x),
        mix(dest.y, dest.w, corner.y),
        0.0,
        1.0,
    );
    out.uv = corner;
    return out;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4<f32> {
    return textureSample(content, content_sampler, in.uv);
}
"#;

impl Composite {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("composite"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("composite"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("composite"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("composite"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Premultiplied source over destination.
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("composite"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            pipeline,
            layout,
            sampler,
        }
    }

    /// Blends `content` over `target` at a pixel rectangle of the frame.
    ///
    /// `dest` is `(x, y, width, height)` from the frame's top-left;
    /// `frame` is the target's full pixel size.
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        content: &wgpu::TextureView,
        dest: (f32, f32, f32, f32),
        frame: (u32, u32),
    ) {
        let (x, y, w, h) = dest;
        let (fw, fh) = (frame.0.max(1) as f32, frame.1.max(1) as f32);
        // Pixel rect to NDC corners; y flips because NDC grows upward.
        let ndc = [
            x / fw * 2.0 - 1.0,
            1.0 - y / fh * 2.0,
            (x + w) / fw * 2.0 - 1.0,
            1.0 - (y + h) / fh * 2.0,
        ];
        // Each encoded draw needs immutable rectangle data. Reusing one
        // queue-written uniform here makes every draw in a command buffer see
        // the final draw's rectangle when the GPU eventually executes it.
        let rect = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("composite dest"),
            contents: bytemuck::cast_slice(&ndc),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("composite"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(content),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: rect.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("composite"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    // Blending over what the frame already holds.
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind, &[]);
        pass.draw(0..6, 0..1);
    }
}
