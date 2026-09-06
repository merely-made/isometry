// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-only follow (DT2): which critter the section's slab is centred on.
//!
//! Split out of `app.rs` for the reason [`super::devtime`] was — the file sits
//! at its six-hundred-line ceiling — and it keeps the same discipline.
//! **Nothing here reaches [`mesocosm_runtime::Runtime::queue`]**, so nothing
//! here can enter the trace or move a replay's hash: following changes where
//! the camera looks and nothing else.
//!
//! # Follow is not control
//!
//! [`Host::followed`] answers *whose body the camera is on*; `World::controlled`
//! answers *whose body a key would move*. The two are deliberately separate and
//! the second never moves here: `crate::section::pose_of` still poses the
//! controlled critter at full fidelity and `roster_of` still excludes it, so
//! following somebody else moves the slab's centre over them and leaves the
//! played body exactly where it was.
//!
//! # A followed critter that dies is reported
//!
//! The one thing follow may not do is fail quietly. When the target stops being
//! alive, [`Host::update_follow`] takes the record's own ending
//! ([`mesocosm_core::History::ending`]) and keeps it as a [`Lost`] for the tile
//! to print, then snaps follow back to the controlled critter. The succession
//! lane is untouched: a death of the *controlled* critter is still the
//! driver's checkpoint and behaves exactly as it did.

use mesocosm_core::{OrganismId, World};
use mesocosm_views::Lost;

use super::Host;
use crate::input;

impl Host {
    /// The critter the camera is on: the follow target while it stands, and
    /// otherwise whoever is under the hand.
    ///
    /// `None` when a follow target is set and there is no hand — a real state,
    /// and the one the follow centre falls back to the origin for.
    pub(super) fn followed(&self) -> Option<OrganismId> {
        self.follow.or_else(|| self.runtime.world().controlled_id())
    }

    /// Where the section's slab is centred, in voxels.
    ///
    /// Presentation only, exactly as the pan beside it is. The followed body's
    /// position, or the played one's, or the origin when the world holds
    /// neither.
    pub(super) fn follow_at(&self) -> [i32; 3] {
        let world = self.runtime.world();
        self.followed()
            .and_then(|id| world.organisms.iter().find(|o| o.id == id))
            .map(|organism| organism.position)
            .or_else(|| world.position())
            .unwrap_or([0, 0, 0])
    }

    /// The whole of DT2's key handling, called from [`Host::try_dev_key`].
    pub(super) fn follow_key(&mut self, action: input::DevKey) {
        let world = self.runtime.world();
        let from = self.followed();
        self.follow = match action {
            // Back to the hand. `None` *is* "the controlled critter", so this
            // is a clear rather than a lookup: control may move afterwards and
            // the camera should follow it there.
            input::DevKey::FollowSelf => None,
            input::DevKey::FollowNext => next_living(world, from, Step::Forward),
            input::DevKey::FollowBack => next_living(world, from, Step::Back),
            _ => self.follow,
        };
        // Following somebody new clears the last one's death notice: it has
        // been read, and the tile is now about a different body.
        if matches!(
            action,
            input::DevKey::FollowSelf | input::DevKey::FollowNext | input::DevKey::FollowBack
        ) {
            self.follow_lost = None;
        }
        // A target equal to the controlled critter is held as "no target", so
        // one state means one thing.
        if self.follow == world.controlled_id() {
            self.follow = None;
        }
    }

    /// Keeps the follow target honest, once a frame.
    ///
    /// A target that stopped being alive is reported and dropped; everything
    /// else is left alone. Called before the frame reads its centre, so the
    /// camera never spends a frame on a body that is gone.
    pub(super) fn update_follow(&mut self) {
        let Some(target) = self.follow else { return };
        let world = self.runtime.world();
        if world
            .organisms
            .iter()
            .any(|o| o.id == target && o.is_alive())
        {
            return;
        }
        let ending = self.runtime.history().ending(target);
        self.follow_lost = Some(mesocosm_views::lost_of(target, ending, world.tick));
        self.follow = None;
    }

    /// The dev lane's follow half, taken fresh each frame. (DT2)
    ///
    /// The accounts come off the driver, which is where a reduction of the flow
    /// stream lives; everything else is a `World` query. Nothing is computed
    /// here — see `mesocosm_views::dev::follow`.
    pub(super) fn follow_reading(&self) -> (Option<mesocosm_views::Follow>, Option<Lost>) {
        let follow = self.followed().and_then(|id| {
            mesocosm_views::follow_of(self.runtime.world(), id, self.runtime.accounts())
        });
        (follow, self.follow_lost)
    }

    /// Points the driver's per-body window at whoever is being followed, so
    /// the accounts on the tile are that body's.
    ///
    /// Idempotent, and called before the frame steps: `Runtime::watch` keeps
    /// the window it has for a body already watched, and starts over for a new
    /// one.
    pub(super) fn watch_followed(&mut self) {
        if !self.config.dev {
            return;
        }
        let followed = self.followed();
        self.runtime.watch(followed);
    }
}

