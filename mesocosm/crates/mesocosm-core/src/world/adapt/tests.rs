// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The scorer and the round, at the scale of the world. (P4b, PE3a)
//!
//! What a round does to a *descendant* is a birth claim and lives in
//! `tests/embodied/lineage.rs`.

use super::*;
use crate::rules::{EpochRule, WorldRules};

fn condition(n: u64) -> Option<ConditionId> {
    Some(ConditionId(n))
}

fn scored(income_mg: u64, rent_mg: u64) -> Score {
    Score {
        ticks: 60,
        income_mg,
        rent_mg,
        outflow_mg: 0,
        born: 0,
    }
}

/// A world whose epoch budget is short enough to reach a boundary in a test,
/// and whose scoring window is short enough that reaching several is cheap.
fn brisk(seed: u64, founders: u32) -> World {
    World::new(seed, founders).with_rules(
        WorldRules::native()
            .ending(EpochRule::Timed { ticks: 8 })
            .scoring_over(4),
    )
}

#[test]
fn a_score_states_its_window_and_derives_one_number_from_it() {
    // Figures, not a verdict: the ordering is the only place a score becomes a
    // rank, and it is income against rent and nothing else.
    let score = scored(900, 400);
    assert_eq!(score.ticks, 60);
    assert_eq!(score.net_mg(), 500);
    assert!(score.beats(scored(900, 401)));
    assert!(!score.beats(scored(900, 400)), "a tie is not a reason");
    assert!(
        !scored(10_000, 10_000).beats(scored(1, 1)),
        "earning more while spending it all is not earning more"
    );
}

#[test]
fn the_ordering_takes_the_best_candidate_that_beats_the_status_quo() {
    // The authored two-candidate case, and the three answers it can give.
    let standing = (None, scored(1_000, 400));
    let better = (condition(11), scored(1_400, 400));
    let best = (condition(13), scored(2_000, 400));

    assert_eq!(
        best_of(&[standing, better]),
        Some(ConditionId(11)),
        "more net income wins"
    );
    assert_eq!(
        best_of(&[standing, better, best]),
        Some(ConditionId(13)),
        "and the most of it wins over merely more"
    );
    assert_eq!(
        best_of(&[standing, best, better]),
        Some(ConditionId(13)),
        "whatever order they were weighed in"
    );
}

#[test]
fn the_status_quo_can_win() {
    // The outcome that keeps a line from revising itself to death out of an
    // obligation to spend. A real answer, and the first entry is what it is
    // measured against.
    let standing = (None, scored(1_000, 400));
    assert_eq!(best_of(&[standing]), None, "nothing else was on the table");
    assert_eq!(
        best_of(&[standing, (condition(7), scored(1_000, 900))]),
        None,
        "and a candidate that earns less net is not taken"
    );
    assert_eq!(
        best_of(&[standing, (condition(7), scored(1_000, 400))]),
        None,
        "nor one that ties it"
    );
}

#[test]
fn a_score_is_the_same_score_twice() {
    // Bounded and deterministic: same world, same candidate, same length.
    let world = brisk(4_242, 24);
    let species = world.controlled().expect("embodied").species;
    let first = world.score(species, None);
    let second = world.score(species, None);
    assert_eq!(first, second);
    assert_eq!(first.ticks, 4, "and it covers the window the rules name");
}

#[test]
fn scoring_leaves_the_world_it_reasons_about_untouched() {
    // The copy is discarded, and the real world's hash is the receipt. It also
    // could not be otherwise: `score` takes `&self`.
    let world = brisk(9_001, 24);
    let species = world.controlled().expect("embodied").species;
    let before = crate::snapshot::state_hash(&world);
    let _ = world.score(species, None);
    let _ = world.score(species, None);
    assert_eq!(crate::snapshot::state_hash(&world), before);
}

#[test]
fn the_founding_revision_is_always_a_candidate() {
    // So that "the status quo beat every candidate" is a real outcome rather
    // than an empty list.
    let world = brisk(7, 24);
    for species in world.initiative() {
        let candidates = world.candidates(species);
        assert_eq!(candidates.first(), Some(&None), "no change is first");
    }
}

