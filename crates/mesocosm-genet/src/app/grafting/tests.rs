// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{HostConfig, PlayedTrace, played::SceneMode};

fn host() -> Host {
    Host::new(HostConfig {
        seed: 7,
        scene: SceneMode::GraftPractice,
        ..HostConfig::default()
    })
}

#[test]
fn browsing_and_cancelling_a_graft_never_change_the_world_or_trace() {
    let mut host = host();
    let hash = host.runtime.state_hash();
    assert!(host.run_action("h"));
    assert!(host.grafting.open);
    assert!(host.grafting.admissible);
    assert!(host.grafting.preview.is_some());
    for action in ["down", "up", "tab", "tab", "w", "e", ".", "g", "f"] {
        assert!(host.run_action(action));
        host.advance();
        assert_eq!(host.runtime.state_hash(), hash);
        assert!(host.runtime.trace().is_empty());
        assert_eq!(host.runtime.queued_len(), 0);
    }
    assert!(host.run_action("escape"));
    assert!(!host.grafting.open);
    assert!(host.grafting.preview.is_none());
    assert_eq!(host.runtime.state_hash(), hash);
}

#[test]
fn confirmation_records_one_graft_and_replays_from_the_named_scene() {
    let mut host = host();
    host.run_action("h");
    let candidate = host
        .grafting
        .preview
        .as_ref()
        .unwrap()
        .controlled()
        .unwrap()
        .phenotype
        .clone();
    host.run_action("enter");
    assert_eq!(host.runtime.trace().len(), 1);
    assert!(matches!(host.runtime.trace()[0], Intent::Graft { .. }));
    assert!(matches!(
        host.runtime.last_outcomes()[0],
        Outcome::Grafted { parts: 2, .. }
    ));
    assert_eq!(
        host.runtime.world().body().unwrap().parts.len(),
        candidate.body().parts.len()
    );
    assert!(host.grafting.preview.is_none());
    assert!(host.grafting.reading.status.starts_with("Grafted"));
    // A second Enter is not a second transaction without another selection.
    host.run_action("enter");
    assert_eq!(host.runtime.trace().len(), 1);
    let trace = PlayedTrace {
        trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        scene: SceneMode::GraftPractice,
        body_layout: host.config.effective_body_layout(),
        seed: 7,
        organisms: host.runtime.receipt().organisms,
        steps: 1,
        state_hash: host.runtime.state_hash(),
        intents: host.runtime.trace().to_vec(),
        content: host.content.clone(),
    };
    let mut replay = Host::new(HostConfig {
        seed: 7,
        replay: Some(trace),
        ..HostConfig::default()
    });
    assert!(replay.advance());
    assert_eq!(replay.runtime.state_hash(), host.runtime.state_hash());
}

#[test]
fn a_stale_preview_requires_review_before_it_can_commit() {
    let mut host = host();
    host.run_action("h");
    host.runtime.queue(Intent::Idle);
    host.runtime.step(1);
    let after = host.runtime.state_hash();
    let count = host.runtime.trace().len();
    host.run_action("enter");
    assert_eq!(host.runtime.state_hash(), after);
    assert_eq!(host.runtime.trace().len(), count);
    assert!(
        host.grafting
            .reading
            .status
            .starts_with("The world changed")
    );
}

#[test]
fn an_empty_menu_is_safe_and_does_not_require_dev_mode() {
    let mut host = Host::new(HostConfig {
        organisms: 0,
        ..HostConfig::default()
    });
    let hash = host.runtime.state_hash();
    host.run_action("h");
    assert!(host.grafting.open);
    assert!(host.grafting.sources.is_empty());
    assert!(host.grafting.reading.detail.starts_with("No reachable"));
    for key in ["up", "down", "tab", "enter", "escape"] {
        host.run_action(key);
    }
    assert_eq!(host.runtime.state_hash(), hash);
    assert!(host.runtime.trace().is_empty());
}
