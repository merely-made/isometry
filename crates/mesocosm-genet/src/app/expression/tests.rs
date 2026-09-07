// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{HostConfig, PlayedTrace, played::SceneMode};

fn host() -> Host {
    Host::new(HostConfig {
        seed: 7,
        scene: SceneMode::ExpressionPractice,
        ..HostConfig::default()
    })
}

#[test]
fn browsing_expression_pauses_and_cancel_preserves_authority() {
    let mut host = host();
    let hash = host.runtime.state_hash();
    assert!(host.run_action("o"));
    assert!(host.grafting.admissible);
    assert!(host.grafting.preview.is_some());
    assert_eq!(host.grafting.reading.headline, "Express discovery");
    for key in ["up", "down", "tab", "w", "e", "h", "f", "g", "."] {
        assert!(host.run_action(key));
        host.advance();
        assert_eq!(host.runtime.state_hash(), hash);
        assert!(host.runtime.trace().is_empty());
        assert_eq!(host.runtime.queued_len(), 0);
    }
    host.run_action("escape");
    assert!(!host.grafting.open);
    assert!(host.grafting.preview.is_none());
    assert_eq!(host.runtime.state_hash(), hash);
}

#[test]
fn expression_commits_once_and_replays_without_revising_the_lineage() {
    let mut host = host();
    let species = host.runtime.world().controlled().unwrap().species;
    let program = host
        .runtime
        .world()
        .lineages()
        .get(species)
        .unwrap()
        .program()
        .clone();
    host.run_action("o");
    let affected = host.grafting.root.unwrap();
    host.run_action("enter");
    assert_eq!(host.runtime.trace().len(), 1);
    assert!(matches!(host.runtime.trace()[0], Intent::Express { .. }));
    assert!(
        matches!(host.runtime.last_outcomes()[0], Outcome::Expressed { part, .. } if part == affected)
    );
    assert_eq!(
        host.runtime
            .world()
            .lineages()
            .get(species)
            .unwrap()
            .program(),
        &program
    );
    assert_eq!(host.grafting.root, Some(affected));
    assert!(host.grafting.preview.is_none());
    assert!(
        host.grafting
            .reading
            .status
            .starts_with("Expressed on part")
    );
    host.run_action("enter");
    assert_eq!(host.runtime.trace().len(), 1);
    let trace = PlayedTrace {
        start: None,
        trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        scene: SceneMode::ExpressionPractice,
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
fn stale_expression_preview_requires_another_deliberate_confirmation() {
    let mut host = host();
    host.run_action("o");
    host.runtime.queue(Intent::Resume);
    host.runtime.step(1);
    let hash = host.runtime.state_hash();
    let count = host.runtime.trace().len();
    host.run_action("enter");
    assert_eq!(host.runtime.state_hash(), hash);
    assert_eq!(host.runtime.trace().len(), count);
    assert!(
        host.grafting
            .reading
            .status
            .starts_with("The world changed")
    );
}

#[test]
fn undiscovered_menu_cannot_apply_an_expression() {
    let mut host = Host::new(HostConfig {
        organisms: 0,
        ..HostConfig::default()
    });
    let hash = host.runtime.state_hash();
    host.run_action("o");
    assert!(host.grafting.conditions.is_empty());
    assert!(!host.grafting.admissible);
    assert!(host.grafting.reading.detail.starts_with("No discoveries"));
    for key in ["down", "enter", "escape"] {
        host.run_action(key);
    }
    assert_eq!(host.runtime.state_hash(), hash);
    assert!(host.runtime.trace().is_empty());
}
