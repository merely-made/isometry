// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Key-to-intent policy: what a keypress means, and whether it is let into
//! the queue.
//!
//! **The backlog problem, and the two-part fix.** `Runtime::queue` is an
//! unbounded `VecDeque` drained at exactly the ruled tempo (10/s), one intent
//! per tick. Two distinct sources can fill it faster than it drains:
//!
//! - **Holding a key.** winit's `Pressed` event fires once for the keydown
//!   and then again and again for as long as the key is held, at the OS's
//!   auto-repeat rate (20-30/s) — much faster than 10 ticks/s. Every one of
//!   those repeats used to queue its own intent, so holding W for a couple of
//!   seconds could queue dozens of copies of the same move, executing tens of
//!   seconds after the key was released. The fix is to drop repeats outright
//!   (`event.repeat`): a held key now contributes exactly the one intent its
//!   initial keydown asked for, and nothing while it is held. This does mean
//!   holding a key no longer walks continuously — press-and-hold stops being
//!   a locomotion mode — but that trade is the point: it is what stops a held
//!   key from multiplying.
//! - **Mashing a key.** Rapid *distinct* presses (real keydown/keyup pairs,
//!   `repeat == false` on every one) are not auto-repeat and are not
//!   filtered by the rule above, so a player who mashes E to eat, or W to
//!   dodge, can still queue faster than the tempo drains. This is where the
//!   queue cap in [`admits`] does its work: a new intent is only queued while
//!   the backlog is still shallow, so the worst case is bounded ticks of lag,
//!   not an ever-growing session-long debt.
//!
//! **Why the cap is per-[`Urgency`] rather than one number.** A movement
//! intent that has to wait behind a backlog is stale by the time it runs — it
//! names a direction from a moment that has passed, and the player has
//! likely already pressed a different one. Queuing it anyway is worse than
//! dropping it: it makes the character visibly walk somewhere the player
//! didn't just ask for. So movement gets the tight cap ([`MOVEMENT_QUEUE_CAP`])
//! and drops rather than queues once the backlog exceeds about a tick or two,
//! which keeps the queue tracking the present. A deliberate press — E to eat,
//! Q to deposit, C to dig — names a decision the player expects to land
//! exactly once, not a direction to keep re-asserting; dropping it silently
//! loses food or a dig. It gets the looser cap ([`DELIBERATE_QUEUE_CAP`]),
//! which only protects against a truly pathological backlog rather than
//! doing routine housekeeping on ordinary mashing.
//!
//! Both filters are host-side and never touch the trace: the trace records
//! intents actually applied, and a replay drives the queue directly
//! ([`Runtime::queue`] called straight from the recorded list), bypassing this
//! module entirely. What was queued and dropped at the keyboard cannot move a
//! replay's hash, only how quickly a played session answers a key.
//!
//! # Dev keys (DT1, DT2, DT3)
//!
//! Live only while `--dev` is set. Twelve, chosen clear of every key play
//! already owns (WASD, E, Space, Q, C, the arrows, Tab, R, T, Enter, Escape):
//!
//! - `P` — pause or unpause the clock. (DT1)
//! - `.` (period) — one step, off the clock.
//! - `,` (comma) — [`crate::app::DEV_STEP_N`] steps, off the clock.
//! - `[` — one rung slower on the speed ladder.
//! - `]` — one rung faster.
//! - `N` — follow the next living critter in id order, wrapping. (DT2)
//! - `B` — follow the previous one, wrapping the other way.
//! - `M` — snap follow back to the critter under the hand.
//! - `X` — end the epoch now. (DT3)
//! - `F` — force a birth from the followed critter.
//! - `K` — kill the followed critter.
//! - `G` — put matter into the ground under the followed critter.
//!
//! **The eight and the four are two different kinds of key, and the split is
//! the dev tools plan's second principle.** DT1's and DT2's eight never reach
//! [`Runtime::queue`]: they are host pacing over `Runtime::advance` and
//! `Runtime::step`, and a host-side camera centre, so none can enter the trace
//! and a run paused a hundred times hashes like one never paused. DT3's four
//! change the world, so every one of them queues an ordinary `Intent` through
//! `Runtime::queue` and lands in the trace — a replay reproduces it, and the
//! receipt counts it. There is no third kind. See [`DevKey`] and [`dev_key`].
//!
//! **Follow is not control.** `N`, `B` and `M` move where the section's slab is
//! centred and nothing else: the played body stays the played body, no intent
//! is queued, and `T` — the one key that does move control, and only at a
//! checkpoint — is untouched. `F`, `K` and `G` act on *the followed* critter
//! rather than the played one, which is what makes the inspector and the three
//! verbs one tool: what the tile is showing is what the key acts on.

