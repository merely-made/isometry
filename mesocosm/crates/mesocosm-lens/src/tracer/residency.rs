// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The tracer's GPU copies of a [`BrickMap`] and everything that fills them:
//! texture creation, the bind group, CPU uploads, and the leased-atlas path.

use modulus::{BrickMap, BrickProjectionRevision};

use super::types::{BrickChange, BrickDiagnostics, BrickFrameInput, BrickRevision};
use super::{BrickTracer, LeasedAtlas};

pub(super) struct ResidentMap {
    pub pointer_extent: [u32; 3],
    pub atlas_extent: [u32; 3],
    pub revision: BrickRevision,
    pub projection_revision: BrickProjectionRevision,
    pub pointer: wgpu::Texture,
    pub atlas: wgpu::Texture,
    pub bind: wgpu::BindGroup,
}

impl BrickTracer {
    pub(super) fn ensure_map(
        &mut self,
        input: BrickFrameInput<'_>,
        diagnostics: &mut BrickDiagnostics,
    ) {
        let recreate = self.map.as_ref().is_none_or(|resident| {
            resident.pointer_extent != input.map.pointer_extent()
                || resident.atlas_extent != input.map.atlas_extent()
        });
        if recreate {
            self.map = Some(self.create_map(input.map));
            diagnostics.resource_creations += 2;
            diagnostics.bind_group_rebuilds += 1;
            diagnostics.map_recreated = true;
        }
        let resident = self.map.as_mut().expect("map created");
        let projection_changed = resident.projection_revision != input.map.projection_revision();
        if resident.revision == input.revision && !projection_changed && !recreate {
            return;
        }
        diagnostics.projection_replaced = projection_changed && !recreate;
        // A leased atlas replaces the CPU upload for the voxels it covers: the
        // producer already has them resident, so they move GPU-side. The
        // pointer volume still uploads from the map, which is tiny and
        // identifies slots rather than carrying material.
        let atlas_from_lease = if let Some(leased) = input.leased_atlas {
            if leased.revision != input.revision {
                diagnostics.stale_lease_rejections += 1;
                false
            } else if leased.projection_revision != input.map.projection_revision() {
                diagnostics.projection_lease_rejections += 1;
                false
            } else if input
                .expected_read_epoch
                .is_some_and(|expected| leased.read_epoch != expected)
            {
                // The frame stated the schedule epoch it trusts; a lease from
                // any other epoch may hold bytes the producer has not yet made
                // safe, or has already moved past.
                diagnostics.epoch_lease_rejections += 1;
                false
            } else if !leased.copyable_into(resident.atlas_extent) {
                // An invalid strided range could copy a neighbouring
                // allocation out of the producer's pool or cross the atlas.
                diagnostics.misfit_lease_rejections += 1;
                false
            } else if !lease_covers_change(
                leased,
                input.map,
                input.change,
                recreate || (projection_changed && matches!(input.change, BrickChange::Full)),
            ) {
                // A valid partial lease must not suppress uploads for changed
                // slots it does not actually contain.
                diagnostics.incomplete_lease_rejections += 1;
                false
            } else {
                copy_leased_atlas(&self.device, &self.queue, &resident.atlas, leased);
                diagnostics.leased_atlas_bytes += leased.byte_len();
                diagnostics.observed_read_epoch = Some(leased.read_epoch);
                true
            }
        } else {
            false
        };

        // The CPU upload stands unless a lease actually took the atlas: a
        // refused lease must not leave the atlas unwritten, or the frame shows
        // whatever was there before.
        let atlas_from_cpu = !atlas_from_lease;
        // A projection advance that declares its changed slots is a retarget
        // over retained textures: the whole (kilobyte-scale) pointer volume
        // moves, but only the declared slots' atlas bytes do.
        let full = recreate || matches!(input.change, BrickChange::Full);
        if full {
            write_texture_3d(
                &self.queue,
                &resident.pointer,
                [0, 0, 0],
                input.map.pointer_extent(),
                4,
                bytemuck::cast_slice(input.map.pointers()),
            );
            diagnostics.brick_upload_bytes += size_of_val(input.map.pointers()) as u64;
            if atlas_from_cpu {
                write_texture_3d(
                    &self.queue,
                    &resident.atlas,
                    [0, 0, 0],
                    input.map.atlas_extent(),
                    1,
                    input.map.atlas(),
                );
                diagnostics.brick_upload_bytes += input.map.atlas().len() as u64;
            }
        } else if let BrickChange::Slots(slots) = input.change {
            if projection_changed {
                write_texture_3d(
                    &self.queue,
                    &resident.pointer,
                    [0, 0, 0],
                    input.map.pointer_extent(),
                    4,
                    bytemuck::cast_slice(input.map.pointers()),
                );
                diagnostics.brick_upload_bytes += size_of_val(input.map.pointers()) as u64;
            } else {
                for slot in slots {
                    let Some(pointer_coord) = input.map.pointer_coord(*slot) else {
                        continue;
                    };
                    let pointer = input.map.pointer_at(pointer_coord).expect("in bounds");
                    write_texture_3d(
                        &self.queue,
                        &resident.pointer,
                        pointer_coord,
                        [1, 1, 1],
                        4,
                        bytemuck::bytes_of(&pointer),
                    );
                    diagnostics.brick_upload_bytes += size_of::<u32>() as u64;
                }
            }
            if atlas_from_cpu {
                diagnostics.brick_upload_bytes +=
                    write_atlas_slot_boxes(&self.queue, &resident.atlas, input.map, slots);
            }
        }
        resident.revision = input.revision;
        resident.projection_revision = input.map.projection_revision();
    }

