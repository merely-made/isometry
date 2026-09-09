// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The evaluator's own claims, without a world in the way.
//!
//! What the world does *with* a discovery — the accumulator, the record, the
//! lexicon, the part-level meal — is `tests/embodied/discovery.rs`. These are
//! about the routing itself.

use super::*;
use crate::process::{Process, Registry};

fn empty() -> BTreeSet<ConditionId> {
    BTreeSet::new()
}

fn hunger(ticks: u64) -> Evidence {
    Evidence::Endured {
        stress: Stress::Hunger,
        ticks,
    }
}

fn meal(role: Role, mass_mg: u64) -> Evidence {
    Evidence::Meal {
        donor: SpeciesId(3),
        part: PartId(2),
        role,
        mass_mg,
    }
}

fn condition(name: &str) -> Condition {
    conditions()
        .into_iter()
        .find(|found| found.name == name)
        .expect("the table holds it")
}

#[test]
fn evidence_reaches_only_the_conditions_that_declared_its_lane() {
    // **The execution boundary.** A meal is not routed to a condition that
    // never asked about meals, and the record says that is why rather than
    // saying the rule went unmet — which would be a different and false claim.
    let verdict = evaluate(meal(Role::Plate, 500), 10, 0, &empty());
    let hunger_id = condition("mesocosm:endured-hunger").id();
    assert_eq!(
        verdict
            .observation
            .missed
            .iter()
            .find(|(id, _)| *id == hunger_id)
            .map(|(_, why)| *why),
        Some(Miss::UndeclaredInput),
        "a meal cannot be offered to an endurance condition at all"
    );

    // And the other way: enduring hunger is never offered to the meal
    // condition, however long it lasted.
    let verdict = evaluate(hunger(HUNGER_TICKS * 10), 10, 0, &empty());
    let plate_id = condition("mesocosm:plate-eaten").id();
    assert_eq!(
        verdict
            .observation
            .missed
            .iter()
            .find(|(id, _)| *id == plate_id)
            .map(|(_, why)| *why),
        Some(Miss::UndeclaredInput)
    );
}

#[test]
fn a_condition_unrelated_to_food_grants_a_candidate() {
    let verdict = evaluate(hunger(HUNGER_TICKS), 40, 2, &empty());
    let discovery = verdict.discovery.expect("coming through it is enough");

    assert_eq!(discovery.route, Input::Endurance);
    assert_eq!(discovery.source, Source::Endured, "there is no donor");
    assert_eq!(discovery.tick, 40);
    assert_eq!(discovery.epoch, 2);
    assert_eq!(
        discovery.candidate.process,
        Registry::native().of_native(Process::Secrete).reference(),
        "the exact admitted definition, not a name"
    );
    assert_eq!(discovery.candidate.site, Role::Plate);
    assert!(discovery.digest != 0);
    assert_eq!(
        name_of(discovery.condition),
        Some("mesocosm:endured-hunger")
    );
}

#[test]
fn a_rule_that_is_not_met_is_a_different_answer_from_a_lane_that_is_not_declared() {
    // One tick short. The lane is right and the evidence is not enough, which
    // is what a quantified stress means.
    let verdict = evaluate(hunger(HUNGER_TICKS - 1), 1, 0, &empty());
    assert!(verdict.discovery.is_none());
    let hunger_id = condition("mesocosm:endured-hunger").id();
    assert_eq!(
        verdict
            .observation
            .missed
            .iter()
            .find(|(id, _)| *id == hunger_id)
            .map(|(_, why)| *why),
        Some(Miss::RuleUnmet)
    );
}

#[test]
fn the_second_time_is_just_a_meal() {
    let id = condition("mesocosm:plate-eaten").id();
    let known = BTreeSet::from([id]);
    let verdict = evaluate(meal(Role::Plate, 500), 5, 0, &known);
    assert!(verdict.discovery.is_none());
    assert_eq!(
        verdict
            .observation
            .missed
            .iter()
            .find(|(found, _)| *found == id)
            .map(|(_, why)| *why),
        Some(Miss::AlreadyKnown)
    );
    assert!(
        verdict.observation.matched.is_none(),
        "but the evidence is still recorded"
    );
}

#[test]
fn a_meal_of_the_wrong_organ_teaches_nothing() {
    // The narrowing that replaced the old lesson: only a plate teaches a
    // plate. Bulk is food.
    for role in [Role::Mass, Role::Limb, Role::Sensor] {
        let verdict = evaluate(meal(role, 5_000), 1, 0, &empty());
        assert!(
            verdict.discovery.is_none(),
            "{role:?} should not have taught anything"
        );
        assert!(
            verdict.observation.matched.is_none(),
            "and the observation says so"
        );
    }
    assert!(
        evaluate(meal(Role::Plate, 5_000), 1, 0, &empty())
            .discovery
            .is_some(),
        "while the organ the condition names does"
    );
}

#[test]
fn a_condition_digest_moves_when_a_rule_bearing_byte_does() {
    let base = condition("mesocosm:plate-eaten");
    let mut widened = base;
    widened.rule = Rule::Consumed {
        role: Role::Plate,
        mass_mg: MEAL_EVIDENCE_MG + 1,
    };
    assert_ne!(base.id(), widened.id(), "the threshold is rule-bearing");

    let mut relaned = base;
    relaned.inputs = &[Input::Meal, Input::Endurance];
    assert_ne!(base.id(), relaned.id(), "so is the declared lane");

    let mut regranted = base;
    regranted.grants.cells = base.grants.cells + 1;
    assert_ne!(base.id(), regranted.id(), "and so are the parameters");
}

#[test]
fn every_condition_declares_at_least_one_lane_and_resolves() {
    // A condition nothing can reach is dead weight, and one whose digest does
    // not resolve cannot be named in a record.
    for condition in conditions() {
        assert!(
            !condition.inputs.is_empty(),
            "{} declares nothing",
            condition.name
        );
        assert_eq!(name_of(condition.id()), Some(condition.name));
    }
}

#[test]
fn an_evidence_word_keeps_its_quantity() {
    // A reading says what happened and how much of it, never a bare verdict.
    assert_eq!(hunger(120).words(), "hunger for 120 ticks");
    assert_eq!(
        meal(Role::Plate, 300).words(),
        "plate part 2 of line 3, 300 mg"
    );
}