use mesocosm_core::{Intent, Placement, World};
use mesocosm_mesh::VolumeMap;
use mesocosm_runtime::Checkpoint;
use winit::keyboard::{Key, NamedKey};

use crate::fixture;

/// How eagerly an intent should be let into an already-backlogged queue.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Urgency {
    /// WASD. Coalesces: a new one is dropped once the queue already holds a
    /// couple of ticks, so held-then-mashed movement tracks the present
    /// rather than replaying stale directions later.
    Movement,
    /// E/Space (metabolize), Q (deposit), C (carve): a single keystroke
    /// naming a decision, not a direction. Worth a longer queue before it is
    /// dropped, and dropping it is a last resort against a runaway backlog
    /// rather than routine housekeeping.
    Deliberate,
}

/// Ticks of backlog a [`Urgency::Movement`] intent tolerates before a new one
/// is dropped rather than queued. At the ruled 10 t/s, 2 ticks is 200ms: the
/// queue stays within about two frames of the present.
pub const MOVEMENT_QUEUE_CAP: usize = 2;

/// Ticks of backlog a [`Urgency::Deliberate`] intent tolerates. Looser than
/// movement's on purpose — see the module docs.
pub const DELIBERATE_QUEUE_CAP: usize = 10;

/// How urgently an intent should be treated once it exists. Everything but a
/// direction is deliberate.
pub fn urgency_of(intent: &Intent) -> Urgency {
    match intent {
        Intent::Move { .. } => Urgency::Movement,
        _ => Urgency::Deliberate,
    }
}

/// Whether a new intent of this urgency should be queued, given the current
/// backlog. `queued_len` is a fresh read of [`mesocosm_runtime::Runtime::queued_len`],
/// taken right before queuing.
pub fn admits(urgency: Urgency, queued_len: usize) -> bool {
    let cap = match urgency {
        Urgency::Movement => MOVEMENT_QUEUE_CAP,
        Urgency::Deliberate => DELIBERATE_QUEUE_CAP,
    };
    queued_len < cap
}

/// Turns a key into an answer while a checkpoint stands. `None` for every
/// other key.
///
/// **The keyboard narrows to the question.** A checkpoint is the world holding
/// still, so the ordinary verbs have nothing to act on and a stray W would
/// only sit in the queue going stale. Two keys, matching the two lines the
/// panel shows: Enter carries on, T takes the body on offer. Both produce
/// ordinary recorded intents — the choice enters the trace and replays like
/// any other.
pub fn answer_for(checkpoint: &Checkpoint, key: &Key) -> Option<Intent> {
    match key {
        Key::Named(NamedKey::Enter) => Some(checkpoint.default_answer()),
        Key::Character(c) if matches!(c.as_str(), "t" | "T") => checkpoint
            .heir()
            .map(|organism| Intent::TakeControl { organism }),
        _ => None,
    }
}

