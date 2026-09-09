// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD3 parity: the packed gland is PD2's gland. (PD3)
//!
//! The gate asks that the packed definition lower to **the same `ProcessDef`
//! and the same game outcome** as the native proof. Those are two claims and
//! there is a test for each:
//!
//! 1. the definition admitted out of `packs/mesocosm/processes/secrete.json`
//!    is the definition `mesocosm-core` holds — same site requirement, same
//!    seeding, same content address, so every allocation already citing it
//!    resolves against the packed ruleset unchanged;
//! 2. a body driven through the packed reference reaches PD2's four states —
//!    a located, paid-for gland; a charged one; a dry one; and a lost one —
//!    with the same numbers the native proof recorded.
//!
//! And the third claim the world half of the gate makes: a snapshot names the
//! ruleset it ran under, and a restore offered a different one refuses.

use std::path::{Path, PathBuf};

use mesocosm_core::{
    Attachment, Intent, Process, ProcessId, ProcessRef, Provenance, Registry, SnapshotError,
    VolumeRef, World, WorldRules, Yaw,
};
use mesocosm_phenotype::admit_dir;

fn shipped_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the repository root")
        .join("packs")
        .join("mesocosm")
}

fn packed() -> Registry {
    admit_dir(&shipped_root()).expect("the shipped pack admits")
}

/// The wide plate PD2's fixtures grow: `[6, 4, 1]` lattices to twelve cells,
/// and a gland on five of them out-holds a fresh soil column, which is what
/// makes the dormant state reachable rather than theoretical.
const FROND: [i32; 3] = [6, 4, 1];

// ---------------------------------------------------------------------------
// 1. The same definition
// ---------------------------------------------------------------------------

#[test]
fn the_packed_gland_lowers_to_the_native_definition() {
    let packed = packed();
    let id = ProcessId::new("mesocosm", "secrete");
    let from_pack = packed.get(&id).expect("the pack declares it");
    let from_core = Registry::native().of_native(Process::Secrete);

    assert_eq!(from_pack.id, from_core.id);
    assert_eq!(from_pack.expressed_by, from_core.expressed_by);
    assert_eq!(from_pack.seeding, from_core.seeding);
    assert_eq!(
        from_pack.digest(),
        from_core.digest(),
        "the content address is the same, so a body already citing it resolves"
    );
    // And the whole ruleset, not only the one definition: a world that
    // admitted the pack runs the biology this build ships.
    assert_eq!(&packed, Registry::native());
    assert_eq!(WorldRules::of(&packed), WorldRules::native());
}

#[test]
fn nothing_the_pack_declares_grows_itself_onto_a_founded_body() {
    // PD2's first done-condition, restated against the packed ruleset: the
    // gland is `acquired`, so a plate admits one and never seeds one, and the
    // whole readable choice depends on that byte surviving the round trip
    // through a file.
    let packed = packed();
    let gland = packed
        .get(&ProcessId::new("mesocosm", "secrete"))
        .expect("declared");
    assert!(!gland.seeded(), "the pack says nothing grows a gland");
    assert!(
        packed
            .seeds(mesocosm_core::Role::Plate)
            .all(|def| def.id.name != "secrete"),
        "and the seeding rule agrees"
    );

    let world = World::new(4_242, 24);
    for organism in &world.organisms {
        assert_eq!(organism.phenotype.secretory_mg(), 0);
    }
    assert_eq!(world.gland(), None);
}

// ---------------------------------------------------------------------------
// 2. The same game outcome
// ---------------------------------------------------------------------------

/// Holds the played body under the starved line, with a hand on it, until the
/// endurance condition lands. The same fixture `tests/embodied/discovery.rs`
/// uses, and the only route into the gland now that PD2's editor operation is
/// deleted.
fn endure(world: &mut World, ticks: u64) {
    for _ in 0..ticks {
        let Some(me) = world.controlled_id() else {
            return;
        };
        let upkeep = world.controlled().expect("alive").upkeep_mg();
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == me)
            .expect("in the roster")
            .energy_mg = upkeep * (mesocosm_core::STARVED_UPKEEP_TICKS - 1);
        world.apply(Intent::Resume);
    }
}

