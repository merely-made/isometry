// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Committing a lineage revision. (P4)
//!
//! **One transaction, two doors.** [`Intent::Revise`](super::Intent) is the
//! played one and this is what it calls; an unplayed lineage reaches
//! [`World::revise`] directly, so the two take the identical path and neither
//! can be a second way for a program to move. Nothing here edits a revision:
//! [`Program::commit`](crate::program::Program::commit) appends, which is the
//! epoch-boundary plan §2 rule made structural.
//!
//! # What a revision is built from
//!
//! The discovery the line came to, and nothing a host said. A [`Candidate`]
//! already names *which admitted process, on what shape, at what bounded
//! capacity* — the three rule-bearing fields a declared site holds — so
//! committing needs no second vocabulary and a host cannot smuggle a cell
//! address or a price through the door.
//!
//! [`Candidate`]: crate::discovery::Candidate

use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;
use crate::discovery::ConditionId;
use crate::flow::Envelope;
use crate::history::Event;
use crate::organism::OrganismId;
use crate::program::{Citation, DeclaredSite, RevisionId};

use super::World;

/// Why a lineage revision was not committed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unrevised {
    /// This world has never heard of that line.
    NoSuchSpecies(SpeciesId),
    /// The line has not come to that condition, so there is nothing to commit.
    Undiscovered(ConditionId),
    /// Not at this point in the run.
    ///
    /// **Reachable since PE3**: bodies change between epochs and not during
    /// them, so a revision is admitted only while the world is standing at its
    /// lineage checkpoint. Every other tick refuses this.
    ///
    /// [`Miss::AnotherTook`]: crate::discovery::Miss::AnotherTook
    NotYet,
    /// The revision would express nothing, so a descendant would be born
    /// exactly as its parents were.
    ///
    /// Today the one way to reach it: this world's ruleset does not hold the
    /// definition the discovery cites, so every descendant's development would
    /// refuse [`UnknownProcess`](crate::Refusal::UnknownProcess) forever.
    /// Refusing at the commit is the honest place — a program that can never
    /// be expressed is not a program.
    Nothing,
}

impl World {
    /// Whether a lineage revision may be committed on this tick.
    ///
    /// **The lineage checkpoint, and nothing else** (PE3). Reproduction is the
    /// checkpoint at the scale of an individual and the epoch boundary is the
    /// one at the scale of a lineage; a program revision belongs to the second
    /// and must not be reachable from the first, or from the middle of a tick
    /// of ordinary play. The placeholder that answered yes at every tick is
    /// gone: this is [`World::at_boundary`], which the epoch rule sets and the
    /// next tick that is not itself a revision clears.
    ///
    /// Two consumers, and both matter: the commit below refuses
    /// [`Unrevised::NotYet`] when this says no, and a host reads it to know
    /// whether the verb is on offer at all.
    pub fn revision_admitted_now(&self) -> bool {
        self.at_boundary()
    }

    /// Commits a revision on a lineage's development program.
    ///
    /// **The world transaction, and the unplayed door.** Since PE3a an
    /// unplayed lineage does take one, at the lineage checkpoint, through
    /// [`World::adapt_round`](crate::World) — and it reaches this function and
    /// not a second one. What an unplayed line may *consider* is still
    /// playable ecology plan §6 ruling 5 and still open at the acquisition
    /// end: it weighs what this world has already come to, and nothing here
    /// discovers anything.
    pub fn revise(
        &mut self,
        species: SpeciesId,
        condition: ConditionId,
    ) -> Result<RevisionId, Unrevised> {
        self.revise_by(species, condition, None)
    }

    /// The same transaction, naming the hand that was on the line.
    pub(super) fn revise_by(
        &mut self,
        species: SpeciesId,
        condition: ConditionId,
        by: Option<OrganismId>,
    ) -> Result<RevisionId, Unrevised> {
        if !self.revision_admitted_now() {
            return Err(Unrevised::NotYet);
        }
        if self.lineages.get(species).is_none() {
            return Err(Unrevised::NoSuchSpecies(species));
        }
        let Some(discovery) = self
            .discoveries
            .iter()
            .find(|discovery| discovery.condition == condition)
            .copied()
        else {
            return Err(Unrevised::Undiscovered(condition));
        };
        // A definition this world did not admit cannot become a program. The
        // same refusal `BodyPhenotype::develop` would give every descendant,
        // asked once, at the commit.
        if self
            .ruleset()
            .resolve(discovery.candidate.process)
            .is_none()
            || discovery.candidate.cells == 0
        {
            return Err(Unrevised::Nothing);
        }

        let sites = vec![DeclaredSite::of(&discovery.candidate)];
        let tick = self.tick;
        let revision = self
            .lineages
            .get_mut(species)
            .expect("checked above")
            .commit(Citation::of(&discovery), sites, tick);

        // The record, at the commit point, for both doors. `records::event_for`
        // is the played door's writer and would leave an unplayed revision
        // unrecorded, so the transaction writes its own — the arrangement
        // `World::observe` already uses for a landed discovery.
        let place = self.acted_at(by);
        self.pending.push(Envelope::new(
            tick,
            place,
            Event::Revised {
                species,
                revision,
                condition,
                by,
            },
        ));
        Ok(revision)
    }
}
