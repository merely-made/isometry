// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained GPU resources and the caller-encoded frame boundary.

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use bytemuck::{Pod, Zeroable};

use crate::{CritterPose, Flight, Grade, MAX_CAPSULES, maps::BiomeMaps};

mod helpers;
mod types;

use helpers::*;
pub use types::{
    Capture, DirtyRect, FrameDiagnostics, FrameInput, LensError, MapChange, MapRevision,
};

#[cfg(test)]
mod tests;

pub const FRAME_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct MarchParams {
    eye: [f32; 3],
    yaw: f32,
    pitch: f32,
    fov: f32,
    far: f32,
    map_side: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct CritterParams {
    bounds: [f32; 4],
    tint_count: [f32; 4],
    eyes: [[f32; 4]; 2],
    pairs: [[f32; 4]; MAX_CAPSULES * 2],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct GradeParams {
    fog: [f32; 3],
    fog_start: f32,
    palette_len: u32,
    dither: f32,
    fog_bands: f32,
    _pad: f32,
}

struct MapResources {
    side: u32,
    revision: MapRevision,
    height: wgpu::Texture,
    height_view: wgpu::TextureView,
    color: wgpu::Texture,
    color_view: wgpu::TextureView,
    palette: wgpu::Texture,
    palette_view: wgpu::TextureView,
    palette_bytes: Vec<u8>,
}

struct TargetResources {
    width: u32,
    height: u32,
    downscale: u32,
    _marched: wgpu::Texture,
    marched_view: wgpu::TextureView,
}

struct CaptureResources {
    width: u32,
    height: u32,
    target: wgpu::Texture,
    target_view: wgpu::TextureView,
    staging: wgpu::Buffer,
    padded_bytes_per_row: u32,
}

/// The retained lens. It owns presentation resources, while its caller owns
/// the simulation, frame encoder, target, and submission.
pub struct Lens {
    device: wgpu::Device,
    queue: wgpu::Queue,
    output_format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    march: wgpu::RenderPipeline,
    grade: wgpu::RenderPipeline,
    march_layout: wgpu::BindGroupLayout,
    grade_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    nearest: wgpu::Sampler,
    march_params: wgpu::Buffer,
    critter_params: wgpu::Buffer,
    grade_params: wgpu::Buffer,
    maps: Option<MapResources>,
    target: Option<TargetResources>,
    capture: Option<CaptureResources>,
    march_bind: Option<wgpu::BindGroup>,
    grade_bind: Option<wgpu::BindGroup>,
    last_march: Option<MarchParams>,
    last_critter: Option<CritterParams>,
    last_grade: Option<GradeParams>,
    pending_resource_creations: u32,
    last_diagnostics: Option<FrameDiagnostics>,
}

impl Lens {
    pub fn headless(width: u32, height: u32) -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
        let adapter =
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
                .ok()?;
        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default())).ok()?;
        Some(Self::with_device(device, queue, width, height))
    }

    pub fn with_device(device: wgpu::Device, queue: wgpu::Queue, width: u32, height: u32) -> Self {
        Self::with_format(device, queue, width, height, FRAME_FORMAT)
    }

    pub fn with_format(
        device: wgpu::Device,
        queue: wgpu::Queue,
        width: u32,
        height: u32,
        output_format: wgpu::TextureFormat,
    ) -> Self {
        let uniform = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };
        let texture = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let sampling = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let march_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lens march layout"),
            entries: &[texture(0), texture(1), sampling(2), uniform(3), uniform(4)],
        });
        let grade_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("lens grade layout"),
            entries: &[texture(0), sampling(1), texture(2), uniform(3)],
        });
        // The pose array size is the Rust cap, injected rather than written
        // twice: a drift between the two layouts is a silent misread.
        let march_source = format!(
            "const POSE_PAIRS = {};\n{}",
            MAX_CAPSULES * 2,
            include_str!("march.wgsl")
        );
        let march = pipeline(
            &device,
            "lens march",
            &march_source,
            &march_layout,
            FRAME_FORMAT,
        );
        let grade = pipeline(
            &device,
            "lens grade",
            include_str!("grade.wgsl"),
            &grade_layout,
            output_format,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lens maps"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });
        let nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lens upscale"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let make_uniform = |label, size| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let march_params = make_uniform("lens march params", size_of::<MarchParams>() as u64);
        let critter_params = make_uniform("lens critter params", size_of::<CritterParams>() as u64);
        let grade_params = make_uniform("lens grade params", size_of::<GradeParams>() as u64);
        Self {
            device,
            queue,
            output_format,
            width: width.max(1),
            height: height.max(1),
            march,
            grade,
            march_layout,
            grade_layout,
            sampler,
            nearest,
            march_params,
            critter_params,
            grade_params,
            maps: None,
            target: None,
            capture: None,
            march_bind: None,
            grade_bind: None,
            last_march: None,
            last_critter: None,
            last_grade: None,
            pending_resource_creations: 9,
            last_diagnostics: None,
        }
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn output_format(&self) -> wgpu::TextureFormat {
        self.output_format
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn last_diagnostics(&self) -> Option<FrameDiagnostics> {
        self.last_diagnostics
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        let size = (width.max(1), height.max(1));
        if size == (self.width, self.height) {
            return;
        }
        (self.width, self.height) = size;
        self.target = None;
        self.capture = None;
        self.grade_bind = None;
    }

    /// Encode march and grade into a caller-owned target. The caller submits
    /// the encoder, so this composes with netrender and other same-device work.
    pub fn encode(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        input: FrameInput<'_>,
    ) -> Result<FrameDiagnostics, LensError> {
        let started = Instant::now();
        validate_maps(input.maps)?;
        if let Some(pose) = input.pose
            && pose.capsules.len() > MAX_CAPSULES
        {
            return Err(LensError::TooManyCapsules {
                actual: pose.capsules.len(),
                maximum: MAX_CAPSULES,
            });
        }
        let mut diagnostics = FrameDiagnostics {
            resource_creations: std::mem::take(&mut self.pending_resource_creations),
            ..Default::default()
        };
        self.ensure_maps(input, &mut diagnostics)?;
        self.ensure_target(input.grade.downscale, &mut diagnostics);
        self.write_uniforms(input, &mut diagnostics);
        self.ensure_bind_groups(&mut diagnostics);
        diagnostics.cpu_prepare_us = started.elapsed().as_micros() as u64;

        let marched = &self.target.as_ref().expect("target ensured").marched_view;
        encode_pass(
            encoder,
            "lens march",
            &self.march,
            self.march_bind.as_ref().unwrap(),
            marched,
        );
        diagnostics.march_passes = 1;
        encode_pass(
            encoder,
            "lens grade",
            &self.grade,
            self.grade_bind.as_ref().unwrap(),
            target,
        );
        diagnostics.grade_passes = 1;
        self.last_diagnostics = Some(diagnostics);
        Ok(diagnostics)
    }

    /// Capture a frame through the live encode path and read back RGBA bytes.
    pub fn capture(&mut self, input: FrameInput<'_>) -> Result<Capture, LensError> {
        if self.output_format != FRAME_FORMAT {
            return Err(LensError::CaptureFormat(self.output_format));
        }
        self.ensure_capture();
        let target_view = self.capture.as_ref().unwrap().target_view.clone();
        let target_texture = self.capture.as_ref().unwrap().target.clone();
        let staging = self.capture.as_ref().unwrap().staging.clone();
        let padded = self.capture.as_ref().unwrap().padded_bytes_per_row;

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("lens capture"),
            });
        let mut diagnostics = self.encode(&mut encoder, &target_view, input)?;
        encoder.copy_texture_to_buffer(
            target_texture.as_image_copy(),
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
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = send.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|error| LensError::DevicePoll(error.to_string()))?;
        receive
            .recv()
            .map_err(|error| LensError::Readback(error.to_string()))?
            .map_err(|error| LensError::Readback(error.to_string()))?;
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
        Ok(Capture {
            width: self.width,
            height: self.height,
            pixels,
            diagnostics,
        })
    }

    /// Static-map compatibility wrapper retained for the existing receipts.
    pub fn render(&mut self, maps: &BiomeMaps, flight: &Flight, grade: &Grade) -> Vec<u8> {
        self.render_with(maps, flight, grade, None)
    }

    /// Static-map compatibility wrapper retained for the existing receipts.
    pub fn render_with(
        &mut self,
        maps: &BiomeMaps,
        flight: &Flight,
        grade: &Grade,
        pose: Option<&CritterPose>,
    ) -> Vec<u8> {
        let mut input = FrameInput::new(maps, MapRevision(0), flight, grade);
        input.pose = pose;
        self.capture(input)
            .expect("valid lens receipt input")
            .pixels
    }

    fn ensure_maps(
        &mut self,
        input: FrameInput<'_>,
        diagnostics: &mut FrameDiagnostics,
    ) -> Result<(), LensError> {
        let recreate = self
            .maps
            .as_ref()
            .is_none_or(|maps| maps.side != input.maps.side);
        if recreate {
            self.maps = Some(create_maps(&self.device, input.maps, diagnostics));
            self.march_bind = None;
            self.grade_bind = None;
            diagnostics.map_recreated = true;
        }
        let maps = self.maps.as_mut().unwrap();
        if diagnostics.map_recreated || maps.revision != input.map_revision {
            let change = if diagnostics.map_recreated {
                MapChange::Full
            } else {
                input.map_change
            };
            upload_maps(&self.queue, maps, input.maps, change, diagnostics)?;
            maps.revision = input.map_revision;
        }
        Ok(())
    }

    fn ensure_target(&mut self, downscale: u32, diagnostics: &mut FrameDiagnostics) {
        let downscale = downscale.max(1);
        let expected = (self.width, self.height, downscale);
        let ready = self
            .target
            .as_ref()
            .is_some_and(|target| (target.width, target.height, target.downscale) == expected);
        if ready {
            return;
        }
        let width = (self.width / downscale).max(1);
        let height = (self.height / downscale).max(1);
        let marched = make_target(&self.device, "lens marched", width, height, FRAME_FORMAT);
        let marched_view = marched.create_view(&Default::default());
        self.target = Some(TargetResources {
            width: self.width,
            height: self.height,
            downscale,
            _marched: marched,
            marched_view,
        });
        self.grade_bind = None;
        diagnostics.resource_creations += 1;
        diagnostics.target_recreated = true;
    }

    fn write_uniforms(&mut self, input: FrameInput<'_>, diagnostics: &mut FrameDiagnostics) {
        let march = MarchParams {
            eye: input.flight.eye,
            yaw: input.flight.yaw,
            pitch: input.flight.pitch,
            fov: input.flight.fov,
            far: input.flight.far,
            map_side: input.maps.side as f32,
        };
        write_changed(
            &self.queue,
            &self.march_params,
            &mut self.last_march,
            march,
            diagnostics,
        );
        let critter = critter_params(input.pose);
        write_changed(
            &self.queue,
            &self.critter_params,
            &mut self.last_critter,
            critter,
            diagnostics,
        );
        let grade = GradeParams {
            fog: input.grade.fog,
            fog_start: input.grade.fog_start,
            palette_len: input.grade.palette_len.min(input.maps.palette.len() as u32),
            dither: input.grade.dither,
            fog_bands: input.grade.fog_bands,
            _pad: 0.0,
        };
        write_changed(
            &self.queue,
            &self.grade_params,
            &mut self.last_grade,
            grade,
            diagnostics,
        );
    }

    fn ensure_bind_groups(&mut self, diagnostics: &mut FrameDiagnostics) {
        if self.march_bind.is_none() {
            let maps = self.maps.as_ref().unwrap();
            self.march_bind = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lens march"),
                layout: &self.march_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&maps.height_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&maps.color_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.march_params.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.critter_params.as_entire_binding(),
                    },
                ],
            }));
            diagnostics.resource_creations += 1;
            diagnostics.bind_group_rebuilds += 1;
        }
        if self.grade_bind.is_none() {
            let maps = self.maps.as_ref().unwrap();
            let target = self.target.as_ref().unwrap();
            self.grade_bind = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("lens grade"),
                layout: &self.grade_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&target.marched_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.nearest),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&maps.palette_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.grade_params.as_entire_binding(),
                    },
                ],
            }));
            diagnostics.resource_creations += 1;
            diagnostics.bind_group_rebuilds += 1;
        }
    }

    fn ensure_capture(&mut self) {
        let ready = self
            .capture
            .as_ref()
            .is_some_and(|capture| capture.width == self.width && capture.height == self.height);
        if ready {
            return;
        }
        let target = make_target(
            &self.device,
            "lens capture target",
            self.width,
            self.height,
            FRAME_FORMAT,
        );
        let target_view = target.create_view(&Default::default());
        let padded_bytes_per_row = (self.width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT)
            * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lens readback"),
            size: (padded_bytes_per_row * self.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        self.capture = Some(CaptureResources {
            width: self.width,
            height: self.height,
            target,
            target_view,
            staging,
            padded_bytes_per_row,
        });
        self.pending_resource_creations += 2;
    }
}
