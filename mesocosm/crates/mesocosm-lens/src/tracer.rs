// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained fragment-only rendering over [`modulus::BrickMap`].
//!
//! The tracer owns GPU copies, never voxel authority. A caller gives it a
//! revision and the slots changed by a projection drain; it encodes into a
//! caller-owned target just like [`crate::Lens`].

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

mod lease;
mod params;
mod residency;
mod types;

use modulus::BRICK_DDA_WGSL;

use crate::{FRAME_FORMAT, MAX_ROSTER, MAX_ROSTER_CAPSULES};
use params::{
    ROSTER_BUFFER_BYTES, ROSTER_HEADER_BYTES, RosterPose, TraceParams, validates_change,
    validates_pose,
};
use residency::ResidentMap;

pub use lease::LeasedAtlas;
pub use types::{
    BrickCapture, BrickChange, BrickDiagnostics, BrickFrameInput, BrickRevision, BrickTraceError,
    SlabWall, TraceCamera,
};

struct CaptureResources {
    width: u32,
    height: u32,
    target: wgpu::Texture,
    view: wgpu::TextureView,
    staging: wgpu::Buffer,
    padded_bytes_per_row: u32,
}

pub struct BrickTracer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    pipeline: wgpu::RenderPipeline,
    /// The depth-join variant: `fs_depth` against a caller-owned
    /// `Depth32Float` attachment. Built on first use so plain tracing pays
    /// nothing for it.
    depth_pipeline: Option<wgpu::RenderPipeline>,
    shader: wgpu::ShaderModule,
    pipeline_layout: wgpu::PipelineLayout,
    layout: wgpu::BindGroupLayout,
    params: wgpu::Buffer,
    /// The pose roster beside `params`' single pose. Its own binding, so a
    /// frame that names no roster uploads nothing here.
    roster: wgpu::Buffer,
    map: Option<ResidentMap>,
    capture: Option<CaptureResources>,
    pending_resource_creations: u32,
    last_diagnostics: Option<BrickDiagnostics>,
    last_params: Option<TraceParams>,
    last_roster: Vec<RosterPose>,
}

