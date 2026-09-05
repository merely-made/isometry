// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's [`Ground`] adapter for the shared sparse-brick ABI.
//!
//! The platform crate owns pointer/atlas layout and traversal. This module
//! owns the product-specific decision that a Ground brick's raw material
//! bytes populate that presentation view.

use std::collections::BTreeMap;
use std::ops::Deref;

use mesocosm_core::places::{BRICK, Ground};
use modulus::BrickMap as SharedBrickMap;

pub use modulus::{BrickMapError, BrickProjectionRevision, RetargetDelta};

/// A Ground-backed adapter over the product-neutral brick map.
#[derive(Clone, Debug)]
pub struct BrickMap(SharedBrickMap);

impl BrickMap {
    pub fn from_ground(ground: &Ground) -> Result<Self, BrickMapError> {
        Self::from_ground_keys(ground, BrickProjectionRevision(0), ground.keys())
    }

    /// Builds a bounded presentation map at an explicit product projection
    /// revision. Selection remains Paredros or Mesocosm policy.
    pub fn from_ground_keys(
        ground: &Ground,
        projection_revision: BrickProjectionRevision,
        keys: impl IntoIterator<Item = [i16; 3]>,
    ) -> Result<Self, BrickMapError> {
        SharedBrickMap::from_keys(projection_revision, keys, |key| {
            ground.brick_materials(key).map(|(brick, _)| brick.raw())
        })
        .map(Self)
    }

    /// Builds a presentation-only cropped map without changing Ground.
    ///
    /// `keep` receives each material's world coordinate. Rejected materials
    /// become air, and a brick that becomes all air has no shared-map slot.
    pub fn from_ground_filtered(
        ground: &Ground,
        projection_revision: BrickProjectionRevision,
        mut keep: impl FnMut([i32; 3], u8) -> bool,
    ) -> Result<Self, BrickMapError> {
        let mut retained = BTreeMap::new();
        for key in ground.keys() {
            let Some((brick, origin)) = ground.brick_materials(key) else {
                continue;
            };
            let mut materials = brick.raw().to_vec();
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        let index = ((y * BRICK + z) * BRICK + x) as usize;
                        let material = materials[index];
                        if !keep([origin[0] + x, origin[1] + y, origin[2] + z], material) {
                            materials[index] = 0;
                        }
                    }
                }
            }
            if materials.iter().any(|material| *material != 0) {
                retained.insert(key, materials);
            }
        }
        SharedBrickMap::from_keys(projection_revision, retained.keys().copied(), |key| {
            retained.get(&key).map(Vec::as_slice)
        })
        .map(Self)
    }

    /// Copies changed Ground bricks into their stable shared-ABI slots.
    pub fn refresh(
        &mut self,
        ground: &Ground,
        changed: impl IntoIterator<Item = [i16; 3]>,
    ) -> Result<Vec<u32>, BrickMapError> {
        self.0.refresh(changed, |key| {
            ground.brick_materials(key).map(|(brick, _)| brick.raw())
        })
    }

    /// An empty capacity-fixed map whose extents never change; see
    /// [`modulus::BrickMap::with_capacity`].
    pub fn with_capacity(
        projection_revision: BrickProjectionRevision,
        capacity_rows: u32,
        pointer_extent: [u32; 3],
    ) -> Result<Self, BrickMapError> {
        SharedBrickMap::with_capacity(projection_revision, capacity_rows, pointer_extent).map(Self)
    }

    /// Replaces the selection from Ground while retained bricks keep their
    /// slots; see [`modulus::BrickMap::retarget`].
    pub fn retarget(
        &mut self,
        ground: &Ground,
        projection_revision: BrickProjectionRevision,
        keys: impl IntoIterator<Item = [i16; 3]>,
    ) -> Result<RetargetDelta, BrickMapError> {
        self.0.retarget(projection_revision, keys, |key| {
            ground.brick_materials(key).map(|(brick, _)| brick.raw())
        })
    }

    pub fn shared(&self) -> &SharedBrickMap {
        &self.0
    }
}

impl Deref for BrickMap {
    type Target = SharedBrickMap;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use mesocosm_core::places::{BRICK, Ground, Places};

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

        let a = BrickMap::from_ground_keys(&ground, BrickProjectionRevision(0), selected.clone())
            .unwrap();
        let b = BrickMap::from_ground_keys(&ground, BrickProjectionRevision(0), reversed).unwrap();
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
            BrickMap::from_ground_keys(&ground, BrickProjectionRevision(0), [[i16::MAX; 3]],),
            Err(BrickMapError::MissingBrick { .. })
        ));
    }

    #[test]
    fn filtered_map_crops_materials_without_changing_ground() {
        let ground = ground();
        let before = ground.clone();
        let revision = ground.revision();
        let retained = ground
            .keys()
            .find_map(|key| {
                let (brick, origin) = ground.brick_materials(key)?;
                for y in 0..BRICK {
                    for z in 0..BRICK {
                        for x in 0..BRICK {
                            let material = brick.get([x, y, z]);
                            if material != 0 {
                                return Some((
                                    [origin[0] + x, origin[1] + y, origin[2] + z],
                                    material,
                                ));
                            }
                        }
                    }
                }
                None
            })
            .expect("generated ground has a solid voxel");
        let map = BrickMap::from_ground_filtered(&ground, BrickProjectionRevision(7), |at, _| {
            at == retained.0
        })
        .unwrap();

        assert_eq!(map.material_at(retained.0), retained.1);
        assert_eq!(
            map.material_at([retained.0[0] + 1, retained.0[1], retained.0[2]]),
            0
        );
        assert_eq!(ground, before);
        assert_eq!(ground.revision(), revision);
    }
}
