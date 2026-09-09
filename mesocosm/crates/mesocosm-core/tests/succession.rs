// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Death is a hinge, not a wall.** PE1's core half: descent is readable off
//! the record the breeding transaction already writes, an eligible descendant
//! can be inhabited when a body ends, and the siblings that were not chosen go
//! on living in the enclosure rather than becoming a list.
//!
//! Nothing here is a second breeding system. Every birth below comes out of
//! `ecology::breeding` — the adult-mass gate, the filial realization, the
//! matter debit and the parent link exactly as they were. What the tests do is
//! put a critter in a state the ecology reaches on its own (mature, past
//! gestation) and then read what the ordinary tick produces, which is the same
//! move `ecology`'s own unit tests make.
//!
//! The unforced end-to-end proof — a natural death at the end of a lifespan,
//! followed by the line continuing — is the recorded demo trace, and it is
//! receipted in `mesocosm-genet`.

use mesocosm_core::history::{Event, History};
use mesocosm_core::{Ineligible, Intent, OrganismId, Outcome, Rejection, Stage, World};

/// Puts somebody in reach of the ordinary breeding gate and hands them the run.
///
/// Returns the organism, or `None` if this world had nobody both eligible and
/// heavy enough — which is a fact about the seed, not a failure.
fn take_a_breeder(world: &mut World) -> Option<OrganismId> {
    let roster: Vec<OrganismId> = world.living().map(|o| o.id).collect();
    for id in roster {
        let ready = {
            let Some(candidate) = world.organisms.iter_mut().find(|o| o.id == id) else {
                continue;
            };
            let was = (candidate.stage, candidate.since_offspring);
            candidate.stage = Stage::Mature;
            // Past any gestation this body's mass could ask for. The gate that
            // matters to the test is the adult-mass one below, which is left
            // exactly as the ecology sets it.
            candidate.since_offspring = u32::MAX;
            if candidate.can_reproduce() {
                true
            } else {
                (candidate.stage, candidate.since_offspring) = was;
                false
            }
        };
        if !ready {
            continue;
        }
        if world.is_eligible(id) {
            assert_eq!(
                world.apply(Intent::TakeControl { organism: id }),
                Outcome::Inhabited { organism: id }
            );
            return Some(id);
        }
    }
    None
}

/// Steps until the played critter bears a child, draining the past as a driver
/// would. Returns the offspring and the history that recorded it.
fn bear(world: &mut World, within: u64) -> (OrganismId, History) {
    let mut history = History::new();
    let parent = world.controlled_id().expect("somebody is playing");
    for _ in 0..within {
        world.apply(Intent::Idle);
        let events = world.drain_events();
        history.record_all(events.iter().copied());
        if let Some(child) = events.iter().find_map(|recorded| match recorded.record {
            Event::Born {
                organism,
                parent: Some(who),
                ..
            } if who == parent => Some(organism),
            _ => None,
        }) {
            return (child, history);
        }
    }
    panic!("the played critter did not breed within {within} ticks");
}

fn world_with_a_breeder(seed: u64) -> (World, OrganismId) {
    let mut world = World::new(seed, 60);
    // A few ticks so the enclosure is running rather than at genesis.
    for _ in 0..5 {
        world.apply(Intent::Idle);
    }
    let breeder = take_a_breeder(&mut world).expect("some founder is ready to breed");
    (world, breeder)
}

#[test]
fn descent_is_readable_off_the_link_the_birth_already_wrote() {
    let (mut world, parent) = world_with_a_breeder(11);
    let (child, history) = bear(&mut world, 40);

    assert_eq!(
        history.descendants(parent),
        vec![child],
        "one birth, one descendant, and no second link on the body to keep in step"
    );
    assert_eq!(
        world.heirs(&history, parent),
        vec![child],
        "and this world would let anyone hold it"
    );
    assert!(
        world.heirs(&history, child).is_empty(),
        "a newborn has no line of its own yet"
    );
}

/// Transitive, because a line continues through a grandchild when the child is
/// gone. Built by hand here: the walk is the claim, not the ecology.
#[test]
fn a_grandchild_is_a_descendant() {
    let mut history = History::new();
    let ids: Vec<OrganismId> = (0..4).map(OrganismId).collect();
    for (tick, (child, parent)) in [(1, 0), (2, 1), (3, 0)].into_iter().enumerate() {
        history.record(mesocosm_core::flow::Envelope::new(
            tick as u64,
            None,
            Event::Born {
                organism: ids[child],
                species: mesocosm_core::SpeciesId(1),
                parent: Some(ids[parent]),
            },
        ));
    }
    assert_eq!(
        history.descendants(ids[0]),
        vec![ids[1], ids[3], ids[2]],
        "both children, eldest first, then the grandchild"
    );
    assert_eq!(history.descendants(ids[2]), Vec::new());
}

