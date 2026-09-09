// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! TD4: the hand, and what happens when it stops.
//!
//! The first real playtest's complaint was that no keypress ever displaced the
//! critter — the ecology was spending its budget wandering it around while the
//! player pressed keys at it. The answer Mark ruled on 2026-08-29 is not to
//! exempt the played body from the ecology: it is to give the hand the body
//! **while the hand is there**, and give it back when the hand goes still.
//! The idle terrarium is the feature, not the failure.
//!
//! Everything here is a function of the trace, which is why it can live in the
//! world at all: count the trailing idles and you know who is driving.

use mesocosm_core::{
    INSTINCT_IDLE_TICKS, Intent, OrganismId, Outcome, Placement, World, state_hash,
};

/// An intent the world always refuses. It is the only way to spend a tick
/// *without* idling and without moving, which is exactly the state a player
/// holding a body but not displacing it is in.
fn refused() -> Intent {
    Intent::Metabolize {
        organism: OrganismId(u32::MAX),
        placement: Placement::Planned,
    }
}

/// A populated world whose played critter starts on an empty budget.
///
/// The seed moved from 4,242 on 2026-08-29: TD8 made travel conditional on
/// carrying an actuator, and that seed founds its played critter from an
/// unlimbed recipe — twenty-two parts and nothing that contracts — so every
/// claim here about who is walking the body would have been vacuously true.
fn restless() -> World {
    emptied(World::new(4_244, 60))
}

/// The same empty budget with nothing in the enclosure to fill it.
///
/// TD5 retired the plain [`restless`] fixture for the wander claim: an NPC's
/// feeding now credits its reserve, so a critter with prey in reach eats its
/// way out of hunger on the first tick and then stands still — correctly.
/// Wandering needs a body that is hungry *and* has nothing to eat.
fn stranded() -> World {
    emptied(World::new(4_244, 0))
}

fn emptied(mut world: World) -> World {
    let me = world.controlled_id().expect("the fixture starts embodied");
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .expect("in the roster")
        .energy_mg = 0;
    world
}

fn played_position(world: &World) -> Option<[i32; 3]> {
    world.controlled().map(|organism| organism.position)
}

#[test]
fn a_held_critter_is_not_walked_by_its_own_instincts() {
    let mut world = restless();
    let start = played_position(&world).unwrap();

    // Sixty ticks of a hand that is present and getting nowhere. Every one of
    // them resets the idle run, so the body stays the player's throughout.
    for _ in 0..60 {
        assert!(matches!(world.apply(refused()), Outcome::Rejected(_)));
        assert_eq!(world.idle_run(), 0, "a refusal is still a hand on the keys");
        assert_eq!(
            played_position(&world),
            Some(start),
            "nothing may displace a held body but the player"
        );
    }
}

#[test]
fn a_held_critter_still_ages_pays_rent_and_can_die() {
    // The gate is narrow on purpose. Sparing the played body its *locomotion*
    // is not the same as exempting it from the enclosure, and a critter that
    // could not starve while you held it would make standing still a strategy.
    let mut world = World::new(4_242, 60);
    let before = world.controlled().expect("embodied").clone();
    let me = before.id;

    // What it ate while being held, counted rather than assumed. TD7's pyramid
    // founds forty producers, so a held grazer now finds a meal most ticks and
    // a falling reserve is no longer the way to see the rent: it is out-earning
    // it. The ledger still shows the payment.
    let mut eaten = 0u64;
    for _ in 0..50 {
        world.apply(refused());
        for event in world.drain_events() {
            if let mesocosm_core::Event::Fed {
                eater, mass_mg: mg, ..
            } = event.record
                && eater == me
            {
                eaten += mg;
            }
        }
    }

    let after = world.controlled().expect("still alive after fifty ticks");
    assert!(after.age > before.age, "a held body still gets older");
    assert!(
        after.energy_mg + after.biomass_mg() < before.energy_mg + before.biomass_mg() + eaten,
        "and still pays for being alive: it holds {} mg against the {} mg it \
         started with plus the {eaten} mg it ate",
        after.energy_mg + after.biomass_mg(),
        before.energy_mg + before.biomass_mg(),
    );
}

#[test]
fn the_enclosure_keeps_moving_while_the_hand_is_on_one_body() {
    // Only the held critter is spared. TD4 is a rule about one organism, and
    // the ant farm has to go on being an ant farm around it.
    let mut world = restless();
    let others: Vec<(OrganismId, [i32; 3])> = world
        .living()
        .filter(|o| Some(o.id) != world.controlled_id())
        .map(|o| (o.id, o.position))
        .collect();
    assert!(
        others.len() > 10,
        "the fixture founds a populated enclosure"
    );

    for _ in 0..30 {
        world.apply(refused());
    }

    let moved = others
        .iter()
        .filter(|(id, was)| world.living().any(|o| o.id == *id && o.position != *was))
        .count();
    assert!(
        moved > 0,
        "the uncontrolled organisms went on living their own lives"
    );
}

#[test]
fn walking_away_gives_the_body_back_to_the_ecology() {
    // The done-condition in a sentence: hold the keys and nothing fights you;
    // put them down and the critter is an animal again.
    let mut held = stranded();
    let mut let_alone = held.clone();
    let start = played_position(&held).unwrap();

    // Long enough to cross the threshold and wander, short enough that an
    // emptied budget has not yet eaten the body out from under the comparison.
    let ticks = INSTINCT_IDLE_TICKS as usize + 20;
    for _ in 0..ticks {
        held.apply(refused());
        let_alone.apply(Intent::Idle);
    }

    assert_eq!(
        played_position(&held),
        Some(start),
        "the hand held it for the whole run"
    );
    assert_ne!(
        played_position(&let_alone),
        Some(start),
        "and the idle terrarium walked it off on its own drives"
    );
}

#[test]
fn the_instincts_wait_out_the_documented_threshold_and_not_a_tick_less() {
    // The count is the whole mechanism, so it is worth reading directly: it
    // rises with consecutive idles, any other intent puts it back to zero, and
    // the hand is judged to be gone exactly at the constant.
    let mut world = restless();

    for tick in 1..INSTINCT_IDLE_TICKS {
        world.apply(Intent::Idle);
        assert_eq!(world.idle_run(), tick);
        assert!(world.held().is_some(), "still under the hand at {tick}");
    }

    world.apply(Intent::Idle);
    assert_eq!(world.idle_run(), INSTINCT_IDLE_TICKS);
    assert!(world.held().is_none(), "let go at the threshold");
    // Control did not move. Nothing was handed over and nothing has to be
    // reclaimed: the next keypress simply lands.
    assert!(world.controlled_id().is_some());

    world.apply(refused());
    assert_eq!(world.idle_run(), 0, "one act and the hand is back");
    assert!(world.held().is_some());
}

#[test]
fn the_idle_run_is_a_function_of_the_trace_and_replays() {
    // Why this could be world state at all. It is derivable from the ordered
    // intents, so recording it costs a replay nothing and buys every host the
    // same answer at every frame rate.
    let trace: Vec<Intent> = (0..50)
        .map(|step| {
            if step % 17 == 16 {
                refused()
            } else {
                Intent::Idle
            }
        })
        .collect();

    let mut once = restless();
    let mut twice = restless();
    once.apply_all(&trace);
    twice.apply_all(&trace);
    assert_eq!(state_hash(&once), state_hash(&twice));

    // And it is not decoration: a run whose hand never let go ends somewhere
    // else entirely.
    let mut busy = restless();
    busy.apply_all(&vec![refused(); trace.len()]);
    assert_ne!(state_hash(&once), state_hash(&busy));
}
