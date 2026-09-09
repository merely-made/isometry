// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    HostConfig,
    played::{BodyLayout, SceneMode},
};
use mesocosm_core::places::route_step_for;

fn host() -> Host {
    Host::new(HostConfig {
        seed: 7,
        scene: SceneMode::FamilyClearing,
        body_layout: BodyLayout::Spaced,
        ..Default::default()
    })
}

fn press(host: &mut Host, script: &mut String, key: &str) {
    assert!(host.run_action(key), "{key}");
    script.push_str(&format!("act {key}\n"));
    if host.runtime.queued_len() > 0 {
        host.steps += host.runtime.step(1);
        host.note_outcomes();
        script.push_str("act .\n"); // headed fixture pauses time, then services this queued play key
    }
}

fn capture(script: &mut String, name: &str) {
    if let Ok(root) = std::env::var("MESOCOSM_CLEARING_RECEIPTS") {
        script.push_str(&format!("settle 3\ncapture {root}/{name}.png\n"));
    }
}

#[test]
fn intake_preview_cancel_and_stale_confirmation_preserve_world() {
    let mut host = host();
    let mut script = String::new();
    let origin = host.runtime.state_hash();
    press(&mut host, &mut script, "y");
    assert_eq!(host.runtime.state_hash(), origin);
    press(&mut host, &mut script, "escape");
    assert_eq!(host.runtime.state_hash(), origin);
    press(&mut host, &mut script, "y");
    host.runtime.queue(Intent::Resume);
    host.runtime.step(1);
    let changed = host.runtime.state_hash();
    press(&mut host, &mut script, "enter");
    assert_eq!(host.runtime.state_hash(), changed);
    assert!(host.grafting.reading.status.contains("world changed"));
}

#[test]
fn family_scene_plays_through_menus_walks_tunnel_and_replays() {
    let mut host = host();
    let original = host.runtime.world().clone();
    let ids = original.family_practice_ids().unwrap();
    let mut script = String::from("act p\n");
    capture(&mut script, "origin");
    press(&mut host, &mut script, "y");
    let intake = host
        .grafting
        .sources
        .iter()
        .position(|s| *s == (ids.donor, ids.intake_part))
        .unwrap();
    for _ in 0..intake {
        press(&mut host, &mut script, "l");
    }
    capture(&mut script, "intake-preview");
    press(&mut host, &mut script, "enter");
    assert!(matches!(
        host.runtime.last_outcomes()[0],
        Outcome::Consumed { .. }
    ));
    press(&mut host, &mut script, "escape");
    press(&mut host, &mut script, "h");
    let branch = host
        .grafting
        .sources
        .iter()
        .position(|s| *s == (ids.donor, ids.graft_part))
        .unwrap();
    for _ in 0..branch {
        press(&mut host, &mut script, "l");
    }
    capture(&mut script, "graft-preview");
    press(&mut host, &mut script, "enter");
    assert!(matches!(
        host.runtime.last_outcomes()[0],
        Outcome::Grafted { .. }
    ));
    press(&mut host, &mut script, "escape");
    press(&mut host, &mut script, "o");
    let condition = host
        .grafting
        .conditions
        .iter()
        .position(|c| *c == ids.condition)
        .unwrap();
    for _ in 0..condition {
        press(&mut host, &mut script, "l");
    }
    press(&mut host, &mut script, "enter");
    assert!(matches!(
        host.runtime.last_outcomes()[0],
        Outcome::Expressed { .. }
    ));
    capture(&mut script, "expression");
    press(&mut host, &mut script, "escape");
    press(&mut host, &mut script, "x");
    let row = host
        .runtime
        .review()
        .unwrap()
        .rows
        .iter()
        .position(|r| r.offer.candidate == Some(ids.condition))
        .unwrap();
    for _ in 0..row {
        press(&mut host, &mut script, "tab");
    }
    capture(&mut script, "lineage-review");
    press(&mut host, &mut script, "r");
    assert!(matches!(
        host.runtime.last_outcomes()[0],
        Outcome::Revised { .. }
    ));
    press(&mut host, &mut script, "enter"); // close the revised lineage review
    let habitat = host.runtime.world().family_clearing_habitat();
    let clearing = host.runtime.world().position().unwrap();
    let outward: Vec<_> = habitat.route.iter().copied().skip(1).collect();
    let homeward: Vec<_> = habitat.route.iter().copied().rev().skip(1).collect();
    for (route, destination, label) in [
        (outward, habitat.chamber, "cave"),
        (homeward, clearing, "returned"),
    ] {
        for target in route {
            for _ in 0..192 {
                let world = host.runtime.world();
                let body = world.controlled().unwrap();
                if body.position == target {
                    break;
                }
                let next = route_step_for(
                    world.ground(),
                    body.walker_shape(),
                    body.position,
                    target,
                    192,
                )
                .unwrap_or_else(|| {
                    panic!(
                        "post-graft body {:?} at {:?} cannot reach {target:?}",
                        body.walker_shape(),
                        body.position
                    )
                });
                let key = match (next[0] - body.position[0], next[2] - body.position[2]) {
                    (x, _) if x > 0 => "d",
                    (x, _) if x < 0 => "a",
                    (_, z) if z > 0 => "s",
                    (_, z) if z < 0 => "w",
                    _ => panic!("no horizontal step"),
                };
                press(&mut host, &mut script, key);
            }
            assert_eq!(host.runtime.world().position(), Some(target));
        }
        assert_eq!(host.runtime.world().position(), Some(destination));
        capture(&mut script, label);
        for turn in 0..4 {
            press(&mut host, &mut script, "v");
            capture(&mut script, &format!("{label}-turn-{turn}"));
        }
    }
    let wait = host.runtime.world().family_practice_birth_run().count();
    for _ in 0..wait {
        press(&mut host, &mut script, "enter");
    }
    assert!(
        host.runtime
            .history()
            .log()
            .entries()
            .iter()
            .any(|r| matches!(r.record,
        mesocosm_core::history::Event::Born {organism,parent:Some(parent),..}
        if parent == ids.parent && organism != ids.relative && organism != ids.donor))
    );
    capture(&mut script, "offspring");
    let mut replay = self::host();
    for intent in host.runtime.trace().iter().cloned() {
        replay.runtime.queue(intent);
        assert_eq!(replay.runtime.step(1), 1);
    }
    assert_eq!(replay.runtime.state_hash(), host.runtime.state_hash());
    assert_eq!(
        host.runtime.dev_intents(),
        1,
        "explicit player-triggered epoch boundary remains counted"
    );
    if let Ok(root) = std::env::var("MESOCOSM_CLEARING_RECEIPTS") {
        script.push_str(&format!(
            "assert snap hash == {:016x}\nassert snap dev-intents == 1\n",
            host.runtime.state_hash()
        ));
        std::fs::write(format!("{root}/play.scenario"), script).unwrap();
        std::fs::write(
            format!("{root}/expected-hash.txt"),
            format!("{:016x}", host.runtime.state_hash()),
        )
        .unwrap();
    }
}
