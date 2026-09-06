// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Paredros's Ground source binding for the shared brick ABI.

use mesocosm_core::places::Ground;
use modulus::{BrickMap, BrickMapError, BrickProjectionRevision};

pub(crate) fn from_ground(ground: &Ground) -> Result<BrickMap, BrickMapError> {
    from_ground_keys(ground, BrickProjectionRevision(0), ground.keys())
}

pub(crate) fn from_ground_keys(
    ground: &Ground,
    projection_revision: BrickProjectionRevision,
    keys: impl IntoIterator<Item = [i16; 3]>,
) -> Result<BrickMap, BrickMapError> {
    BrickMap::from_keys(projection_revision, keys, |key| {
        ground.brick_materials(key).map(|(brick, _)| brick.raw())
    })
}

/// Retargets a capacity-fixed map's selection from Ground; retained
/// bricks keep their slots and are never refetched.
pub(crate) fn retarget_from_ground(
    map: &mut BrickMap,
    ground: &Ground,
    projection_revision: BrickProjectionRevision,
    keys: impl IntoIterator<Item = [i16; 3]>,
) -> Result<modulus::RetargetDelta, BrickMapError> {
    map.retarget(projection_revision, keys, |key| {
        ground.brick_materials(key).map(|(brick, _)| brick.raw())
    })
}
