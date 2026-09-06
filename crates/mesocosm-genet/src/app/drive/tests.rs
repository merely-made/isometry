// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The scenario driver, without a window. (DT4)
//!
//! `Host::frame` advances the world before it touches the device, so a headless
//! pump is the same three calls in the same order minus the drawing — which is
//! what [`pump`] is. Everything the driver decides (what `busy` means, what a
//! snapshot says, whether an assertion holds) is decided off world state and
//! retained DOMs, so none of it needs a GPU to be tested.

use std::cell::RefCell;
use std::rc::Rc;

use cambium::GenetAppRunner;
use genet_probe::{ProbeSurface, Scenario, text_present};
use genet_scripted_dom::ScriptedDom;
use mesocosm_core::Intent;

use super::*;
use crate::HostConfig;
use crate::played::{self, PlayedTrace};

/// A short recorded run, its own hash included — the golden fixture's shape at
/// a size a test run can afford.
fn recorded() -> PlayedTrace {
    played::record_demo(played::DEMO_SEED, 40, 10, 240)
}

fn replaying(trace: PlayedTrace) -> Host {
    Host::new(HostConfig {
        seed: trace.seed,
        organisms: trace.organisms,
        replay: Some(trace),
        ..HostConfig::default()
    })
}

/// The headless equivalent of the frame loop: advance, note what the world
/// answered, pump one scenario step. Capped so a scenario that never finishes
/// fails the test rather than hanging it.
fn pump(host: &mut Host, text: &str) -> genet_probe::Outcome {
    let mut scenario = Scenario::parse(text).expect("the scenario parses");
    for _ in 0..20_000 {
        host.advance();
        host.note_outcomes();
        host.frames += 1;
        if scenario.tick(host) == Progress::Done {
            return scenario.finish();
        }
    }
    panic!("the scenario did not finish");
}

/// **The claim DT4 is for**: a recorded trace replays through the shared
/// scenario driver and lands on the hash it recorded, asserted by the scenario
/// rather than by bespoke harness code.
#[test]
fn a_recorded_trace_replays_through_the_driver_at_its_recorded_hash() {
    let trace = recorded();
    let want = format!("{:016x}", trace.state_hash);
    let mut host = replaying(trace);

    let outcome = pump(
        &mut host,
        &format!(
            "log replaying the fixture\n\
             wait 5000\n\
             assert snap mode == replay\n\
             assert snap matches == yes\n\
             assert snap hash == {want}\n\
             assert snap dev-intents == 0\n\
             assert snap assisted == unassisted"
        ),
    );
    assert!(outcome.ok, "log: {:?}", outcome.log);
}

/// And the other half, which is what makes the first one evidence: a trace
/// whose recorded hash has been falsified fails the same scenario.
#[test]
fn a_falsified_hash_fails_the_same_scenario() {
    let mut trace = recorded();
    let want = format!("{:016x}", trace.state_hash);
    trace.state_hash ^= 1;
    let mut host = replaying(trace);

    let outcome = pump(
        &mut host,
        &format!("wait 5000\nassert snap matches == yes\nassert snap expected == {want}"),
    );
    assert!(!outcome.ok, "a falsified hash must fail: {:?}", outcome.log);
    assert!(
        outcome.log.iter().any(|line| line.contains("matches")),
        "and say which assertion caught it: {:?}",
        outcome.log
    );
}

/// **What `busy` means here**, in the four states the module documents.
#[test]
fn busy_is_scripted_work_in_flight_and_a_checkpoint_is_not_it() {
    let trace = recorded();
    let mut host = replaying(trace);
    assert_eq!(
        host.busy(),
        Some(true),
        "a replay with trace left to feed is busy"
    );

    // Feed it all, then it is quiet.
    while host.replay_pending() {
        host.advance();
    }
    assert_eq!(host.busy(), Some(false), "a spent trace is quiet");

    // An intent still in the queue is busy: the world has not answered it yet.
    let mut played = Host::new(HostConfig {
        organisms: 12,
        ..HostConfig::default()
    });
    assert_eq!(played.busy(), Some(false));
    played.runtime.queue(Intent::Idle);
    assert_eq!(played.busy(), Some(true), "a queued intent is in flight");
    played.runtime.step(1);
    assert_eq!(played.busy(), Some(false));

    // A demo still pumping is busy, and stops being so when it runs out.
    assert!(played.run_action("demo 2"));
    assert_eq!(played.busy(), Some(true));
    played.pump_frame();
    assert_eq!(played.busy(), Some(false));
}

