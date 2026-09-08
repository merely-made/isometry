// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The host's configuration: every flag `main.rs` can set.
//!
//! Split out of `app.rs` at the 600-line ceiling.

use std::path::PathBuf;

use crate::played::{PlayedTrace, SceneMode};
use crate::section::{self, CameraMode};

#[derive(Clone, Debug)]
pub struct HostConfig {
    /// Open the local starting-life picker before the ecology clock begins.
    /// Preview changes are disposable until the player confirms a candidate.
    pub creator_request: Option<mesocosm_core::world::generation::Request>,
    /// Explicit destination for saving creator criteria, separate from play.
    pub creator_draft: Option<PathBuf>,
    pub start: Option<mesocosm_core::world::generation::Selection>,
    pub seed: u64,
    pub organisms: u32,
    pub ticks_per_second: u32,
    pub width: u32,
    pub height: u32,
    /// Run this many frames and exit. Makes the windowed path verifiable
    /// without a person sitting in front of it.
    pub frames: Option<u32>,
    /// Write the last frame here before exiting.
    pub capture: Option<PathBuf>,
    /// Write the session's intent trace here before exiting. Skipped on a
    /// replay, whose trace is an input rather than a result.
    pub trace: Option<PathBuf>,
    /// Write the run's receipt here before exiting.
    pub receipt: Option<PathBuf>,
    /// Drive the run from this trace instead of the keyboard. The self-driving
    /// receipt: exactly one recorded intent per fixed step, then a hash
    /// assertion against what the trace recorded.
    pub replay: Option<PlayedTrace>,
    /// The scenario text driving this run, when `--scenario` gave it one. (DT4)
    ///
    /// The text rather than a parsed [`genet_probe::Scenario`], because a
    /// config is cloned and compared and a parsed scenario is neither. `Host::new`
    /// parses it, so a typo stops the run before a window opens.
    pub scenario: Option<String>,
    /// Half the height of the section's orthographic slab, in voxels — how much
    /// world the terrarium view frames.
    ///
    /// The default is ruled ([`section::SLAB_HALF_HEIGHT`], 28 since
    /// 2026-08-29); the knob stays so every framing remains reproducible from
    /// the tree. Presentation only: it never reaches an intent, so it cannot
    /// move a replay hash.
    pub slab_half_height: f32,
    /// Which way the section looks (DC4, Q9). `oblique` is the shipped
    /// section and the default, ruled 2026-09-04; `side` looks straight down
    /// `-z` and draws bodies end-on, and `across` turns a quarter so they
    /// chain across the view. The two level arms stay because they are the
    /// measurement the ruling rests on.
    ///
    /// **Presentation only, exactly like `slab_half_height` beside it.** It
    /// picks rays, not rules: no intent, no snapshot field the world reads,
    /// no state hash. The measured slice replays one golden trace under all
    /// three and asserts the same hash from each.
    pub camera: CameraMode,
    pub camera_explicit: bool,
    pub terrarium_pitch: f32,
    pub cutaway: section::Cutaway,
    /// Presentation preset; Auto selects the habitat look only for CP1.
    pub terrain_style: section::TerrainStyle,
    /// Voxel anatomy or the legacy capsule comparison, presentation only.
    pub body_mode: section::BodyMode,
    pub body_budget: usize,
    /// New worlds admit generated voxel content. Replays use their saved pack.
    pub generated_content: bool,
    /// New-world recipe set. A replay always uses its recorded choice.
    pub body_layout: crate::played::BodyLayout,
    pub scene: SceneMode,
    /// Off by default (DT1). On, the dev lane draws and its keys go live;
    /// recorded in the receipt either way.
    pub dev: bool,
    /// Which critter the section starts centred on (DT2).
    ///
    /// **Only where the camera starts.** `N`, `B` and `M` move it from here
    /// like any other frame, and a target that is not alive is reported and
    /// dropped on the first frame. It exists so an unattended `--frames` run
    /// can be captured with the follow target off the played body, which is
    /// what DT2's headed receipt has to show; nothing about it reaches an
    /// intent, so it cannot move a replay hash.
    pub follow: Option<u32>,
}

