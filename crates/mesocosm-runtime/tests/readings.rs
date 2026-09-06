// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PE0's reading receipts: replay reproduces them, and the warning separates a
//! stressed enclosure from an untouched one.

use mesocosm_core::snapshot::encode;
use mesocosm_core::{Intent, Kingdom, Placement, Stage, World, state_hash};
use mesocosm_runtime::{FlowWindows, Runtime};

fn scripted(world: &World) -> Vec<Intent> {
    let me = world.controlled_id().expect("a played critter");
    let mut trace = vec![
        Intent::Move { delta: [1, 0, 1] },
        Intent::Idle,
        Intent::Deposit { mass_mg: 40 },
    ];
    for organism in world.living().map(|o| o.id).take(6) {
        if organism != me {
            trace.push(Intent::Metabolize {
                organism,
                placement: Placement::Planned,
            });
        }
    }
    trace.extend(std::iter::repeat_n(Intent::Idle, 400));
    trace
}

#[test]
fn a_replay_reads_the_same_windows_byte_for_byte() {
    // The windows are not in the snapshot, so nothing forces them to agree.
    // Reducing the replay's own streams through the same reducer and comparing
    // the encodings is what shows that they do.
    let mut driven = Runtime::new(4_242, 60, 10);
    let trace = scripted(driven.world());
    for intent in &trace {
        driven.queue(intent.clone());
    }
    driven.step(trace.len() as u64);

    let replayed = Runtime::replayed(4_242, 60, driven.trace());

    assert_eq!(
        encode(driven.windows()).expect("encodable"),
        encode(&replayed.readings).expect("encodable"),
        "a replayed run reads exactly what the run it replays read"
    );
    assert_eq!(driven.trend(), replayed.readings.trend());
    assert_eq!(state_hash(&replayed.world), driven.state_hash());
    assert_eq!(&replayed.history, driven.history());
}

#[test]
fn reducing_readings_leaves_the_world_hash_alone() {
    // A driven run reduces every tick; a bare world never looks. They must
    // still land on the same hash, or a panel would be simulation authority.
    let mut driven = Runtime::new(555, 40, 10);
    driven.step(300);

    let mut bare = World::new(555, 40);
    for _ in 0..300 {
        bare.apply(Intent::Idle);
        bare.drain_events();
    }

    assert_eq!(state_hash(&bare), driven.state_hash());
    assert!(
        driven.trend().replacement_ticks > 0,
        "and it read something"
    );
}

#[test]
fn the_windows_stay_bounded_however_long_the_run() {
    // The retention length is a claim about memory as well as about meaning.
    let mut short = Runtime::new(9, 40, 10);
    short.step(50);
    let mut long = Runtime::new(9, 40, 10);
    long.step(2_000);

    let small = encode(short.windows()).expect("encodable").len();
    let big = encode(long.windows()).expect("encodable").len();
    assert!(
        big < small * 8,
        "a run forty times as long encoded to {big} bytes against {small}"
    );
    assert_eq!(
        long.trend().replacement_ticks,
        mesocosm_runtime::RETENTION_TICKS as u64
    );
}

// ---------------------------------------------------------------------------
// The two arms.
// ---------------------------------------------------------------------------

/// The seed the paired arms run.
///
/// Chosen for the property [`QUIET_SEEDS`] asserts of it and of three others —
/// an untouched enclosure that holds its stand for the whole run — and the
/// assertion is what keeps the choice honest: if this world ever starts
/// declining on its own, these fail loudly instead of quietly passing.
const ARM_SEED: u64 = 7;

/// Seeds whose untouched enclosure holds its stand across [`ARM_TICKS`].
///
/// These control fixtures may show short transient dips, but none reaches
/// [`mesocosm_core::WARN_AFTER_TICKS`]. Under TG1, seed 555 reaches 14 short
/// ticks and recovers before the 60-tick warning threshold.
const QUIET_SEEDS: [u64; 4] = [1, 7, 99, 555];

/// Founders per arm, and how long each runs. Two thousand ticks is the horizon
/// the threshold was measured over.
const ARM_FOUNDERS: u32 = 200;
const ARM_TICKS: u64 = 2_000;

/// Runs one arm and reports the longest shortfall streak it reached.
fn arm(seed: u64, overdraw: bool) -> u64 {
    let mut world = World::new(seed, ARM_FOUNDERS);
    if overdraw {
        overdraw_the_producer_path(&mut world);
    }
    let mut windows = FlowWindows::new();
    let mut peak = 0;
    for _ in 0..ARM_TICKS {
        world.apply(Intent::Idle);
        let events = world.drain_events();
        windows.absorb(&events, &world.drain_flows());
        peak = peak.max(windows.trend().shortfall_ticks);
    }
    peak
}

/// The induced stress: half the stand dies off, and every surviving mouth
/// starts standing on what is left.
///
/// **One support path, deliberately overdrawn.** Both halves are things the
/// world can do to itself — a die-off and a herd converging — and neither
/// conjures or destroys a milligram: a corpse keeps every milligram it had and
/// returns it through the ordinary decay path, and a position is not matter at
/// all. So the control can be the *same* world with nothing done to it, which is
/// what makes the comparison mean anything.
fn overdraw_the_producer_path(world: &mut World) {
    let mut seen = 0;
    for organism in world.organisms.iter_mut() {
        if organism.kingdom() == Kingdom::Producer && organism.is_alive() {
            seen += 1;
            if seen % 2 != 0 {
                organism.stage = Stage::Carrion;
            }
        }
    }
    let stand: Vec<[i32; 3]> = world
        .living()
        .filter(|o| o.kingdom() == Kingdom::Producer)
        .map(|o| o.position)
        .collect();
    assert!(!stand.is_empty(), "the die-off left a stand to converge on");
    let mut next = 0;
    for organism in world.organisms.iter_mut() {
        if organism.kingdom() == Kingdom::Consumer && organism.is_alive() {
            organism.position = stand[next % stand.len()];
            next += 1;
        }
    }
}

#[test]
fn a_neutral_control_does_not_raise_the_warning_a_stressed_arm_raises() {
    let control = arm(ARM_SEED, false);
    let stressed = arm(ARM_SEED, true);

    assert_eq!(
        control, 0,
        "the untouched enclosure never read short across {ARM_TICKS} ticks"
    );
    assert!(
        stressed >= mesocosm_core::WARN_AFTER_TICKS,
        "the induced overdraw reached {stressed} short ticks, under the {} the \
         warning is ruled at",
        mesocosm_core::WARN_AFTER_TICKS
    );
}

#[test]
fn an_enclosure_holding_its_stand_never_raises_it() {
    // The other half of the control: not one seed that happened to stay quiet,
    // but every seed measured to hold its stand. A warning that fired on a
    // healthy enclosure would be worth nothing on a sick one.
    for seed in QUIET_SEEDS {
        assert!(
            arm(seed, false) < mesocosm_core::WARN_AFTER_TICKS,
            "seed {seed} reached the warning threshold with nothing done to it"
        );
    }
}
