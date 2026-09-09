// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G1's projection receipt: world bricks feed the same greedy mesher
//! bodies do. One truth, two consumers, no conversion layer beyond a
//! byte copy into `Volume`'s layout.

use mesocosm_core::places::{BRICK, Ground, Places};
use mesocosm_mesh::{Volume, mesh_volume};

/// A brick's materials as a mesh volume. The brick is y-major (y, z, x);
/// `Volume::get(x, y, z)` indexes x + y*sx + z*sx*sy, so this walks the
/// brick in the volume's order rather than assuming the layouts agree.
fn volume_of(ground: &Ground, key: [i16; 3]) -> Option<Volume> {
    let (brick, _origin) = ground.brick_materials(key)?;
    let side = BRICK as u32;
    let mut voxels = Vec::with_capacity((side * side * side) as usize);
    for z in 0..side as i32 {
        for y in 0..side as i32 {
            for x in 0..side as i32 {
                voxels.push(brick.get([x, y, z]));
            }
        }
    }
    Volume::new([side, side, side], voxels).ok()
}

#[test]
fn world_bricks_mesh_like_bodies_do() {
    let grown = Places::grown(4_242, 4, 64);
    let ground = Ground::grow(&grown, 64);

    let mut meshed = 0;
    let mut quads = 0;
    for key in ground.keys() {
        let volume = volume_of(&ground, key).expect("a stored brick builds a volume");
        let mesh = mesh_volume(&volume);
        if volume.solid_count() > 0 {
            assert!(
                !mesh.quads.is_empty(),
                "brick {key:?} holds matter but meshed to nothing"
            );
            meshed += 1;
            quads += mesh.quads.len();
        }
    }
    assert!(meshed > 8, "a world is more than a few bricks: {meshed}");
    assert!(quads > meshed, "greedy quads should outnumber bricks");
}

#[test]
fn a_carve_changes_exactly_its_bricks_meshes() {
    let grown = Places::grown(4_242, 4, 64);
    let mut ground = Ground::grow(&grown, 64);
    let before: std::collections::BTreeMap<_, _> = ground
        .keys()
        .map(|key| (key, mesh_volume(&volume_of(&ground, key).unwrap()).quads))
        .collect();

    let top = ground.surface(6, -6).expect("a column");
    ground.carve([6, top, -6], 1);
    let dirty: std::collections::BTreeSet<_> = ground.drain_dirty().into_iter().collect();
    assert!(!dirty.is_empty());

    for key in ground.keys() {
        let now = mesh_volume(&volume_of(&ground, key).unwrap()).quads;
        let then = before.get(&key);
        if dirty.contains(&key) {
            continue; // May differ; the point is the clean ones may not.
        }
        assert_eq!(Some(&now), then, "clean brick {key:?} changed its mesh");
    }
}
