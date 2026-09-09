// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S0's replay receipt: the same trace, the same hash, across a save.
//!
//! The position log is the thing hashed, not the final position. Two runs
//! can arrive in the same corner by different routes, and only one of those
//! is determinism.

use mesocosm_core::snapshot;
use paredros_room::probe::{ProbeError, Save};
use paredros_room::room::SEED;
use paredros_room::{Probe, TICKS};

fn spent() -> Probe {
    let mut probe = Probe::new(SEED).expect("the room probe grows its world");
    probe.run(TICKS);
    probe
}

#[test]
fn the_same_trace_walks_the_same_path_twice() {
    let (a, b) = (spent(), spent());
    assert_eq!(a.at(), b.at());
    assert_eq!(a.log(), b.log());
    assert_eq!(a.hash(), b.hash());
    println!(
        "S0 receipt: position-log hash {:#018x}, ground hash {:#018x}, final {:?}",
        a.hash(),
        a.ground_hash(),
        a.at()
    );
}

#[test]
fn a_save_at_half_time_replays_to_the_straight_run() {
    let straight = spent();

    let mut split = Probe::new(SEED).unwrap();
    split.run(TICKS / 2);
    let saved = split.save().expect("a probe encodes");

    let mut restored = Probe::restore(&saved).expect("a fresh probe takes the save");
    assert_eq!(restored.tick_count(), TICKS / 2);
    assert_eq!(restored.at(), split.at());
    restored.run(TICKS - TICKS / 2);

    assert_eq!(restored.tick_count(), TICKS);
    assert_eq!(restored.at(), straight.at(), "the run ended elsewhere");
    assert_eq!(restored.log(), straight.log(), "the run took another route");
    assert_eq!(restored.hash(), straight.hash(), "the hashes diverged");
}

#[test]
fn a_save_reloads_into_the_same_room() {
    let probe = spent();
    let restored = Probe::restore(&probe.save().unwrap()).unwrap();
    assert_eq!(restored.room().centre, probe.room().centre);
    assert_eq!(restored.ground_hash(), probe.ground_hash());
}

#[test]
fn a_save_whose_world_regrew_differently_is_refused() {
    // The save carries a seed, not a world, so a restore has to prove the
    // world it regrew is the world the save came from. Divergence must be
    // reported, never replayed over.
    let probe = spent();
    let mut save: Save = snapshot::decode(&probe.save().unwrap()).unwrap();
    save.ground_hash ^= 1;
    let bytes = snapshot::encode(&save).unwrap();
    assert!(matches!(
        Probe::restore(&bytes),
        Err(ProbeError::GroundDiverged)
    ));
}
