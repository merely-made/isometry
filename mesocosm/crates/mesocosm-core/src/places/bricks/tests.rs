// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::Places;
use super::*;

fn world() -> (Grown, Ground) {
    let grown = Places::grown(4_242, 4, 64);
    let ground = Ground::grow(&grown, 64);
    (grown, ground)
}

#[test]
fn nest_routes_never_cross_the_resident_wall() {
    // Before the depth cap in `nest_entry`, routes drifted up to 12
    // voxels past the real ENCLOSURE bound across these seeds. (TD2b)
    let extent = 16i32;
    for seed in 1u64..500 {
        let grown = Places::grown(seed, 3, extent);
        for (nest, entry) in grown.nest_entries(extent) {
            for at in &entry.route {
                assert!(
                    at[0].abs() <= extent && at[2].abs() <= extent,
                    "nest {nest:?} route point {at:?} crosses the wall at extent {extent}"
                );
            }
        }
    }
}

#[test]
fn the_same_world_raises_the_same_ground() {
    let (_, a) = world();
    let (_, b) = world();
    assert_eq!(a, b);
    let bytes = crate::snapshot::encode(&a).unwrap();
    assert_eq!(crate::snapshot::decode::<Ground>(&bytes).unwrap(), a);
}

#[test]
fn the_ground_is_somewhere_and_walkable() {
    let (_, ground) = world();
    assert!(ground.brick_count() > 0);
    let top = ground.surface(0, 0).expect("a column at the origin");
    assert!(
        ground.stands([0, top + 1, 0], 2),
        "the surface holds you up"
    );
    assert!(
        !ground.stands([0, top - 1, 0], 2),
        "inside rock is not a stance"
    );
}

#[test]
fn nests_are_roofed_voids() {
    // The graph promised interiors. An interior is a hole with a
    // ceiling: air with solid directly above it, near the host. An
    // open pit does not count.
    let (grown, ground) = world();
    assert!(!grown.nests.is_empty(), "this seed grows nests");
    for nest in &grown.nests {
        let [x, z] = grown.places.get(nest.host).unwrap().centre;
        let mut roofed = false;
        'search: for dz in -12..=12 {
            for dx in -12..=12 {
                for y in 1..SURFACE_BAND {
                    let at = [x + dx, y, z + dz];
                    if !ground.solid(at) && ground.solid([at[0], y + 1, at[2]]) {
                        roofed = true;
                        break 'search;
                    }
                }
            }
        }
        assert!(roofed, "nest at {:?} has no roofed room", nest.host);
    }
}

#[test]
fn nest_entries_are_walkable_routes_to_the_roofed_interior() {
    let (grown, ground) = world();
    for nest in &grown.nests {
        let entry = nest_entry(&grown, 64, *nest).unwrap();
        for pair in entry.route.windows(2) {
            assert!(ground.stands(pair[0], 2), "bad entry stance: {:?}", pair[0]);
            assert_eq!(
                super::super::step(&ground, pair[0], pair[1]),
                pair[1],
                "entry step failed: {:?} -> {:?}",
                pair[0],
                pair[1]
            );
        }
        let inside = *entry.route.last().unwrap();
        assert!(ground.stands(inside, 2));
        assert!(
            ground.solid([inside[0], inside[1] + 2, inside[2]]),
            "nest at {:?} has no roof over its entry room",
            nest.host
        );
    }
}

#[test]
fn a_carve_is_a_revision_and_a_dirty_brick() {
    let (_, mut ground) = world();
    ground.drain_dirty();
    assert_eq!(ground.revision(), 0);

    let top = ground.surface(4, 4).unwrap();
    let removed = ground.carve([4, top, 4], 1);
    assert!(removed > 0);
    assert_eq!(ground.revision(), 1);
    let dirty = ground.drain_dirty();
    assert!(!dirty.is_empty());
    assert!(
        dirty.len() <= 8,
        "a radius-1 carve touches at most 8 bricks"
    );
    assert!(ground.drain_dirty().is_empty(), "drained means drained");

    // Carving air is not an edit.
    let removed = ground.carve([4, SURFACE_BAND + 40, 4], 1);
    assert_eq!(removed, 0);
    assert_eq!(ground.revision(), 1);
    assert!(ground.drain_dirty().is_empty());
}

#[test]
fn the_same_carve_replays_to_the_same_ground() {
    let (_, mut a) = world();
    let (_, mut b) = world();
    let top = a.surface(-3, 5).unwrap();
    a.carve([-3, top, 5], 2);
    b.carve([-3, top, 5], 2);
    assert_eq!(
        crate::snapshot::encode(&a).unwrap(),
        crate::snapshot::encode(&b).unwrap()
    );
}

#[test]
fn draining_projection_dirt_does_not_change_the_snapshot() {
    let (_, mut ground) = world();
    let top = ground.surface(4, 4).unwrap();
    ground.carve([4, top, 4], 1);
    let before = crate::snapshot::encode(&ground).unwrap();
    assert!(!ground.drain_dirty().is_empty());
    assert_eq!(crate::snapshot::encode(&ground).unwrap(), before);
}

#[test]
fn hills_block_sight_and_tunnels_grant_it() {
    let (_, mut ground) = world();
    // A horizontal sightline with genuinely solid ground between the
    // endpoints. Solidity is asserted at the crossing height itself,
    // because a roofed burrow can hollow a column whose surface reads
    // high (the first version of this scan walked into one).
    let mut wall = None;
    'scan: for z in -40..40 {
        for x in -40..30 {
            let (a, b) = (
                ground.surface(x, z).unwrap_or(0),
                ground.surface(x + 8, z).unwrap_or(0),
            );
            let eye = a.max(b) + 1;
            if ground.stands([x, eye, z], 1)
                && ground.stands([x + 8, eye, z], 1)
                && ground.solid([x + 4, eye, z])
            {
                wall = Some(([x, eye, z], [x + 8, eye, z]));
                break 'scan;
            }
        }
    }
    let (from, to) = wall.expect("this seed has a hill somewhere");
    assert!(!ground.sees(from, to), "a hill is opaque");

    // Bore straight through at eye height; sight follows the tunnel.
    for x in from[0]..=to[0] {
        ground.carve([x, from[1], from[2]], 1);
    }
    assert!(
        ground.sees(from, [to[0], from[1], from[2]]),
        "a tunnel is not"
    );
}
