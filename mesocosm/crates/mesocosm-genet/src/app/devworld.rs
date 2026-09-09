// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The four world-changing dev keys (DT3): what each one asks the world for.
//!
//! Split out of `app.rs` for the reason [`super::devtime`] and [`super::follow`]
//! were — that file sits at its six-hundred-line ceiling — but the discipline
//! here is the **opposite** of theirs, and deliberately so. Those two are host
//! state that never reaches [`mesocosm_runtime::Runtime::queue`]. Every key
//! here does nothing but build an ordinary [`Intent`] and queue it, so what it
//! does to the world goes into the trace, replays, and is counted on the
//! receipt.
//!
//! # Nothing is decided here
//!
//! There is no legality check in this file. Whether the epoch may end now,
//! whether a parent can provision an offspring, whether a body is alive, and
//! whether a cell is on the grid are all the world's questions, and the world
//! answers them by name through `Outcome::Rejected` — which the vitals lane
//! already prints in plain words. A host that pre-checked would be a second
//! authority quietly agreeing with the first until the day it did not.
//!
//! # The subject is the followed critter
//!
//! [`Host::followed`] and not `World::controlled`, which is what makes the DT2
//! inspector and these three verbs one tool: what the tile is showing is what
//! `F`, `K` and `G` act on. `X` names nobody, because ending an epoch is a
//! world rule's business rather than a body's.
//!
//! [`Host::followed`]: super::Host::followed

use mesocosm_core::Intent;

use super::Host;
use crate::input::{self, DevKey};

/// Matter one press of `G` puts into the ground. (DT3)
///
/// Half of `PLACE_MATTER_MAX_MG`, so the key is visibly under the bound rather
/// than sitting on it: a press that always placed the maximum could not
/// distinguish a working bound from an absent one, and the refusal case would
/// only ever be reachable by editing this constant.
pub const DEV_PLACE_MG: u64 = mesocosm_core::PLACE_MATTER_MAX_MG / 2;

/// Checked where it cannot drift: a press that placed nothing, or that sat on
/// the world's bound, would make `OverBound` unreachable from the keyboard.
const _: () = assert!(DEV_PLACE_MG > 0 && DEV_PLACE_MG < mesocosm_core::PLACE_MATTER_MAX_MG);

impl Host {
    /// The intent one of DT3's four keys asks for, or `None` when there is
    /// nobody to ask it about.
    ///
    /// `None` only for the three that name a body, and only when nothing is
    /// followed and nobody is embodied — a world with no one in it is a
    /// legitimate state, and there is no body for `F`, `K` or `G` to be about.
    pub(super) fn dev_intent(&self, action: DevKey) -> Option<Intent> {
        match action {
            DevKey::EndEpoch => Some(Intent::EndEpoch),
            DevKey::ForceBirth => Some(Intent::ForceBirth {
                organism: self.followed()?,
            }),
            DevKey::Kill => Some(Intent::Kill {
                organism: self.followed()?,
            }),
            // Under the followed body rather than under the played one, for
            // the module's own reason: the tile is showing that critter, and
            // enriching the ground it is standing on is the readable version
            // of this verb.
            DevKey::PlaceMatter => Some(Intent::PlaceMatter {
                at: self.follow_at(),
                mass_mg: DEV_PLACE_MG,
            }),
            _ => None,
        }
    }

    /// Queues one of DT3's four, through the same door and the same backlog
    /// cap every play key uses.
    ///
    /// `Runtime::queue` and nothing else: a dev intent that reached the world
    /// by a side door would be exactly the second authority the plan's stop
    /// rules forbid, and it would not be in the trace.
    pub(super) fn dev_world_key(&mut self, action: DevKey) {
        let Some(intent) = self.dev_intent(action) else {
            return;
        };
        let urgency = input::urgency_of(&intent);
        if input::admits(urgency, self.runtime.queued_len()) {
            self.runtime.queue(intent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostConfig;
    use mesocosm_core::Outcome;
    use winit::keyboard::Key;

    fn host() -> Host {
        Host::new(HostConfig {
            organisms: 8,
            dev: true,
            ..HostConfig::default()
        })
    }

    /// **Every one of the four queues, and every one lands in the trace.** The
    /// other half of DT1's and DT2's claim: those keys queue nothing, these
    /// queue an ordinary intent, and there is no third route to the world.
    #[test]
    fn each_world_changing_dev_key_queues_an_intent_that_reaches_the_trace() {
        for (key, expected) in [
            ("x", "EndEpoch"),
            ("f", "ForceBirth"),
            ("k", "Kill"),
            ("g", "PlaceMatter"),
        ] {
            let mut host = host();
            assert!(host.try_dev_key(&Key::Character(key.into())), "{key}");
            assert_eq!(host.runtime.queued_len(), 1, "{key} queued nothing");

            host.runtime.step(1);
            let applied = host
                .runtime
                .trace()
                .last()
                .expect("the step applied something")
                .clone();
            assert!(applied.is_dev(), "{key} applied {applied:?}");
            assert!(
                format!("{applied:?}").starts_with(expected),
                "{key} applied {applied:?}"
            );
        }
    }

    /// The four are the world's to refuse, not the host's. `K` on a body that
    /// is already dead reaches the world and comes back refused by name, rather
    /// than being swallowed here.
    #[test]
    fn a_dev_key_the_world_refuses_is_refused_by_the_world() {
        let mut host = host();
        let target = host
            .runtime
            .world()
            .living()
            .find(|o| Some(o.id) != host.runtime.world().controlled_id())
            .expect("somebody else is alive")
            .id;
        host.follow = Some(target);

        // Once, which lands.
        assert!(host.try_dev_key(&Key::Character("k".into())));
        host.runtime.step(1);
        assert!(matches!(
            host.runtime.last_outcomes().first(),
            Some(Outcome::Killed { .. })
        ));
        assert_eq!(host.runtime.dev_intents(), 1);

        // And again, on the corpse. Follow snaps back on the next frame, so
        // the target is set again here to keep the refusal the subject.
        host.follow = Some(target);
        assert!(host.try_dev_key(&Key::Character("k".into())));
        host.runtime.step(1);
        assert!(
            matches!(
                host.runtime.last_outcomes().first(),
                Some(Outcome::Rejected(mesocosm_core::Rejection::NotLiving(who))) if *who == target
            ),
            "{:?}",
            host.runtime.last_outcomes()
        );
        assert_eq!(
            host.runtime.dev_intents(),
            1,
            "a refused dev intent applied nothing, so it is not counted"
        );
    }

    /// The dev keys are dead outside `--dev`, DT3's four included.
    #[test]
    fn nothing_is_queued_when_the_dev_flag_is_off() {
        let mut host = Host::new(HostConfig {
            organisms: 8,
            dev: false,
            ..HostConfig::default()
        });
        for key in ["x", "f", "k", "g"] {
            assert!(!host.try_dev_key(&Key::Character(key.into())), "{key}");
        }
        assert_eq!(host.runtime.queued_len(), 0);
        assert_eq!(host.runtime.dev_intents(), 0);
    }
}
