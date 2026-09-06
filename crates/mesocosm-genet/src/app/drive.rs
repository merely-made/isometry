// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host, driven by a text scenario. (DT4)
//!
//! Mesocosm's replay-and-demo harness used to be entirely its own: a
//! `--replay` cursor, a `--record-demo` script, an `--auto-eat` interval and a
//! one-off `examples/dt3_script.rs`, each a separate way of running the game
//! without a person in front of it. The dev tools plan's fourth principle says
//! that folds toward genet-probe rather than growing, and this is the fold: the
//! host implements [`Automatable`] and [`Driveable`], and
//! [`genet_probe::Scenario`] — the same driver turnstone uses — pumps a text
//! file against it one step per rendered frame.
//!
//! # What each verb means here
//!
//! | Verb | Here |
//! | --- | --- |
//! | `act <name>` | one of the host's own documented key names, or one of five host actions — see [`super::actions`] |
//! | `click .class text` | **attributed, not delivered** — see the gap below |
//! | `settle N` | pump N frames |
//! | `wait [cap]` | hold until [`Automatable::busy`] reads quiet |
//! | `assert text <s>` | `s` appears on a chrome lane that is actually on screen |
//! | `assert snap <f> <op> <v>` | a field of [`Host::snapshot`] — the state hash, the tick, the camera mode, the dev-intent count, the assisted label |
//! | `assert event <s>` | `s` appears in what the world answered |
//! | `capture <name>` | the run's own capture path, through [`super::Host::capture_to`] |
//! | `log <words>` | into the run's log |
//!
//! # `busy`, and what it means for this host
//!
//! Busy means **scripted work is still in flight that the next step must not
//! race**: a replay whose cursor has not reached the end of its trace, a demo
//! or a hunt still pumping, or an intent still sitting in the queue.
//!
//! **A checkpoint holding the world with nothing queued is quiet, not busy**,
//! and that is a decision rather than an oversight. A checkpoint is the world
//! stopped, waiting to be answered; nothing about it resolves on its own, so
//! reporting busy would burn the whole `wait` cap and then proceed anyway — a
//! hang dressed as a wait. It is exactly the moment the scenario's next step
//! (`act enter`, `act t`, `act r`) is needed, so `wait` hands control back and
//! the script answers. An answer already in the queue *is* busy, because the
//! hold lifts on the step that reaches it.
//!
//! # Stack gaps found doing this, reported rather than built
//!
//! - **Pointer delivery.** `Automatable` requires `press`/`moved`/`release`,
//!   and `mesocosm-genet` has nowhere to route a window point: DT2 already
//!   found that click-to-select in the section needs picking machinery this
//!   host does not have, and the chrome lanes are rasters composited over the
//!   frame with no hit-test path back into cambium. So the three are
//!   **attributed no-ops**: they record `pointer-unrouted x y` into the event
//!   stream, which an `assert event` can catch, rather than silently swallowing
//!   a click and letting the scenario believe it landed. `click` therefore
//!   resolves a selector correctly and then goes nowhere, which is the honest
//!   state of this host and not a defect in genet-probe.
//! - **No verb for founding a world.** A scenario is pumped inside an app that
//!   already exists, so the seed, the founder count and a trace to replay are
//!   necessarily flags. Expressed as `--seed`, `--replay` and `--scenario`
//!   rather than as scenario lines.
//! - **No loop or repeat verb.** The demo's three thousand steps and a hunt's
//!   standing interval are host actions taking a count (`act demo 3100`,
//!   `act hunt 12`) because the grammar has no way to say "again".

use std::path::PathBuf;

use genet_probe::{Automatable, Driveable, ProbeSnapshot, ProbeSurface, Progress};
use mesocosm_core::Outcome;
use winit::event_loop::ActiveEventLoop;

use super::Host;
use crate::played;

impl Host {
    /// Pumps the scenario one step, after the frame it is asserting about was
    /// drawn. Ends the run when the steps are exhausted, or when a frame limit
    /// ran out with steps left — which is a failure, because a scenario that
    /// did not finish asserted nothing about the rest of itself.
    pub(super) fn drive_scenario(&mut self, event_loop: &ActiveEventLoop, hit_limit: bool) {
        let Some(mut scenario) = self.scenario.take() else {
            return;
        };
        let progress = scenario.tick(self);
        if progress == Progress::Running && !hit_limit {
            self.scenario = Some(scenario);
            return;
        }
        let outcome = scenario.finish();
        for line in &outcome.log {
            println!("scenario: {line}");
        }
        let finished = progress == Progress::Done;
        if !finished {
            println!("scenario: the frame limit ran out with steps left");
        }
        let ok = outcome.ok && finished;
        println!("scenario: {}", if ok { "ok" } else { "FAILED" });
        if !ok {
            self.code = 1;
        }
        self.finish(event_loop);
    }

