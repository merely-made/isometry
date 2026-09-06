// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a scenario's `act` can ask for, and the one route a key takes. (DT4)
//!
//! # The mapping is the host's own key names
//!
//! `act` carries **the same names `--help` documents**, one per key, rather
//! than a second vocabulary of intent names and fields. Two reasons, and the
//! second is the load-bearing one:
//!
//! - It is smaller. `act e` is the E key, `act x` is the X key, and there is
//!   nothing to look up: a scenario reads like the controls list.
//! - It cannot drift. [`Host::press_key`] is the *same* function the window's
//!   keyboard handler calls, so a scripted `act e` goes through
//!   `crate::input::intent_for`, the checkpoint's answers, the board's two
//!   keys, the dev-key split and the queue's backlog cap in exactly the order a
//!   person's keypress does. A mapping that built intents directly would be a
//!   second input policy, quietly agreeing with the first until the day it did
//!   not — and the dev tools plan's stop rules forbid exactly that.
//!
//! An intent name and fields would have bought the ability to script things no
//! key can express (metabolize *that* organism, place matter *there*), and the
//! host does not need them: what DT3's keys act on is the followed critter, and
//! following is scriptable.
//!
//! # And five host-side names for what no key says
//!
//! A script has no eye and no hand, so a handful of things a person does
//! without thinking have no key at all. These are host actions rather than new
//! genet-probe verbs, per the plan's instruction: where the stack lacks a verb,
//! express it through `act` with a host-side name and report the gap.
//!
//! | Name | What it does | Why no key |
//! | --- | --- | --- |
//! | `follow <id>` | camera onto one critter | `--follow`'s job; the keys only cycle |
//! | `follow-nearest` | camera onto the nearest living neighbour | a person looks; a script cannot |
//! | `follow-child` | camera onto the last forced birth's offspring | the id is not knowable in advance |
//! | `hunt <steps>` | eat, or hunt, for N ticks | where `--auto-eat` went |
//! | `demo <steps>` | N ticks of the recorded demo script | where `--record-demo` went |
//!
//! # The two pumps run off the clock, and that is the point
//!
//! `--auto-eat` metabolized every N *steps* while wall time drove the ticks, so
//! how much of an enclosure a capture run had eaten depended on the frame rate
//! it was captured at. A scenario asks for ticks and gets exactly those:
//! [`Pump`] calls [`mesocosm_runtime::Runtime::step`] directly, the way DT1's
//! manual step key does. A scripted run is therefore the same run on any
//! machine — which is what lets one assert a hash at the end of it.
//!
//! A scenario that also wants wall time back pairs a pump with DT1's pause
//! (`act p`), so nothing but the pump moves the world.

use mesocosm_core::{Intent, OrganismId, Placement};
use winit::keyboard::{Key, NamedKey};

use super::Host;
use crate::input;
use crate::played::Script;
use crate::{fixture, played};

/// The least mass a critter must carry to be picked by `follow-nearest`.
///
/// A birth costs its parent a quarter of itself, so the neighbour a scenario
/// reaches for is one that could actually bear. Below this the enclosure's
/// newest juveniles would be chosen constantly and `act f` would refuse every
/// time — a scripted receipt that only ever exercised the refusal.
pub const NEIGHBOUR_MIN_MG: u64 = 400;

/// Scripted steps one frame takes.
///
/// Twenty-five, so three thousand steps is a hundred and twenty-odd frames
/// rather than three thousand. It changes nothing about what is recorded:
/// each step is one intent and one tick whatever pumps it, so the trace and
/// the hash are the headless recording's either way. It is only how long the
/// window is open for.
pub const PUMP_STEPS_PER_FRAME: u64 = 25;

/// A stretch of scripted ticks, mid-pump.
#[derive(Clone, Copy, Debug)]
pub enum Pump {
    /// Ordinary play with nobody at the keyboard: eat what is in reach, walk
    /// toward what is not, and where there is neither, say carry on.
    ///
    /// **`Resume` rather than `Idle` for the third case**, deliberately: a run
    /// of idles hands the body back to its instincts after
    /// `INSTINCT_IDLE_TICKS` and the hand is gone, which would take the trait
    /// board with it at a boundary. `Resume` is a hand saying carry on.
    Hunt { left: u64 },
    /// The recorded demo script, through [`crate::played::demo_step`].
    Demo {
        /// Which step of the script comes next. The script branches on it, so
        /// it counts steps taken rather than steps left.
        step: u64,
        until: u64,
        script: Script,
    },
}

