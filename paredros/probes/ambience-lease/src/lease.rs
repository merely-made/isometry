// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The producer half: a resident padded-3D position buffer that
//! explicit-regime kernels advance, plus the adapter dispatch that
//! publishes it into renderling's slab.
//!
//! This is mere's resident-graph shape copied, not shared, per the
//! plan: sharing is what P4 promotion is for, and the copy is what
//! makes the neutrality claim testable at all. If the contract only
//! worked because both sides were the same code, it would prove
//! nothing.

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Params {
    n: u32,
    dt: f32,
    time: f32,
    swirl: f32,
    updraft: f32,
    extent: f32,
    transform_base: u32,
    transform_stride: u32,
}

/// What a consumer must be told to read the lease: the plan's
/// `SpatialBufferLease` in its P3 form.
pub struct Lease {
    pub count: u32,
    pub extent: f32,
    /// Padded 3D, `vec4f` stride, xyz meaningful.
    pub stride_bytes: u32,
}

pub struct Ambience {
    device: wgpu::Device,
    queue: wgpu::Queue,
    n: u32,
    params: wgpu::Buffer,
    positions: wgpu::Buffer,
    velocities: wgpu::Buffer,
    bind: Option<wgpu::BindGroup>,
    layout: wgpu::BindGroupLayout,
    drift: wgpu::ComputePipeline,
    publish: wgpu::ComputePipeline,
    extent: f32,
    time: f32,
}

/// Deterministic scatter, seeded: the same cloud every run.
fn scatter(n: usize, extent: f32) -> Vec<[f32; 4]> {
    let mut state = 0x2026_0813u64;
    let mut next = move || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as f32 / u32::MAX as f32) * 2.0 - 1.0
    };
    (0..n)
        .map(|_| [next() * extent, next() * extent, next() * extent, 0.0])
        .collect()
}

impl Ambience {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, n: u32, extent: f32) -> Self {
        use wgpu::util::DeviceExt;

        let positions = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ambience positions"),
            contents: bytemuck::cast_slice(&scatter(n as usize, extent)),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
        let velocities = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("ambience velocities"),
            contents: bytemuck::cast_slice(&vec![[0.0f32; 4]; n as usize]),
            usage: wgpu::BufferUsages::STORAGE,
        });
        let params = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ambience params"),
            size: std::mem::size_of::<Params>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ambience kernels"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ambience.wgsl").into()),
        });

        let entries: Vec<wgpu::BindGroupLayoutEntry> = (0..4u32)
            .map(|binding| wgpu::BindGroupLayoutEntry {
                binding,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: if binding == 0 {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage { read_only: false }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            })
            .collect();
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ambience layout"),
            entries: &entries,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ambience pipelines"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = |entry: &str| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some(entry),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some(entry),
                compilation_options: Default::default(),
                cache: None,
            })
        };

        Self {
            device: device.clone(),
            queue: queue.clone(),
            n,
            params,
            positions,
            velocities,
            bind: None,
            layout,
            drift: pipeline("drift"),
            publish: pipeline("publish"),
            extent,
            time: 0.0,
        }
    }

    pub fn lease(&self) -> Lease {
        Lease {
            count: self.n,
            extent: self.extent,
            stride_bytes: 16,
        }
    }

    /// Bind the consumer's destination: renderling's geometry slab.
    /// Deferred because the slab buffer does not exist until the stage
    /// has committed, and it is replaced when the allocator grows,
    /// which is the epoch problem the lease names.
    pub fn attach_slab(&mut self, slab: &wgpu::Buffer) {
        self.bind = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ambience bind"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.positions.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.velocities.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: slab.as_entire_binding(),
                },
            ],
        }));
    }

    /// One ambience frame: drift, then publish into the consumer's
    /// slab. Both dispatches in one pass on the shared queue, so the
    /// renderer's draw sees this frame's positions by submission order
    /// alone.
    pub fn step(&mut self, dt: f32, transform_base: u32, transform_stride: u32) {
        self.time += dt;
        let params = Params {
            n: self.n,
            dt,
            time: self.time,
            swirl: 22.0,
            updraft: 5.0,
            extent: self.extent,
            transform_base,
            transform_stride,
        };
        self.queue
            .write_buffer(&self.params, 0, bytemuck::bytes_of(&params));

        let bind = self.bind.as_ref().expect("attach_slab before step");
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ambience frame"),
            });
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("ambience"),
                timestamp_writes: None,
            });
            let groups = self.n.div_ceil(256);
            pass.set_bind_group(0, bind, &[]);
            pass.set_pipeline(&self.drift);
            pass.dispatch_workgroups(groups, 1, 1);
            pass.set_pipeline(&self.publish);
            pass.dispatch_workgroups(groups, 1, 1);
        }
        self.queue.submit([encoder.finish()]);
    }

    /// One-time diagnostic readback of the resident positions, for the
    /// receipt that renderling drew *these* motes. Outside the frame
    /// discipline, like mere's bounds read.
    pub fn read_positions_once(&self) -> Vec<[f32; 4]> {
        let size = (self.n as u64) * 16;
        let staging = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("positions staging"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("positions read"),
            });
        encoder.copy_buffer_to_buffer(&self.positions, 0, &staging, 0, size);
        self.queue.submit([encoder.finish()]);
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .expect("device poll");
        rx.recv().expect("map channel").expect("positions map");
        let data = slice.get_mapped_range();
        let out: Vec<[f32; 4]> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        staging.unmap();
        out
    }
}