/// What a key means while the trait board is standing. (PE3b)
///
/// **Two, and only one of them is an intent.** Moving the cursor changes what
/// the panel is pointing at and nothing else, so it never reaches the queue;
/// committing sends an ordinary `Intent::Revise`, which enters the trace and
/// replays like every other answer. Enter and T stay where they are — the board
/// is a checkpoint, and its "carry on" is the same one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoardKey {
    /// Move the selection to the next candidate, wrapping.
    Next,
    /// Commit the selected candidate.
    Commit,
}

/// Turns a key into a board action. `None` for every other key, which then
/// falls through to [`answer_for`].
///
/// Tab rather than an arrow: the arrows pan the section and keep doing so at a
/// checkpoint, because where the camera is looking is presentation and does not
/// stop when the world does.
pub fn board_key(key: &Key) -> Option<BoardKey> {
    match key {
        Key::Named(NamedKey::Tab) => Some(BoardKey::Next),
        Key::Character(c) if matches!(c.as_str(), "r" | "R") => Some(BoardKey::Commit),
        _ => None,
    }
}

/// Turns a key into an intent. Pure, given the world state it needs to
/// resolve a meal's target: the host does not decide whether the intent is
/// legal, the core does, and reports a rejection through `Outcome`.
pub fn intent_for(world: &World, volumes: &VolumeMap, key: &Key) -> Option<Intent> {
    let step = 2;
    match key {
        Key::Character(c) => match c.as_str() {
            "w" | "W" => Some(Intent::Move {
                delta: [0, 0, -step],
            }),
            "s" | "S" => Some(Intent::Move {
                delta: [0, 0, step],
            }),
            "a" | "A" => Some(Intent::Move {
                delta: [-step, 0, 0],
            }),
            "d" | "D" => Some(Intent::Move {
                delta: [step, 0, 0],
            }),
            // The one verb, one key. The second one (F, for burning) is gone:
            // Mark ruled the hotkey pair unworkable as an interface and the
            // destination diegetic, so there is nothing left for a second key
            // to say.
            "e" | "E" => meal(world, volumes),
            "q" | "Q" => Some(Intent::Deposit { mass_mg: 60 }),
            // Digging at your own feet. Legality is embodiment plus reach,
            // and one voxel down is inside the shortest reach.
            "c" | "C" => world.position().map(|at| Intent::Carve {
                at: [at[0], at[1] - 1, at[2]],
                radius: 1,
            }),
            _ => None,
        },
        Key::Named(NamedKey::Space) => meal(world, volumes),
        _ => None,
    }
}

/// A dev-only action a key maps to.
///
/// **Two kinds, and [`DevKey::changes_the_world`] is which.** The first eight
/// are host-only — time control and the camera — and none of them reaches
/// [`mesocosm_runtime::Runtime::queue`], so none can enter the trace or move a
/// replay's hash. The last four are DT3's world-changing intents and every one
/// of them does queue. See the module docs and the dev tools plan's §2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevKey {
    /// `P`. Toggles whether the host passes elapsed time to the clock at
    /// all.
    TogglePause,
    /// `.` (period). Runs exactly one step, off the clock, whatever the
    /// pause state.
    Step,
    /// `,` (comma). Runs [`crate::app::DEV_STEP_N`] steps the same way.
    StepN,
    /// `[`. One rung slower on the speed ladder, floored at its slowest.
    SlowDown,
    /// `]`. One rung faster, capped at its fastest.
    SpeedUp,
    /// `N`. The next living critter in id order, wrapping. (DT2)
    FollowNext,
    /// `B`. The previous one, wrapping the other way.
    FollowBack,
    /// `M`. Back to the critter under the hand.
    FollowSelf,
    /// `X`. End the epoch now. (DT3)
    EndEpoch,
    /// `F`. Bear an offspring from the followed critter now.
    ForceBirth,
    /// `K`. End the followed critter's life now.
    Kill,
    /// `G`. Put [`crate::app::DEV_PLACE_MG`] into the ground under the followed
    /// critter.
    PlaceMatter,
}

