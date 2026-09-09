// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-only time control (DT1): pause, speed, and the manual step keys.
//!
//! Split out of `app.rs` at the 600-line ceiling. Everything here is pacing
//! over [`mesocosm_runtime::Runtime::advance`] and `::step`: nothing reaches
//! `Runtime::queue`, so nothing here can enter the trace or move a replay's
//! hash — the dev tools plan's second principle. See `crate::dev` for the
//! panel this state is read into, and `crate::input::dev_key` for the keys
//! that call in here.

use super::Host;
use crate::input;

/// The speed ladder the `[`/`]` dev keys move through, paired with the word
/// the dev lane shows for each rung. A multiplier over the elapsed
/// microseconds the host was already going to pass to `Runtime::advance` —
/// it never reaches the runtime itself, so it cannot move a replay's hash.
const DEV_SPEED_LADDER: [(f64, &str); 5] = [
    (0.25, "1/4"),
    (0.5, "1/2"),
    (1.0, "1"),
    (2.0, "2"),
    (4.0, "4"),
];

/// Where the ladder starts: ordinary speed.
pub(super) const DEV_SPEED_DEFAULT_IDX: usize = 2;

impl Host {
    /// Presentation-only terrarium quarter turns. They are handled before
    /// inspection and dev/world keys so Z/V always remain camera controls.
    pub(super) fn try_camera_key(&mut self, key: &winit::keyboard::Key) -> bool {
        let backwards = match key {
            winit::keyboard::Key::Character(c) if matches!(c.as_str(), "z" | "Z") => true,
            winit::keyboard::Key::Character(c) if matches!(c.as_str(), "v" | "V") => false,
            _ => return false,
        };
        if !self.config.camera.is_terrarium() {
            return true;
        }
        self.config.camera = self.config.camera.quarter_turn(backwards);
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.section.set_mode(self.config.camera);
        }
        true
    }

    /// Applies pause and speed to a played frame's elapsed time. `advance`
    /// calls this every frame, dev build or not: outside `--dev` it is the
    /// identity, so an ordinary build pays one branch and nothing else.
    ///
    /// **Pause drops the elapsed time rather than banking it** — the same
    /// "do not bank it" rule a checkpoint hold already uses inside
    /// `Runtime::advance` — and speed scales it before the clock ever sees
    /// it. `Runtime::advance` takes a `u64` of microseconds and does not know
    /// or care why this frame's was zero, or four times the wall clock's.
    pub(super) fn dev_paced_elapsed(&self, elapsed_us: u64) -> u64 {
        if !self.config.dev {
            elapsed_us
        } else if self.dev_paused {
            0
        } else {
            let (multiplier, _) = DEV_SPEED_LADDER[self.dev_speed_idx];
            ((elapsed_us as f64) * multiplier) as u64
        }
    }

    /// The whole of the key handler's dev interception: off outside
    /// `--dev`, and `true` (having already applied the action) for one of
    /// the twelve keys `--dev` makes live. Kept off unless the flag is set, so
    /// an ordinary build's keyboard is exactly what it was before DT1.
    ///
    /// The three follow keys are handed to [`super::follow`] and DT3's four to
    /// [`super::devworld`]. The split is the dev tools plan's principle 2:
    /// nothing above the last arm reaches `Runtime::queue`, and everything in
    /// it does nothing but.
    pub(super) fn try_dev_key(&mut self, key: &winit::keyboard::Key) -> bool {
        if !self.config.dev {
            return false;
        }
        let Some(action) = input::dev_key(key) else {
            return false;
        };
        match action {
            input::DevKey::TogglePause => self.dev_paused = !self.dev_paused,
            input::DevKey::Step => self.dev_step(1),
            input::DevKey::StepN => self.dev_step(super::DEV_STEP_N),
            input::DevKey::SlowDown => self.dev_speed_idx = self.dev_speed_idx.saturating_sub(1),
            input::DevKey::SpeedUp => {
                self.dev_speed_idx = (self.dev_speed_idx + 1).min(DEV_SPEED_LADDER.len() - 1);
            },
            // DT2: the camera's centre, and nothing else.
            input::DevKey::FollowNext | input::DevKey::FollowBack | input::DevKey::FollowSelf => {
                self.follow_key(action)
            },
            // DT3: an ordinary intent, queued. The only arm here that can
            // reach the world at all.
            action if action.changes_the_world() => self.dev_world_key(action),
            _ => {},
        }
        true
    }

    /// The step and step-N dev keys both land here: off the clock entirely,
    /// exactly [`mesocosm_runtime::Runtime::step`]'s own contract — `n`
    /// unless a checkpoint holds it, then fewer.
    fn dev_step(&mut self, n: u64) {
        let taken = self.runtime.step(n);
        self.steps += taken;
        self.dev_manual_steps += taken;
    }

    /// The dev lane's reading, taken fresh each frame. `None` when `--dev` is
    /// off, which is also when nothing calls into `crate::dev` at all.
    ///
    /// The time half is this module's; the follow half is `super::follow`'s,
    /// and every line of it comes back out of a core query.
    pub(super) fn dev_reading(&self) -> Option<mesocosm_views::Dev> {
        if !self.config.dev {
            return None;
        }
        let (follow, lost) = self.follow_reading();
        Some(mesocosm_views::Dev {
            running: !self.dev_paused,
            speed: DEV_SPEED_LADDER[self.dev_speed_idx].1,
            tick: self.steps,
            manual_steps: self.dev_manual_steps,
            follow,
            lost,
            inspection: self.inspection.open.then(|| {
                self.inspection.selected.map_or_else(
                    || mesocosm_views::PartInspection {
                        reading: None,
                        notice: Some(self.inspection.notice.into()),
                    },
                    |selected| {
                        mesocosm_views::part_of(
                            self.runtime.world(),
                            selected.organism,
                            selected.part,
                            self.runtime.history(),
                        )
                    },
                )
            }),
        })
    }
}

/// Refreshes and composites the dev lane, doing nothing when `dev` is
/// `None` — which is every frame `--dev` is off, since [`Host::dev_reading`]
/// is what produces `Some`. A free function rather than a method: by the
/// point `frame` calls this, `lanes` is already a reborrow out of
/// `self.gpu`, so a method taking `&self` alongside it would be a second,
/// conflicting borrow.
pub(super) fn composite_dev_lane(
    lanes: &mut super::Lanes,
    dev: Option<&mesocosm_views::Dev>,
    encoder: &mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    frame: (u32, u32),
) {
    let Some(dev) = dev else { return };
    lanes.dev.refresh(&lanes.device, dev);
    lanes.dev.composite(&lanes.device, encoder, view, frame);
}
