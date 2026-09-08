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
    host.run_action("v");
    assert_ne!(host.config.camera, preview_camera);
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
