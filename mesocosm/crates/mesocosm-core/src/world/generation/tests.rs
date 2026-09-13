// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Intent, restore, snapshot, state_hash};

fn palette() -> PartPalette {
    Founding::Drawn.palette()
}

#[test]
fn older_requests_are_rejected_before_material_interpretation() {
    let old = Request {
        version: 1,
        ..Request::default()
    };
    let new = Request::default();
    assert_eq!(old.validate(), Err(Error::Version(1)));
    assert!(new.preview(palette()).is_ok());
    let mut invalid = old;
    invalid.criteria.body_plan = BodyPlan::Branched;
    assert!(invalid.validate().is_err());
}

#[test]
fn branching_corpus_changes_geometry_and_survives_restore_and_replay() {
    for seed in [0, 7, 42] {
        for role in [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer] {
            let mut request = Request {
                seed,
                ..Request::default()
            };
            request.criteria.body_plan = BodyPlan::Branched;
            request.criteria.role = Some(role);
            let prepared = request.prepare(palette()).unwrap();
            let draft = prepared.draft();
            assert_eq!(
                draft.candidates.len(),
                4,
                "{seed}/{role:?}: {:?}",
                draft.rejected
            );
            for candidate in &draft.candidates {
                assert_eq!(candidate.role, role);
                assert!(candidate.recipe.layout.len() >= 4);
                assert_eq!(candidate.recipe.layout[2].parent, Some(1));
                assert_eq!(candidate.recipe.layout[3].parent, Some(1));
                let mut axial = candidate.recipe.clone();
                axial.layout.clear();
                let soma = Soma::develop(&axial, candidate.seed);
                let body = crate::develop_body(
                    SpeciesId(1),
                    &axial,
                    &soma,
                    request.criteria.mass_mg,
                    palette(),
                )
                .unwrap();
                assert_ne!(
                    body.parts, candidate.body.parts,
                    "layout must change realized geometry"
                );
            }
            let mut world = prepared.enter(0).unwrap();
            assert_eq!(
                world.lineages.get(SpeciesId(1)).unwrap().recipe,
                draft.candidates[0].recipe
            );
            let mut saved = restore(&snapshot(&world).unwrap()).unwrap();
            let mut replay = request.prepare(palette()).unwrap().enter(0).unwrap();
            let matter = world.total_matter_mg();
            for _ in 0..8 {
                for state in [&mut world, &mut saved, &mut replay] {
                    state.apply(Intent::Idle);
                }
                assert_eq!(world.total_matter_mg(), matter);
                assert_eq!(state_hash(&world), state_hash(&saved));
                assert_eq!(state_hash(&world), state_hash(&replay));
            }
            request.variation += 1;
            let varied = request.preview(palette()).unwrap();
            assert_eq!(draft.habitat, varied.habitat);
            assert_ne!(draft.candidates, varied.candidates);
        }
    }
}

#[test]
fn prepared_previews_are_disposable_and_match_fresh_entry() {
    let request = Request::default();
    let prepared = request.prepare(palette()).unwrap();
    let expected = Selection {
        request,
        candidate: 1,
    }
    .enter(palette())
    .unwrap();
    let mut disposable = prepared.enter(1).unwrap();
    assert_eq!(state_hash(&disposable), state_hash(&expected));
    disposable.apply(Intent::Move { delta: [1, 0, 0] });
    assert_ne!(state_hash(&disposable), state_hash(&expected));
    assert_eq!(
        state_hash(&prepared.enter(1).unwrap()),
        state_hash(&expected)
    );
    assert!(prepared.enter(usize::MAX).is_err());
}

#[test]
fn generated_creator_entry_installs_the_selected_tissue() {
    let mut request = Request::default();
    request.criteria.role = Some(Kingdom::Producer);
    let world = request.prepare(palette()).unwrap().enter(0).unwrap();
    let founder = world.controlled().expect("generated founder is controlled");
    let expected = crate::matter::Material::Producer;
    assert_eq!(
        world
            .lineages()
            .get(founder.species)
            .expect("founder lineage")
            .initial_tissue,
        crate::InitialTissueRecipe::single(expected)
    );
    assert_eq!(
        founder.phenotype.total_stock().unwrap(),
        crate::matter::Stock::single(expected, founder.biomass_mg())
    );
}