    /// What the world answered this frame, as grep-friendly strings an
    /// `assert event` matches, plus the one thing a scenario cannot know in
    /// advance: the id a forced birth produced.
    ///
    /// Nothing accumulates without a scenario to drain it, so an ordinary
    /// session pays one branch a frame and no memory. It is called from both
    /// sides — the frame that advanced and the action that stepped — and the
    /// applied-intent count keeps one batch from being recorded twice.
    pub(super) fn note_outcomes(&mut self) {
        if self.config.scenario.is_none() {
            return;
        }
        let applied = self.runtime.trace().len();
        if applied == self.noted {
            return;
        }
        self.noted = applied;
        for outcome in self.runtime.last_outcomes().to_vec() {
            if let Outcome::Bore { offspring, .. } = outcome {
                self.last_child = Some(offspring);
            }
            self.events.push(format!("outcome {outcome:?}"));
        }
    }

    /// Where a `capture <name>` writes.
    ///
    /// A name carrying a separator is a path and is taken as one, so a receipt
    /// can name an explicit non-default file; a bare name lands beside the
    /// fixtures in the workspace's headed-verify home, which is where the
    /// run's own capture goes.
    fn capture_path(&self, name: &str) -> PathBuf {
        let path = PathBuf::from(name);
        if path.is_absolute() || name.contains('/') || name.contains('\\') {
            path
        } else {
            played::default_out_dir().join(format!("{name}.png"))
        }
    }

    /// Whether a replay still has trace left to feed.
    fn replay_pending(&self) -> bool {
        self.config
            .replay
            .as_ref()
            .is_some_and(|trace| self.cursor < trace.intents.len())
    }
}

