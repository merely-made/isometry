// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wave 1.1 done-conditions, as executable checks.
//!
//! 1. The fixture replays identically.
//! 2. Provenance round-trips.
//! 3. Attaching a part changes the core's mass and collision state.
//!
//! These are also the receipts the host probe (wave 1.3) compares against:
//! both hosts run this fixture and must agree on the final state hash.

use mesocosm_core::body::Attachment;
use mesocosm_core::snapshot::{decode, encode, restore};
use mesocosm_core::{
    BodyDocument, Intent, Origin, Outcome, PartId, Placement, Provenance, SpeciesId, VolumeRef,
    World, Yaw, snapshot, state_hash,
};

const SEED: u64 = 0x5E5E_1234;
const MORSELS: u32 = 32;

/// The shared fixture trace. Hosts replay exactly this.
///
/// **Recorded by driving, not by guessing.** It used to pick targets within a
/// hardcoded eight voxels, which stopped being true when reach became anatomy:
/// a starting critter touches about three. So the trace is captured the way a
/// player would produce one, by walking to prey and eating what is actually in
/// reach. Deterministic because the scratch world uses the same seed.
fn fixture_trace(_world: &World) -> Vec<Intent> {
    let mut scratch = World::new(SEED, MORSELS);
    let mut trace = vec![Intent::Move { delta: [1, 0, 1] }, Intent::Idle];
    scratch.apply_all(&trace);

    let mut meals = 0;
    for _ in 0..600 {
        if meals >= 3 {
            break;
        }
        let Some(here) = scratch.position() else {
            break;
        };
        let Some((id, at)) = scratch
            .organisms
            .iter()
            .filter(|m| Some(m.id) != scratch.controlled_id() && m.is_alive())
            .map(|m| (m.id, m.position))
            .min_by_key(|(_, at): &(_, [i32; 3])| {
                (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0)
            })
        else {
            break;
        };

        let intent = if scratch.in_reach(at) {
            meals += 1;
            Intent::Metabolize {
                organism: id,
                placement: Placement::Planned,
            }
        } else {
            Intent::Move {
                delta: [0, 1, 2].map(|a| (at[a] - here[a]).signum()),
            }
        };
        scratch.apply(intent.clone());
        trace.push(intent);
    }

    trace.push(Intent::Deposit { mass_mg: 40 });
    trace.push(Intent::Move { delta: [-1, 0, 2] });
    trace
}

fn run_fixture() -> World {
    let mut world = World::new(SEED, MORSELS);
    let trace = fixture_trace(&world);
    world.apply_all(&trace);
    world
}

#[test]
fn fixture_replays_identically() {
    let a = run_fixture();
    let b = run_fixture();
    assert_eq!(
        state_hash(&a),
        state_hash(&b),
        "same seed and trace must agree"
    );
    assert_eq!(a, b);
}

#[test]
fn fixture_survives_snapshot_and_restore_midway() {
    let mut world = World::new(SEED, MORSELS);
    let trace = fixture_trace(&world);
    let split = trace.len() / 2;

    world.apply_all(&trace[..split]);
    let bytes = snapshot(&world).expect("world encodes");
    let mut resumed = restore(&bytes).expect("world decodes");

    world.apply_all(&trace[split..]);
    resumed.apply_all(&trace[split..]);

    assert_eq!(state_hash(&world), state_hash(&resumed));
}

#[test]
fn fixture_incorporates_and_the_body_grows() {
    let mut world = World::new(SEED, MORSELS);
    let trace = fixture_trace(&world);

    let mass_before = world.total_mass_mg();
    let box_before = world.collision().unwrap();
    let parts_before = world.body().unwrap().len();

    let outcomes = world.apply_all(&trace);
    let incorporated = outcomes
        .iter()
        .filter(|o| matches!(o, Outcome::Incorporated { .. }))
        .count();

    assert!(incorporated > 0, "fixture must actually eat something");
    assert!(world.total_mass_mg() > mass_before, "mass must grow");
    assert!(
        world.body().unwrap().len() > parts_before,
        "body must gain parts"
    );

    let after = world.collision();
    assert!(
        after
            .unwrap()
            .extent()
            .into_iter()
            .zip(box_before.extent())
            .any(|(after, before)| after > before),
        "the incorporated part must enlarge the developed body's collision"
    );
}

#[test]
fn provenance_round_trips_through_the_wire() {
    let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2]);
    let donor = SpeciesId(17);
    let part = body
        .attach(
            VolumeRef::from_tag(5),
            750,
            [1, 2, 1],
            Attachment {
                parent: body.root,
                offset: [3, 0, 0],
                yaw: Yaw::Quarter,
            },
            Provenance {
                origin: Origin::Incorporated {
                    from_species: donor,
                    from_part: PartId(2),
                },
                epoch: 4,
            },
        )
        .expect("root exists");

    let bytes = encode(&body).expect("body encodes");
    let restored: BodyDocument = decode(&bytes).expect("body decodes");

    assert_eq!(body, restored);
    let provenance = &restored.part(part).unwrap().provenance;
    assert_eq!(provenance.epoch, 4);
    assert_eq!(
        provenance.origin,
        Origin::Incorporated {
            from_species: donor,
            from_part: PartId(2)
        }
    );
}

#[test]
fn provenance_survives_a_whole_world_snapshot() {
    let world = run_fixture();
    let donors: Vec<SpeciesId> = world
        .body()
        .unwrap()
        .incorporated()
        .map(|p| match p.provenance.origin {
            Origin::Incorporated { from_species, .. } => from_species,
            Origin::Founding => unreachable!("filtered to incorporated"),
        })
        .collect();
    assert!(!donors.is_empty(), "fixture must incorporate something");

    let bytes = snapshot(&world).unwrap();
    let restored = restore(&bytes).unwrap();
    let restored_donors: Vec<SpeciesId> = restored
        .body()
        .unwrap()
        .incorporated()
        .map(|p| match p.provenance.origin {
            Origin::Incorporated { from_species, .. } => from_species,
            Origin::Founding => unreachable!("filtered to incorporated"),
        })
        .collect();

    assert_eq!(
        donors, restored_donors,
        "every part still knows whose it was"
    );
}

#[test]
fn a_different_seed_produces_a_different_world() {
    let mut other = World::new(SEED ^ 0xFFFF, MORSELS);
    let trace = fixture_trace(&other);
    other.apply_all(&trace);
    assert_ne!(state_hash(&run_fixture()), state_hash(&other));
}