/// **The done-condition.** A body ends, the world says whose it was, and an
/// eligible descendant carries the run on. Before this the session simply
/// refused every input for the rest of its life.
#[test]
fn death_continues_through_an_eligible_descendant() {
    let (mut world, parent) = world_with_a_breeder(11);
    let (child, history) = bear(&mut world, 40);

    // The end of a body, reached the short way. What the ecology does to get
    // here is its own business and is proved next door in `matter.rs`; what
    // this test is about is what happens afterwards.
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == parent)
        .expect("the parent is still in the roster")
        .stage = Stage::Carrion;

    world.apply(Intent::Idle);
    assert_eq!(
        world.control_lost(),
        Some(parent),
        "the world names whose body it was"
    );
    assert!(!world.is_embodied(), "and nobody is playing until they say");

    // Declining is an answer, and the world keeps running through it.
    assert_eq!(world.apply(Intent::Resume), Outcome::Resumed);
    assert!(!world.is_embodied());

    let heirs = world.heirs(&history, parent);
    assert_eq!(heirs, vec![child], "the line survives its founder");
    assert_eq!(
        world.apply(Intent::TakeControl { organism: child }),
        Outcome::Inhabited { organism: child },
        "and the run continues in it"
    );
    assert_eq!(world.controlled_id(), Some(child));

    // Play resumes for real: an ordinary verb lands again.
    assert!(
        !matches!(
            world.apply(Intent::Deposit { mass_mg: 1 }),
            Outcome::Rejected(Rejection::Disembodied)
        ),
        "the wall is gone"
    );
}

/// Siblings stay organisms. Nothing about a checkpoint removes one from the
/// enclosure, reserves one, or holds one still — which is the difference
/// between an ecology and an inventory screen.
#[test]
fn siblings_persist_in_the_ecology_rather_than_becoming_inventory() {
    let (mut world, parent) = world_with_a_breeder(11);
    let (first, mut history) = bear(&mut world, 40);

    // A second child, so there is a sibling to not choose.
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == parent)
        .expect("alive")
        .since_offspring = u32::MAX;
    let (second, more) = bear(&mut world, 40);
    for recorded in more.log().entries() {
        history.record(*recorded);
    }
    assert_ne!(first, second);

    // The run takes one of them. The other is untouched by that.
    assert_eq!(
        world.apply(Intent::TakeControl { organism: second }),
        Outcome::Inhabited { organism: second }
    );

    let before = world
        .living()
        .find(|o| o.id == first)
        .map(|o| (o.position, o.age, o.biomass_mg()))
        .expect("the sibling is alive");
    for _ in 0..60 {
        world.apply(Intent::Idle);
    }
    let after = world.living().find(|o| o.id == first).map(|o| o.age);
    assert!(
        after.is_none_or(|age| age > before.1),
        "the unchosen sibling went on living its own life, or was eaten doing it"
    );
    assert!(
        world.heirs(&history, parent).contains(&first) || after.is_none(),
        "and it is still simply an organism in the enclosure"
    );
}

/// The two answers are ordinary intents, so a trace carries the choice, a
/// replay makes it again, and a run that answered is not the same run as one
/// that let the question stand.
#[test]
fn the_choice_is_in_the_trace_and_replays() {
    let taken = {
        let mut world = World::new(11, 60);
        world.apply(Intent::Idle);
        world
            .living()
            .map(|o| o.id)
            .find(|id| Some(*id) != world.controlled_id() && world.is_eligible(*id))
            .expect("somebody in this enclosure is inhabitable")
    };
    let answered = vec![
        Intent::Idle,
        Intent::Resume,
        Intent::TakeControl { organism: taken },
        Intent::Idle,
    ];
    let unanswered = vec![Intent::Idle, Intent::Idle, Intent::Idle, Intent::Idle];

    let run = |trace: &[Intent]| {
        let mut world = World::new(11, 60);
        world.apply_all(trace);
        world
    };
    let once = run(&answered);
    assert_eq!(once.controlled_id(), Some(taken));

    assert_eq!(
        mesocosm_core::state_hash(&once),
        mesocosm_core::state_hash(&run(&answered)),
        "the same trace against the same seed lands on the same world"
    );
    assert_ne!(
        mesocosm_core::state_hash(&once),
        mesocosm_core::state_hash(&run(&unanswered)),
        "and a run that answered is distinguishable from one that never did"
    );
}

/// Answering is a hand on the critter. A run that resumed and then went quiet
/// keeps its body for the full idle window rather than being handed back to
/// instinct for having been asked something.
#[test]
fn resuming_is_a_hand_and_not_an_idle() {
    let mut world = World::new(11, 60);
    for _ in 0..mesocosm_core::INSTINCT_IDLE_TICKS {
        world.apply(Intent::Idle);
    }
    assert_eq!(world.held(), None, "the hand is off");

    assert_eq!(world.apply(Intent::Resume), Outcome::Resumed);
    assert_eq!(world.idle_run(), 0, "answering reset the run");
    assert_eq!(
        world.held(),
        world.controlled_id(),
        "and the critter is back under the hand"
    );
}

/// A descendant is reached through the one control gate, not a private one.
#[test]
fn an_heir_that_is_not_eligible_is_not_offered() {
    let (mut world, parent) = world_with_a_breeder(11);
    let (child, history) = bear(&mut world, 40);

    world
        .organisms
        .iter_mut()
        .find(|o| o.id == child)
        .expect("the newborn is in the roster")
        .stage = Stage::Carrion;

    assert_eq!(
        history.descendants(parent),
        vec![child],
        "the record still says it was born"
    );
    assert!(
        world.heirs(&history, parent).is_empty(),
        "but a carcass is not a critter you can play"
    );
    assert_eq!(
        world.eligibility(child),
        Err(Ineligible::NotAlive),
        "and the same gate says so"
    );
}