impl Automatable for Host {
    /// The chrome lanes that are **actually on screen**, topmost first.
    ///
    /// The board and the checkpoint are listed only while they stand and the
    /// dev tile only under `--dev`, because that is what makes `assert text` a
    /// claim about what a person would see rather than about a retained tree
    /// nobody is looking at. The minimap is not here at all: it is the painted
    /// lane, and by lane discipline it holds no words.
    fn with_surfaces<R>(&self, f: impl FnOnce(&[ProbeSurface<'_>]) -> R) -> R {
        let Some(gpu) = &self.gpu else {
            return f(&[]);
        };
        let Some(lanes) = &gpu.chrome else {
            return f(&[]);
        };
        let frame = (gpu.config.width, gpu.config.height);
        let (board_dom, board_rect, board_sheet) = lanes.board.probe(frame);
        let (held_dom, held_rect, held_sheet) = lanes.checkpoint.probe(frame);
        let (dev_dom, dev_rect, dev_sheet) = lanes.dev.probe(frame);
        let (vitals_dom, vitals_rect, vitals_sheet) = lanes.vitals.probe(frame);
        let board = board_dom.borrow();
        let held = held_dom.borrow();
        let dev = dev_dom.borrow();
        let vitals = vitals_dom.borrow();

        let mut surfaces = Vec::with_capacity(4);
        if lanes.board.standing() {
            surfaces.push(ProbeSurface {
                name: "board",
                dom: &board,
                rect: board_rect,
                sheet: board_sheet,
            });
        }
        if lanes.checkpoint.standing() {
            surfaces.push(ProbeSurface {
                name: "checkpoint",
                dom: &held,
                rect: held_rect,
                sheet: held_sheet,
            });
        }
        if self.config.dev {
            surfaces.push(ProbeSurface {
                name: "dev",
                dom: &dev,
                rect: dev_rect,
                sheet: dev_sheet,
            });
        }
        surfaces.push(ProbeSurface {
            name: "vitals",
            dom: &vitals,
            rect: vitals_rect,
            sheet: vitals_sheet,
        });
        f(&surfaces)
    }

    /// Everything a scenario asserts that the lanes cannot say in words.
    ///
    /// The hashes are hex to sixteen places, matching the receipt line and the
    /// trace file, so a scenario's literal is copied from what a run printed.
    fn snapshot(&self) -> ProbeSnapshot {
        let world = self.runtime.world();
        let expected = self
            .config
            .replay
            .as_ref()
            .filter(|trace| self.cursor >= trace.intents.len())
            .map(|trace| trace.state_hash);
        let hash = self.runtime.state_hash();
        let dev_intents = self.runtime.dev_intents();
        let selected_reading = self.inspection.selected.and_then(|selected| {
            mesocosm_views::part_of(
                world,
                selected.organism,
                selected.part,
                self.runtime.history(),
            )
            .reading
        });
        ProbeSnapshot {
            focused: self.followed().map(|id| format!("critter {}", id.0)),
            fields: [
                ("hash", format!("{hash:016x}")),
                (
                    "expected",
                    expected.map_or(String::new(), |hash| format!("{hash:016x}")),
                ),
                (
                    "matches",
                    match expected {
                        None => String::new(),
                        Some(want) if want == hash => "yes".to_string(),
                        Some(_) => "no".to_string(),
                    },
                ),
                ("mode", self.mode().to_string()),
                // Which way the section is looking (DC4). Presentation, so a
                // scenario asserts it to prove the arm it captured rather
                // than to prove anything about the world.
                ("camera", self.config.camera.name().to_string()),
                ("scene", self.config.effective_scene().name().to_string()),
                (
                    "terrain-style",
                    self.config
                        .terrain_style
                        .resolved(self.habitat.is_some())
                        .name()
                        .to_string(),
                ),
                (
                    "controlled-x",
                    world
                        .controlled_id()
                        .and_then(|id| world.organisms.iter().find(|o| o.id == id))
                        .map_or(String::new(), |o| o.position[0].to_string()),
                ),
                (
                    "controlled-y",
                    world
                        .controlled_id()
                        .and_then(|id| world.organisms.iter().find(|o| o.id == id))
                        .map_or(String::new(), |o| o.position[1].to_string()),
                ),
                (
                    "controlled-z",
                    world
                        .controlled_id()
                        .and_then(|id| world.organisms.iter().find(|o| o.id == id))
                        .map_or(String::new(), |o| o.position[2].to_string()),
                ),
                (
                    "burrow-occupied",
                    self.habitat
                        .as_ref()
                        .map_or("unknown", |habitat| {
                            world
                                .controlled_id()
                                .and_then(|id| world.organisms.iter().find(|o| o.id == id))
                                .map_or("unknown", |o| {
                                    if crate::section::terrarium_occupied(habitat, o.position) {
                                        "yes"
                                    } else {
                                        "no"
                                    }
                                })
                        })
                        .to_string(),
                ),
                ("tick", world.tick.to_string()),
                ("steps", self.steps.to_string()),
                ("frames", self.frames.to_string()),
                ("epoch", world.epoch.to_string()),
                ("dev", self.config.dev.to_string()),
                ("dev-intents", dev_intents.to_string()),
                // The receipt line prints nothing at all for an unaided run —
                // there is no label to skim past. A scenario needs a word to
                // compare against, and the grammar has no way to say "equals
                // nothing", so the absence is spelled here.
                (
                    "assisted",
                    match played::assisted_label(dev_intents) {
                        label if label.is_empty() => "unassisted".to_string(),
                        label => label,
                    },
                ),
                ("queued", self.runtime.queued_len().to_string()),
                (
                    "controlled",
                    world
                        .controlled_id()
                        .map_or("none".to_string(), |id| id.0.to_string()),
                ),
                (
                    "follow",
                    self.followed()
                        .map_or("none".to_string(), |id| id.0.to_string()),
                ),
                ("living", world.living().count().to_string()),
                ("checkpoint", yes_no(self.runtime.checkpoint().is_some())),
                ("boundary", yes_no(world.at_boundary())),
                ("paused", yes_no(self.dev_paused)),
                ("inspecting", yes_no(self.inspection.open)),
                (
                    "selected-intake",
                    selected_reading
                        .as_ref()
                        .map_or("none".into(), |r| r.intake.clone()),
                ),
                (
                    "selected-diet",
                    selected_reading
                        .as_ref()
                        .map_or("none".into(), |r| r.feeding.clone()),
                ),
                (
                    "selected-part",
                    self.inspection
                        .selected
                        .map_or("none".into(), |s| s.part.0.to_string()),
                ),
                (
                    "selected-organism",
                    self.inspection
                        .selected
                        .map_or("none".into(), |s| s.organism.0.to_string()),
                ),
                (
                    "selected-revision",
                    self.inspection
                        .selected
                        .map_or("none".into(), |s| format!("{:016x}", s.revision.0)),
                ),
            ]
            .into_iter()
            .map(|(name, value)| (name.to_string(), value))
            .collect(),
        }
    }

    fn drain_events(&mut self) -> Vec<String> {
        std::mem::take(&mut self.events)
    }

    fn act(&mut self, label: &str) -> bool {
        self.run_action(label)
    }

    // Attributed, not delivered. See the stack gap in the module docs: this
    // host has no route from a window point to either the section or a chrome
    // raster, and a silent swallow would let a scenario believe a click landed.
    fn press(&mut self, x: f32, y: f32) {
        self.events.push(format!("pointer-unrouted {x} {y}"));
    }

    fn moved(&mut self, x: f32, y: f32) {
        self.events.push(format!("pointer-unrouted {x} {y}"));
    }

    fn release(&mut self, x: f32, y: f32) {
        self.events.push(format!("pointer-unrouted {x} {y}"));
    }

    /// See the module docs: scripted work in flight, and a checkpoint with
    /// nothing queued is quiet.
    fn busy(&mut self) -> Option<bool> {
        Some(self.replay_pending() || self.pump.is_some() || self.runtime.queued_len() > 0)
    }
}

impl Driveable for Host {
    fn capture(&mut self, name: &str) -> bool {
        let path = self.capture_path(name);
        match self.capture_to(&path) {
            Ok(()) => {
                self.events.push(format!("captured {}", path.display()));
                true
            },
            Err(why) => {
                eprintln!("capture: {why}");
                false
            },
        }
    }
}

fn yes_no(flag: bool) -> String {
    if flag { "yes" } else { "no" }.to_string()
}

#[cfg(test)]
mod tests;
