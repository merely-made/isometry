// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared-device draw adapter for rigid, greedy-meshed body volumes.
//!
//! The cache is deliberately keyed only by immutable `VolumeRef` bytes. A
//! placement, an organism's movement, scale, and its presentation tint are
//! per-frame instance data, so they do not cause static geometry uploads.

use std::collections::BTreeMap;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use mesocosm_core::{PartId, VolumeRef, Yaw};
use mesocosm_mesh::{BodyMesh, PartMesh};
use wgpu::util::DeviceExt;

use crate::geometry::{Vertex, face_shade, material_colour};

mod materials;

pub use materials::PartMaterial;

use materials::{part_appearance, valid_materials};

/// A body owned by the caller's projection. Identity remains alongside this
/// lightweight draw item in the host's attributed projection.
#[derive(Clone, Copy, Debug)]
pub struct LiveBody<'a> {
    pub mesh: &'a BodyMesh,
    /// World-space location of the body's local origin, in voxel units.
    pub origin: [f32; 3],
    /// Uniform world scale for this body.
    pub scale: f32,
    /// Linear multiplier applied after the volume material colour.
    pub tint: [f32; 3],
    /// Gives this body a restrained inspection emphasis.
    pub focused: bool,
    /// The one addressed part that receives the inspection colour.
    pub selected_part: Option<PartId>,
    /// Per-part expression summaries from the host's phenotype projection.
    /// They are per-instance data and never alter the immutable volume cache.
    pub materials: &'a [PartMaterial],
}

impl<'a> LiveBody<'a> {
    pub fn new(mesh: &'a BodyMesh, origin: [f32; 3]) -> Self {
        Self {
            mesh,
            origin,
            scale: 1.0,
            tint: [1.0; 3],
            focused: false,
            selected_part: None,
            materials: &[],
        }
    }
}

/// Inclusive cut interval in world coordinates. This is the tracer's
/// world-vertical slab, not a camera-forward near/far substitute.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipSlab {
    pub normal: [f32; 3],
    pub min: f32,
    pub max: f32,
    /// Optional fixed world volume, independent of camera orientation.
    pub bounds: Option<([f32; 3], [f32; 3])>,
}

impl ClipSlab {
    pub const fn new(normal: [f32; 3], min: f32, max: f32) -> Self {
        Self {
            normal,
            min,
            max,
            bounds: None,
        }
    }
}

/// Receipts for the last draw. Upload counters stay at zero for an unchanged
/// frame, even though it still submits its draw calls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BodyDrawStats {
    pub cached_meshes: usize,
    /// Distinct immutable volume buffers created for this call.
    pub mesh_builds: usize,
    pub mesh_uploads: usize,
    pub mesh_upload_bytes: usize,
    pub static_vertices_uploaded: usize,
    pub frame_upload_bytes: usize,
    pub instances: usize,
    pub instance_upload_bytes: usize,
    /// Rigid part placements submitted, before batching shared volumes.
    pub draw_parts: usize,
    pub draws: usize,
    pub evictions: usize,
}

/// The adapter declines a frame before recording a partial body draw when its
/// bounded immutable-volume cache cannot represent every required volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiveBodyError {
    /// A public `BodyMesh` placement did not resolve to its promised geometry.
    /// Refuse the entire call so a host can use its counted fallback.
    MissingMesh {
        volume: VolumeRef,
    },
    CacheOverflow {
        required_meshes: usize,
        capacity: usize,
    },
    InvalidBody,
    /// The caller supplied a non-finite, negative, or over-capacity process
    /// summary. Refuse rather than making a decorative claim about tissue.
    InvalidMaterials,
    InvalidClip,
}

struct CachedMesh {
    vertices: wgpu::Buffer,
    vertex_count: u32,
    last_used: u64,
}

struct CachedInstances {
    buffer: wgpu::Buffer,
    capacity: usize,
    bytes: Vec<u8>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq)]
