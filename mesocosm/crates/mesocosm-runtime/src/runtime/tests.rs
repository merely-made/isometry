// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The driver's own claims: frame delivery does not reach the simulation, a
//! step cap defers work rather than dropping it, and a trace replays to the
//! world it recorded.
//!
//! Split out of `runtime.rs` at the 600-line ceiling when PE3b added the
//! review. What the *checkpoint* and the *board* do to stepping are driven
//! claims and live in `tests/checkpoint.rs` and `tests/review.rs`.

use super::*;
use mesocosm_core::{PartId, Placement, Yaw};

fn scripted() -> Vec<Intent> {
    vec![
        Intent::Move { delta: [1, 0, 0] },
        Intent::Idle,
        Intent::Move { delta: [0, 0, 2] },
        Intent::Deposit { mass_mg: 25 },
        Intent::Metabolize {
            organism: mesocosm_core::OrganismId(0),
            placement: Placement::Explicit {
                parent: PartId(0),
                offset: [4, 0, 0],
                yaw: Yaw::Zero,
            },
        },
        Intent::Move { delta: [-1, 0, -1] },
    ]
}

#[test]
fn uneven_frames_do_not_change_the_simulation() {
    let steady = {
        let mut rt = Runtime::new(2024, 24, 60).with_max_steps(u64::MAX);
        for intent in scripted() {
            rt.queue(intent);
        }
        for _ in 0..6 {
            rt.advance(16_666);
        }
        rt
    };

    let ragged = {
        let mut rt = Runtime::new(2024, 24, 60).with_max_steps(u64::MAX);
        for intent in scripted() {
            rt.queue(intent);
        }
        // The same total time, delivered badly: a stall, a burst, a crawl.
        for chunk in [33u64, 1, 79_998, 12, 19_952] {
            rt.advance(chunk);
        }
        rt
    };

    assert_eq!(
        steady.trace(),
        ragged.trace(),
        "same intents in the same order"
    );
    assert_eq!(steady.state_hash(), ragged.state_hash());
}

#[test]
fn a_step_cap_delays_work_without_changing_it() {
    let uncapped = {
        let mut rt = Runtime::new(77, 16, 60).with_max_steps(u64::MAX);
        for intent in scripted() {
            rt.queue(intent);
        }
        rt.advance(100_000);
        rt
    };

    let capped = {
        let mut rt = Runtime::new(77, 16, 60).with_max_steps(2);
        for intent in scripted() {
            rt.queue(intent);
        }
        // One big frame, then idle frames while it catches up.
        rt.advance(100_000);
        for _ in 0..8 {
            rt.advance(0);
        }
        rt
    };

    assert_eq!(uncapped.trace().len(), capped.trace().len());
    assert_eq!(uncapped.state_hash(), capped.state_hash());
}

#[test]
fn empty_queue_idles_rather_than_stalling() {
    let mut rt = Runtime::new(3, 4, 60).with_max_steps(u64::MAX);
    rt.advance(16_666 * 4);
    assert_eq!(rt.trace().len(), 3);
    assert!(rt.trace().iter().all(|i| matches!(i, Intent::Idle)));
}

#[test]
fn trace_replays_to_the_same_world() {
    let mut rt = Runtime::new(555, 20, 60).with_max_steps(u64::MAX);
    for intent in scripted() {
        rt.queue(intent);
    }
    rt.advance(200_000);

    let (replayed, past) = Runtime::replay(555, 20, rt.trace());
    assert_eq!(state_hash(&replayed), rt.state_hash());
    assert_eq!(&past, rt.history(), "and the same run has the same past");
}