/// A world whose played critter is a plain mature consumer of a known size.
///
/// The same fixture `tests/embodied.rs` opens with: a founded body varies with
/// the seed, and every number below is about what a development does to a
/// body, not about which body worldgen drew.
fn bulk_world(seed: u64, founders: u32) -> World {
    let mut world = World::new(seed, founders);
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
        ..mesocosm_core::Organism::founding(
            me,
            species,
            mesocosm_core::Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    world
}

fn frond_on(world: &mut World) -> mesocosm_core::PartId {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches above the root")
}

fn hunger() -> mesocosm_core::ConditionId {
    mesocosm_core::discovery::conditions()
        .into_iter()
        .find(|condition| condition.name == "mesocosm:endured-hunger")
        .expect("the table holds it")
        .id()
}

#[test]
fn the_packed_definition_reaches_pd2s_four_states_through_the_packed_door() {
    // **The parity receipt the deletion checklist waited on.** Everything the
    // native fixture proved, proved again with the definition read off disk:
    // the reference the discovery grants is the packed reference, and it walks
    // the one validator to the same result.
    let packed = packed();
    let packed_gland = packed
        .get(&ProcessId::new("mesocosm", "secrete"))
        .expect("declared")
        .reference();

    let mut world = bulk_world(4_242, 24);
    endure(&mut world, mesocosm_core::discovery::HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()), "the condition landed");
    let candidate = world
        .discoveries()
        .iter()
        .find(|discovery| discovery.condition == hunger())
        .expect("recorded")
        .candidate;
    assert_eq!(
        candidate.process, packed_gland,
        "what the line came to is the definition the pack declares"
    );

    // Availability is not expression: a bulk consumer has nowhere to put one.
    assert!(world.candidate_intent(hunger()).is_none());
    let part = frond_on(&mut world);

    // **Located, and charged.** Through PD3's bounded door, which is the only
    // developmental verb a host has now.
    let cell_mg = world.phenotype().unwrap().cell_mg(part);
    let matter_before = world.total_matter_mg();
    let intent = world
        .candidate_intent(hunger())
        .expect("the frond is somewhere to put it");
    assert_eq!(
        intent,
        Intent::Express {
            condition: hunger()
        },
        "the whole of what a host says about a development"
    );
    let outcome = world.apply(intent);
    let mesocosm_core::Outcome::Expressed {
        part: on,
        cost_mg,
        revision,
    } = outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!(on, part);
    assert_eq!(cost_mg, u64::from(candidate.cells) * cell_mg);
    assert_eq!(revision, 1);
    assert_eq!(
        world.total_matter_mg(),
        matter_before,
        "the development conserved matter"
    );

    // **Useful.** The tissue is where the reading says, it stings, and it
    // costs rent from here on.
    let reading = world.gland().expect("it has one now");
    assert_eq!(reading.sites, vec![(part, candidate.cells)]);
    assert!(reading.charged);
    assert!(reading.rent_mg > 0);
    assert!(
        world
            .phenotype()
            .unwrap()
            .expressing(packed_gland)
            .any(|found| found == part),
        "and what it expresses resolves against the packed ruleset"
    );

    // **Dormant.** Two columns over, where the ground cannot supply what the
    // gland holds — and nothing about the allocation moved.
    let mut dry_world = world.clone();
    dry_world.apply(Intent::Move { delta: [2, 0, 0] });
    dry_world.apply(Intent::Move { delta: [2, 0, 0] });
    let dry = dry_world.gland().expect("still has one");
    assert!(!dry.charged, "{} against {}", dry.ground_mg, dry.potency_mg);
    assert_eq!(dry.cells, reading.cells);
    assert_eq!(dry.rent_mg, reading.rent_mg);

    // **Severed, and gone** — and the branch still says what it did.
    let me = world.controlled_id().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .phenotype
        .sever(part);
    let gone = world.gland().expect("the loss is still readable");
    assert!(gone.sites.is_empty());
    assert_eq!(gone.rent_mg, 0);
    assert_eq!(gone.lost, vec![part]);
}