impl Host {
    /// **The one route a key takes into the world**, whether a person pressed
    /// it or a scenario asked for it by name.
    ///
    /// Dev keys first: none of the twelve collides with a play key (see
    /// [`crate::input`]'s module docs), but checking here means a dev build's
    /// controls never drift one key handler's accident away from live.
    ///
    /// At a checkpoint the keyboard narrows to the answers: the world is
    /// stopped, so a move has nothing to move and would only go stale in the
    /// queue. At a lineage checkpoint the board's own two keys come first — one
    /// of which sends no intent at all.
    pub(super) fn press_key(&mut self, key: &Key) {
        if self.try_graft_key(key) {
            return;
        }
        if self.try_camera_key(key) {
            return;
        }
        if self.try_inspection_key(key) {
            return;
        }
        if self.try_dev_key(key) {
            return;
        }
        let intent = match self.board_key(key) {
            Some(taken) => taken,
            None => match self.runtime.checkpoint() {
                Some(checkpoint) => input::answer_for(checkpoint, key),
                None => input::intent_for(self.runtime.world(), &self.volumes, key),
            },
        };
        if let Some(intent) = intent {
            let urgency = input::urgency_of(&intent);
            if input::admits(urgency, self.runtime.queued_len()) {
                self.runtime.queue(intent);
            }
        }
    }

    /// The trait board's own keys, while it is standing. (PE3b)
    ///
    /// `None` means the board did not take this key and it falls through to the
    /// checkpoint's answers. `Some(None)` means the board took it and produced
    /// no intent — moving the cursor is presentation, and putting a cursor
    /// move in the queue would be putting it in the trace.
    fn board_key(&mut self, key: &Key) -> Option<Option<Intent>> {
        let action = input::board_key(key)?;
        let review = self.runtime.review()?;
        match action {
            // `Review::commit` refuses the status quo and every untakeable row,
            // so the key cannot send a revision the world would only reject.
            input::BoardKey::Commit => Some(review.commit(self.board_row)),
            input::BoardKey::Next => {
                // Wrapping, and over every row including the untakeable ones:
                // a candidate you cannot take yet is a thing to read, and
                // skipping it would hide the reason it is there for.
                let rows = review.rows.len();
                self.board_row = if rows == 0 {
                    0
                } else {
                    (self.board_row + 1) % rows
                };
                Some(None)
            },
        }
    }

    /// Runs one named action. `false` when there is no such name, which the
    /// scenario driver reports as a failed step rather than a silent no-op.
    pub(super) fn run_action(&mut self, label: &str) -> bool {
        let (name, rest) = match label.trim().split_once(char::is_whitespace) {
            Some((name, rest)) => (name, rest.trim()),
            None => (label.trim(), ""),
        };
        if let Some(key) = key_named(name) {
            if self.try_camera_key(&key) {
                return true;
            }
            // A replay's trace is the whole of what it applies. An `act` that
            // queued beside it would put an intent in the run the recording
            // never had, and the hash would be the first thing to say so.
            if self.config.replay.is_some() {
                self.events
                    .push(format!("act-refused {name}: a replay drives itself"));
                return false;
            }
            self.press_key(&key);
            // A dev step key takes ticks here rather than in a frame, so what
            // the world answered to it belongs to this step, not the next one.
            self.note_outcomes();
            return true;
        }
        if self.grafting.open {
            return false;
        }
        // The host actions below set state and start pumps; none of them takes
        // a tick, so there is nothing here for the world to have answered.
        match name {
            "follow" => match rest.parse::<u32>() {
                Ok(id) => {
                    self.follow = Some(OrganismId(id));
                    self.follow_lost = None;
                    true
                },
                Err(_) => false,
            },
            "follow-nearest" => match self.nearest_neighbour() {
                Some(id) => {
                    self.follow = Some(id);
                    self.follow_lost = None;
                    self.events.push(format!("followed-nearest {}", id.0));
                    true
                },
                None => {
                    self.events
                        .push("follow-nearest: nobody big enough is alive nearby".to_string());
                    false
                },
            },
            "follow-child" => match self.last_child {
                Some(id) => {
                    self.follow = Some(id);
                    self.follow_lost = None;
                    self.events.push(format!("followed-child {}", id.0));
                    true
                },
                None => {
                    self.events
                        .push("follow-child: no birth has been forced yet".to_string());
                    false
                },
            },
            "hunt" => match rest.parse::<u64>() {
                Ok(left) => {
                    self.pump = (left > 0).then_some(Pump::Hunt { left });
                    true
                },
                Err(_) => false,
            },
            "demo" => match rest.parse::<u64>() {
                Ok(until) => {
                    self.pump = (until > 0).then_some(Pump::Demo {
                        step: 0,
                        until,
                        script: Script::default(),
                    });
                    true
                },
                Err(_) => false,
            },
            _ => false,
        }
    }