#[test]
fn an_undiscovered_world_gives_every_line_nothing_to_weigh() {
    // Discovery is played-only (PE2) and PE3a does not change that: an
    // enclosure nobody has played holds no candidate for anybody, so its
    // rounds are empty. This is why the headless population instrument sees
    // the boundary and nothing else.
    let mut world = brisk(3, 24);
    assert!(world.discoveries().is_empty());
    for species in world.initiative() {
        assert_eq!(world.candidates(species).len(), 1);
    }
    for _ in 0..8 {
        world.apply(Intent::Idle);
    }
    assert!(world.at_boundary(), "the epoch still ends");
    assert!(
        world.last_round().turns.is_empty(),
        "and nobody took a turn in it"
    );
}

#[test]
fn initiative_is_descending_recipe_complexity_then_id() {
    // The named reading: `Species::recipe.complexity()`, which is what
    // `World::intricacy` reads and the frontier binds on.
    let world = brisk(11, 40);
    let order = world.initiative();
    assert!(order.len() > 1, "the roster founds several lines");

    let key = |id: &SpeciesId| {
        let complexity = world
            .lineages()
            .get(*id)
            .map(|line| line.recipe.complexity() as i64)
            .unwrap_or(0);
        (-complexity, id.0)
    };
    let mut expected = order.clone();
    expected.sort_by_key(key);
    assert_eq!(order, expected);

    let living: std::collections::BTreeSet<SpeciesId> = world.living().map(|o| o.species).collect();
    assert_eq!(
        order
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>(),
        living,
        "every living lineage, and only living ones"
    );
}

#[test]
fn the_timed_rule_ends_the_epoch_at_the_budget_and_not_before() {
    let mut world = World::new(4_242, 24)
        .with_rules(WorldRules::native().ending(EpochRule::Timed { ticks: 5 }));
    assert_eq!(world.epoch, 0);
    for tick in 1..5 {
        world.apply(Intent::Idle);
        assert_eq!(world.epoch, 0, "not at tick {tick}");
        assert!(!world.at_boundary());
    }
    world.apply(Intent::Idle);
    assert_eq!(world.epoch, 1, "and exactly at the budget");
    assert!(world.at_boundary());
    assert_eq!(world.epoch_began(), 5, "the next budget starts here");

    // And the boundary is one tick wide unless something holds it.
    world.apply(Intent::Idle);
    assert!(!world.at_boundary());
    assert_eq!(world.epoch, 1);

    for _ in 0..4 {
        world.apply(Intent::Idle);
    }
    assert_eq!(
        world.epoch, 2,
        "and it comes round again on the same budget"
    );
}

#[test]
fn a_world_under_an_unbuilt_rule_never_ends_an_epoch() {
    // Named as data, and honest about it: no condition behind the rule means
    // the epoch does not end rather than ending on a guess.
    for rule in [EpochRule::Gated, EpochRule::PlayerTriggered] {
        let mut world = World::new(4_242, 24).with_rules(WorldRules::native().ending(rule));
        for _ in 0..40 {
            world.apply(Intent::Idle);
        }
        assert_eq!(world.epoch, 0, "{rule:?} ends nothing");
        assert!(!world.at_boundary());
        assert!(!world.revision_admitted_now());
    }
}

#[test]
fn a_revision_is_admitted_only_at_the_lineage_checkpoint() {
    // The placeholder replaced. Bodies change between epochs and not during
    // them, so every other tick refuses by name.
    let mut world = World::new(4_242, 24)
        .with_rules(WorldRules::native().ending(EpochRule::Timed { ticks: 3 }));
    let species = world.controlled().expect("embodied").species;

    assert!(!world.revision_admitted_now(), "not at the founding tick");
    assert_eq!(
        world.revise(species, ConditionId(1)),
        Err(super::super::Unrevised::NotYet),
        "and the door says which refusal it is"
    );

    for _ in 0..3 {
        world.apply(Intent::Idle);
    }
    assert!(world.revision_admitted_now(), "and inside, it is admitted");
    assert_eq!(
        world.revise(species, ConditionId(1)),
        Err(super::super::Unrevised::Undiscovered(ConditionId(1))),
        "past the gate, the next refusal is the honest one"
    );
}
