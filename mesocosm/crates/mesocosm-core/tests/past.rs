// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The world acquires a past.
//!
//! `World::apply` used to return an outcome and drop it, so nothing that reads
//! history had anything to read. These tests run a real enclosure and check
//! that what comes out is a causal graph rather than a list of things that
//! happened to occur in that order.

use mesocosm_core::{Event, History, Intent, Placement, World, snapshot, state_hash};

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
fn an_enclosure_left_alone_still_has_a_past() {
    // Nobody played, and things still happened. That is the difference between
    // an ecology and a field of pickups.
    let (_, history) = lived(300);

    assert!(!history.is_empty(), "something happened");
    assert!(
        history
            .log()
            .entries()
            .iter()
            .any(|e| matches!(e.record, Event::Fed { .. })),
        "including creatures eating each other"
    );
}

#[test]
fn a_creature_gets_a_line_of_its_own() {
    let (_, history) = lived(300);

    let subject = history
        .log()
        .entries()
        .iter()
        .flat_map(|event| event.record.subjects())
        .find(|who| history.line_of(*who).len() > 2)
        .expect("somebody had a life");

    let line = history.line_of(subject);
    assert!(line.windows(2).all(|w| w[0] < w[1]), "oldest first");
    for seq in &line {
        assert!(
            history
                .get(*seq)
                .unwrap()
                .record
                .subjects()
                .contains(&subject),
            "every event in a creature's line is about it"
        );
    }
}

#[test]
fn feeding_joins_lines_that_were_independent() {
    // The property a flat log cannot express. Two creatures born apart have
    // nothing to do with each other until one eats the other.
    let (_, history) = lived(300);

    let meal = history
        .log()
        .entries()
        .iter()
        .position(|e| matches!(e.record, Event::Fed { .. }))
        .map(|i| muniment::Seq(i as u64))
        .expect("something fed");

    let Some(Event::Fed { eater, from, .. }) = history.event(meal).copied() else {
        unreachable!("filtered to a meal")
    };
    assert_ne!(eater, from, "nothing eats itself");
    assert!(
        !history.antecedents(meal).is_empty(),
        "it followed from something"
    );
}

#[test]
fn most_of_what_happens_is_concurrent() {
    // The honest shape of an ecology: creatures act independently, and a list
    // would have to invent an order between them and then imply it meant
    // something.
    let (_, history) = lived(200);
    assert!(history.len() > 10, "enough happened to sample");

    let mut concurrent = 0;
    let mut compared = 0;
    for a in 0..history.len().min(40) {
        for b in a + 1..history.len().min(40) {
            compared += 1;
            if history.concurrent(muniment::Seq(a as u64), muniment::Seq(b as u64)) {
                concurrent += 1;
            }
        }
    }

    assert!(
        concurrent * 2 > compared,
        "most pairs are unrelated ({concurrent} of {compared}), which is what a \
         sequence alone would misrepresent"
    );
}

#[test]
fn a_death_follows_from_what_drained_it() {
    // The retroactive question, on real data: what led here.
    let (_, history) = lived(400);

    let died = history
        .log()
        .entries()
        .iter()
        .position(|e| matches!(e.record, Event::Died { .. }))
        .map(|i| muniment::Seq(i as u64))
        .expect("something died");

    let before = history.antecedents(died);
    assert!(!before.is_empty(), "a death has a history behind it");

    // And the reachability runs the other way too.
    for cause in &before {
        assert!(
            history.consequences(*cause).contains(&died),
            "if it led to the death, the death is among its consequences"
        );
    }
}

#[test]
fn the_player_act_is_recorded_before_the_tick_it_happened_in() {
    // Order matters: a creature's last act must precede its death, not follow
    // it. An earlier cut pushed the player's event after the ecology's.
    let mut world = World::new(4_242, 40);
    let me = world.controlled_id().unwrap();
    // The founding population's births are already pending; they belong to the
    // world's beginning rather than to the tick the burn happens in.
    world.drain_events();

    // Walk to something and burn it.
    let mut burned = None;
    for _ in 0..400 {
        let here = world.position().unwrap();
        let Some((prey, at)) = world
            .organisms
            .iter()
            .filter(|o| Some(o.id) != world.controlled_id() && o.is_alive())
            .map(|o| (o.id, o.position))
            .min_by_key(|(_, at): &(_, [i32; 3])| {
                (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0)
            })
        else {
            break;
        };
        if world.in_reach(at) {
            // Empty the budget first: since TD4 the body routes its own meal,
            // and a burn is what a starved one does with it.
            world
                .organisms
                .iter_mut()
                .find(|o| o.id == me)
                .expect("still embodied")
                .energy_mg = 0;
            world.apply(Intent::Metabolize {
                organism: prey,
                placement: Placement::Planned,
            });
            burned = Some(world.drain_events());
            break;
        }
        world.apply(Intent::Move {
            delta: [0, 1, 2].map(|a| (at[a] - here[a]).signum()),
        });
        world.drain_events();
    }

    let events = burned.expect("something was burned");
    let mine = events
        .iter()
        .position(|e| matches!(e.record, Event::Burned { organism, .. } if organism == me))
        .expect("the burn was recorded");

    assert_eq!(mine, 0, "the player's act leads the tick it happened in");
}

#[test]
fn the_world_holds_only_one_tick_of_events() {
    // The bounded buffer that keeps snapshots from growing. History lives
    // beside the world precisely so the world does not accumulate it.
    let mut world = World::new(4_242, 40);
    for _ in 0..200 {
        world.apply(Intent::Idle);
    }

    let undrained = world.events().len();
    world.apply(Intent::Idle);
    assert!(
        world.events().len() < undrained + 50,
        "two hundred undrained ticks did not accumulate two hundred ticks of events"
    );
}

#[test]
fn history_is_derivable_and_therefore_reproducible() {
    // Why it is safe to keep history outside the snapshot: the same seed and
    // the same intents produce the same past, so nothing is lost by not
    // capturing it.
    let (a_world, a) = lived(120);
    let (b_world, b) = lived(120);

    assert_eq!(state_hash(&a_world), state_hash(&b_world));
    assert_eq!(a, b, "the same run has the same past");
}

#[test]
fn a_past_persists_on_its_own() {
    let (_, history) = lived(120);
    let bytes = mesocosm_core::snapshot::encode(&history).unwrap();
    assert_eq!(
        mesocosm_core::snapshot::decode::<History>(&bytes).unwrap(),
        history
    );
}

#[test]
fn a_snapshot_does_not_carry_the_past() {
    // The thing that would have broken cheap whole-state capture. Two worlds
    // that lived equally long snapshot to the same size regardless of how much
    // history was recorded beside them.
    let (mut short, _) = lived(20);
    let (mut long, long_history) = lived(300);
    short.drain_events();
    long.drain_events();

    assert!(
        long_history.len() > 20,
        "the long run really did accumulate a past"
    );
    let brief = snapshot(&short).unwrap().len();
    let lengthy = snapshot(&long).unwrap().len();

    // Sizes differ because the enclosures differ, not because one carries more
    // history: neither carries any.
    assert!(
        lengthy < brief * 4,
        "a long-lived world's snapshot is not proportional to its history \
         ({brief} then {lengthy})"
    );
}
