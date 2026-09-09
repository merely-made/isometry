// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Presentation identity for a part that was actually drawn as a voxel body.

use mesocosm_core::{OrganismId, PartId, World};
use mesocosm_mesh::BodyDependencyRevision;

use super::Section;

/// A part address carried by the last successful voxel-body projection.
///
/// The revision makes a selection expire when attachment geometry changes.
/// It is presentation state, never a world address or trace input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BodySelection {
    pub organism: OrganismId,
    pub part: PartId,
    pub revision: BodyDependencyRevision,
}

impl Section {
    /// An isolated body view reuses the resident body renderer and depth target.
    /// The host restores ordinary scene rendering when its menu closes.
    pub fn set_body_preview(&mut self, isolated: bool, depth: f32) {
        self.bodies.isolated = isolated;
        self.bodies.preview_depth = if depth.is_finite() && depth > 0.0 {
            depth
        } else {
            super::SLAB_DEPTH
        };
    }

    /// Walks parts in the last successful voxel-body draw for `subject`.
    /// Capsule fallbacks deliberately contribute no selectable identity.
    pub fn select_part(
        &self,
        subject: OrganismId,
        current: Option<BodySelection>,
        backwards: bool,
    ) -> Option<BodySelection> {
        if self.body_mode != super::BodyMode::Voxels {
            return None;
        }
        self.bodies.select_part(subject, current, backwards)
    }

    /// Confirms that a selected part remains both drawable and geometrically
    /// current before the host presents it again.
    pub fn validate_selection(
        &mut self,
        selection: BodySelection,
        world: &World,
        volumes: &mesocosm_mesh::VolumeMap,
    ) -> bool {
        if self.body_mode != super::BodyMode::Voxels {
            return false;
        }
        self.bodies.validate_selection(selection, world, volumes)
    }

    /// Sets host-owned inspection emphasis for the next body draw.
    pub fn set_body_focus(&mut self, subject: Option<OrganismId>, selected: Option<BodySelection>) {
        self.bodies.set_focus(subject, selected);
    }
}
