// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A retained, presentation-only view of [`Ground`].
//!
//! A pointer volume identifies a brick slot for each world brick; an R8 atlas
//! holds its dense material voxels. Slot zero means air. This is deliberately
//! a copy of world truth: it can be rebuilt or uploaded at any cadence without
//! becoming a second world representation.

use std::collections::{BTreeMap, BTreeSet};

use mesocosm_core::places::{BRICK, Ground};

const SLOTS_X: u32 = 16;
const SLOTS_Z: u32 = 16;
const MAX_SLOTS_Y: u32 = 8;
const MAX_BRICKS: usize = (SLOTS_X * SLOTS_Z * MAX_SLOTS_Y - 1) as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrickMapError {
    TooManyBricks { actual: usize, maximum: usize },
    GroundShapeChanged,
}

impl std::fmt::Display for BrickMapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooManyBricks { actual, maximum } => {
                write!(
                    f,
                    "ground has {actual} bricks; this tracer atlas admits {maximum}"
                )
            }
            Self::GroundShapeChanged => write!(f, "ground gained or lost a brick slot"),
        }
    }
}

impl std::error::Error for BrickMapError {}

/// Dense material data arranged for a GPU `texture_3d<u32>` pointer and an
/// `texture_3d<u32>` material atlas.
#[derive(Clone, Debug)]
pub struct BrickMap {
    origin: [i16; 3],
    pointer_extent: [u32; 3],
    slots: [u32; 3],
    pointers: Vec<u32>,
    atlas: Vec<u8>,
    key_slots: BTreeMap<[i16; 3], u32>,
}

impl BrickMap {
    pub fn from_ground(ground: &Ground) -> Result<Self, BrickMapError> {
        Self::from_ground_keys(ground, ground.keys())
    }

    /// Builds a bounded presentation map from selected world bricks.
    ///
    /// Selection is presentation policy rather than world authority. Keys are
    /// deduplicated and ordered here so camera traversal cannot make slot
    /// assignment nondeterministic. A missing key is refused rather than
    /// silently presented as air.
    pub fn from_ground_keys(
        ground: &Ground,
        keys: impl IntoIterator<Item = [i16; 3]>,
    ) -> Result<Self, BrickMapError> {
        let keys: Vec<_> = keys
            .into_iter()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        if keys.len() > MAX_BRICKS {
            return Err(BrickMapError::TooManyBricks {
                actual: keys.len(),
                maximum: MAX_BRICKS,
            });
        }
        let (origin, pointer_extent) = bounds(&keys);
        let slot_count = keys.len() as u32 + 1; // zero is the air sentinel.
        let slots = [
            SLOTS_X,
            slot_count.div_ceil(SLOTS_X * SLOTS_Z).max(1),
            SLOTS_Z,
        ];
        let pointer_len = pointer_extent.iter().product::<u32>() as usize;
        let atlas_extent = atlas_extent(slots);
        let atlas_len = atlas_extent.iter().product::<u32>() as usize;
        let key_slots = keys
            .iter()
            .enumerate()
            .map(|(index, key)| (*key, index as u32 + 1))
            .collect();
        let mut map = Self {
            origin,
            pointer_extent,
            slots,
            pointers: vec![0; pointer_len],
            atlas: vec![0; atlas_len],
            key_slots,
        };
        map.refresh(ground, keys)?;
        Ok(map)
    }

    /// Copies only changed world bricks and returns their stable atlas slots.
    /// The caller uses those slots for narrow GPU updates.
    pub fn refresh(
        &mut self,
        ground: &Ground,
        changed: impl IntoIterator<Item = [i16; 3]>,
    ) -> Result<Vec<u32>, BrickMapError> {
        let mut slots = BTreeSet::new();
        for key in changed {
            let Some(&slot) = self.key_slots.get(&key) else {
                return Err(BrickMapError::GroundShapeChanged);
            };
            let Some((brick, _)) = ground.brick_materials(key) else {
                return Err(BrickMapError::GroundShapeChanged);
            };
            let pointer = self.pointer_index(key);
            self.pointers[pointer] = if brick.is_empty() { 0 } else { slot };
            self.write_slot(slot, brick.raw());
            slots.insert(slot);
        }
        Ok(slots.into_iter().collect())
    }

