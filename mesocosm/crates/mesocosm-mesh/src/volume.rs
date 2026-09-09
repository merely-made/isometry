// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Voxel volumes and where they come from.
//!
//! The core carries [`VolumeRef`] content addresses and never the voxels
//! themselves, so a projection resolves them through a [`VolumeSource`]. That
//! keeps the portable body document free of the bytes a particular renderer
//! wants, and it is what lets one part's mesh be built once and reused
//! wherever that part appears.

use std::collections::BTreeMap;

use mesocosm_core::VolumeRef;
use serde::{Deserialize, Serialize};

/// A part's occupancy grid. `0` is empty; any other value is a material id
/// that a projection maps to a colour or palette entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Volume {
    pub size: [u32; 3],
    voxels: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VolumeError {
    /// `size` does not match the number of voxels supplied.
    SizeMismatch { expected: usize, got: usize },
}

impl Volume {
    pub fn new(size: [u32; 3], voxels: Vec<u8>) -> Result<Self, VolumeError> {
        let expected = size.iter().map(|d| *d as usize).product::<usize>();
        if voxels.len() != expected {
            return Err(VolumeError::SizeMismatch {
                expected,
                got: voxels.len(),
            });
        }
        Ok(Self { size, voxels })
    }

    /// A solid box of one material.
    pub fn solid(size: [u32; 3], material: u8) -> Self {
        let count = size.iter().map(|d| *d as usize).product::<usize>();
        Self {
            size,
            voxels: vec![material; count],
        }
    }

    pub fn empty(size: [u32; 3]) -> Self {
        Self::solid(size, 0)
    }

    /// Consumes the volume, yielding its cells in `x + y * sx + z * sx * sy`
    /// order. Used by the interchange profile, which carries a flat array
    /// rather than this type so a foreign reader needs none of this crate.
    pub fn into_voxels(self) -> Vec<u8> {
        self.voxels
    }

    /// Number of stored cells, including empty cells. A deserialised volume
    /// must still agree with its dimensions before a resolver can index it.
    pub fn voxel_count(&self) -> usize {
        self.voxels.len()
    }

    fn index(&self, x: u32, y: u32, z: u32) -> usize {
        (x + y * self.size[0] + z * self.size[0] * self.size[1]) as usize
    }

    pub fn get(&self, x: u32, y: u32, z: u32) -> u8 {
        if x >= self.size[0] || y >= self.size[1] || z >= self.size[2] {
            return 0;
        }
        self.voxels[self.index(x, y, z)]
    }

    pub fn set(&mut self, x: u32, y: u32, z: u32, material: u8) {
        if x >= self.size[0] || y >= self.size[1] || z >= self.size[2] {
            return;
        }
        let i = self.index(x, y, z);
        self.voxels[i] = material;
    }

    /// Reads by signed coordinate, treating anything outside as empty. The
    /// mesher uses this so a face on the boundary is visible.
    pub fn get_signed(&self, coord: [i64; 3]) -> u8 {
        if coord.iter().any(|c| *c < 0) {
            return 0;
        }
        self.get(coord[0] as u32, coord[1] as u32, coord[2] as u32)
    }

    pub fn is_empty(&self) -> bool {
        self.voxels.iter().all(|v| *v == 0)
    }

    pub fn solid_count(&self) -> usize {
        self.voxels.iter().filter(|v| **v != 0).count()
    }
}

/// Resolves the volumes a body's parts refer to.
pub trait VolumeSource {
    fn volume(&self, reference: VolumeRef) -> Option<&Volume>;
}

/// An in-memory source. Ordered so iteration never depends on hashing.
#[derive(Clone, Debug, Default)]
pub struct VolumeMap {
    volumes: BTreeMap<[u8; 32], Volume>,
}

impl VolumeMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, reference: VolumeRef, volume: Volume) {
        self.volumes.insert(reference.0, volume);
    }

    pub fn len(&self) -> usize {
        self.volumes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.volumes.is_empty()
    }
}

impl VolumeSource for VolumeMap {
    fn volume(&self, reference: VolumeRef) -> Option<&Volume> {
        self.volumes.get(&reference.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solid_volume_is_fully_occupied() {
        let v = Volume::solid([2, 3, 4], 7);
        assert_eq!(v.solid_count(), 24);
        assert_eq!(v.get(1, 2, 3), 7);
        assert!(!v.is_empty());
    }

    #[test]
    fn out_of_bounds_reads_as_empty() {
        let v = Volume::solid([2, 2, 2], 1);
        assert_eq!(v.get(9, 0, 0), 0);
        assert_eq!(v.get_signed([-1, 0, 0]), 0);
        assert_eq!(v.get_signed([0, 0, 0]), 1);
    }

    #[test]
    fn size_and_voxels_must_agree() {
        let err = Volume::new([2, 2, 2], vec![1; 7]).unwrap_err();
        assert_eq!(
            err,
            VolumeError::SizeMismatch {
                expected: 8,
                got: 7
            }
        );
    }

    #[test]
    fn map_resolves_by_reference() {
        let mut map = VolumeMap::new();
        let r = VolumeRef::from_tag(3);
        map.insert(r, Volume::solid([1, 1, 1], 2));
        assert!(map.volume(r).is_some());
        assert!(map.volume(VolumeRef::from_tag(4)).is_none());
    }
}