    /// The nearest living critter to the **played** one that is big enough to
    /// bear.
    ///
    /// Measured from the played body rather than the camera, so two
    /// `follow-nearest` in a row walk outward from one place instead of
    /// chaining off wherever the last one landed. Manhattan distance and then
    /// id, so the answer does not depend on the roster's order — the same
    /// tie-break every ordered read in core uses.
    fn nearest_neighbour(&self) -> Option<OrganismId> {
        let world = self.runtime.world();
        let me = world.controlled_id();
        let here = world.position().unwrap_or_else(|| self.follow_at());
        world
            .living()
            .filter(|o| Some(o.id) != me && o.biomass_mg() >= NEIGHBOUR_MIN_MG)
            .map(|o| {
                let distance: i32 = (0..3)
                    .map(|axis| (o.position[axis] - here[axis]).abs())
                    .sum();
                (distance, o.id)
            })
            .min()
            .map(|(_, id)| id)
    }

    /// One frame of a scripted stretch: up to [`PUMP_STEPS_PER_FRAME`] ticks,
    /// off the clock.
    pub(super) fn pump_frame(&mut self) {
        let Some(pump) = self.pump.take() else { return };
        self.pump = match pump {
            Pump::Hunt { left } => {
                let taking = left.min(PUMP_STEPS_PER_FRAME);
                for _ in 0..taking {
                    let intent = self.hunting_intent();
                    self.runtime.queue(intent);
                    self.steps += self.runtime.step(1);
                }
                let left = left - taking;
                if left == 0 {
                    self.events.push("hunt-finished".to_string());
                }
                (left > 0).then_some(Pump::Hunt { left })
            },
            Pump::Demo {
                mut step,
                until,
                mut script,
            } => {
                let end = (step + PUMP_STEPS_PER_FRAME).min(until);
                while step < end {
                    played::demo_step(&mut self.runtime, &self.volumes, step, &mut script);
                    step += 1;
                    self.steps += 1;
                }
                if step >= until {
                    self.events.push(format!("demo-finished {step} steps"));
                    None
                } else {
                    Some(Pump::Demo {
                        step,
                        until,
                        script,
                    })
                }
            },
        };
        self.note_outcomes();
    }

    /// One tick of ordinary play with nobody at the keyboard. See [`Pump::Hunt`].
    fn hunting_intent(&self) -> Intent {
        let world = self.runtime.world();
        // A checkpoint holds the world, and a hunt that walked into one would
        // queue moves nobody can take. Carry on is the answer that lifts it.
        if let Some(checkpoint) = self.runtime.checkpoint() {
            return checkpoint.default_answer();
        }
        if let Some(target) = fixture::reachable(world) {
            // What it eats, it eats; whether that grows a body or refills a
            // budget is the body's to decide.
            return fixture::metabolize(world, target, &self.volumes, Placement::Planned);
        }
        // **Hunt, do not wait.** Reach became anatomy in P2, so a starting
        // critter touches about three voxels. An unattended run that stood
        // still grew nothing in nine hundred frames.
        match fixture::toward_prey(world) {
            Some(delta) => Intent::Move { delta },
            None => Intent::Resume,
        }
    }
}