/// A checkpoint holds the world and nothing resolves it on its own, so the
/// driver reports **quiet** and hands the script its turn. Reporting busy would
/// burn the whole wait cap and then proceed anyway.
#[test]
fn a_checkpoint_with_nothing_queued_is_quiet_so_the_script_can_answer_it() {
    let mut host = Host::new(HostConfig {
        organisms: 12,
        dev: true,
        ..HostConfig::default()
    });
    // An accepted loss opens a real checkpoint immediately. The assertion is
    // about the held runtime state a script observes, not a long demo search.
    let controlled = host
        .runtime
        .world()
        .controlled_id()
        .expect("a founder is controlled");
    host.runtime.queue(Intent::Kill {
        organism: controlled,
    });
    host.runtime.step(1);
    let Some(_) = host.runtime.checkpoint() else {
        panic!("an accepted loss opens a checkpoint");
    };
    assert_eq!(host.runtime.queued_len(), 0);
    assert_eq!(
        host.busy(),
        Some(false),
        "a held world with nothing queued is the script's turn, not a wait"
    );
    assert_eq!(host.snapshot().field("checkpoint"), Some("yes"));
}

/// The snapshot answers what the lanes cannot say in words, and the hash it
/// reports is the driver's own.
#[test]
fn the_snapshot_reports_the_run_the_receipt_reports() {
    let mut host = Host::new(HostConfig {
        organisms: 12,
        dev: true,
        ..HostConfig::default()
    });
    host.runtime.step(3);
    let snap = host.snapshot();
    assert_eq!(
        snap.field("hash"),
        Some(format!("{:016x}", host.runtime.state_hash()).as_str())
    );
    assert_eq!(snap.field("tick"), Some("3"));
    assert_eq!(snap.field("mode"), Some("played"));
    assert_eq!(snap.field("dev"), Some("true"));
    assert_eq!(snap.field("assisted"), Some("unassisted"));
    assert_eq!(snap.field("expected"), Some(""), "and nothing to match");

    // A dev intent applied labels the run, in the receipt line's own words.
    host.runtime.queue(Intent::EndEpoch);
    host.runtime.step(1);
    assert_eq!(host.runtime.dev_intents(), 1);
    assert_eq!(
        host.snapshot().field("assisted"),
        Some("assisted (1 dev intents)"),
        "the same string the receipt line leads with"
    );
}

/// **An `assert text` reads a chrome lane's own words.**
///
/// The lane's retained tree is what the driver searches, so this builds the
/// vitals runner exactly as `VitalsChrome::new` does — minus the raster, which
/// wants a device — and asserts against the sentences `mesocosm-views` wrote.
#[test]
fn an_assertion_reads_the_words_a_chrome_lane_actually_holds() {
    type Runner = GenetAppRunner<
        mesocosm_views::Vitals,
        fn(&mesocosm_views::Vitals) -> mesocosm_views::VitalsChild,
        mesocosm_views::VitalsChild,
    >;
    let mut world = mesocosm_core::World::new(0x1234, 12);
    world.apply(Intent::Idle);
    let reading = mesocosm_views::vitals_of(&world, 1_000, Some("grew"), None);

    let mut runner = Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        mesocosm_views::vitals_root as fn(&mesocosm_views::Vitals) -> mesocosm_views::VitalsChild,
        mesocosm_views::Vitals::default(),
    );
    runner.update(|vitals| *vitals = reading);
    let dom = runner.dom();
    let dom = dom.borrow();
    let surfaces = [ProbeSurface {
        name: "vitals",
        dom: &dom,
        rect: [12.0, 208.0, 300.0, 320.0],
        sheet: mesocosm_views::vitals_css(),
    }];

    assert!(
        text_present(&surfaces, "energy"),
        "the panel names the budget"
    );
    assert!(text_present(&surfaces, "grew"), "and shows the notice");
    assert!(
        !text_present(&surfaces, "kleptoplasty"),
        "and a miss is a miss"
    );
}

/// A `capture` names a file. A bare name lands in the headed-verify home; a
/// path is taken as written, which is how a receipt asks for an explicit
/// non-default file.
#[test]
fn a_capture_name_resolves_to_the_headed_verify_home_or_to_the_path_given() {
    let host = Host::new(HostConfig::default());
    assert_eq!(
        host.capture_path("dt4_scenario"),
        played::default_out_dir().join("dt4_scenario.png")
    );
    let explicit = if cfg!(windows) {
        r"C:\tmp\shot.png"
    } else {
        "/tmp/shot.png"
    };
    assert_eq!(
        host.capture_path(explicit),
        std::path::PathBuf::from(explicit)
    );
}

/// A pointer gesture this host cannot route attributes itself into the event
/// stream rather than passing silently. The stack gap, made assertable.
#[test]
fn an_unrouted_pointer_gesture_says_so_in_the_stream() {
    let mut host = Host::new(HostConfig::default());
    host.press(10.0, 20.0);
    host.release(10.0, 20.0);
    let events = host.drain_events();
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|e| e.starts_with("pointer-unrouted")));
}