#[test]
fn a_host_cannot_ask_for_what_its_line_has_not_come_to() {
    // The bound on the door, and the reason it replaced an arrangement with a
    // condition: expressing is asking for something already discovered, and a
    // body with nowhere to put it is told so rather than refused vaguely.
    let mut world = bulk_world(4_242, 24);
    assert_eq!(
        world.apply(Intent::Express {
            condition: hunger()
        }),
        mesocosm_core::Outcome::Rejected(mesocosm_core::Rejection::Undiscovered(hunger()))
    );

    endure(&mut world, mesocosm_core::discovery::HUNGER_TICKS + 1);
    assert_eq!(
        world.apply(Intent::Express {
            condition: hunger()
        }),
        mesocosm_core::Outcome::Rejected(mesocosm_core::Rejection::Nowhere(hunger())),
        "discovered, and this body is not a shape that can carry it"
    );
}

// ---------------------------------------------------------------------------
// 3. The snapshot names the ruleset
// ---------------------------------------------------------------------------

#[test]
fn a_snapshot_names_the_exact_admitted_ruleset() {
    let world = World::new(4_242, 24);
    assert_eq!(
        world.rules(),
        WorldRules::of(&packed()),
        "a world founded under this build is running the pack's biology"
    );

    let bytes = mesocosm_core::snapshot(&world).expect("encodes");
    let restored = mesocosm_core::restore_under(&bytes, std::sync::Arc::new(packed()))
        .expect("the same ruleset restores");
    assert_eq!(restored.rules(), world.rules());
    assert_eq!(
        mesocosm_core::state_hash(&restored),
        mesocosm_core::state_hash(&world),
        "the ruleset is world state, so it is inside the hash"
    );
}

#[test]
fn a_replay_against_a_different_admitted_ruleset_is_refused_identifiably() {
    // The missing-ruleset rule (plan §6) at the world scale: a save restored
    // under a biology that is not the one it ran is refused by name, with both
    // digests, rather than continuing against whatever this build holds.
    let world = World::new(4_242, 24);
    let bytes = mesocosm_core::snapshot(&world).expect("encodes");

    // A pack with one rule-bearing byte changed: the gland grows on plates.
    let mut defs: Vec<_> = packed().all().cloned().collect();
    for def in &mut defs {
        if def.id.name == "secrete" {
            def.seeding = mesocosm_core::Seeding::Geometry;
        }
    }
    let other = Registry::admit(defs).expect("no collision");
    assert_ne!(other.digest(), packed().digest());

    let other_digest = other.digest();
    let refused = mesocosm_core::restore_under(&bytes, std::sync::Arc::new(other));
    assert_eq!(
        refused.err(),
        Some(SnapshotError::Ruleset {
            expected: world.rules().processes,
            found: other_digest,
        }),
        "it must say which ruleset it wanted, not simply fail"
    );

    // And the same bytes still restore under the ruleset they ran under, so
    // the refusal is about the biology rather than about the save.
    assert!(mesocosm_core::restore_under(&bytes, world.admitted()).is_ok());
}

#[test]
fn an_allocation_does_not_resolve_against_a_ruleset_that_lost_its_definition() {
    // The other half of the same rule, one scale down: `resolve` answers
    // `None` rather than substituting a similar local definition, so a pack
    // that dropped the gland cannot quietly run a body that has one.
    let mut defs: Vec<_> = packed().all().cloned().collect();
    defs.retain(|def| def.id.name != "secrete");
    let without = Registry::admit(defs).expect("no collision");

    let gland: ProcessRef = Registry::native().of_native(Process::Secrete).reference();
    assert!(without.resolve(gland).is_none());
    assert!(packed().resolve(gland).is_some());
}