#[test]
fn deterministic_preview_and_structural_variety() {
    let request = Request::default();
    let draft = request.preview(palette()).unwrap();
    assert_eq!(draft, request.preview(palette()).unwrap());
    assert_eq!(
        draft.candidates.len(),
        request.candidates as usize,
        "{:?}",
        draft.rejected
    );
    assert!(
        draft
            .candidates
            .windows(2)
            .any(|pair| pair[0].recipe != pair[1].recipe)
    );
    for candidate in &draft.candidates {
        let entered = Selection {
            request: request.clone(),
            candidate: draft
                .candidates
                .iter()
                .position(|c| c == candidate)
                .unwrap(),
        }
        .enter(palette())
        .unwrap();
        assert_eq!(entered.body().unwrap(), &candidate.body);
        assert_eq!(entered.position(), Some(candidate.position));
        assert!(
            entered
                .controlled()
                .unwrap()
                .walker_shape()
                .stands(entered.ground(), candidate.position)
        );
    }
}

#[test]
fn varying_bodies_preserves_habitat_and_locked_criteria() {
    let mut request = Request::default();
    request.criteria.role = Some(Kingdom::Consumer);
    request.criteria.movement_organs = Some(true);
    let (first, first_world) = request.draft_world(palette()).unwrap();
    request.variation = 1;
    let (second, second_world) = request.draft_world(palette()).unwrap();
    assert_eq!(first.habitat, second.habitat);
    assert_eq!(state_hash(&first_world), state_hash(&second_world));
    assert_ne!(first.candidates, second.candidates);
    assert!(!second.candidates.is_empty(), "{:?}", second.rejected);
    for candidate in second.candidates {
        assert_eq!(candidate.role, Kingdom::Consumer);
        assert!(candidate.actuator_span > 0);
        assert_eq!(candidate.body.total_mass_mg(), request.criteria.mass_mg);
    }
    request.criteria.role = Some(Kingdom::Producer);
    request.criteria.movement_organs = Some(false);
    let plants = request.preview(palette()).unwrap();
    assert!(!plants.candidates.is_empty(), "{:?}", plants.rejected);
    assert!(plants.candidates.iter().all(|c| c.actuator_span == 0));
}

#[test]
fn impossible_requests_are_bounded_and_never_silently_relaxed() {
    let mut request = Request::default();
    request.criteria.max_parts = 1;
    request.attempts = 8;
    let draft = request.preview(palette()).unwrap();
    assert!(draft.candidates.is_empty());
    assert_eq!(draft.attempted, 8);
    assert_eq!(draft.rejected.values().sum::<u32>(), 8);
    assert!(matches!(
        Selection {
            request,
            candidate: 0
        }
        .enter(palette()),
        Err(Error::CandidateUnavailable { .. })
    ));
    assert!(matches!(
        Request {
            version: 99,
            ..Request::default()
        }
        .preview(palette()),
        Err(Error::Version(99))
    ));
}

#[test]
fn habitat_resources_really_constrain_founding() {
    let request = Request {
        soil_min_mg: 1,
        soil_max_mg: 1,
        ..Request::default()
    };
    let draft = request.preview(palette()).unwrap();
    assert!(draft.candidates.is_empty());
    assert!(draft.rejected.keys().any(|s| s.contains("founding matter")));
}

#[test]
fn entry_conserves_budget_and_runs_through_save_and_replay() {
    let selection = Selection {
        request: Request::default(),
        candidate: 1,
    };
    let (_, provisional) = selection.request.draft_world(palette()).unwrap();
    let mut world = selection.enter(palette()).unwrap();
    let matter = world.total_matter_mg();
    assert_eq!(matter, provisional.total_matter_mg());
    let mut reopened = restore(&snapshot(&world).unwrap()).unwrap();
    let mut replayed = selection.enter(palette()).unwrap();
    let initial_position = world.position();
    for intent in [
        Intent::Move { delta: [1, 0, 0] },
        Intent::Idle,
        Intent::Move { delta: [0, 0, 1] },
        Intent::Idle,
    ]
    .into_iter()
    .cycle()
    .take(20)
    {
        world.apply(intent.clone());
        reopened.apply(intent.clone());
        replayed.apply(intent);
        assert_eq!(world.total_matter_mg(), matter);
        assert_eq!(state_hash(&world), state_hash(&reopened));
        assert_eq!(state_hash(&world), state_hash(&replayed));
    }
    assert_ne!(
        world.position(),
        initial_position,
        "the generated life must actually move"
    );
}

#[test]
fn seed_corpus_has_admitted_varied_bodies() {
    let mut recipes = Vec::new();
    for seed in [0, 1, 13, 42, 99, 1024, u64::MAX] {
        let request = Request {
            seed,
            ..Request::default()
        };
        let draft = request.preview(palette()).unwrap();
        assert_eq!(
            draft.candidates.len(),
            4,
            "seed {seed}: {:?}",
            draft.rejected
        );
        for candidate in draft.candidates {
            assert!(candidate.parts <= request.criteria.max_parts);
            assert!(
                (request.criteria.min_segments..=request.criteria.max_segments)
                    .contains(&candidate.segments)
            );
            assert!(candidate.local_soil_mg >= candidate.body.total_mass_mg() * 2);
            if !recipes.contains(&candidate.recipe) {
                recipes.push(candidate.recipe);
            }
        }
    }
    assert!(
        recipes.len() > 7,
        "the corpus must produce more than one recipe per habitat"
    );
}

