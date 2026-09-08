// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{Intent, restore, snapshot, state_hash};

fn palette() -> PartPalette {
    Founding::Drawn.palette()
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
