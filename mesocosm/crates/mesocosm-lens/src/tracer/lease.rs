// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The GPU-resident atlas contract: a plain wgpu buffer range a producer
//! lends the tracer, plus the fit checks that keep a bad one out.

use modulus::BrickProjectionRevision;

use super::types::BrickRevision;

/// A GPU-resident atlas the tracer may fill its texture from without the
/// CPU seeing a voxel.
///
/// The tracer samples `texture_3d` and holds no storage buffer, because
/// it is fragment-only for downlevel reach (WebGL2 has neither compute
/// nor storage buffers). So a resident producer does not change the
/// tracer's bindings; it changes where the atlas texture's bytes come
/// from. This is deliberately a plain wgpu triple rather than a
/// producer's own view type: the buffer contract is the meeting point,
/// so the lens depends on no compute stack.
///
/// The producer owns the allocation and its lifetime. `revision` must
/// be the world revision those bytes were materialized at, so a stale
/// lease cannot be presented as current.
#[derive(Clone, Copy, Debug)]
pub struct LeasedAtlas<'a> {
    pub buffer: &'a wgpu::Buffer,
    /// Start and length of the producer allocation. Source coordinates below
    /// are relative to this range rather than forged into the buffer offset.
    pub offset: u64,
    pub size: u64,
    /// Source voxel coordinate and row/image strides inside the producer's
    /// R8 allocation. These permit one brick to be leased from a larger atlas.
    pub source_origin: [u32; 3],
    pub source_bytes_per_row: u32,
    pub source_rows_per_image: u32,
    /// Where in the atlas these voxels belong, and how many.
    pub slot_origin: [u32; 3],
    pub extent: [u32; 3],
    pub revision: BrickRevision,
    /// Selected brick projection these resident bytes materialize.
    pub projection_revision: BrickProjectionRevision,
    /// Host-issued schedule epoch at which the producer made these bytes
    /// safe for reader tenants.
    pub read_epoch: u64,
}

impl LeasedAtlas<'_> {
    /// The bytes `extent` describes, at one byte per voxel (the atlas is
    /// `R8Uint`).
    pub fn byte_len(&self) -> u64 {
        self.extent
            .into_iter()
            .try_fold(1u64, |total, axis| total.checked_mul(u64::from(axis)))
            .unwrap_or(u64::MAX)
    }

    /// Whether the strided source extent fits inside the leased range.
    ///
    /// Load-bearing rather than defensive: a producer's allocator pools
    /// many planes into one buffer, so copying an extent larger than the
    /// lease does not fault, it reads whatever plane happens to sit
    /// next in the pool and paints it into the world. Silent corruption
    /// is worse than a refusal, so an ill-fitting lease is refused.
    pub fn fits(&self) -> bool {
        if self.extent.contains(&0)
            || self.source_bytes_per_row == 0
            || self.source_rows_per_image == 0
        {
            return false;
        }
        let Some(source_x_end) = self.source_origin[0].checked_add(self.extent[0]) else {
            return false;
        };
        let Some(source_y_end) = self.source_origin[1].checked_add(self.extent[1]) else {
            return false;
        };
        if source_x_end > self.source_bytes_per_row || source_y_end > self.source_rows_per_image {
            return false;
        }
        self.source_end().is_some_and(|end| end <= self.size)
    }

    /// Whether wgpu can copy this source into the destination atlas without
    /// crossing either range or violating buffer-copy alignment.
    pub fn copyable_into(&self, atlas_extent: [u32; 3]) -> bool {
        let destination_fits = (0..3).all(|axis| {
            self.slot_origin[axis]
                .checked_add(self.extent[axis])
                .is_some_and(|end| end <= atlas_extent[axis])
        });
        let alignment = wgpu::COPY_BUFFER_ALIGNMENT;
        self.fits()
            && destination_fits
            && self.extent[0].is_multiple_of(alignment as u32)
            && self.source_bytes_per_row.is_multiple_of(alignment as u32)
            && self
                .source_start()
                .and_then(|start| self.offset.checked_add(start))
                .is_some_and(|start| start.is_multiple_of(alignment))
    }

    /// Whether this destination lease contains another atlas box completely.
    pub fn covers(&self, origin: [u32; 3], extent: [u32; 3]) -> bool {
        (0..3).all(|axis| {
            let Some(required_end) = origin[axis].checked_add(extent[axis]) else {
                return false;
            };
            let Some(leased_end) = self.slot_origin[axis].checked_add(self.extent[axis]) else {
                return false;
            };
            origin[axis] >= self.slot_origin[axis] && required_end <= leased_end
        })
    }

    fn source_start(&self) -> Option<u64> {
        let bytes_per_row = u64::from(self.source_bytes_per_row);
        let rows_per_image = u64::from(self.source_rows_per_image);
        let image_stride = bytes_per_row.checked_mul(rows_per_image)?;
        u64::from(self.source_origin[2])
            .checked_mul(image_stride)?
            .checked_add(u64::from(self.source_origin[1]).checked_mul(bytes_per_row)?)?
            .checked_add(u64::from(self.source_origin[0]))
    }

    fn source_end(&self) -> Option<u64> {
        let bytes_per_row = u64::from(self.source_bytes_per_row);
        let rows_per_image = u64::from(self.source_rows_per_image);
        let image_stride = bytes_per_row.checked_mul(rows_per_image)?;
        self.source_start()?
            .checked_add(u64::from(self.extent[2] - 1).checked_mul(image_stride)?)?
            .checked_add(u64::from(self.extent[1] - 1).checked_mul(bytes_per_row)?)?
            .checked_add(u64::from(self.extent[0]))
    }
}