impl DevKey {
    /// Whether this key queues an intent rather than moving host state.
    ///
    /// The two kinds the dev tools plan keeps apart, in one predicate. `true`
    /// for DT3's four and `false` for DT1's and DT2's eight, and there is
    /// nothing in between — a host-only action that queued, or a
    /// world-changing one that did not, would each break a different half of
    /// principle 2.
    pub fn changes_the_world(self) -> bool {
        matches!(
            self,
            Self::EndEpoch | Self::ForceBirth | Self::Kill | Self::PlaceMatter
        )
    }
}

/// Turns a key into a dev action. `None` for every other key, which then
/// falls through to the board, the checkpoint, or the ordinary play keys.
pub fn dev_key(key: &Key) -> Option<DevKey> {
    match key {
        Key::Character(c) => match c.as_str() {
            "p" | "P" => Some(DevKey::TogglePause),
            "." => Some(DevKey::Step),
            "," => Some(DevKey::StepN),
            "[" => Some(DevKey::SlowDown),
            "]" => Some(DevKey::SpeedUp),
            "n" | "N" => Some(DevKey::FollowNext),
            "b" | "B" => Some(DevKey::FollowBack),
            "m" | "M" => Some(DevKey::FollowSelf),
            "x" | "X" => Some(DevKey::EndEpoch),
            "f" | "F" => Some(DevKey::ForceBirth),
            "k" | "K" => Some(DevKey::Kill),
            "g" | "G" => Some(DevKey::PlaceMatter),
            _ => None,
        },
        _ => None,
    }
}