    pub fn origin(&self) -> [i16; 3] {
        self.origin
    }

    pub fn pointer_extent(&self) -> [u32; 3] {
        self.pointer_extent
    }

    pub fn atlas_extent(&self) -> [u32; 3] {
        atlas_extent(self.slots)
    }

    pub fn slots(&self) -> [u32; 3] {
        self.slots
    }

    pub fn pointers(&self) -> &[u32] {
        &self.pointers
    }

    pub fn pointer_at(&self, coord: [u32; 3]) -> Option<u32> {
        let [x, y, z] = coord;
        let [width, height, depth] = self.pointer_extent;
        (x < width && y < height && z < depth)
            .then(|| self.pointers[(z * height * width + y * width + x) as usize])
    }

    pub fn atlas(&self) -> &[u8] {
        &self.atlas
    }

    pub fn pointer_coord(&self, slot: u32) -> Option<[u32; 3]> {
        self.key_slots
            .iter()
            .find_map(|(key, found)| (*found == slot).then(|| self.key_coord(*key)))
    }

    /// Bytes for one atlas slot in `wgpu::Queue::write_texture` order.
    pub fn slot_texels(&self, slot: u32) -> Option<Vec<u8>> {
        let [base_x, base_y, base_z] = self.atlas_slot_origin(slot)?;
        let [width, height, _] = self.atlas_extent();
        let mut out = Vec::with_capacity((BRICK * BRICK * BRICK) as usize);
        for z in 0..BRICK as u32 {
            for y in 0..BRICK as u32 {
                for x in 0..BRICK as u32 {
                    out.push(
                        self.atlas[((base_z + z) * height * width
                            + (base_y + y) * width
                            + base_x
                            + x) as usize],
                    );
                }
            }
        }
        Some(out)
    }

    pub fn material_at(&self, at: [i32; 3]) -> u8 {
        let key = at.map(|axis| axis.div_euclid(BRICK) as i16);
        let local = at.map(|axis| axis.rem_euclid(BRICK) as u32);
        let Some(&slot) = self.key_slots.get(&key) else {
            return 0;
        };
        if self.pointers[self.pointer_index(key)] == 0 {
            return 0;
        }
        let Some([base_x, base_y, base_z]) = self.atlas_slot_origin(slot) else {
            return 0;
        };
        let [width, height, _] = self.atlas_extent();
        self.atlas[((base_z + local[2]) * height * width
            + (base_y + local[1]) * width
            + base_x
            + local[0]) as usize]
    }

    fn key_coord(&self, key: [i16; 3]) -> [u32; 3] {
        [0, 1, 2].map(|axis| (key[axis] - self.origin[axis]) as u32)
    }

    fn pointer_index(&self, key: [i16; 3]) -> usize {
        let [x, y, z] = self.key_coord(key);
        let [width, height, _] = self.pointer_extent;
        (z * height * width + y * width + x) as usize
    }

    pub fn atlas_slot_origin(&self, slot: u32) -> Option<[u32; 3]> {
        if slot == 0 || slot as usize > self.key_slots.len() {
            return None;
        }
        let index = slot - 1;
        let x = index % self.slots[0];
        let z = (index / self.slots[0]) % self.slots[2];
        let y = index / (self.slots[0] * self.slots[2]);
        Some([x * BRICK as u32, y * BRICK as u32, z * BRICK as u32])
    }

    fn write_slot(&mut self, slot: u32, raw: &[u8]) {
        let [base_x, base_y, base_z] = self.atlas_slot_origin(slot).expect("assigned slot");
        let [width, height, _] = self.atlas_extent();
        for z in 0..BRICK as u32 {
            for y in 0..BRICK as u32 {
                for x in 0..BRICK as u32 {
                    let source = ((y * BRICK as u32 + z) * BRICK as u32 + x) as usize;
                    let destination =
                        ((base_z + z) * height * width + (base_y + y) * width + base_x + x)
                            as usize;
                    self.atlas[destination] = raw[source];
                }
            }
        }
    }
}