    fn create_map(&self, map: &BrickMap) -> ResidentMap {
        let texture = |label: &str, extent: [u32; 3], format: wgpu::TextureFormat| {
            self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: extent[0],
                    height: extent[1],
                    depth_or_array_layers: extent[2],
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            })
        };
        let pointer = texture(
            "brick pointers",
            map.pointer_extent(),
            wgpu::TextureFormat::R32Uint,
        );
        let pointer_view = pointer.create_view(&Default::default());
        let atlas = texture(
            "brick atlas",
            map.atlas_extent(),
            wgpu::TextureFormat::R8Uint,
        );
        let atlas_view = atlas.create_view(&Default::default());
        let bind = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("brick tracer map"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&pointer_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.params.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.roster.as_entire_binding(),
                },
            ],
        });
        ResidentMap {
            pointer_extent: map.pointer_extent(),
            atlas_extent: map.atlas_extent(),
            revision: BrickRevision(u64::MAX),
            projection_revision: BrickProjectionRevision(u64::MAX),
            pointer,
            atlas,
            bind,
        }
    }
}

fn lease_covers_change(
    leased: LeasedAtlas<'_>,
    map: &BrickMap,
    change: BrickChange<'_>,
    recreate: bool,
) -> bool {
    if recreate || matches!(change, BrickChange::Full) {
        return leased.covers([0; 3], map.atlas_extent());
    }
    let BrickChange::Slots(slots) = change else {
        return false;
    };
    slots.iter().all(|slot| {
        map.atlas_slot_origin(*slot)
            .is_some_and(|origin| leased.covers(origin, [8; 3]))
    })
}