/// **A trace carrying dev intents replays like any other** (DT3), and the
/// replay counts the same dev intents the recording did.
///
/// This is the whole of the plan's second principle at the world-changing end:
/// a dev action that changed the world is an ordinary `Intent` in the trace, so
/// it reproduces exactly, and the count on the receipt is a function of the
/// trace rather than of who was at the keyboard.
#[test]
fn a_trace_carrying_dev_intents_replays_to_the_same_world_and_the_same_count() {
    let mut rt = Runtime::new(555, 20, 60).with_max_steps(u64::MAX);
    let parent = rt
        .world()
        .living()
        .find(|o| Some(o.id) != rt.world().controlled_id() && o.biomass_mg() > 400)
        .expect("somebody has a body to divide")
        .id;
    let doomed = rt
        .world()
        .living()
        .find(|o| Some(o.id) != rt.world().controlled_id() && o.id != parent)
        .expect("somebody else is alive")
        .id;
    let here = rt.world().position().expect("a played critter");

    for intent in [
        Intent::ForceBirth { organism: parent },
        Intent::Kill { organism: doomed },
        Intent::PlaceMatter {
            at: here,
            mass_mg: 400,
        },
        // One the world refuses, which must not be counted.
        Intent::Kill { organism: doomed },
        // Last, because the boundary it opens holds the world: an intent
        // queued behind it would wait for the question to be answered, which
        // is the checkpoint doing its job rather than a dev intent failing.
        Intent::EndEpoch,
    ] {
        rt.queue(intent);
    }
    assert_eq!(rt.step(5), 5, "nothing held the world until the last one");
    assert_eq!(
        rt.dev_intents(),
        4,
        "four applied, and the refused one is not one of them"
    );
    assert_eq!(
        rt.trace().iter().filter(|i| i.is_dev()).count(),
        5,
        "all five are in the trace, refused or not"
    );
    // **The demand runs PE3a's boundary and stops at PE3a's question.**
    assert!(matches!(
        rt.checkpoint().map(|held| held.occasion),
        Some(crate::succession::Occasion::Epoch(_))
    ));
    assert_eq!(rt.world().epoch, 1);

    let (replayed, past) = Runtime::replay(555, 20, rt.trace());
    assert_eq!(
        state_hash(&replayed),
        rt.state_hash(),
        "a dev intent replays like every other intent"
    );
    assert_eq!(&past, rt.history(), "and leaves the same past");

    // The driven route too, which is what `--replay` takes: it counts the same
    // four, because it is counting the world's answers rather than a keyboard.
    let mut driven = Runtime::new(555, 20, 60).with_max_steps(u64::MAX);
    for intent in rt.trace() {
        driven.queue(intent.clone());
    }
    driven.step(rt.trace().len() as u64);
    assert_eq!(driven.state_hash(), rt.state_hash());
    assert_eq!(driven.dev_intents(), 4);
}

/// A run with no dev intents in it counts none.
#[test]
fn an_ordinary_run_applies_no_dev_intents() {
    let mut rt = Runtime::new(555, 20, 60).with_max_steps(u64::MAX);
    for intent in scripted() {
        rt.queue(intent);
    }
    rt.advance(200_000);
    rt.step(50);
    assert_eq!(rt.dev_intents(), 0);
}

#[test]
fn a_driven_run_keeps_its_past() {
    // The world buffers one tick and drops it if nobody drains. Before the
    // driver recorded, every shipped run had a present and no history.
    let mut rt = Runtime::new(4_242, 40, 60);
    rt.step(200);

    assert!(!rt.history().is_empty(), "two hundred ticks left a record");
    assert!(!rt.readings().is_empty(), "and it comes to something");
}

/// **One boundary, one door** (DT4). The driver has no manual `end_epoch` any
/// more: a caller that wants the epoch closed now queues `Intent::EndEpoch`,
/// the world's own boundary block runs, and the driver reckons it through
/// `reckon_if_ended` exactly as it reckons a spent budget's. What used to be
/// two code paths that disagreed about whether the unplayed lines get a turn is
/// this one.
#[test]
fn ending_an_epoch_notes_what_the_run_did() {
    let mut rt = Runtime::new(4_242, 40, 60);
    rt.step(200);

    assert_eq!(rt.world().record().filled(), 0);
    rt.queue(Intent::EndEpoch);
    rt.step(1);

    assert!(!rt.reckoning().is_empty(), "the driver reckoned it");
    assert!(rt.world().record().filled() > 0, "the record has it now");
    assert_eq!(rt.world().epoch, 1);
    assert!(
        rt.world().at_boundary(),
        "and the world stands at its lineage checkpoint, which the deleted \
         manual door never left it doing"
    );
    assert_eq!(rt.world().last_round().epoch, 1, "the round ran");
}