/// Which way the roster is walked.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Forward,
    Back,
}

/// The next living organism in id order, wrapping, skipping the dead.
///
/// `World::living` is the core reading and it is already in id order, because
/// the roster itself is; this is only which end of it to take. `None` when the
/// enclosure holds nothing alive at all, which is the honest answer rather than
/// a stale target.
fn next_living(world: &World, from: Option<OrganismId>, step: Step) -> Option<OrganismId> {
    let ids: Vec<OrganismId> = world.living().map(|organism| organism.id).collect();
    let from = from?;
    match step {
        Step::Forward => ids
            .iter()
            .find(|id| **id > from)
            .or_else(|| ids.first())
            .copied(),
        Step::Back => ids
            .iter()
            .rev()
            .find(|id| **id < from)
            .or_else(|| ids.last())
            .copied(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostConfig;
    use winit::keyboard::Key;

    /// A small enclosure with the dev lane live. Eight founders is enough for
    /// the roster to have somebody else in it and small enough to build in a
    /// unit test.
    fn host() -> Host {
        Host::new(HostConfig {
            organisms: 8,
            dev: true,
            ..HostConfig::default()
        })
    }

    /// **The DT2 done-condition, executable**: following a critter that is not
    /// under control does not move control, queues nothing, and cannot move the
    /// world.
    #[test]
    fn following_does_not_move_control_and_queues_nothing() {
        let mut host = host();
        let controlled = host.runtime.world().controlled_id();
        let hash = host.runtime.state_hash();
        let at = host.runtime.world().position();

        assert!(host.try_dev_key(&Key::Character("n".into())));
        let followed = host
            .follow
            .expect("N takes a target off the controlled one");
        assert_ne!(Some(followed), controlled, "and it is somebody else");

        assert_eq!(
            host.runtime.world().controlled_id(),
            controlled,
            "control did not move"
        );
        assert_eq!(host.runtime.queued_len(), 0, "nothing was queued");
        assert_eq!(host.runtime.state_hash(), hash, "the world did not move");
        assert_eq!(host.runtime.world().position(), at);
        assert!(
            host.runtime.trace().is_empty(),
            "and nothing reached a trace"
        );

        // The camera did move, which is the whole of what the keys do.
        let centre = host.follow_at();
        let followed_at = host
            .runtime
            .world()
            .organisms
            .iter()
            .find(|o| o.id == followed)
            .expect("the target is in the roster")
            .position;
        assert_eq!(centre, followed_at);
    }

    /// The snap-back key returns the camera to the played body, and nothing
    /// else changes.
    #[test]
    fn the_snap_back_key_returns_the_camera_to_the_controlled_critter() {
        let mut host = host();
        assert!(host.try_dev_key(&Key::Character("n".into())));
        assert!(host.follow.is_some());
        assert!(host.try_dev_key(&Key::Character("m".into())));
        assert_eq!(host.follow, None);
        assert_eq!(host.followed(), host.runtime.world().controlled_id());
        assert_eq!(host.follow_at(), host.runtime.world().position().unwrap());
        assert_eq!(host.runtime.queued_len(), 0);
    }

    /// Cycling walks the living roster in id order, wraps at both ends, and
    /// never lands on the dead.
    #[test]
    fn cycling_wraps_in_id_order_and_skips_the_dead() {
        let world = World::new(0x00A7_7AC4, 8);
        let living: Vec<OrganismId> = world.living().map(|o| o.id).collect();
        assert!(living.len() > 2, "the fixture needs a roster to walk");
        assert!(
            living.windows(2).all(|pair| pair[0] < pair[1]),
            "the living roster is already in id order"
        );

        // Forward from each, all the way round and back to where it started.
        let mut at = living[0];
        for expected in living.iter().skip(1).chain(living.first()) {
            at = next_living(&world, Some(at), Step::Forward).expect("a living roster answers");
            assert_eq!(at, *expected);
        }
        assert_eq!(at, living[0], "forward wraps to the first");

        // And backward, the other way round: from the first id, one step back
        // wraps to the last and walks down to where it started.
        let mut at = living[0];
        for expected in living.iter().rev() {
            at = next_living(&world, Some(at), Step::Back).expect("a living roster answers");
            assert_eq!(at, *expected);
        }
        assert_eq!(at, living[0], "back wraps all the way round");

        // The dead are not in `World::living`, so they are never landed on —
        // and an id no longer alive still finds its neighbours rather than
        // stranding the camera.
        let dead = OrganismId(u32::MAX);
        assert_eq!(
            next_living(&world, Some(dead), Step::Forward),
            living.first().copied(),
            "past the end wraps to the first living id"
        );
        assert_eq!(
            next_living(&world, Some(dead), Step::Back),
            living.last().copied()
        );
    }

    /// A followed critter that stops being alive is reported with the record's
    /// own tick and reason, and follow snaps back.
    #[test]
    fn a_followed_critter_that_dies_is_reported_and_follow_snaps_back() {
        let mut host = host();
        // A target that is not in the roster at all stands in for one that has
        // been eaten to nothing: the roster cannot answer, and the report has
        // to come from somewhere else or be dropped.
        host.follow = Some(OrganismId(9_999));
        host.update_follow();
        assert_eq!(host.follow, None, "follow snapped back");
        let lost = host
            .follow_lost
            .expect("the loss was reported, not dropped");
        assert_eq!(lost.id, 9_999);
        assert_eq!(lost.tick, host.runtime.world().tick);

        // And the tile carries it: the reading is built with the notice on it.
        let (follow, notice) = host.follow_reading();
        assert!(follow.is_some(), "the camera is back on the played body");
        assert_eq!(notice, Some(lost));

        // Following somebody else clears it — it has been read.
        assert!(host.try_dev_key(&Key::Character("n".into())));
        assert_eq!(host.follow_lost, None);
    }

    /// **A dev kill reads on the tile exactly as a natural death does** (DT3).
    ///
    /// The DT2 done-condition was that a followed critter that dies is
    /// reported rather than silently dropped; the DT3 claim on top is that the
    /// report cannot tell which death it was. Both are taken here from the
    /// same fixture — one body killed by the `K` key, one left to the ecology —
    /// and the tile's notice is compared word for word.
    #[test]
    fn a_dev_kill_is_reported_on_the_tile_the_way_a_natural_death_is() {
        let mut host = host();
        let target = host
            .runtime
            .world()
            .living()
            .find(|o| Some(o.id) != host.runtime.world().controlled_id())
            .expect("somebody else is alive")
            .id;
        host.follow = Some(target);

        assert!(host.try_dev_key(&Key::Character("k".into())));
        host.runtime.step(1);
        assert!(matches!(
            host.runtime.last_outcomes().first(),
            Some(mesocosm_core::Outcome::Killed { .. })
        ));

        host.update_follow();
        assert_eq!(host.follow, None, "follow snapped back, as it does anyway");
        let killed = host.follow_lost.expect("the death was reported");
        assert_eq!(killed.id, target.0);
        assert_eq!(
            killed.how, "died",
            "the record's own word, and it is the natural death's word"
        );
        let ending = host
            .runtime
            .history()
            .ending(target)
            .expect("a dev kill leaves the ordinary ending in the record");
        assert_eq!(
            killed.tick, ending.tick,
            "the record's tick, not the frame's"
        );

        // Beside it, a death the ecology took: same shape, same words but for
        // the id, and nothing anywhere says a hand was involved.
        let natural: Vec<_> = host
            .runtime
            .world()
            .living()
            .filter(|organism| Some(organism.id) != host.runtime.world().controlled_id())
            .map(|organism| organism.id)
            .collect();
        for _ in 0..4_000 {
            host.runtime.step(1);
            if natural.iter().any(|organism| {
                matches!(
                    host.runtime.history().ending(*organism),
                    Some(mesocosm_core::Ending {
                        how: mesocosm_core::Passing::Died,
                        ..
                    })
                )
            }) {
                break;
            }
        }
        let natural = natural
            .into_iter()
            .find(|organism| {
                matches!(
                    host.runtime.history().ending(*organism),
                    Some(mesocosm_core::Ending {
                        how: mesocosm_core::Passing::Died,
                        ..
                    })
                )
            })
            .expect("an unplayed founder dies and leaves a corpse inside four thousand ticks");
        let natural_ending = host
            .runtime
            .history()
            .ending(natural)
            .expect("the recorded natural death has an ending");
        let natural_lost =
            mesocosm_views::lost_of(natural, Some(natural_ending), host.runtime.world().tick);
        assert_eq!(
            natural_lost.how, killed.how,
            "one word for one kind of thing happening"
        );
        assert_eq!(
            mesocosm_views::lost_words(killed),
            format!("critter {} died at tick {}", target.0, ending.tick),
            "and the tile's sentence is the same sentence"
        );
    }

    /// A world's own death, end to end: a critter that really dies is reported
    /// with the tick and the word the record carries.
    #[test]
    fn a_real_death_under_the_camera_carries_the_records_tick_and_word() {
        let mut host = host();
        let target = host
            .runtime
            .world()
            .living()
            .find(|o| Some(o.id) != host.runtime.world().controlled_id())
            .expect("somebody else is alive")
            .id;
        host.follow = Some(target);

        // Run the enclosure until that body's life ends. Founders age out well
        // inside this, and the loop stops the moment the record has an ending.
        for _ in 0..4_000 {
            host.runtime.step(1);
            if host.runtime.history().ending(target).is_some() {
                break;
            }
        }
        let ending = host
            .runtime
            .history()
            .ending(target)
            .expect("a founder's life ends inside four thousand ticks");

        host.update_follow();
        assert_eq!(host.follow, None);
        let lost = host.follow_lost.expect("the death was reported");
        assert_eq!(lost.tick, ending.tick, "the record's tick, not the frame's");
        assert!(
            ["died", "went back to the ground"].contains(&lost.how),
            "the record's own word: {}",
            lost.how
        );
    }
}
