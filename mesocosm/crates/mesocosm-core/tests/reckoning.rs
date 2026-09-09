// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An enclosure has somewhere in it, and an epoch comes to something.
//!
//! `WorldRecord::note` was built and left uncalled through three sessions,
//! because scoring a run needs the log, the species tree, and places all at
//! once. These run a real enclosure and check that the record fills from what
//! actually happened.

use std::collections::BTreeSet;

use mesocosm_core::{Feat, History, Intent, Scale, SpeciesId, World, snapshot};

/// Runs an enclosure for a while, recording everything.
fn lived(ticks: usize) -> (World, History) {
    let mut world = World::new(4_242, 40);
    let mut history = History::new();

    for _ in 0..ticks {
        world.apply(Intent::Idle);
        history.record_all(world.drain_events());
    }
    (world, history)
}

#[test]
fn everything_in_the_enclosure_is_somewhere() {
    let (world, _) = lived(120);

    assert!(
        world
            .living()
            .any(|organism| world.places().at(organism.position).is_some()),
        "a living organism is in a region"
    );
    for organism in world.living() {
        assert!(
            world.places().at(organism.position).is_some(),
            "{:?} is nowhere",
            organism.id
        );
    }
}

#[test]
fn the_regions_hold_still() {
    // A place that moved would make "it happened here" meaningless. The
    // division is drawn once from the seed and never redrawn.
    let fresh = World::new(4_242, 40);
    let (worn, _) = lived(300);
    assert_eq!(fresh.places(), worn.places());
}

#[test]
fn a_range_only_grows() {
    // The high-water rule, one scale down from the frontier. A lineage that
    // withdrew from half the enclosure still reached it.
    let mut world = World::new(4_242, 40);
    let mut before: Vec<BTreeSet<_>> = Vec::new();

    for _ in 0..200 {
        world.apply(Intent::Idle);
        let now: Vec<BTreeSet<_>> = (1..6).map(|id| world.range(SpeciesId(id))).collect();
        for (was, is) in before.iter().zip(&now) {
            assert!(
                was.is_subset(is),
                "a lineage un-reached somewhere it had been"
            );
        }
        before = now;
    }
    assert!(
        before.iter().any(|range| range.len() > 1),
        "something got about"
    );
}

#[test]
fn a_lineage_that_has_not_moved_is_local() {
    // Scale is read off reach rather than declared. A single critter idling in
    // place has been exactly one region, and that is what Local means.
    let mut world = World::new(4_242, 40);
    let history = History::new();
    world.apply(Intent::Idle);

    let mine = world.controlled().expect("embodied").species;
    assert_eq!(world.range(mine).len(), 1, "one critter, one region");

    let readings = world.reckon(&history);
    for reading in readings.iter().filter(|r| r.species == mine) {
        assert_eq!(reading.scale, Scale::Local);
    }
}

#[test]
fn a_scale_is_the_reach_of_whoever_did_it() {
    let (mut world, history) = lived(300);
    let places = world.places().clone();

    for reading in world.reckon(&history) {
        assert_eq!(
            reading.scale,
            places.scale(&world.range(reading.species)),
            "a reading's scale is not its lineage's"
        );
    }
}

/// **The boundary and the reckoning, in the order the driver does them** (DT4).
/// There is one door into a boundary — `World::apply`'s block, reached by the
/// world's own epoch rule or by a hand's `Intent::EndEpoch` — and the reckoning
/// is the separate read-the-past call after it. The manual `World::end_epoch`
/// that used to do both at once is deleted; it skipped the adaptation round and
/// left the world standing at no checkpoint, which is two answers to one
/// question.
#[test]
fn ending_an_epoch_finally_writes_the_record() {
    let (mut world, history) = lived(300);
    assert_eq!(world.record().filled(), 0, "nothing has been noted yet");
    assert_eq!(world.epoch, 0, "the budget is nowhere near spent");

    assert_eq!(
        world.apply(Intent::EndEpoch),
        mesocosm_core::Outcome::EpochEnded { epoch: 0 }
    );
    assert_eq!(world.epoch, 1);
    assert!(
        world.at_boundary(),
        "and standing at its lineage checkpoint"
    );

    let readings = world.reckon(&history);
    assert!(!readings.is_empty(), "an epoch of living came to something");
    assert!(world.record().filled() > 0, "and the record has it now");
    for reading in &readings {
        let mark = world
            .record()
            .standing(reading.feat, reading.scale)
            .expect("noted");
        assert!(mark.high >= reading.value, "a mark is at least what set it");
    }
}

#[test]
fn the_first_of_anything_takes_the_record() {
    // What an epoch-boundary screen is made of: not the numbers, but which of
    // them nobody had reached before.
    let (mut world, history) = lived(200);
    let readings = world.reckon(&history);
    assert!(
        readings.iter().any(|r| r.took),
        "an empty record is all firsts"
    );
}

#[test]
fn reckoning_twice_changes_nothing() {
    // The record joins by taking the higher mark, which is idempotent, so a
    // boundary that fires twice is the same boundary. That is the property
    // that lets two worlds hand each other records without a protocol, seen
    // from inside one world.
    let (mut world, history) = lived(250);
    world.reckon(&history);
    let after = world.record().clone();

    let again = world.reckon(&history);
    assert_eq!(*world.record(), after, "a second reckoning moved a mark");
    assert!(
        again.iter().all(|r| !r.took),
        "and nothing was a first the second time"
    );
}

#[test]
fn nobody_has_built_anything_yet() {
    // Two axes stay open, and that is the point: `untouched` answers "has
    // anyone ever", so writing a zero would close the question permanently.
    let (mut world, history) = lived(300);
    world.reckon(&history);

    for scale in [Scale::Local, Scale::Regional, Scale::Worldwide] {
        assert!(world.record().untouched(Feat::Construction, scale));
        assert!(world.record().untouched(Feat::Symbiosis, scale));
    }
    assert!(!world.record().untouched(Feat::Growth, Scale::Local) || world.record().filled() > 0);
}

#[test]
fn a_world_carries_its_record_and_its_regions() {
    // Both belong in the snapshot: a few integers and nine sites, so whole-state
    // capture stays the cheap thing the wing's rollback thinking rests on.
    let (mut world, history) = lived(150);
    world.reckon(&history);

    let bytes = snapshot(&world).unwrap();
    let restored: World = mesocosm_core::restore(&bytes).unwrap();
    assert_eq!(restored, world);
    assert_eq!(restored.record().filled(), world.record().filled());
}

#[test]
fn the_same_seed_reckons_the_same() {
    let (mut a_world, a) = lived(150);
    let (mut b_world, b) = lived(150);
    assert_eq!(a_world.reckon(&a), b_world.reckon(&b));
    assert_eq!(a_world.record(), b_world.record());
}
