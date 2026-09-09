// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The attachment hypothesis, end to end.
//!
//! The body pipeline plan names this as the wing's unproven assumption: a part
//! attaches to a living body **during play**, acquires collision and mass,
//! moves the centre of balance, and stays legible.
//!
//! These tests run the real simulation, eat a real organism, and check all four
//! at once. What they cannot check is whether it *looks* good on screen; that
//! stays a judgment for the windowed host. What they do establish is that
//! everything the screen would need is derivable, deterministic, and cheap.

use mesocosm_core::{
    Intent, Origin, Outcome, PartId, Placement, STARVED_UPKEEP_TICKS, VolumeRef, World, Yaw,
};
use mesocosm_mesh::{Volume, VolumeMap, mesh_body};

/// Volumes for the fixture: a body, and one for every primitive tag the world
/// may mint. Developmental roles now choose their volume vocabulary from the
/// realized body, so this adapter fixture must not assume one narrow tag band.
fn source() -> VolumeMap {
    let mut map = VolumeMap::new();
    map.insert(VolumeRef::from_tag(1), Volume::solid([3, 3, 3], 1));
    for tag in 2..=u8::MAX {
        map.insert(VolumeRef::from_tag(tag), Volume::solid([2, 2, 2], tag));
    }
    map
}

/// Walks the critter to its nearest neighbour and returns it.
///
/// Reach is anatomy since P2, so a starting critter touches about three
/// voxels and a fixture has to travel like a player instead of assuming a
/// meal is adjacent.
fn reachable_organism(world: &mut World) -> mesocosm_core::OrganismId {
    for _ in 0..600 {
        let here = world.position().expect("embodied");
        let Some((id, at)) = world
            .organisms
            .iter()
            // The played critter is an organism too since P1; never eat yourself.
            .filter(|m| Some(m.id) != world.controlled_id() && m.is_alive())
            .map(|m| (m.id, m.position))
            .min_by_key(|(_, at): &(_, [i32; 3])| {
                (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0)
            })
        else {
            break;
        };
        if world.in_reach(at) {
            return id;
        }
        world.apply(Intent::Move {
            delta: [0, 1, 2].map(|a| (at[a] - here[a]).signum()),
        });
    }
    panic!("nothing came within reach")
}

/// Fills the played critter's reserve so its next meal *builds*.
///
/// TD4 made the body route a meal — starved burns, provisioned incorporates —
/// and TD7 raised what a motile body's rent is, so the starved horizon
/// (`upkeep x STARVED_UPKEEP_TICKS`) is now further from empty than the walk to
/// a neighbour leaves it. These tests are about attachment, not about routing,
/// so they say which state the body is in rather than hoping.
fn provisioned(world: &mut World) {
    let id = world.controlled_id().expect("embodied");
    let organism = world
        .organisms
        .iter_mut()
        .find(|o| o.id == id)
        .expect("still on the roster");
    organism.energy_mg = organism.upkeep_mg() * (STARVED_UPKEEP_TICKS + 1);
}

#[test]
fn eating_changes_mass_balance_collision_and_geometry() {
    let source = source();
    let mut world = World::new(0x00A7_7AC4, 40);

    let mass_before = world.total_mass_mg();
    let centre_before = world.body().unwrap().centre_of_mass();
    let collision_before = world.collision().unwrap();
    let drawn_before = mesh_body(world.body().unwrap(), &source).unwrap();

    let target = reachable_organism(&mut world);
    provisioned(&mut world);
    let outcome = world.apply(Intent::Metabolize {
        organism: target,
        placement: Placement::Explicit {
            parent: PartId(0),
            offset: [9, 0, 0],
            yaw: Yaw::Zero,
        },
    });
    assert!(
        matches!(outcome, Outcome::Incorporated { .. }),
        "{outcome:?}"
    );

    let drawn_after = mesh_body(world.body().unwrap(), &source).unwrap();

    // Mass.
    assert!(world.total_mass_mg() > mass_before, "the body got heavier");

    // Balance. The part landed to the +x side, so the centre must follow it.
    let centre_after = world.body().unwrap().centre_of_mass();
    assert!(
        centre_after[0] > centre_before[0],
        "centre of mass moved toward the new part: {centre_before:?} -> {centre_after:?}"
    );

    // Collision.
    assert!(
        world.collision().unwrap().extent()[0] > collision_before.extent()[0],
        "the collision box grew"
    );

    // Geometry.
    assert_eq!(
        drawn_after.placement_count(),
        drawn_before.placement_count() + 1
    );
    assert!(drawn_after.drawn_quads() > drawn_before.drawn_quads());
    let (_, max_before) = drawn_before.bounds().unwrap();
    let (_, max_after) = drawn_after.bounds().unwrap();
    assert!(
        max_after[0] > max_before[0],
        "the drawn body reaches further"
    );
}

