// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PE3a: the round, in a world. One lineage turn, end to end.
//!
//! Split out of `lineage.rs` at the 600-line ceiling. What lives next door is
//! P4a and PD5 — what a *commit* is and what a birth makes of one; what lives
//! here is the epoch boundary running that commit on its own, for a line
//! nobody is holding, off a score nobody wrote down in advance.
//!
//! The scorer's own claims — determinism, the untouched hash, the ordering —
//! are unit tests beside it in `src/world/adapt/tests.rs`. This is the join.

use mesocosm_core::discovery::HUNGER_TICKS;
use mesocosm_core::history::Event;
use mesocosm_core::rules::{EpochRule, WorldRules};
use mesocosm_core::{Intent, OrganismId, Stage};

use super::bulk_world;
use super::discovery::{endure, hunger};
use super::gland::{frond_on, gland};

#[test]
fn an_unplayed_line_takes_its_turn_at_the_boundary_and_its_next_birth_expresses() {
    // **PE3a's headline** (P4b + PE3a joined). The epoch's budget runs out, the
    // world runs a round, every unplayed line scores its candidates by growing
    // them in copies of itself, and a line that earns more net income with the
    // gland than without it commits — through `World::revise`, the identical
    // transaction the played door reaches. Then the ordinary birth pass hands
    // its next descendant the program, through PD5's identical code.
    let mut world = bulk_world(4_242, 24);
    frond_on(&mut world);
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()));

    // **One line grown up.** A revision only ever shows up in descendants, so
    // a scoring window that sees no birth of the line scores the candidate and
    // the status quo identically. This matures the biggest line that could
    // carry a gland at all, and leaves the rest of the enclosure exactly as it
    // was: the breeding gate is untouched, and the fixture pays for one line's
    // births rather than the whole roster's.
    let grown = world
        .initiative()
        .into_iter()
        .filter(|line| Some(*line) != world.controlled().map(|o| o.species))
        .filter(|line| world.candidates(*line).len() > 1)
        .max_by_key(|line| world.living().filter(|o| o.species == *line).count())
        .expect("some unplayed line can carry a gland");
    for organism in world.organisms.iter_mut() {
        if organism.species == grown {
            organism.stage = Stage::Mature;
        }
    }

    // To the next boundary, on a budget short enough for a test. The scoring
    // window is shortened with it: what this test is about is the round, and
    // the shipped window's own behaviour is receipted at demo scale.
    let mut world = world.with_rules(
        WorldRules::native()
            .ending(EpochRule::Timed { ticks: 1 })
            .scoring_over(240),
    );
    world.apply(Intent::Idle);

    let round = world.last_round().clone();
    assert!(
        !round.turns.is_empty(),
        "unplayed lines had a candidate to weigh"
    );
    let mine = world.controlled().expect("embodied").species;
    assert!(
        round.turn(mine).is_none(),
        "and the played line is left to the player: its turn is the review"
    );
    for turn in &round.turns {
        assert_eq!(
            turn.considered.first().map(|(candidate, _)| *candidate),
            Some(None),
            "the status quo is always on the table"
        );
        assert_eq!(
            turn.considered[0].1.ticks, 240,
            "over the window the rules name"
        );
        // The ordering is net income and nothing else, at the point it was used.
        let standing = turn.considered[0].1.net_mg();
        match turn.chosen {
            Some(condition) => {
                let taken = turn
                    .considered
                    .iter()
                    .find(|(candidate, _)| *candidate == Some(condition))
                    .expect("what it chose is what it weighed")
                    .1;
                assert!(taken.net_mg() > standing, "it earns more net than standing");
                assert!(turn.committed.is_some(), "and the commit went through");
            },
            None => assert!(
                turn.considered
                    .iter()
                    .skip(1)
                    .all(|(_, score)| score.net_mg() <= standing),
                "nothing beat the status quo"
            ),
        }
    }

    let Some(changed) = round.changes().next() else {
        panic!("no unplayed line took the gland: {round:?}");
    };
    let line = changed.lineage;
    let revision = changed.committed.expect("committed");
    assert_eq!(
        world
            .lineages()
            .get(line)
            .expect("the line")
            .program()
            .current()
            .map(|current| current.id),
        Some(revision),
        "and the line is now born under it"
    );

    // Its next descendant, through the ordinary birth pass, with nobody in it.
    // The line is already past the gate, so this waits for the ecology to bear
    // one rather than reaching in and making it — and the epoch budget goes
    // back to this build's own first, because a round per tick would score the
    // same candidates eighty more times to learn nothing.
    let mut world = world.with_rules(WorldRules::native());
    let mut expressed = None;
    for _ in 0..80 {
        world.apply(Intent::Idle);
        let events: Vec<Event> = world
            .drain_events()
            .into_iter()
            .map(|recorded| recorded.record)
            .collect();
        let born: Vec<OrganismId> = events
            .iter()
            .filter_map(|event| match *event {
                Event::Born {
                    organism,
                    species,
                    parent: Some(_),
                } if species == line => Some(organism),
                _ => None,
            })
            .collect();
        if born.is_empty() {
            continue;
        }
        assert!(
            born.iter()
                .all(|child| Some(*child) != world.controlled_id()),
            "nobody is holding these"
        );
        if let Some(child) = born.into_iter().find(|child| {
            events.iter().any(|event| {
                matches!(
                    *event,
                    Event::Inherited { organism, revision: under, .. }
                        if organism == *child && under == revision
                )
            })
        }) {
            expressed = Some(child);
            break;
        }
    }
    let child = expressed.expect("a descendant of the line was born under the revision");
    assert!(
        world
            .living()
            .find(|o| o.id == child)
            .expect("alive")
            .phenotype
            .expresses(gland()),
        "and it is born expressing it"
    );
}
