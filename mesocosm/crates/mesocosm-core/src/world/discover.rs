// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where the world offers evidence, and what it keeps of the answer. (PE2)
//!
//! [`crate::discovery`] owns the rules; this owns the two things only a world
//! can supply — the **authoritative accumulator** a sustained-stress condition
//! reads, and the consequences a landed discovery has on the line.
//!
//! # Played-only, and structurally so
//!
//! Both routes into [`World::observe`] are the played critter's: a meal is an
//! ordered intent somebody sent, and the hunger run only advances while
//! [`World::held`] says a hand is on the body — the same gate PE1's checkpoint
//! uses, for the same reason. An idle terrarium is therefore never asked
//! anything and never discovers anything, which is why the population
//! instrument cannot observe this phase at all.
//!
//! The *evidence* is organism-neutral: nothing in [`crate::discovery::Evidence`]
//! names the player. Whether unplayed lineages acquire through the same rules is
//! open (playable ecology plan §6, ruling 5); what it would need is a per-body
//! accumulator (or a declared cohort reduction for it) and a proposal sink in
//! the ecology's own step, not a second evaluator.

use std::collections::BTreeSet;

use crate::discovery::{ConditionId, Discovery, Evidence, HUNGER_TICKS, Observation, Stress};
use crate::flow::Envelope;
use crate::history::Event;

use super::World;

impl World {
    /// Routes one accepted piece of evidence, and keeps what it came to.
    ///
    /// Called at the point the evidence is accepted — a landed meal, a
    /// threshold crossed — never on a sweep. A discovery that lands takes
    /// three consequences with it: the record, the causal event, and the word
    /// the line may now say.
    pub(super) fn observe(&mut self, evidence: Evidence) {
        let Some((me, lineage)) = self.controlled().map(|o| (o.id, o.species)) else {
            return;
        };
        let known: BTreeSet<ConditionId> = self
            .discoveries
            .iter()
            .map(|discovery| discovery.condition)
            .collect();
        let verdict = crate::discovery::evaluate(evidence, self.tick, self.epoch, &known);

        if let Some(discovery) = verdict.discovery {
            // **Inheritance**, and the only production caller `Recipe::acquire`
            // has ever had. It is the narrowed remains of `learn_from`: a word
            // for the organ that was actually observed, rather than every word
            // the donor's recipe happened to hold.
            if let Some(word) = discovery.candidate.word
                && let Some(species) = self.lineages.get_mut(lineage)
            {
                species.recipe.acquire(word);
            }
            let place = self.acted_at(Some(me));
            self.pending.push(Envelope::new(
                self.tick,
                place,
                Event::Discovered {
                    organism: me,
                    species: lineage,
                    condition: discovery.condition,
                },
            ));
            self.discoveries.push(discovery);
        }
        self.last_observation = Some(verdict.observation);
    }

    /// Advances the one accumulator a stress condition is allowed to read, and
    /// offers its crossing once.
    ///
    /// **Authoritative, bounded, and not a trend.** It is world state, so it is
    /// hashed, snapshotted and replayed with everything else; it is one integer,
    /// so it does not grow with the run; and it is read here rather than from
    /// the driver's presentation windows, which the boundary forbids.
    ///
    /// A hand that lets go neither advances the run nor throws it away — the
    /// ecology is driving the body then, so nobody is enduring anything — while
    /// a body that gets fed, or stops being a body, ends the stress.
    pub(super) fn endure(&mut self) {
        let starved = self.is_starved();
        self.hunger_run = match self.held() {
            Some(_) if starved => self.hunger_run.saturating_add(1),
            _ if !starved => 0,
            _ => self.hunger_run,
        };
        // The crossing is the event. Offering the evidence on every tick past
        // the horizon would be the polling the boundary rules out, and would
        // put a fresh observation in the record for a fact that did not change.
        if u64::from(self.hunger_run) == HUNGER_TICKS {
            self.observe(Evidence::Endured {
                stress: Stress::Hunger,
                ticks: HUNGER_TICKS,
            });
        }
    }

    /// What this line has come to, in the order it came to it.
    ///
    /// Bounded by the condition table: a condition discovers once, and a
    /// second crossing is recorded as an observation rather than a discovery.
    pub fn discoveries(&self) -> &[Discovery] {
        &self.discoveries
    }

    /// Whether this line already holds a condition's candidate.
    pub fn discovered(&self, condition: ConditionId) -> bool {
        self.discoveries
            .iter()
            .any(|discovery| discovery.condition == condition)
    }

    /// The most recent evidence a condition was offered, and what every
    /// condition made of it — including the ones that refused it.
    pub fn last_observation(&self) -> Option<&Observation> {
        self.last_observation.as_ref()
    }

    /// Consecutive ticks a hand has held this body under the starved line.
    pub fn hunger_run(&self) -> u32 {
        self.hunger_run
    }

    /// The proposal a discovered candidate would submit for the played body.
    ///
    /// **The door between discovery and expression.** A discovery grants
    /// availability; this is what availability turns into, and it is an
    /// ordinary [`AllocationProposal`] bound for the one validator. `None`
    /// means the body has nowhere to put it yet — a real state, and the
    /// difference between having the option and being able to take it.
    ///
    /// Until PE3's review exists, a player takes it through
    /// [`Intent::Express`](super::Intent::Express), PD3's bounded door.
    ///
    /// [`AllocationProposal`]: crate::phenotype::AllocationProposal
    pub fn candidate_proposal(
        &self,
        condition: ConditionId,
        source: crate::phenotype::Arrangement,
    ) -> Option<crate::phenotype::AllocationProposal> {
        let discovery = self
            .discoveries
            .iter()
            .find(|discovery| discovery.condition == condition)?;
        discovery
            .candidate
            .propose(&self.controlled()?.phenotype, source)
    }

    /// The intent that would express a discovered candidate on this body.
    ///
    /// The same proposal as [`Self::candidate_proposal`], reduced to the shape
    /// a host sends: the condition, and nothing else. `None` still means the
    /// body has nowhere to put it, which is the difference between having the
    /// option and being able to take it.
    ///
    /// **The whole of what a host may say about a development** since PD3
    /// deleted `Intent::Rearrange`. A host cannot name cells, and cannot name
    /// a definition; it can only ask for what its line already came to.
    pub fn candidate_intent(&self, condition: ConditionId) -> Option<super::Intent> {
        self.candidate_proposal(condition, crate::phenotype::Arrangement::Direct)?;
        Some(super::Intent::Express { condition })
    }
}
