// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    HostConfig,
    played::{PlayedTrace, SceneMode},
};

fn opened(request: Request) -> Host {
    Host::new(HostConfig {
        creator_request: Some(request),
        ..Default::default()
    })
}

fn ready(host: &mut Host) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    while host.creator.as_ref().is_some_and(|c| c.pending) {
        assert!(
            std::time::Instant::now() < deadline,
            "generation did not finish"
        );
        host.advance();
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn body_plan_switch_upgrades_old_draft_and_preserves_habitat() {
    use mesocosm_core::world::generation::{BodyPlan, VERSION};
    let mut host = opened(serde_json::from_str(r#"{"version":1,"seed":7}"#).unwrap());
    let original = host.runtime.state_hash();
    ready(&mut host);
    let habitat = host
        .creator
        .as_ref()
        .unwrap()
        .prepared
        .as_ref()
        .unwrap()
        .draft()
        .habitat
        .clone();
    host.run_action("b");
    host.run_action("enter");
    assert!(host.creator.is_some());
    ready(&mut host);
    let creator = host.creator.as_ref().unwrap();
    assert_eq!(creator.request.version, VERSION);
    assert_eq!(creator.request.criteria.body_plan, BodyPlan::Branched);
    let saved = serde_json::to_vec(&creator.request).unwrap();
    assert_eq!(
        serde_json::from_slice::<Request>(&saved).unwrap(),
        creator.request
    );
    assert_eq!(creator.prepared.as_ref().unwrap().draft().habitat, habitat);
    assert!(creator.count() > 0);
    host.run_action("k");
    ready(&mut host);
    host.run_action("r");
    ready(&mut host);
    host.run_action("u");
    ready(&mut host);
    assert_eq!(
        host.creator.as_ref().unwrap().request.criteria.body_plan,
        BodyPlan::Branched
    );
    assert_eq!(host.runtime.state_hash(), original);
    let expected = state_hash(host.creator.as_ref().unwrap().world());
    host.run_action("enter");
    assert!(host.creator.is_none());
    assert_eq!(host.runtime.state_hash(), expected);
}

#[test]
fn retained_traits_survive_variation_and_can_be_cleared() {
    let mut host = opened(Request::default());
    let original = host.runtime.state_hash();
    ready(&mut host);
    host.run_action("l");
    let draft = host
        .creator
        .as_ref()
        .unwrap()
        .prepared
        .as_ref()
        .unwrap()
        .draft();
    let chosen = draft.candidates[1].clone();
    let habitat = draft.habitat.clone();
    host.run_action("k");
    ready(&mut host);
    let before = host
        .creator
        .as_ref()
        .unwrap()
        .prepared
        .as_ref()
        .unwrap()
        .draft()
        .candidates
        .clone();
    host.run_action("r");
    ready(&mut host);
    let creator = host.creator.as_ref().unwrap();
    let draft = creator.prepared.as_ref().unwrap().draft();
    assert_eq!(draft.habitat, habitat);
    assert_eq!(draft.candidates.len(), 4);
    assert!(draft.candidates.iter().all(|c| c.role == chosen.role
        && c.segments == chosen.segments
        && (c.actuator_span > 0) == (chosen.actuator_span > 0)));
    assert!(
        draft
            .candidates
            .iter()
            .any(|c| !before.iter().any(|old| old.recipe == c.recipe))
    );
    assert_eq!(host.runtime.state_hash(), original);
    assert!(host.runtime.trace().is_empty());
    let request = creator.request.clone();
    host.run_action("u");
    let cleared = &host.creator.as_ref().unwrap().request;
    assert_eq!(cleared.criteria.role, None);
    assert_eq!(cleared.criteria.movement_organs, None);
    assert_eq!(
        (cleared.criteria.min_segments, cleared.criteria.max_segments),
        (1, 32)
    );
    assert_eq!(
        (
            cleared.seed,
            cleared.variation,
            cleared.criteria.mass_mg,
            cleared.criteria.max_parts
        ),
        (
            request.seed,
            request.variation,
            request.criteria.mass_mg,
            request.criteria.max_parts
        )
    );
}

#[test]
fn saved_criteria_reopen_without_entry_and_save_failure_keeps_world_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("criteria.json");
    let mut host = Host::new(HostConfig {
        creator_request: Some(Request::default()),
        creator_draft: Some(path.clone()),
        ..Default::default()
    });
    ready(&mut host);
    host.run_action("l");
    host.run_action("k");
    host.run_action("r");
    ready(&mut host);
    let original = host.runtime.state_hash();
    host.run_action("s");
    let saved = crate::creator_draft::load(&path).unwrap().unwrap();
    assert_eq!(saved, host.creator.as_ref().unwrap().request);
    let mut reopened = opened(saved);
    ready(&mut reopened);
    assert_eq!(
        state_hash(reopened.creator.as_ref().unwrap().world()),
        state_hash(host.creator.as_ref().unwrap().world())
    );
    host.creator.as_mut().unwrap().request.criteria.max_parts = 1;
    host.creator.as_mut().unwrap().regenerate();
    ready(&mut host);
    host.run_action("s");
    let status = &host.creator.as_ref().unwrap().reading.status;
    assert!(status.contains("0 candidates") && status.contains("Criteria saved"));
    host.creator.as_mut().unwrap().draft_path = Some(dir.path().to_owned());
    host.run_action("s");
    assert!(
        host.creator
            .as_ref()
            .unwrap()
            .notice
            .contains("Could not save")
    );
    assert_eq!(host.runtime.state_hash(), original);
    assert!(host.runtime.trace().is_empty());
}

#[test]
fn browse_and_cancel_leave_the_parked_world_untouched() {
    let mut host = opened(Request::default());
    let original = host.runtime.state_hash();
    host.run_action("enter"); // Pending is never an implicit choice.
    assert!(host.creator.is_some());
    ready(&mut host);
    assert_eq!(host.runtime.state_hash(), original);
    assert!(host.run_action("l"));
    assert_eq!(host.creator.as_ref().unwrap().selected, 1);
    assert!(
        host.creator.as_ref().unwrap().reading.rows[1]
            .offer
            .contains("Consumer")
    );
    host.run_action("w");
    host.run_action(".");
    host.advance();
    assert_eq!(host.runtime.state_hash(), original);
    assert_eq!(host.runtime.queued_len(), 0);
    let preview_camera = host.config.camera;
    assert_eq!(preview_camera, crate::section::CameraMode::Oblique);
    host.run_action("v");
    assert_ne!(host.config.camera, preview_camera);
    for _ in 0..4 {
        host.run_action("v");
    }
    assert_eq!(host.config.camera, preview_camera);
    host.run_action("z");
    host.run_action("v");
    assert_eq!(host.config.camera, preview_camera);
    host.run_action("v"); // Cancel must restore from a different view.
    assert!(host.run_action("escape"));
    assert!(host.creator.is_none());
    assert_eq!(host.config.camera, crate::section::CameraMode::Oblique);
    assert_eq!(host.runtime.state_hash(), original);
    assert!(host.runtime.trace().is_empty());
}

#[test]
fn new_requests_retire_old_candidates_and_failed_search_has_no_body() {
    let mut host = opened(Request::default());
    ready(&mut host);
    let habitat = host
        .creator
        .as_ref()
        .unwrap()
        .prepared
        .as_ref()
        .unwrap()
        .draft()
        .habitat
        .clone();
    for _ in 0..5 {
        host.run_action("r");
    }
    assert!(host.creator.as_ref().unwrap().world().body().is_none());
    host.run_action("enter");
    assert!(host.creator.is_some());
    ready(&mut host);
    let creator = host.creator.as_ref().unwrap();
    let draft = creator.prepared.as_ref().unwrap().draft();
    assert_eq!(draft.request.variation, 5);
    assert_eq!(draft.habitat, habitat);
    let creator = host.creator.as_mut().unwrap();
    creator.request.criteria.max_parts = 1;
    creator.request.attempts = 8;
    creator.regenerate();
    ready(&mut host);
    assert!(host.creator.as_ref().unwrap().world().body().is_none());
    assert!(
        host.creator
            .as_ref()
            .unwrap()
            .reading
            .status
            .contains("0 candidates")
    );
    assert!(
        host.creator
            .as_ref()
            .unwrap()
            .reading
            .choice
            .contains("Change the criteria")
    );
    host.run_action("enter");
    assert!(host.creator.is_some());
}

#[test]
fn selected_world_is_revalidated_entered_once_and_replayed() {
    let mut host = opened(Request {
        candidates: 8,
        ..Request::default()
    });
    ready(&mut host);
    host.creator.as_mut().unwrap().select(7);
    assert_eq!(host.creator.as_ref().unwrap().reading.rows[1].source, "8");
    let expected = state_hash(host.creator.as_ref().unwrap().world());
    host.run_action("enter");
    assert!(host.creator.is_none());
    assert_eq!(host.runtime.state_hash(), expected);
    assert_eq!(host.config.start.as_ref().unwrap().candidate, 7);
    host.run_action("enter");
    assert_eq!(host.runtime.state_hash(), expected);
    for _ in 0..8 {
        host.runtime.queue(mesocosm_core::Intent::Idle);
        host.runtime.step(1);
    }
    let trace = PlayedTrace {
        start: host.config.start.clone(),
        trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        scene: SceneMode::Ecology,
        body_layout: host.config.body_layout,
        seed: host.config.seed,
        organisms: host.config.organisms,
        steps: host.runtime.trace().len() as u64,
        state_hash: host.runtime.state_hash(),
        intents: host.runtime.trace().to_vec(),
        content: host.content.clone(),
    };
    let mut replay = Host::new(HostConfig {
        replay: Some(trace.clone()),
        ..Default::default()
    });
    while !replay.advance() {}
    assert_eq!(replay.runtime.state_hash(), trace.state_hash);
}

#[test]
fn held_body_world_controls_preserve_subject_and_parked_runtime() {
    let mut host = opened(Request::default());
    ready(&mut host);
    let original = host.runtime.state_hash();
    host.run_action("l");
    let body = host
        .creator
        .as_ref()
        .unwrap()
        .world()
        .controlled()
        .unwrap()
        .phenotype
        .clone();
    host.run_action("h");
    ready(&mut host);
    let criteria = host.creator.as_ref().unwrap().request.criteria.clone();
    for key in ["b", "r", "k", "u", "m", "c", "+"] {
        host.run_action(key);
    }
    assert_eq!(host.creator.as_ref().unwrap().request.criteria, criteria);
    for key in ["n", "f", ".", "p"] {
        host.run_action(key);
        ready(&mut host);
    }
    let creator = host.creator.as_ref().unwrap();
    assert_eq!(creator.world().controlled().unwrap().phenotype, body);
    assert!(creator.observation.is_some());
    assert_eq!(host.runtime.state_hash(), original);
    host.run_action("escape");
    assert_eq!(host.runtime.state_hash(), original);
}

#[test]
fn refused_held_habitat_cannot_enter_and_keeps_saved_body() {
    let mut host = opened(Request::default());
    ready(&mut host);
    host.run_action("h");
    ready(&mut host);
    let original = host.runtime.state_hash();
    let creator = host.creator.as_mut().unwrap();
    let fixed = creator.request.fixed_body.clone();
    creator.request.soil_min_mg = 1;
    creator.request.soil_max_mg = 1;
    creator.regenerate();
    ready(&mut host);
    host.run_action("enter");
    let creator = host.creator.as_ref().unwrap();
    assert_eq!(creator.count(), 0);
    assert!(creator.world().controlled().is_none());
    assert_eq!(creator.request.fixed_body, fixed);
    let saved = serde_json::to_vec(&creator.request).unwrap();
    assert_eq!(
        serde_json::from_slice::<Request>(&saved).unwrap(),
        creator.request
    );
    assert_eq!(host.runtime.state_hash(), original);
}