impl HostConfig {
    pub fn effective_start(&self) -> Option<&mesocosm_core::world::generation::Selection> {
        match &self.replay {
            Some(trace) => trace.start.as_ref(),
            None => self.start.as_ref(),
        }
    }
    pub fn effective_body_layout(&self) -> crate::played::BodyLayout {
        self.replay
            .as_ref()
            .map_or(self.body_layout, |trace| trace.body_layout)
    }

    pub fn effective_scene(&self) -> SceneMode {
        self.replay.as_ref().map_or(self.scene, |trace| trace.scene)
    }
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            creator_request: None,
            creator_draft: None,
            start: None,
            seed: 0x00A7_7AC4,
            // The world's own area-scaled cohort, not a literal: S1 tied the
            // founding population to the enclosure's floor area so a wider
            // terrarium is bigger rather than emptier.
            organisms: mesocosm_core::world::FOUNDERS,
            // The canonical played tempo (TD2, ruled 2026-08-29). Sixty was
            // never chosen; it was the frame rate, and driving the ecology's
            // tick-tuned life history at it mapped a whole lifetime onto
            // seventeen seconds. Ten gives 100ms input granularity and puts a
            // starter's life at about five minutes. Headless labs and the
            // population instrument keep their own rates.
            ticks_per_second: 10,
            width: 960,
            height: 540,
            frames: None,
            capture: None,
            trace: None,
            receipt: None,
            replay: None,
            scenario: None,
            slab_half_height: section::SLAB_HALF_HEIGHT,
            camera: CameraMode::default(),
            camera_explicit: false,
            terrarium_pitch: section::TERRARIUM_DEGREES,
            cutaway: section::Cutaway::Occupied,
            terrain_style: section::TerrainStyle::Auto,
            body_mode: section::BodyMode::default(),
            body_budget: section::DEFAULT_BODY_BUDGET,
            generated_content: true,
            body_layout: crate::played::BodyLayout::Spaced,
            scene: SceneMode::default(),
            dev: false,
            follow: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::played::{BodyLayout, PlayedTrace, SceneMode};

    #[test]
    fn old_trace_without_layout_stays_axial() {
        let trace: PlayedTrace = serde_json::from_str(
            r#"{"seed":1,"organisms":1,"steps":0,"state_hash":0,"intents":[]}"#,
        )
        .unwrap();
        assert_eq!(trace.body_layout, BodyLayout::Axial);
    }

    #[test]
    fn saved_jointed_layout_overrides_new_spaced_default() {
        let trace: PlayedTrace = serde_json::from_str(
            r#"{"body_layout":"jointed","seed":1,"organisms":1,"steps":0,"state_hash":0,"intents":[]}"#,
        )
        .unwrap();
        let config = HostConfig {
            replay: Some(trace),
            ..HostConfig::default()
        };
        assert_eq!(config.body_layout, BodyLayout::Spaced);
        assert_eq!(config.effective_body_layout(), BodyLayout::Jointed);
    }

    #[test]
    fn old_trace_defaults_ecology_and_saved_scene_wins() {
        let old: PlayedTrace = serde_json::from_str(
            r#"{"seed":1,"organisms":1,"steps":0,"state_hash":0,"intents":[]}"#,
        )
        .unwrap();
        assert_eq!(old.scene, SceneMode::Ecology);
        let saved: PlayedTrace = serde_json::from_str(
            r#"{"scene":"terrarium","seed":1,"organisms":1,"steps":0,"state_hash":0,"intents":[]}"#,
        )
        .unwrap();
        let config = HostConfig {
            scene: SceneMode::Ecology,
            replay: Some(saved),
            ..HostConfig::default()
        };
        assert_eq!(config.effective_scene(), SceneMode::Terrarium);
    }
}
