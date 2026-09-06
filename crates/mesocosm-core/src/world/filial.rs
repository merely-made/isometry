// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a birth does with its line's program. (PD5)
//!
//! The ecology's birth pass realizes the recipe and seeds allocation from
//! geometry — **the founding revision**, and what every birth in this world
//! does until a line commits one. This is the tick's other half: a descendant
//! of a line that has committed a revision is developed under it, here, before
//! anything reads the world.
//!
//! # Through the one validator, and paid for
//!
//! Nothing here arranges tissue.
//! [`program::express`](crate::program::express) builds the same
//! [`AllocationProposal`](crate::AllocationProposal) `candidate_proposal`
//! builds and offers it to
//! [`BodyPhenotype::develop`](crate::BodyPhenotype::develop), so a descendant
//! expressing its program and a player expressing a discovery reach one
//! developmental authority. The price is PD2's, unchanged: cells whose
//! expression changed, at the part's own cell price, out of the child's reserve
//! and into the column under it. Rebuilding an organ is work, and work in this
//! world is matter moving somewhere else. (TD6)
//!
//! # A refusal is a named fact
//!
//! A child body with no part of the declared shape, or too little banked to pay
//! for one, is **born anyway** — under geometry seeding — and the record says
//! which revision it could not express and why. That is PE2's residue one
//! generation down: a candidate that cannot be taken is the ordinary case, and
//! a silent fallback would make an inherited program unfalsifiable.
//!
//! # Unplayed lineages take this path
//!
//! Nothing below reads [`World::controlled`](crate::World::controlled). The
//! pass runs over every organism the tick's birth pass allocated an id for, so
//! a descendant of an NPC line is developed by the identical code.

use crate::flow::{Account, FlowEvent, Subject};
use crate::history::Event;
use crate::organism::OrganismId;
use crate::program::{self, Conditions};

use super::World;

impl World {
    /// Develops this tick's newborns under their lines' current revisions.
    ///
    /// `first_new` is the organism counter as it stood before the ecology
    /// stepped. Ids are allocated only by the birth pass and never reused, so
    /// everything at or above it is a body that did not exist a moment ago —
    /// an exact answer rather than a scan for something that looks new.
    pub(super) fn express_filially(&mut self, first_new: u32) {
        // Most ticks in a terrarium bear nobody. The counter not having moved
        // is exactly "no id was allocated", so this is the same answer the
        // filter below would give and costs one comparison instead of a walk
        // over the whole roster on every tick of every run.
        if self.next_organism == first_new {
            return;
        }
        let born: Vec<OrganismId> = self
            .organisms
            .iter()
            .filter(|organism| organism.id.0 >= first_new)
            .map(|organism| organism.id)
            .collect();
        for organism in born {
            self.express_born(organism);
        }
    }

    fn express_born(&mut self, id: OrganismId) {
        // The shared handle first, so it outlives the mutable borrow of the
        // roster below. **This world's admitted ruleset** (PD4), never a
        // global one.
        let ruleset = self.admitted();
        let Some(child) = self.organisms.iter().find(|organism| organism.id == id) else {
            return;
        };
        let (species, position, held_mg) = (child.species, child.position, child.energy_mg);
        let phenotype = child.phenotype.clone();

        let Some(lineage) = self.lineages.get(species) else {
            return;
        };
        // The founding revision: nothing declared, nothing to express, and no
        // record — a birth under it is an ordinary birth and always was.
        let Some(revision) = lineage.program().current() else {
            return;
        };
        let revision_id = revision.id;
        let column = self.soil.column_at(position);
        let conditions = Conditions {
            ground_mg: self.soil.matter_mg(column),
            material_mg: held_mg,
        };

        let outcome = program::express(revision, &ruleset, &phenotype, conditions);
        let place = self.acted_at(Some(id));
        let tick = self.tick;

        match outcome {
            Ok((expressed, filial)) => {
                let Some(child) = self.organisms.iter_mut().find(|o| o.id == id) else {
                    return;
                };
                let subject = Subject::of(child);
                child.phenotype = expressed;
                child.energy_mg -= filial.cost_mg;

                self.soil.deposit(column, filial.cost_mg);
                self.flow(
                    position,
                    FlowEvent::returned(
                        crate::flow::Process::Develop,
                        subject,
                        Account::Reserve,
                        filial.cost_mg,
                    ),
                );
                self.pending.push(crate::flow::Envelope::new(
                    tick,
                    place,
                    Event::Inherited {
                        organism: id,
                        species,
                        revision: revision_id,
                        part: filial.part,
                        cost_mg: filial.cost_mg,
                    },
                ));
            },
            Err(why) => self.pending.push(crate::flow::Envelope::new(
                tick,
                place,
                Event::Unexpressed {
                    organism: id,
                    species,
                    revision: revision_id,
                    why,
                },
            )),
        }
    }
}