struct Instance {
    model: [[f32; 4]; 4],
    tint: [f32; 4],
    expression: [f32; 4],
    expression_tail: [f32; 4],
}

impl Instance {
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 2,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 16,
                shader_location: 3,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 32,
                shader_location: 4,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 48,
                shader_location: 5,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 64,
                shader_location: 6,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 80,
                shader_location: 7,
                format: wgpu::VertexFormat::Float32x4,
            },
            wgpu::VertexAttribute {
                offset: 96,
                shader_location: 8,
                format: wgpu::VertexFormat::Float32x4,
            },
        ],
    };
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct FrameUniform {
    clip_from_world: [[f32; 4]; 4],
    slab_normal_min: [f32; 4],
    slab_max_enabled: [f32; 4],
    bounds_min: [f32; 4],
    bounds_max: [f32; 4],
}

/// Draws cached volume meshes into a pass that the caller owns.
pub struct LiveBodyRenderer {
    pipeline: wgpu::RenderPipeline,
    frame_buffer: wgpu::Buffer,
    frame_bind_group: wgpu::BindGroup,
    meshes: BTreeMap<[u8; 32], CachedMesh>,
    instances: BTreeMap<[u8; 32], CachedInstances>,
    max_cached_meshes: usize,
    clock: u64,
    last_frame: Option<FrameUniform>,
    last_stats: BodyDrawStats,
}

