// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::BTreeSet;

fn sites() -> Vec<BodySite> {
    vec![
        BodySite {
            part: PartRef {
                subject: 4,
                part: 0,
            },
            role: SiteRole::Source,
        },
        BodySite {
            part: PartRef {
                subject: 4,
                part: 1,
            },
            role: SiteRole::Store,
        },
        BodySite {
            part: PartRef {
                subject: 4,
                part: 2,
            },
            role: SiteRole::Gate,
        },
        BodySite {
            part: PartRef {
                subject: 4,
                part: 3,
            },
            role: SiteRole::Actuator,
        },
        BodySite {
            part: PartRef {
                subject: 4,
                part: 4,
            },
            role: SiteRole::Actuator,
        },
        BodySite {
            part: PartRef {
                subject: 4,
                part: 5,
            },
            role: SiteRole::Actuator,
        },
    ]
}

#[test]
fn same_seed_is_byte_stable_and_seeds_vary() {
    let settings = GeneratorSettings::default();
    assert_eq!(
        generate(9, &settings, &sites()),
        generate(9, &settings, &sites())
    );
    assert_ne!(
        generate(9, &settings, &sites()),
        generate(10, &settings, &sites())
    );
}

#[test]
fn settings_and_supplied_sites_bound_candidates() {
    let mut settings = GeneratorSettings::default();
    settings.forms = vec![Form::Creature];
    settings.min_actuators = 3;
    settings.max_actuators = 3;
    let batch = generate(2, &settings, &sites());
    assert_eq!(batch.candidates[0].blueprint.actuators.len(), 3);
    let fewer = sites()
        .into_iter()
        .filter(|s| s.role != SiteRole::Actuator || s.part.part < 5)
        .collect::<Vec<_>>();
    assert!(generate(2, &settings, &fewer).candidates.is_empty());
}

#[test]
fn form_order_does_not_change_per_form_seed_or_blueprint() {
    let mut a = GeneratorSettings::default();
    a.forms = vec![Form::Creature, Form::Staff];
    a.max_actuators = 3;
    let mut b = a.clone();
    b.forms.reverse();
    let left = generate(91, &a, &sites());
    let right = generate(91, &b, &sites());
    for form in [Form::Creature, Form::Staff] {
        let x = left
            .candidates
            .iter()
            .find(|c| c.blueprint.form == form)
            .unwrap();
        let y = right
            .candidates
            .iter()
            .find(|c| c.blueprint.form == form)
            .unwrap();
        assert_eq!(x.blueprint, y.blueprint);
    }
}

#[test]
fn live_part_mutation_changes_availability_with_reason() {
    let settings = GeneratorSettings {
        forms: vec![Form::Creature],
        min_actuators: 1,
        max_actuators: 1,
        ..Default::default()
    };
    let candidate = generate(3, &settings, &sites())
        .candidates
        .remove(0)
        .blueprint;
    let mut live: BTreeSet<_> = sites().iter().map(|s| s.part).collect();
    let rules = WorldRules {
        allowed_operators: [Operator::Strengthen].into_iter().collect(),
        allowed_costs: [10].into_iter().collect(),
        max_range: 2,
        max_hops: 4,
    };
    let before = preview_techniques(&candidate, &live, &rules, &PreviewSettings::default());
    live.remove(&candidate.actuators[0]);
    let after = preview_techniques(&candidate, &live, &rules, &PreviewSettings::default());
    assert!(before.iter().any(|x| x.result.is_ok()));
    assert!(
        after
            .iter()
            .any(|x| x.result.as_ref().is_err_and(|e| e.contains("unavailable")))
    );
}

#[test]
fn accepted_blueprint_and_continuation_round_trip() {
    let batch = generate(77, &GeneratorSettings::default(), &sites());
    let bytes = serde_json::to_vec(&batch).unwrap();
    let restored: GenerationBatch = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(batch, restored);
}