#[test]
fn held_body_survives_habitat_changes_and_is_never_substituted() {
    let mut request = Request::default();
    let chosen = request.preview(palette()).unwrap().candidates[1].clone();
    request.fixed_body = Some(FixedBody {
        seed: chosen.seed,
        role: chosen.role,
    });
    for seed in [7, 8, 42] {
        request.seed = seed;
        request.soil_pattern = SoilPattern::Uniform;
        request.organisms = 30;
        let draft = request.preview(palette()).unwrap();
        assert_eq!(draft.attempted, 1);
        assert_eq!(draft.candidates.len(), 1);
        let c = &draft.candidates[0];
        assert_eq!(c.seed, chosen.seed);
        assert_eq!(c.recipe, chosen.recipe);
        assert_eq!(c.body, chosen.body);
        assert!(draft.habitat.iter().all(|h| h.soil_mg_per_column == 120));
    }
    request.soil_min_mg = 1;
    request.soil_max_mg = 1;
    let refused = request.preview(palette()).unwrap();
    assert!(refused.candidates.is_empty());
    assert_eq!(refused.attempted, 1);
    assert_eq!(refused.rejected.values().sum::<u32>(), 1);
    assert!(request.fixed_body.is_some());
}

#[test]
fn habitat_trial_is_bounded_conservative_and_does_not_advance_entry() {
    let prepared = Request::default().prepare(palette()).unwrap();
    let before = state_hash(&prepared.enter(1).unwrap());
    let observation = prepared.observe(1, 32).unwrap();
    assert_eq!(observation.matter_before_mg, observation.matter_after_mg);
    assert_eq!(state_hash(&prepared.enter(1).unwrap()), before);
    assert_eq!(observation, prepared.observe(1, 32).unwrap());
    assert_eq!(observation.evidence.controller.held_ticks, 29);
    assert_eq!(observation.evidence.controller.instinct_ticks, 3);
    assert!(prepared.observe(1, 129).is_err());
    assert!(prepared.observe(999, 32).is_err());
}

#[test]
fn habitat_trial_records_actual_intake_without_explaining_its_absence() {
    let prepared = Request::default().prepare(palette()).unwrap();
    let none = prepared.observe(1, 1).unwrap();
    assert_eq!(none.evidence.subject_feeding_mg, 0);
    assert_eq!(none.evidence.subject_uptake_mg, 0);
    assert!(none.evidence.subject_alive);
    assert!(none.evidence.summary().contains("no intake"));

    let observed = prepared.observe(0, 128).unwrap();
    assert!(
        observed.evidence.subject_uptake_mg > 0,
        "the selected generated producer records accepted soil uptake"
    );
    assert!(
        !observed.evidence.summary().contains("compatible"),
        "nearby diet admission is not used as an explanation"
    );
}

#[test]
fn dense_habitat_trial_records_feeding_and_one_subject_ending() {
    let prepared = Request {
        organisms: 120,
        ..Request::default()
    }
    .prepare(palette())
    .unwrap();
    let observation = prepared.observe(1, 128).unwrap();
    assert!(observation.evidence.subject_feeding_mg > 0);
    assert!(!observation.evidence.subject_alive);
    assert!(observation.evidence.subject_death_recorded);
    assert_eq!(
        observation.before.into_iter().sum::<u32>() + observation.evidence.births
            - observation.after.into_iter().sum::<u32>(),
        observation.evidence.deaths,
        "one ending is counted for each initially living or trial-born critter"
    );
    let controller = observation.evidence.controller;
    assert_eq!(
        controller.held_ticks + controller.instinct_ticks + controller.inactive_ticks,
        observation.ticks
    );
}

#[test]
fn habitat_criteria_validate_versions_and_distribution() {
    let mut request = Request {
        soil_pattern: SoilPattern::Contrasting,
        ..Request::default()
    };
    let draft = request.preview(palette()).unwrap();
    assert!(
        draft
            .habitat
            .iter()
            .all(|h| [60, 180].contains(&h.soil_mg_per_column))
    );
    request.min_open_steps = 5;
    assert!(request.validate().is_err());
    request.min_open_steps = 0;
    for version in [1, 2] {
        request.version = version;
        assert!(request.validate().is_err());
    }
}