impl LiveBodyRenderer {
    /// Builds against the application's existing device. The adapter never
    /// requests a device or owns a target, depth texture, or command encoder.
    pub fn new(
        device: &wgpu::Device,
        colour_format: wgpu::TextureFormat,
        max_cached_meshes: usize,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("mesocosm live body"),
            source: wgpu::ShaderSource::Wgsl(include_str!("live_body.wgsl").into()),
        });
        let frame_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesocosm live body frame"),
            contents: bytemuck::bytes_of(&FrameUniform::zeroed()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("mesocosm live body frame layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mesocosm live body frame bind group"),
            layout: &frame_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: frame_buffer.as_entire_binding(),
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("mesocosm live body layout"),
            bind_group_layouts: &[Some(&frame_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mesocosm live body pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(Vertex::LAYOUT), Some(Instance::LAYOUT)],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: colour_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            frame_buffer,
            frame_bind_group,
            meshes: BTreeMap::new(),
            instances: BTreeMap::new(),
            max_cached_meshes: max_cached_meshes.max(1),
            clock: 0,
            last_frame: None,
            last_stats: BodyDrawStats::default(),
        }
    }

    pub fn last_stats(&self) -> BodyDrawStats {
        self.last_stats
    }

    /// Records a `Load`/`Load` pass into caller-owned display-encoded colour
    /// and `Depth32Float` attachments. The supplied matrix must be the same
    /// world-to-clip convention used by the terrain tracer.
    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        colour: &wgpu::TextureView,
        depth: &wgpu::TextureView,
        clip_from_world: [[f32; 4]; 4],
        clip_slab: Option<ClipSlab>,
        bodies: &[LiveBody<'_>],
    ) -> Result<BodyDrawStats, LiveBodyError> {
        if !clip_from_world
            .iter()
            .flatten()
            .all(|value| value.is_finite())
        {
            return Err(LiveBodyError::InvalidClip);
        }
        if let Some(slab) = clip_slab
            && (!slab.normal.iter().all(|value| value.is_finite())
                || !slab.min.is_finite()
                || !slab.max.is_finite()
                || slab.min > slab.max
                || slab.normal.iter().map(|value| value * value).sum::<f32>() <= f32::EPSILON)
        {
            return Err(LiveBodyError::InvalidClip);
        }
        for body in bodies {
            if !body.origin.iter().all(|value| value.is_finite())
                || !body.scale.is_finite()
                || body.scale <= 0.0
                || !body.tint.iter().all(|value| value.is_finite())
            {
                return Err(LiveBodyError::InvalidBody);
            }
            for placement in &body.mesh.placements {
                if body.mesh.mesh_for(placement.volume).is_none() {
                    return Err(LiveBodyError::MissingMesh {
                        volume: placement.volume,
                    });
                }
            }
            if !valid_materials(body.materials) {
                return Err(LiveBodyError::InvalidMaterials);
            }
        }
        self.clock = self.clock.wrapping_add(1);
        let mut stats = BodyDrawStats::default();
        let mut instances: BTreeMap<[u8; 32], Vec<Instance>> = BTreeMap::new();
        for body in bodies {
            for placement in &body.mesh.placements {
                let key = placement.volume.0;
                let model = model_matrix(*body, placement.yaw, placement.pivot, placement.pivot_at);
                let appearance = part_appearance(*body, placement.part);
                instances.entry(key).or_default().push(Instance {
                    model: model.to_cols_array_2d(),
                    tint: appearance.tint,
                    expression: appearance.expression,
                    expression_tail: appearance.expression_tail,
                });
            }
        }
        if instances.len() > self.max_cached_meshes {
            self.last_stats = BodyDrawStats {
                cached_meshes: self.meshes.len(),
                ..stats
            };
            return Err(LiveBodyError::CacheOverflow {
                required_meshes: instances.len(),
                capacity: self.max_cached_meshes,
            });
        }
        for body in bodies {
            for placement in &body.mesh.placements {
                let Some(part_mesh) = body.mesh.mesh_for(placement.volume) else {
                    continue;
                };
                self.cache_mesh(device, placement.volume.0, part_mesh, &mut stats);
            }
        }
        let slab = clip_slab.unwrap_or(ClipSlab::new([0.0; 3], 0.0, 0.0));
        let frame = FrameUniform {
            clip_from_world,
            bounds_min: slab
                .bounds
                .map_or([0.0; 4], |(min, _)| [min[0], min[1], min[2], 1.0]),
            bounds_max: slab
                .bounds
                .map_or([0.0; 4], |(_, max)| [max[0], max[1], max[2], 0.0]),
            slab_normal_min: [slab.normal[0], slab.normal[1], slab.normal[2], slab.min],
            slab_max_enabled: [
                slab.max,
                0.0,
                0.0,
                if clip_slab.is_some() { 1.0 } else { 0.0 },
            ],
        };
        if self.last_frame.as_ref().map(bytemuck::bytes_of) != Some(bytemuck::bytes_of(&frame)) {
            queue.write_buffer(&self.frame_buffer, 0, bytemuck::bytes_of(&frame));
            stats.frame_upload_bytes = size_of::<FrameUniform>();
            self.last_frame = Some(frame);
        }
        for (key, batch) in &instances {
            stats.instances += batch.len();
            stats.draw_parts += batch.len();
            self.update_instances(device, queue, *key, batch, &mut stats);
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("mesocosm live bodies"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: colour,
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
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.frame_bind_group, &[]);
        for (key, batch) in &instances {
            let cached = self.meshes.get(key).expect("cached before batching");
            if cached.vertex_count == 0 {
                continue;
            }
            let instance_buffer = &self.instances.get(key).expect("updated before pass").buffer;
            pass.set_vertex_buffer(0, cached.vertices.slice(..));
            pass.set_vertex_buffer(1, instance_buffer.slice(..));
            pass.draw(0..cached.vertex_count, 0..batch.len() as u32);
            stats.draws += 1;
        }
        drop(pass);
        stats.cached_meshes = self.meshes.len();
        self.last_stats = stats;
        Ok(stats)
    }

    fn cache_mesh(
        &mut self,
        device: &wgpu::Device,
        key: [u8; 32],
        mesh: &PartMesh,
        stats: &mut BodyDrawStats,
    ) {
        if let Some(cached) = self.meshes.get_mut(&key) {
            cached.last_used = self.clock;
            return;
        }
        while self.meshes.len() >= self.max_cached_meshes {
            let oldest = self
                .meshes
                .iter()
                .min_by_key(|(_, mesh)| mesh.last_used)
                .map(|(key, _)| *key)
                .expect("nonempty over capacity");
            self.meshes.remove(&oldest);
            self.instances.remove(&oldest);
            stats.evictions += 1;
        }
        let vertices = part_vertices(mesh);
        let vertex_count = vertices.len() as u32;
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mesocosm live body volume"),
            contents: if vertices.is_empty() {
                bytemuck::bytes_of(&Vertex {
                    position: [0.0; 3],
                    color: [0.0; 3],
                })
            } else {
                bytemuck::cast_slice(&vertices)
            },
            usage: wgpu::BufferUsages::VERTEX,
        });
        self.meshes.insert(
            key,
            CachedMesh {
                vertices: buffer,
                vertex_count,
                last_used: self.clock,
            },
        );
        stats.mesh_builds += 1;
        stats.mesh_uploads += 1;
        stats.mesh_upload_bytes += size_of::<Vertex>() * vertices.len();
        stats.static_vertices_uploaded += vertices.len();
    }

    fn update_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        key: [u8; 32],
        instances: &[Instance],
        stats: &mut BodyDrawStats,
    ) {
        let bytes = bytemuck::cast_slice(instances);
        let required = bytes.len().max(size_of::<Instance>());
        let replace = self
            .instances
            .get(&key)
            .is_none_or(|cached| cached.capacity < required);
        if replace {
            let capacity = required.next_power_of_two();
            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("mesocosm live body instances"),
                size: capacity as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.instances.insert(
                key,
                CachedInstances {
                    buffer,
                    capacity,
                    bytes: Vec::new(),
                },
            );
        }
        let cached = self.instances.get_mut(&key).expect("inserted or present");
        if cached.bytes != bytes {
            queue.write_buffer(&cached.buffer, 0, bytes);
            stats.instance_upload_bytes += bytes.len();
            cached.bytes.clear();
            cached.bytes.extend_from_slice(bytes);
        }
    }
}