/// Copy a producer's resident voxels into the atlas texture, GPU-side.
///
/// Buffer-to-texture copies require `COPY_BYTES_PER_ROW_ALIGNMENT`-byte rows
/// and a brick row is eight bytes, so the leased rows are repacked into an
/// aligned staging buffer first. That repack is device-local: the CPU still
/// never sees a voxel.
fn copy_leased_atlas(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    atlas: &wgpu::Texture,
    leased: LeasedAtlas<'_>,
) {
    let [width, height, depth] = leased.extent;
    if width == 0 || height == 0 || depth == 0 {
        return;
    }
    let aligned_row = width.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
    let rows = height * depth;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("leased atlas repack"),
    });
    let staging = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("leased atlas staging"),
        size: (aligned_row * rows) as u64,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    for z in 0..depth {
        for y in 0..height {
            let destination_row = z * height + y;
            let source_row = (leased.source_origin[2] + z) * leased.source_rows_per_image
                + leased.source_origin[1]
                + y;
            let source_offset = leased.offset
                + u64::from(source_row) * u64::from(leased.source_bytes_per_row)
                + u64::from(leased.source_origin[0]);
            encoder.copy_buffer_to_buffer(
                leased.buffer,
                source_offset,
                &staging,
                (destination_row * aligned_row) as u64,
                width as u64,
            );
        }
    }
    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer: &staging,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(aligned_row),
                rows_per_image: Some(height),
            },
        },
        wgpu::TexelCopyTextureInfo {
            texture: atlas,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: leased.slot_origin[0],
                y: leased.slot_origin[1],
                z: leased.slot_origin[2],
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: depth,
        },
    );
    queue.submit([encoder.finish()]);
}

/// Upload the named slots' atlas texels as contiguous box writes.
///
/// Slot index `i` sits at atlas cell `(i % sx, i / (sx*sz), (i/sx) % sz)`
/// (index 0 is the reserved air slot, so texture slots start at 1). A run of
/// consecutive slot indices inside one x-row is one box; a run of whole x-rows
/// inside one y-layer is one wider box. The data comes straight from the map's
/// own atlas slice with the atlas's row and image strides, so nothing is
/// gathered on the CPU. Returns the texel bytes uploaded.
fn write_atlas_slot_boxes(
    queue: &wgpu::Queue,
    atlas: &wgpu::Texture,
    map: &BrickMap,
    slots: &[u32],
) -> u64 {
    let [sx, _, sz] = map.slots();
    let [width, height, _] = map.atlas_extent();
    let data = map.atlas();
    let mut sorted: Vec<u32> = slots.to_vec();
    sorted.sort_unstable();
    sorted.dedup();

    let mut uploaded = 0u64;
    let mut write_box = |origin_slots: [u32; 3], extent_slots: [u32; 3]| {
        let origin = origin_slots.map(|axis| axis * 8);
        let extent = extent_slots.map(|axis| axis * 8);
        let offset = ((origin[2] * height + origin[1]) * width + origin[0]) as u64;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: atlas,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: origin[0],
                    y: origin[1],
                    z: origin[2],
                },
                aspect: wgpu::TextureAspect::All,
            },
            &data[offset as usize..],
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width: extent[0],
                height: extent[1],
                depth_or_array_layers: extent[2],
            },
        );
        uploaded += u64::from(extent[0]) * u64::from(extent[1]) * u64::from(extent[2]);
    };

    let mut runs = sorted.iter().peekable();
    while let Some(&start) = runs.next() {
        let mut end = start;
        while runs.peek().is_some_and(|&&next| next == end + 1) {
            end = *runs.next().expect("peeked");
        }
        // The run [start, end] of texture slot spots, split at row and layer
        // boundaries into boxes.
        let mut at = start - 1;
        let last = end - 1;
        while at <= last {
            let x = at % sx;
            let z = (at / sx) % sz;
            let y = at / (sx * sz);
            let remaining = last - at + 1;
            if x == 0 && remaining >= sx {
                let rows_left_in_layer = sz - z;
                let rows = (remaining / sx).min(rows_left_in_layer);
                write_box([0, y, z], [sx, 1, rows]);
                at += rows * sx;
            } else {
                let run = (sx - x).min(remaining);
                write_box([x, y, z], [run, 1, 1]);
                at += run;
            }
        }
    }
    uploaded
}

fn write_texture_3d(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    origin: [u32; 3],
    extent: [u32; 3],
    bytes_per_texel: u32,
    data: &[u8],
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: origin[0],
                y: origin[1],
                z: origin[2],
            },
            aspect: wgpu::TextureAspect::All,
        },
        data,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(extent[0] * bytes_per_texel),
            rows_per_image: Some(extent[1]),
        },
        wgpu::Extent3d {
            width: extent[0],
            height: extent[1],
            depth_or_array_layers: extent[2],
        },
    );
}