/// The next meal in reach. `None` when nothing is close enough.
///
/// One meal, one key. Where it goes is the body's answer, not a second
/// keystroke (TD4): a starved critter burns what it eats and a provisioned
/// one builds with it, and the budget that decides is on the panel in the
/// corner.
fn meal(world: &World, volumes: &VolumeMap) -> Option<Intent> {
    fixture::reachable(world).map(|m| fixture::metabolize(world, m, volumes, Placement::Planned))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admits_up_to_the_cap_and_no_further() {
        for len in 0..MOVEMENT_QUEUE_CAP {
            assert!(admits(Urgency::Movement, len), "len {len} should admit");
        }
        assert!(!admits(Urgency::Movement, MOVEMENT_QUEUE_CAP));
        assert!(!admits(Urgency::Movement, MOVEMENT_QUEUE_CAP + 5));

        for len in 0..DELIBERATE_QUEUE_CAP {
            assert!(admits(Urgency::Deliberate, len), "len {len} should admit");
        }
        assert!(!admits(Urgency::Deliberate, DELIBERATE_QUEUE_CAP));
    }

    #[test]
    fn the_board_has_two_keys_and_neither_is_a_verb() {
        assert_eq!(board_key(&Key::Named(NamedKey::Tab)), Some(BoardKey::Next));
        assert_eq!(
            board_key(&Key::Character("r".into())),
            Some(BoardKey::Commit)
        );
        // Everything else falls through to the checkpoint's own two answers,
        // which is what keeps Enter meaning "carry on" everywhere.
        for key in ["w", "e", "q", "c", "t"] {
            assert_eq!(board_key(&Key::Character(key.into())), None, "{key}");
        }
        assert_eq!(board_key(&Key::Named(NamedKey::Enter)), None);
    }

    #[test]
    fn only_move_is_movement_urgency() {
        assert_eq!(
            urgency_of(&Intent::Move { delta: [1, 0, 0] }),
            Urgency::Movement
        );
        assert_eq!(
            urgency_of(&Intent::Deposit { mass_mg: 60 }),
            Urgency::Deliberate
        );
        assert_eq!(
            urgency_of(&Intent::Carve {
                at: [0, 0, 0],
                radius: 1
            }),
            Urgency::Deliberate
        );
    }

    /// The twelve dev keys, and nothing else, resolve to a dev action.
    #[test]
    fn the_dev_keys_resolve_and_nothing_play_owns_collides() {
        assert_eq!(
            dev_key(&Key::Character("p".into())),
            Some(DevKey::TogglePause)
        );
        assert_eq!(
            dev_key(&Key::Character("P".into())),
            Some(DevKey::TogglePause)
        );
        assert_eq!(dev_key(&Key::Character(".".into())), Some(DevKey::Step));
        assert_eq!(dev_key(&Key::Character(",".into())), Some(DevKey::StepN));
        assert_eq!(dev_key(&Key::Character("[".into())), Some(DevKey::SlowDown));
        assert_eq!(dev_key(&Key::Character("]".into())), Some(DevKey::SpeedUp));
        // DT2's three, in both cases.
        assert_eq!(
            dev_key(&Key::Character("n".into())),
            Some(DevKey::FollowNext)
        );
        assert_eq!(
            dev_key(&Key::Character("N".into())),
            Some(DevKey::FollowNext)
        );
        assert_eq!(
            dev_key(&Key::Character("b".into())),
            Some(DevKey::FollowBack)
        );
        assert_eq!(
            dev_key(&Key::Character("m".into())),
            Some(DevKey::FollowSelf)
        );

        // DT3's four, in both cases.
        for (key, action) in [
            ("x", DevKey::EndEpoch),
            ("f", DevKey::ForceBirth),
            ("k", DevKey::Kill),
            ("g", DevKey::PlaceMatter),
        ] {
            assert_eq!(dev_key(&Key::Character(key.into())), Some(action), "{key}");
            assert_eq!(
                dev_key(&Key::Character(key.to_uppercase().into())),
                Some(action),
                "{key} uppercase"
            );
        }

        // Every key play already owns falls through undisturbed — `t` above
        // all, because it is the one key that does move control.
        for key in ["w", "a", "s", "d", "e", "q", "c", "t", "r"] {
            assert_eq!(dev_key(&Key::Character(key.into())), None, "{key}");
        }
        assert_eq!(dev_key(&Key::Named(NamedKey::Space)), None);
        assert_eq!(dev_key(&Key::Named(NamedKey::Tab)), None);
        assert_eq!(dev_key(&Key::Named(NamedKey::Enter)), None);
        assert_eq!(dev_key(&Key::Named(NamedKey::Escape)), None);
    }

    /// **The two kinds of dev key, and the line between them** (dev tools plan
    /// §2, principle 2). DT3's four queue an intent; DT1's and DT2's eight
    /// never do, which is what keeps a paused run hashing like a straight one.
    #[test]
    fn only_the_four_world_changing_dev_keys_say_they_change_the_world() {
        for action in [
            DevKey::EndEpoch,
            DevKey::ForceBirth,
            DevKey::Kill,
            DevKey::PlaceMatter,
        ] {
            assert!(action.changes_the_world(), "{action:?}");
        }
        for action in [
            DevKey::TogglePause,
            DevKey::Step,
            DevKey::StepN,
            DevKey::SlowDown,
            DevKey::SpeedUp,
            DevKey::FollowNext,
            DevKey::FollowBack,
            DevKey::FollowSelf,
        ] {
            assert!(!action.changes_the_world(), "{action:?}");
        }
    }

    #[test]
    fn wasd_and_space_resolve_to_intents_a_bare_world_admits() {
        let world = World::new(0x1234, 40);
        let volumes = fixture::volumes();
        for key in ["w", "a", "s", "d"] {
            let k = Key::Character(key.into());
            assert!(matches!(
                intent_for(&world, &volumes, &k),
                Some(Intent::Move { .. })
            ));
        }
        // Q always resolves; whether the world accepts a deposit is the
        // core's call, not this function's.
        let q = Key::Character("q".into());
        assert!(matches!(
            intent_for(&world, &volumes, &q),
            Some(Intent::Deposit { .. })
        ));
    }
}
