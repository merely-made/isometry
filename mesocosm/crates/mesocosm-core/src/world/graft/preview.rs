// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A graft's checked candidate, before the world publishes it.

use crate::body::{Attachment, PartId, SpeciesId};
use crate::graft::{Crossing, Verdict};
use crate::organism::OrganismId;
use crate::phenotype::BodyPhenotype;

use super::{Rejection, World};

/// What a graft would publish if it were applied against this unchanged world.
///
/// The candidate phenotype is the exact body validated by the graft gate.
/// Reading it has no effect on the world's hash, random stream, flows, or
/// donor anatomy. A later intent always prepares again, so an intervening
/// world change cannot commit this value stale.
#[derive(Clone, Debug)]
pub struct GraftPreview {
    pub recipient: OrganismId,
    pub donor: OrganismId,
    pub donor_line: SpeciesId,
    pub donor_part: PartId,
    pub crossing: Crossing,
    pub verdict: Verdict,
    /// The body-plan attachment selected for the branch root.
    pub attachment: Attachment,
    /// The exact checked recipient phenotype, including the arriving branch.
    pub phenotype: BodyPhenotype,
    pub root: PartId,
    pub parts: Vec<PartId>,
    pub mass_mg: u64,
    pub cost_mg: u64,
    pub revision: u32,
}

impl World {
    /// Reachable carcass branches a player may inspect for grafting.
    ///
    /// This is intentionally cheaper than [`Self::preview_graft`]: it only
    /// filters the stable, present eligibility a menu needs. The chosen entry
    /// still goes through the complete preparation gate before it can land.
    pub fn graft_sources(&self) -> Vec<(OrganismId, PartId)> {
        let Some(here) = self.position() else {
            return Vec::new();
        };
        let mut sources = Vec::new();
        for donor in self
            .organisms
            .iter()
            .filter(|organism| !organism.is_alive() && Some(organism.id) != self.controlled_id())
        {
            if self.reach_to(donor.position).is_err() {
                continue;
            }
            let distance = (0..3)
                .map(|axis| (donor.position[axis] - here[axis]).abs())
                .max()
                .expect("three axes");
            let body = donor.body();
            for part in body
                .living()
                .filter(|part| part.id != body.root && part.mass_mg > 0)
            {
                sources.push((distance, donor.id, part.id));
            }
        }
        sources.sort_unstable();
        sources
            .into_iter()
            .map(|(_, donor, part)| (donor, part))
            .collect()
    }

    /// Checks a branch transfer without changing this world.
    pub fn preview_graft(
        &self,
        donor: OrganismId,
        part: PartId,
        crossing: Crossing,
    ) -> Result<GraftPreview, Rejection> {
        self.prepare_graft(donor, part, crossing)
            .map(|prepared| prepared.preview)
    }
}