impl BrickTracer {
    pub fn headless(width: u32, height: u32) -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .ok()?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).ok()?;
        Some(Self::with_device(device, queue, width, height))
    }

    /// The device this tracer renders on, so a resident producer can
    /// allocate its lease against the same one. A lease from another
    /// device cannot be copied into this tracer's atlas.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue, width: u32, height: u32) -> Self {
        Self::with_format(device, queue, width, height, FRAME_FORMAT)
    }

    pub fn with_format(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Uint,
                view_dimension: wgpu::TextureViewDimension::D3,
                multisampled: false,
            },
            count: None,
        };
        let uniform = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("brick tracer layout"),
            entries: &[texture(0), texture(1), uniform(2), uniform(3)],
        });
        // The roster's array sizes are the Rust caps, injected rather than
        // written twice: a drift between the two layouts is a silent
        // misread of the uniform.
        let shader_source = format!(
            "{BRICK_DDA_WGSL}\nconst ROSTER_MEMBERS = {MAX_ROSTER};\nconst ROSTER_PAIRS = {};\n{}",
            MAX_ROSTER_CAPSULES * 2,
            include_str!("tracer.wgsl")
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("brick tracer"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("brick tracer"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("brick tracer"),
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
                    blend: None,
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
        let uniform_buffer = |label, size| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let params = uniform_buffer("brick tracer parameters", size_of::<TraceParams>() as u64);
        // Zero-initialized, so an unwritten roster reads as no members.
        let roster = uniform_buffer("brick tracer roster", ROSTER_BUFFER_BYTES);
        Self {
            device,
            queue,
            format,
            width: width.max(1),
            height: height.max(1),
            pipeline,
            depth_pipeline: None,
            shader,
            pipeline_layout,
            layout,
            params,
            roster,
            map: None,
            capture: None,
            pending_resource_creations: 2,
            last_diagnostics: None,
            last_params: None,
            last_roster: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        self.capture = None;
    }

    pub fn last_diagnostics(&self) -> Option<BrickDiagnostics> {
        self.last_diagnostics
    }

    pub fn encode(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        input: BrickFrameInput<'_>,
    ) -> Result<BrickDiagnostics, BrickTraceError> {
        let mut diagnostics = self.prepare_frame(input)?;
        let bind = &self.map.as_ref().expect("map ensured").bind;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("brick trace"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        diagnostics.trace_passes = 1;
        self.last_diagnostics = Some(diagnostics);
        Ok(diagnostics)
    }

    /// The depth join: trace into a raster tenant's colour target against
    /// its stored `Depth32Float` depth, so the two occlude each other per
    /// pixel.
    ///
    /// The caller renders its raster pass first with depth stored, then
    /// this pass loads both attachments, writes `@builtin(frag_depth)` from
    /// the frame's `clip_from_world`, and tests `LessEqual` with depth
    /// write on. Standard-z is assumed, matching a depth cleared to 1.0
    /// under a `Less` raster compare; `LessEqual` here lets the traced sky,
    /// clamped to the far plane, replace the raster background while never
    /// covering nearer geometry.
    pub fn encode_with_depth(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        input: BrickFrameInput<'_>,
    ) -> Result<BrickDiagnostics, BrickTraceError> {
        if input.clip_from_world.is_none() {
            return Err(BrickTraceError::MissingClipFromWorld);
        }
        if self.depth_pipeline.is_none() {
            self.depth_pipeline = Some(self.create_depth_pipeline());
            self.pending_resource_creations += 1;
        }
        let mut diagnostics = self.prepare_frame(input)?;
        let bind = &self.map.as_ref().expect("map ensured").bind;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("brick trace depth join"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(self.depth_pipeline.as_ref().expect("depth pipeline built"));
        pass.set_bind_group(0, bind, &[]);
        pass.draw(0..3, 0..1);
        drop(pass);
        diagnostics.trace_passes = 1;
        self.last_diagnostics = Some(diagnostics);
        Ok(diagnostics)
    }

    /// Everything a trace pass needs before it begins: validation, resident
    /// textures, and the two uniforms, shared by both encode paths.
    fn prepare_frame(
        &mut self,
        input: BrickFrameInput<'_>,
    ) -> Result<BrickDiagnostics, BrickTraceError> {
        let started = Instant::now();
        validates_pose(input)?;
        validates_change(input)?;
        let mut diagnostics = BrickDiagnostics {
            resource_creations: std::mem::take(&mut self.pending_resource_creations),
            ..Default::default()
        };
        self.ensure_map(input, &mut diagnostics);
        let params = TraceParams::from_input(input);
        if self.last_params != Some(params) {
            self.queue
                .write_buffer(&self.params, 0, bytemuck::bytes_of(&params));
            diagnostics.uniform_upload_bytes += size_of::<TraceParams>() as u64;
            self.last_params = Some(params);
        }
        self.write_roster(input, &mut diagnostics);
        diagnostics.cpu_prepare_us = started.elapsed().as_micros() as u64;
        Ok(diagnostics)
    }

    /// Upload the roster's used prefix only, so a frame pays for the bodies
    /// in view rather than for the cap.
    fn write_roster(&mut self, input: BrickFrameInput<'_>, diagnostics: &mut BrickDiagnostics) {
        let roster = params::roster_of(input);
        diagnostics.roster_members = roster.len() as u32;
        diagnostics.roster_dropped = input.roster.len().saturating_sub(MAX_ROSTER) as u32;
        diagnostics.roster_capsules_dropped = params::roster_capsules_dropped(input);
        if self.last_roster == roster {
            return;
        }
        let count = [roster.len() as u32, 0, 0, 0];
        self.queue
            .write_buffer(&self.roster, 0, bytemuck::bytes_of(&count));
        diagnostics.uniform_upload_bytes += ROSTER_HEADER_BYTES;
        if !roster.is_empty() {
            self.queue.write_buffer(
                &self.roster,
                ROSTER_HEADER_BYTES,
                bytemuck::cast_slice(&roster),
            );
            diagnostics.uniform_upload_bytes += size_of_val(roster.as_slice()) as u64;
        }
        self.last_roster = roster;
    }

    fn create_depth_pipeline(&self) -> wgpu::RenderPipeline {
        self.device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("brick tracer depth join"),
                layout: Some(&self.pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &self.shader,
                    entry_point: Some("vs"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.shader,
                    entry_point: Some("fs_depth"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
    }

    pub fn capture(&mut self, input: BrickFrameInput<'_>) -> Result<BrickCapture, BrickTraceError> {
        if self.format != FRAME_FORMAT {
            return Err(BrickTraceError::CaptureFormat(self.format));
        }
        self.ensure_capture();
        let capture = self.capture.as_ref().expect("capture ensured");
        let view = capture.view.clone();
        let target = capture.target.clone();
        let staging = capture.staging.clone();
        let padded = capture.padded_bytes_per_row;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("brick trace capture"),
            });
        let mut diagnostics = self.encode(&mut encoder, &view, input)?;
        encoder.copy_texture_to_buffer(
            target.as_image_copy(),
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
        self.queue.submit([encoder.finish()]);
        let slice = staging.slice(..);
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = send.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| BrickTraceError::DevicePoll(error.to_string()))?;
        receive
            .recv()
            .map_err(|error| BrickTraceError::Readback(error.to_string()))?
            .map_err(|error| BrickTraceError::Readback(error.to_string()))?;
        let data = slice.get_mapped_range().expect("map range");
        let mut pixels = Vec::with_capacity((self.width * self.height * 4) as usize);
        for row in 0..self.height {
            let start = (row * padded) as usize;
            pixels.extend_from_slice(&data[start..start + (self.width * 4) as usize]);
        }
        drop(data);
        staging.unmap();
        diagnostics.readback_bytes = pixels.len() as u64;
        self.last_diagnostics = Some(diagnostics);
        Ok(BrickCapture {
            width: self.width,
            height: self.height,
            pixels,
            diagnostics,
        })
    }

    fn ensure_capture(&mut self) {
        if self
            .capture
            .as_ref()
            .is_some_and(|capture| (capture.width, capture.height) == (self.width, self.height))
        {
            return;
        }
        let target = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("brick trace capture"),
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
        let padded_bytes_per_row =
            (self.width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("brick trace readback"),
            size: u64::from(padded_bytes_per_row) * u64::from(self.height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.capture = Some(CaptureResources {
            width: self.width,
            height: self.height,
            view: target.create_view(&Default::default()),
            target,
            staging,
            padded_bytes_per_row,
        });
    }
}