fn model_matrix(body: LiveBody<'_>, yaw: Yaw, pivot: [i32; 3], pivot_at: [i32; 3]) -> Mat4 {
    let angle = match yaw {
        Yaw::Zero => 0.0,
        Yaw::Quarter => core::f32::consts::FRAC_PI_2,
        Yaw::Half => core::f32::consts::PI,
        Yaw::ThreeQuarter => -core::f32::consts::FRAC_PI_2,
    };
    Mat4::from_translation(glam::Vec3::from_array(body.origin))
        * Mat4::from_scale(glam::Vec3::splat(body.scale))
        * Mat4::from_translation(glam::Vec3::new(
            pivot_at[0] as f32,
            pivot_at[1] as f32,
            pivot_at[2] as f32,
        ))
        * Mat4::from_rotation_y(angle)
        * Mat4::from_translation(glam::Vec3::new(
            -pivot[0] as f32,
            -pivot[1] as f32,
            -pivot[2] as f32,
        ))
}

fn part_vertices(mesh: &PartMesh) -> Vec<Vertex> {
    let mut vertices = Vec::with_capacity(mesh.quads.len() * 6);
    for quad in &mesh.quads {
        let base = material_colour(quad.material);
        let shade = face_shade(quad.axis, quad.positive);
        let colour = [base[0] * shade, base[1] * shade, base[2] * shade];
        let corners = quad.corners();
        for index in [0, 1, 2, 0, 2, 3] {
            vertices.push(Vertex {
                position: corners[index].map(|value| value as f32),
                color: colour,
            });
        }
    }
    vertices
}

#[cfg(test)]
#[path = "live_body_tests.rs"]
mod tests;