fn bounds(keys: &[[i16; 3]]) -> ([i16; 3], [u32; 3]) {
    let mut min = [0; 3];
    let mut max = [0; 3];
    if let Some(first) = keys.first() {
        min = *first;
        max = *first;
        for key in keys {
            for axis in 0..3 {
                min[axis] = min[axis].min(key[axis]);
                max[axis] = max[axis].max(key[axis]);
            }
        }
    }
    let extent = [0, 1, 2].map(|axis| (max[axis] - min[axis] + 1) as u32);
    (min, extent)
}

fn atlas_extent(slots: [u32; 3]) -> [u32; 3] {
    slots.map(|axis| axis * BRICK as u32)
}

#[cfg(test)]
mod tests {
    use mesocosm_core::places::{Ground, Places};

    use super::*;

    fn ground() -> Ground {
        let grown = Places::grown(4_242, 4, 64);
        Ground::grow(&grown, 64)
    }

    #[test]
    fn every_ground_voxel_has_one_atlas_reading() {
        let ground = ground();
        let map = BrickMap::from_ground(&ground).unwrap();
        for key in ground.keys() {
            let (brick, origin) = ground.brick_materials(key).unwrap();
            for z in 0..BRICK {
                for y in 0..BRICK {
                    for x in 0..BRICK {
                        assert_eq!(
                            map.material_at([origin[0] + x, origin[1] + y, origin[2] + z]),
                            brick.get([x, y, z]),
                            "brick {key:?} local [{x}, {y}, {z}]"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn a_carve_rewrites_only_its_assigned_slots() {
        let mut ground = ground();
        let mut map = BrickMap::from_ground(&ground).unwrap();
        let top = ground.surface(4, 4).unwrap();
        assert!(ground.carve([4, top, 4], 1) > 0);
        let dirty = ground.drain_dirty();
        let changed = map.refresh(&ground, dirty.clone()).unwrap();
        assert_eq!(changed.len(), dirty.len());
        for key in ground.keys() {
            let (brick, origin) = ground.brick_materials(key).unwrap();
            assert_eq!(
                map.material_at(origin),
                brick.get([0, 0, 0]),
                "brick {key:?} stayed coherent"
            );
        }
    }

    #[test]
    fn a_selected_working_set_is_deterministic_and_outside_is_air() {
        let ground = ground();
        let selected: Vec<_> = ground
            .keys()
            .filter(|key| key[0].abs() <= 2 && key[2].abs() <= 2)
            .collect();
        let mut reversed = selected.clone();
        reversed.reverse();
        reversed.extend(selected.iter().copied());

        let a = BrickMap::from_ground_keys(&ground, selected.clone()).unwrap();
        let b = BrickMap::from_ground_keys(&ground, reversed).unwrap();
        assert_eq!(a.origin(), b.origin());
        assert_eq!(a.pointer_extent(), b.pointer_extent());
        assert_eq!(a.pointers(), b.pointers());
        assert_eq!(a.atlas(), b.atlas());

        for key in &selected {
            let (brick, origin) = ground.brick_materials(*key).unwrap();
            assert_eq!(a.material_at(origin), brick.get([0, 0, 0]));
        }
        let outside = ground
            .keys()
            .find(|key| !selected.contains(key))
            .expect("the working set excludes some ground");
        let (_, origin) = ground.brick_materials(outside).unwrap();
        assert_eq!(a.material_at(origin), 0);
    }

    #[test]
    fn a_selected_working_set_refuses_a_missing_brick() {
        let ground = ground();
        assert!(matches!(
            BrickMap::from_ground_keys(&ground, [[i16::MAX; 3]]),
            Err(BrickMapError::GroundShapeChanged)
        ));
    }
}