#[test]
fn receipts_match_when_runs_match() {
    let run = |max: u64, chunks: &[u64]| {
        let mut rt = Runtime::new(8, 12, 60).with_max_steps(max);
        for intent in scripted() {
            rt.queue(intent);
        }
        for c in chunks {
            rt.advance(*c);
        }
        rt.receipt()
    };
    let a = run(u64::MAX, &[100_000]);
    let b = run(2, &[100_000, 0, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(a, b);
}

/// DT1's determinism receipt: pause (dropped elapsed time) and speed (scaled
/// elapsed time) are host pacing over `advance`, and the manual step keys are
/// `step` off the clock entirely — none of the three is a second input to
/// the simulation, only a second way of asking for the same one. A run driven
/// through the identical intents with pauses, speed changes and manual steps
/// interleaved between calls reaches the same hash as one driven straight.
///
/// `uneven_frames_do_not_change_the_simulation` above already proves ragged
/// `advance` chunks agree; this adds the two things DT1 introduces that test
/// does not exercise — a chunk of exactly zero (a pause) and `step` mixed
/// into the same run as `advance` (the step keys) — and states the claim in
/// the plan's own terms.
#[test]
fn pauses_speed_changes_and_manual_steps_do_not_change_the_hash() {
    // Six intents, repeated: long enough to run through every pacing shape
    // below more than once, short enough that nothing here grows a body far
    // enough to open a checkpoint (see `tests/checkpoint.rs` for what
    // `step`'s contract does once one stands).
    let intents: Vec<Intent> = scripted().into_iter().cycle().take(42).collect();
    let ticks_per_second = 10;
    let nominal_us = 1_000_000 / ticks_per_second as u64;

    let straight = {
        let mut rt = Runtime::new(9_001, 20, ticks_per_second).with_max_steps(u64::MAX);
        for intent in &intents {
            rt.queue(intent.clone());
        }
        for _ in 0..intents.len() {
            rt.advance(nominal_us);
        }
        rt
    };

    let paced = {
        let mut rt = Runtime::new(9_001, 20, ticks_per_second).with_max_steps(u64::MAX);
        for intent in &intents {
            rt.queue(intent.clone());
        }
        let total = intents.len() as u64;
        let mut taken = 0u64;
        let mut cycle = 0usize;
        while taken < total {
            // Every branch is capped at what remains, so a chunk near the
            // end of the run can never authorise a tick past the recording's
            // last one and pull in a synthetic `Idle` the straight run never
            // saw. The cap changes nothing about what a call is *asked* for
            // in the ordinary case — `remaining` only binds at the tail.
            let remaining = total - taken;
            taken += match cycle % 5 {
                // Paused: elapsed time dropped rather than banked, same as a
                // checkpoint hold — nothing runs. Never needs the cap: zero
                // can never overshoot.
                0 => rt.advance(0),
                // Quarter speed: on its own this authorises nothing (a
                // quarter tick is under the clock's threshold); several in a
                // row carry the remainder past a whole tick. Never needs the
                // cap either: the clock's own remainder is always under one
                // tick between calls, so a single quarter-tick addition can
                // push it past at most one boundary.
                1 => rt.advance(nominal_us / 4),
                // Ordinary speed: exactly one tick.
                2 => rt.advance(remaining.min(1) * nominal_us),
                // Quadruple speed: several ticks in one call.
                3 => rt.advance(remaining.min(4) * nominal_us),
                // The manual step key, off the clock entirely.
                _ => rt.step(remaining.min(3)),
            };
            cycle += 1;
        }
        rt
    };

    assert_eq!(
        straight.trace(),
        paced.trace(),
        "the same intents in the same order, however the host paced them"
    );
    assert_eq!(straight.state_hash(), paced.state_hash());
}

/// The pause half of DT1's contract, isolated: no elapsed time authorises no
/// steps. This is what a host does every frame while paused — it still calls
/// `advance`, but with nothing in it.
#[test]
fn advancing_with_zero_elapsed_time_takes_no_steps() {
    let mut rt = Runtime::new(9, 8, 60).with_max_steps(u64::MAX);
    rt.queue(Intent::Idle);
    assert_eq!(
        rt.advance(0),
        0,
        "no elapsed time authorises no steps — a host-side pause looks \
         exactly like a very slow frame"
    );
    assert!(rt.trace().is_empty());
}

#[test]
fn manual_stepping_matches_clocked_stepping() {
    let clocked = {
        let mut rt = Runtime::new(41, 10, 60).with_max_steps(u64::MAX);
        for intent in scripted() {
            rt.queue(intent);
        }
        // Exactly six steps at 60 Hz is 100_000 us, not 16_666 * 6. The
        // clock is drift-free, so the shortfall would run five.
        rt.advance(100_000);
        rt.state_hash()
    };
    let manual = {
        let mut rt = Runtime::new(41, 10, 60);
        for intent in scripted() {
            rt.queue(intent);
        }
        rt.step(6);
        rt.state_hash()
    };
    assert_eq!(clocked, manual);
}

/// DT2's determinism receipt, DT1's stated in the same terms: watching a body
/// reduces the stream the readings already reduce and writes nothing back, so
/// a run with an inspector on it and one without are the same world.
#[test]
fn watching_a_body_does_not_change_the_hash() {
    let intents: Vec<Intent> = scripted().into_iter().cycle().take(24).collect();
    let run = |watch: Option<OrganismId>| {
        let mut rt = Runtime::new(9_001, 20, 10).with_max_steps(u64::MAX);
        rt.watch(watch);
        for intent in &intents {
            rt.queue(intent.clone());
        }
        rt.step(intents.len() as u64);
        rt
    };
    let unwatched = run(None);
    let watched = run(Some(OrganismId(0)));
    assert_eq!(unwatched.trace(), watched.trace());
    assert_eq!(unwatched.state_hash(), watched.state_hash());

    // And the window is a real reading: it covers exactly the ticks it was
    // watched for, and the played body pays rent in every one of them.
    let accounts = watched.accounts();
    assert_eq!(accounts.ticks, intents.len() as u64);
    assert!(
        accounts.rent_mg > 0,
        "a living body spends something on standing still"
    );
    assert_eq!(unwatched.accounts(), mesocosm_core::Accounts::default());
}

/// Watching is idempotent, and a change of subject starts the window over —
/// figures carried across one would be somebody else's.
#[test]
fn rewatching_the_same_body_keeps_its_window_and_a_new_one_starts_over() {
    let mut rt = Runtime::new(9_001, 20, 10).with_max_steps(u64::MAX);
    rt.watch(Some(OrganismId(0)));
    rt.step(10);
    assert_eq!(rt.accounts().ticks, 10);

    rt.watch(Some(OrganismId(0)));
    assert_eq!(rt.accounts().ticks, 10, "the same body keeps its window");
    assert_eq!(rt.watched(), Some(OrganismId(0)));

    rt.watch(Some(OrganismId(1)));
    assert_eq!(rt.accounts(), mesocosm_core::Accounts::default());
    rt.step(4);
    assert_eq!(rt.accounts().ticks, 4);

    rt.watch(None);
    assert_eq!(rt.accounts(), mesocosm_core::Accounts::default());
    assert_eq!(rt.watched(), None);
}