#[test]
fn an_eaten_part_still_says_whose_it_was() {
    let source = source();
    // Use the open-surface fixture above. Provenance is the subject here;
    // whether anatomy fits a confined stance is covered in mesocosm-core.
    let mut world = World::new(0x00A7_7AC4, 40);
    let target = reachable_organism(&mut world);
    let donor = world
        .organisms
        .iter()
        .find(|m| m.id == target)
        .map(|m| m.species)
        .unwrap();

    provisioned(&mut world);
    let outcome = world.apply(Intent::Metabolize {
        organism: target,
        placement: Placement::Explicit {
            parent: PartId(0),
            offset: [9, 0, 0],
            yaw: Yaw::Quarter,
        },
    });
    let Outcome::Incorporated { part } = outcome else {
        panic!("expected incorporation, got {outcome:?}");
    };

    // The projection can place it...
    let mesh = mesh_body(world.body().unwrap(), &source).unwrap();
    let placement = mesh
        .placements
        .iter()
        .find(|p| p.part == part)
        .expect("the new part is placed");
    assert_eq!(placement.yaw, Yaw::Quarter);

    // ...and the record of where it came from survives alongside the geometry.
    match world.body().unwrap().part(part).unwrap().provenance.origin {
        Origin::Incorporated { from_species, .. } => assert_eq!(from_species, donor),
        Origin::Founding => panic!("an eaten part is not founding stock"),
    }
}

#[test]
fn attaching_remeshes_only_what_is_new() {
    let source = source();
    let mut world = World::new(31337, 40);

    let before = mesh_body(world.body().unwrap(), &source).unwrap();
    let root_mesh_before = before.mesh_for(VolumeRef::from_tag(1)).cloned();

    let target = reachable_organism(&mut world);
    world.apply(Intent::Metabolize {
        organism: target,
        placement: Placement::Explicit {
            parent: PartId(0),
            offset: [6, 0, 0],
            yaw: Yaw::Zero,
        },
    });

    let after = mesh_body(world.body().unwrap(), &source).unwrap();

    assert_eq!(
        after.mesh_for(VolumeRef::from_tag(1)).cloned(),
        root_mesh_before,
        "the existing body's geometry is untouched by an attachment"
    );
    assert!(after.mesh_count() >= before.mesh_count());
}

#[test]
fn a_body_grown_over_many_meals_stays_deterministic() {
    let source = source();

    let grow = || {
        let mut world = World::new(9_001, 60);
        for _ in 0..5 {
            let target = reachable_organism(&mut world);
            world.apply(Intent::Metabolize {
                organism: target,
                placement: Placement::Explicit {
                    parent: PartId(0),
                    offset: [5, 0, 0],
                    yaw: Yaw::Zero,
                },
            });
        }
        world
    };

    let a = grow();
    let b = grow();
    assert_eq!(a.body(), b.body());

    let mesh_a = mesh_body(a.body().unwrap(), &source).unwrap();
    let mesh_b = mesh_body(b.body().unwrap(), &source).unwrap();
    assert_eq!(mesh_a.placements, mesh_b.placements);
    assert_eq!(mesh_a.drawn_quads(), mesh_b.drawn_quads());
    assert!(
        mesh_a.placement_count() > 1,
        "the fixture actually ate something"
    );
}