/// The key a documented name stands for. `None` for a name that is not a key,
/// which then falls through to the host actions above.
///
/// Exactly the letters and named keys the `--help` controls list prints, and
/// nothing invented here: `space`, `enter` and `tab` are spelled out because a
/// scenario line is whitespace-delimited and a literal space cannot be a token.
fn key_named(name: &str) -> Option<Key> {
    match name {
        "space" => Some(Key::Named(NamedKey::Space)),
        "enter" => Some(Key::Named(NamedKey::Enter)),
        "tab" => Some(Key::Named(NamedKey::Tab)),
        "escape" => Some(Key::Named(NamedKey::Escape)),
        "up" => Some(Key::Named(NamedKey::ArrowUp)),
        "down" => Some(Key::Named(NamedKey::ArrowDown)),
        "left" => Some(Key::Named(NamedKey::ArrowLeft)),
        "right" => Some(Key::Named(NamedKey::ArrowRight)),
        // Play: WASD, E, Q, C; T at a checkpoint; R at the board.
        // Dev (live only under `--dev`): P . , [ ] N B M X F K G.
        "w" | "a" | "s" | "d" | "e" | "q" | "c" | "t" | "r" | "p" | "." | "," | "[" | "]" | "n"
        | "b" | "m" | "x" | "f" | "k" | "g" | "h" | "i" | "j" | "l" | "u" | "z" | "v" => {
            Some(Key::Character(name.into()))
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostConfig;

    fn host() -> Host {
        Host::new(HostConfig {
            organisms: 12,
            dev: true,
            ..HostConfig::default()
        })
    }

    /// Every documented key is an `act` name, and every `act` name that looks
    /// like a key is one. The list cannot drift from `input`'s own mapping
    /// without this failing.
    #[test]
    fn every_key_the_host_documents_is_an_act_name() {
        for name in [
            "w", "a", "s", "d", "e", "q", "c", "t", "r", "space", "enter", "tab",
        ] {
            assert!(key_named(name).is_some(), "{name} is a documented play key");
        }
        for name in ["p", ".", ",", "[", "]", "n", "b", "m", "x", "f", "k", "g"] {
            let key = key_named(name).expect("a documented dev key");
            assert!(
                crate::input::dev_key(&key).is_some(),
                "{name} must resolve through input::dev_key, not a second table"
            );
        }
        assert!(key_named("wasd").is_none());
        assert!(key_named("follow").is_none(), "a host action, not a key");
    }

    /// A world-changing dev key asked for by name queues exactly what the same
    /// key pressed by hand queues — the one route, proven.
    #[test]
    fn an_act_named_for_a_dev_key_takes_the_same_route_a_press_does() {
        let mut acted = host();
        assert!(acted.run_action("x"));
        let mut pressed = host();
        pressed.press_key(&Key::Character("x".into()));

        assert_eq!(acted.runtime.queued_len(), 1);
        assert_eq!(
            acted.runtime.queued_len(),
            pressed.runtime.queued_len(),
            "the act and the press agree"
        );
        acted.runtime.step(1);
        pressed.runtime.step(1);
        assert_eq!(acted.runtime.trace(), pressed.runtime.trace());
        assert_eq!(acted.runtime.state_hash(), pressed.runtime.state_hash());
    }

    #[test]
    fn terrarium_camera_keys_turn_without_touching_world_hash() {
        let mut host = Host::new(HostConfig {
            dev: false,
            scene: crate::played::SceneMode::Terrarium,
            camera: crate::section::CameraMode::TerrariumEast,
            ..HostConfig::default()
        });
        let before = host.runtime.state_hash();
        assert!(host.run_action("v"));
        assert_eq!(
            host.config.camera,
            crate::section::CameraMode::TerrariumSouth
        );
        assert_eq!(host.runtime.state_hash(), before);
        assert!(host.run_action("z"));
        assert_eq!(
            host.config.camera,
            crate::section::CameraMode::TerrariumEast
        );
        assert_eq!(host.runtime.state_hash(), before);
    }

    /// An unknown name fails loudly rather than passing as a no-op.
    #[test]
    fn an_unknown_action_is_refused() {
        let mut host = host();
        assert!(!host.run_action("teleport"));
        assert!(!host.run_action("follow not-a-number"));
        assert!(!host.run_action("hunt sometimes"));
    }

    /// `follow-nearest` picks somebody alive, somebody else, and somebody big
    /// enough to be worth forcing a birth from.
    #[test]
    fn follow_nearest_takes_a_living_neighbour_big_enough_to_bear() {
        let mut host = host();
        assert!(host.run_action("follow-nearest"));
        let picked = host.follow.expect("it followed somebody");
        let world = host.runtime.world();
        assert_ne!(Some(picked), world.controlled_id(), "somebody else");
        let organism = world
            .living()
            .find(|o| o.id == picked)
            .expect("and somebody alive");
        assert!(organism.biomass_mg() >= NEIGHBOUR_MIN_MG);
    }

    /// `follow-child` refuses until there is a child to follow, rather than
    /// silently following nobody.
    #[test]
    fn follow_child_refuses_before_any_birth_is_forced() {
        let mut host = host();
        assert!(!host.run_action("follow-child"));
        assert!(
            host.events.iter().any(|e| e.contains("no birth")),
            "{:?}",
            host.events
        );
    }

    /// **The demo pumped by frames is the demo recorded by a loop.** The whole
    /// claim of moving `--record-demo` behind the driver: `demo_step` is one
    /// function, so how many of it a frame takes changes the window's runtime
    /// and nothing about the trace.
    #[test]
    fn a_frame_pumped_demo_reaches_the_headless_recordings_hash() {
        const STEPS: u64 = 120;
        const FOUNDERS: u32 = 40;
        let recorded = crate::played::record_demo(crate::played::DEMO_SEED, FOUNDERS, 10, STEPS);

        let mut host = Host::new(HostConfig {
            seed: crate::played::DEMO_SEED,
            organisms: FOUNDERS,
            generated_content: false, // This test compares the historical fixture recorder.
            body_layout: crate::played::BodyLayout::Axial,
            ..HostConfig::default()
        });
        assert!(host.run_action(&format!("demo {STEPS}")));
        while host.pump.is_some() {
            host.pump_frame();
        }

        assert_eq!(host.runtime.trace(), recorded.intents.as_slice());
        assert_eq!(host.runtime.state_hash(), recorded.state_hash);
        assert_eq!(host.steps, STEPS);
    }

    /// `hunt` is where `--auto-eat` went, and it takes exactly the ticks it was
    /// asked for — no clock in it, so a scripted run is the same run at any
    /// frame rate.
    #[test]
    fn hunt_takes_exactly_the_ticks_it_was_asked_for() {
        let mut host = host();
        assert!(host.pump.is_none());
        assert!(host.run_action("hunt 12"));
        assert!(matches!(host.pump, Some(Pump::Hunt { left: 12 })));
        while host.pump.is_some() {
            host.pump_frame();
        }
        assert_eq!(host.steps, 12);
        assert_eq!(host.runtime.trace().len(), 12);
        // And a hand is still on the body, which a run of idles would have lost.
        assert!(host.runtime.world().controlled_id().is_some());

        // `hunt 0` is a hunt of nothing, not a standing setting to turn off.
        assert!(host.run_action("hunt 0"));
        assert!(host.pump.is_none());
    }

    /// Two `follow-nearest` in a row walk outward from the played body rather
    /// than chaining off wherever the last one landed — which is what lets a
    /// scenario kill one neighbour and then bear from the next.
    #[test]
    fn two_follow_nearest_in_a_row_take_two_different_neighbours() {
        // A denser enclosure than the tests above, so there is a second
        // neighbour to find at all.
        let mut host = Host::new(HostConfig {
            organisms: 120,
            dev: true,
            ..HostConfig::default()
        });
        assert!(host.run_action("follow-nearest"));
        let first = host.follow.expect("somebody");
        assert!(host.run_action("k"));
        host.runtime.step(1);
        assert!(host.run_action("follow-nearest"));
        let second = host.follow.expect("somebody else");
        assert_ne!(first, second, "the corpse is not living, so it is not next");
    }
}
