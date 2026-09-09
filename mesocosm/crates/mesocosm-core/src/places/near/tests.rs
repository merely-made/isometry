// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::{Places, SURFACE_BAND};
use super::*;
use crate::body::{Attachment, BodyDocument, Provenance, SpeciesId, VolumeRef, Yaw};
use crate::world::{ENCLOSURE, PLACE_SALT, PLACE_SIDE};

fn ground() -> Ground {
    let grown = Places::grown(4_242, 4, 64);
    Ground::grow(&grown, 64)
}

fn a_stance(ground: &Ground) -> [i32; 3] {
    for z in -40..40 {
        for x in -40..40 {
            if let Some(top) = ground.surface(x, z) {
                let at = [x, top + 1, z];
                if ground.stands(at, WALKER_HEIGHT) {
                    return at;
                }
            }
        }
    }
    unreachable!("a world with no footing");
}

#[test]
fn live_anatomy_decides_the_turning_cross_section() {
    let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1]);
    let compact = WalkerShape::from_aabb(body.aabb());
    assert_eq!((compact.radius(), compact.height()), (0, 1));

    let plate = body
        .attach(
            VolumeRef::from_tag(2),
            50,
            [3, 1, 3],
            Attachment {
                parent: body.root,
                offset: [0, 2, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    let broad = WalkerShape::from_aabb(body.aabb());
    assert_eq!(broad.radius(), 1);

    assert_eq!(body.sever(plate), vec![plate]);
    assert_eq!(WalkerShape::from_aabb(body.aabb()), compact);

    let stalk = body
        .attach(
            VolumeRef::from_tag(3),
            1,
            [1, 5, 1],
            Attachment {
                parent: body.root,
                offset: [0, 6, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    let tall = WalkerShape::from_aabb(body.aabb());
    assert_eq!((tall.radius(), tall.height()), (0, 3));
    assert_eq!(compact.sight_point([4, 7, 9]), [4, 8, 9]);
    assert_eq!(tall.sight_point([4, 7, 9]), [4, 9, 9]);

    assert_eq!(body.sever(stalk), vec![stalk]);
    assert_eq!(WalkerShape::from_aabb(body.aabb()), compact);
}

#[test]
fn one_generated_burrow_admits_a_compact_body_and_excludes_a_broad_one() {
    let grown = Places::grown(PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let ground = Ground::grow(&grown, ENCLOSURE);
    let route = grown
        .nest_entries(ENCLOSURE)
        .next()
        .expect("seed 0 grows a burrow entry")
        .1
        .route;
    let compact = WalkerShape::STANDARD;
    let broad = WalkerShape::from_aabb(Aabb {
        min: [-3, -2, -3],
        max: [3, 2, 3],
    });
    let threshold = route[1];

    assert!(compact.stands(&ground, threshold));
    assert_eq!(step_for(&ground, compact, route[0], threshold), threshold);
    assert_eq!(broad.radius(), 1);
    assert!(
        !broad.stands(&ground, threshold),
        "the generated one-voxel threshold unexpectedly admits a broad body"
    );
}

#[test]
fn a_step_never_ends_inside_rock_and_never_teleports() {
    let ground = ground();
    let mut at = a_stance(&ground);
    // March hard toward a far corner; every step must stay legal.
    for _ in 0..120 {
        let next = step(&ground, at, [60, at[1], 60]);
        assert!(
            (next[0] - at[0]).abs() <= 1 && (next[2] - at[2]).abs() <= 1,
            "teleported: {at:?} -> {next:?}"
        );
        assert!(
            ground.stands(next, WALKER_HEIGHT),
            "ended unstandable at {next:?}"
        );
        at = next;
    }
}

#[test]
fn walls_slide_and_ledges_climb() {
    let ground = ground();
    let start = a_stance(&ground);
    // Wherever the terrain blocks a heading, the step still makes
    // progress or holds; across a long march it must cover ground.
    let mut at = start;
    for _ in 0..80 {
        at = step(&ground, at, [start[0] + 50, at[1], start[2]]);
    }
    assert!(
        (at[0] - start[0]).abs() + (at[2] - start[2]).abs() > 10,
        "eighty steps went nowhere: {start:?} -> {at:?}"
    );
}

#[test]
fn the_enclosure_edge_refuses_a_step_off_but_not_along_or_away() {
    let ground = ground();
    let bound = ground.extent();
    // Find any stance on the resident edge column.
    let mut edge = None;
    'search: for z in -bound..=bound {
        if let Some(top) = ground.surface(bound, z) {
            let at = [bound, top + 1, z];
            if ground.stands(at, WALKER_HEIGHT) {
                edge = Some(at);
                break 'search;
            }
        }
    }
    let start = edge.expect("the generated edge column has footing somewhere");

    // Marching straight off the map never crosses the bound.
    let mut at = start;
    for _ in 0..40 {
        at = step(&ground, at, [bound + 50, at[1], start[2]]);
        assert!(at[0] <= bound, "stepped past the resident bound: {at:?}");
    }

    // Sliding along the edge (target x held at the bound) is not
    // read as blocked: only leaving the bound is refused.
    let mut at = start;
    for _ in 0..40 {
        at = step(&ground, at, [start[0], at[1], start[2] + 50]);
    }
    assert!(
        (at[2] - start[2]).abs() > 5,
        "could not slide along the enclosure wall: {start:?} -> {at:?}"
    );

    // The same body can still leave the edge toward the interior: no trap.
    let mut at = start;
    for _ in 0..40 {
        at = step(&ground, at, [start[0] - 50, at[1], start[2]]);
    }
    assert!(
        at[0] < start[0],
        "the edge trapped a body that tried to walk inward: {start:?} -> {at:?}"
    );
}

#[test]
fn a_bounded_route_uses_legal_steps_around_generated_ground() {
    let mut ground = ground();
    // An authoritative L-shaped bore is the smallest turning interior the
    // player can make with the same carve primitive the run records.
    for [x, z] in [[0, 0], [4, 0], [4, 4]] {
        let top = ground.surface(x, z).unwrap();
        assert!(ground.carve([x, top, z], 1) > 0);
    }
    let mut stances = Vec::new();
    for z in -2..=6 {
        for x in -2..=6 {
            for y in 1..SURFACE_BAND {
                let at = [x, y, z];
                if ground.stands(at, WALKER_HEIGHT) {
                    stances.push(at);
                }
            }
        }
    }
    for from in &stances {
        for target in &stances {
            let mut greedy = *from;
            for _ in 0..16 {
                let next = step(&ground, greedy, *target);
                if next == greedy {
                    break;
                }
                greedy = next;
            }
            if greedy == *target {
                continue;
            }
            let mut routed = *from;
            for _ in 0..16 {
                let Some(next) = route_step(&ground, routed, *target, 8) else {
                    break;
                };
                assert_eq!(step(&ground, routed, next), next);
                routed = next;
                if routed == *target {
                    return;
                }
            }
            if routed != *target {
                continue;
            }
        }
    }
    panic!("seeded L-shaped bore offered no bounded detour");
}

#[test]
fn the_tier_line_does_not_flap() {
    let grown = Places::grown(4_242, 4, 64);
    let ground = Ground::grow(&grown, 64);
    let line = TierLine::default();
    let focus = a_stance(&ground);
    // Walk an agent straight out and back across the band; count
    // transitions. Hysteresis admits at most one each way.
    let mut tier = Tier::Near;
    let mut flips = 0;
    let mut previous = tier;
    for leg in 0..2 {
        for step_i in 0..60 {
            let d = if leg == 0 { step_i } else { 60 - step_i };
            let agent = [focus[0] + d, focus[1], focus[2]];
            tier = line.tick(&grown.places, tier, agent, focus);
            if tier != previous {
                flips += 1;
                previous = tier;
            }
        }
    }
    assert!(flips <= 2, "tier flapped {flips} times");
}

#[test]
fn spotting_respects_walls_and_range() {
    let ground = ground();
    let at = a_stance(&ground);
    assert!(spot(&ground, at, at, 20), "you can see where you stand");
    assert!(
        !spot(&ground, at, [at[0] + 200, at[1], at[2]], 20),
        "range caps sight"
    );
}
