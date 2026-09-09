// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Taking one organ off a carcass. (PE2)
//!
//! **One bounded proof, and the bound is the point.** The playable ecology plan
//! asks part-level eating to start with a single claim: consuming a corpse part
//! settles *that part's* exact matter and donor evidence, and cannot teach
//! unrelated parts from the donor's recipe. It explicitly does **not** attach a
//! functioning source branch — live subtree transfer is phenotype P3 — and full
//! live dismemberment is not a prerequisite for it.
//!
//! # Why a corpse, and not a severed branch on a living body
//!
//! Both are in the plan's sentence, and only one of them conserves matter
//! today. A severed part is tombstoned: `BodyDocument::total_mass_mg` already
//! skips it, so its milligrams have left the conservation account, and eating
//! one would *create* matter. A corpse's living parts still weigh what they
//! weigh. So this verb takes an unsevered part off something that is no longer
//! alive, and the severed half waits for the dismemberment gate (phenotype D3a)
//! that would put those milligrams somewhere honest in the first place.
//!
//! # It is not the meal verb wearing a part
//!
//! [`Intent::Metabolize`](super::Intent::Metabolize) is still how a body eats
//! for calories, and the body still routes it. This is a deliberate act on a
//! named organ: it always incorporates, and a body with nowhere to put the
//! organ refuses rather than quietly burning it. That is what makes the
//! provenance it writes worth writing — `Origin::Incorporated { from_part }`
//! finally names the part it came off rather than the donor's root.

use crate::body::{Origin, PartId, Provenance};
use crate::discovery::Evidence;
use crate::flow::{Account, FlowEvent, Subject};
use crate::organism::OrganismId;
use crate::plan::classify;

use super::{Outcome, Rejection, World};

impl World {
    /// Takes one part off a carcass, settling exactly what that part weighs.
    pub(super) fn consume(&mut self, organism: OrganismId, part: PartId) -> Outcome {
        if Some(organism) == self.controlled {
            return Outcome::Rejected(Rejection::Itself);
        }
        let Some(index) = self.organisms.iter().position(|o| o.id == organism) else {
            return Outcome::Rejected(Rejection::NoSuchOrganism(organism));
        };
        let donor = &self.organisms[index];
        // A living body's organs are not on offer. Taking one would be live
        // dismemberment, which this proof deliberately does not open.
        if donor.is_alive() {
            return Outcome::Rejected(Rejection::StillLiving(organism));
        }
        if !self
            .controlled()
            .is_some_and(|eater| eater.admits(donor.kingdom().nis_kind(), true))
        {
            return Outcome::Rejected(Rejection::Inedible(organism));
        }
        let Some(found) = donor.body().part(part) else {
            return Outcome::Rejected(Rejection::NoSuchPart(part));
        };
        // Severed tissue weighs nothing in the conservation account, and an
        // emptied part has already been taken.
        if found.severed || found.mass_mg == 0 {
            return Outcome::Rejected(Rejection::NothingLeft(part));
        }
        let (mass_mg, half_extent, volume, at, lineage) = (
            found.mass_mg,
            found.half_extent,
            found.volume,
            donor.position,
            donor.species,
        );
        let carrion = Subject::of(donor);
        if let Err(unmet) = self.reach_to(at) {
            return Outcome::Rejected(Rejection::OutOfReach(unmet));
        }

        // Where it would go, resolved before anything is taken, so a body with
        // nowhere to put an organ refuses without disturbing the corpse.
        let Some(body) = self.body() else {
            return Outcome::Rejected(Rejection::Disembodied);
        };
        let Some(growth) = crate::growth::resolve(body, half_extent) else {
            return Outcome::Rejected(Rejection::NoRoom);
        };
        let Some((eater, eater_at)) = self.controlled().map(|me| (Subject::of(me), me.position))
        else {
            return Outcome::Rejected(Rejection::Disembodied);
        };
        // The rollback point is the whole phenotype, for the reason a meal's
        // is: a restore that put back the anatomy and left the mosaic grown
        // would be exactly the split account the wrapper exists to prevent.
        let before = self
            .controlled()
            .map(|me| me.phenotype.clone())
            .expect("just read");

        // **The field that finally names something.** Every other caller
        // writes the donor's root here; this one writes the organ a player
        // chose.
        let provenance = Provenance {
            origin: Origin::Incorporated {
                from_species: lineage,
                from_part: part,
            },
            epoch: self.epoch,
        };
        // **No mirrored pair.** A meal's mass may split across a bilateral
        // plan; an organ is one organ, and settling *its exact matter* means
        // one part carrying all of it.
        let attached = self.controlled_phenotype_mut().attach(
            volume,
            mass_mg,
            half_extent,
            crate::growth::attachment(&growth),
            provenance,
        );
        let Ok(grown) = attached else {
            *self.controlled_phenotype_mut() = before;
            return Outcome::Rejected(Rejection::NoRoom);
        };
        if self
            .controlled()
            .is_some_and(|me| !me.walker_shape().stands(&self.ground, me.position))
        {
            *self.controlled_phenotype_mut() = before;
            return Outcome::Rejected(Rejection::NoRoom);
        }

        // Only now does the corpse give it up, and only this part's own
        // milligrams: its children keep theirs and stay where they are.
        let taken = self.organisms[index].phenotype.take_part_mass(part);
        debug_assert_eq!(taken, mass_mg, "the part was read a moment ago");
        self.flow(
            eater_at,
            FlowEvent::between(
                crate::flow::Process::Feeding,
                carrion,
                Account::Substance,
                eater,
                Account::Substance,
                taken,
            ),
        );
        // The observation the meal supplied: this organ, off this donor. Not
        // the donor's recipe, and not the rest of its body.
        self.observe(Evidence::Meal {
            donor: lineage,
            part,
            role: classify(half_extent),
            mass_mg: taken,
        });
        Outcome::Consumed {
            part: grown,
            from: organism,
            from_part: part,
            mass_mg: taken,
        }
    }
}
