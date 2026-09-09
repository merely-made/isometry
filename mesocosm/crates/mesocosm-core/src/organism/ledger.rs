// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a body can hold, what it spends, and what it can afford.
//!
//! Split out of `organism.rs` at the 600-line ceiling when PD1b routed mass
//! through the phenotype wrapper — the same split-before-adding move that put
//! `kingdom` in a file of its own.
//!
//! Everything here is a **reading or a movement of one account**: body mass is
//! the sum of the surviving parts and nothing else, which is what PD0's
//! migration bought and what keeps a forty-part critter from costing what a
//! single cell costs. Mass is not allocation: growing and starving change what
//! a body weighs and never where its organs are.

use super::Organism;
use super::ecology;

impl Organism {
    /// The adult mass this body plan describes.
    ///
    /// **Determinate growth** (TD6, ruled 2026-08-29). Income, upkeep and the
    /// reproduction tax all scale as `m^0.75`, so the sign of net growth never
    /// depended on size and no body ever arrived at an adult mass. The ceiling
    /// is derived per part from what the plan already knows — the part's own
    /// voxel volume — and summed, so a longer or bulkier recipe is a larger
    /// adult and gigantism is a lineage strategy rather than an authored
    /// number. It is no longer the stability mechanism (conservation is), so
    /// it can be exactly this simple.
    ///
    /// The way past it is the game's own verb: eating adds *parts*, and every
    /// part brings its own ceiling with it.
    pub fn mass_ceiling_mg(&self) -> u64 {
        self.body()
            .living()
            .map(|part| ecology::part_ceiling_mg(part.half_extent))
            .sum()
    }

    /// How much more matter this body could take in, as substance or reserve.
    ///
    /// What a feeding body should ask for: drawing beyond it would only have
    /// to be handed straight back to the world.
    pub fn intake_room_mg(&self) -> u64 {
        let ceiling = self.mass_ceiling_mg();
        ceiling.saturating_sub(self.biomass_mg()) + ceiling.saturating_sub(self.energy_mg)
    }

    /// Adds substance, to the root part, up to [`Self::mass_ceiling_mg`].
    ///
    /// Growth by feeding thickens what is already there. Gaining a *part* is
    /// incorporation, which is a different act with a different cost.
    ///
    /// Returns what would not fit. Matter is conserved, so a caller has to put
    /// that somewhere real rather than letting it evaporate.
    pub fn gain_mass(&mut self, mg: u64) -> u64 {
        let room = self.mass_ceiling_mg().saturating_sub(self.biomass_mg());
        let kept = mg.min(room);
        // Mass is not allocation: thickening the root moves no organ, so this
        // needs no developmental event and creates no causal record.
        if self.phenotype.gain_root_mass(kept) {
            mg - kept
        } else {
            mg
        }
    }

    /// Removes substance across the living body in stable part order.
    ///
    /// Returns what could not be paid. A body that cannot cover a cost is
    /// starving, and the caller decides what that means.
    pub fn spend_mass(&mut self, mg: u64) -> u64 {
        self.phenotype.spend_mass(mg)
    }

    /// How elaborate this organism is.
    ///
    /// **Derived from anatomy**, like everything else a body decides. Parts
    /// carry more weight than substance, because a creature is complex by
    /// having many things rather than by being large, and each part is
    /// something that has to be fed and connected.
    ///
    /// The provisional `epoch::Lineage` had a rival complexity over a scalar
    /// trait array. It was deleted with its module on 2026-09-04 (phenotype
    /// plan §D4), so this is the only one left and the one the world uses.
    pub fn complexity(&self) -> i32 {
        let parts = self.body().living().count() as i32;
        parts * 4 + (self.biomass_mg() / 500) as i32
    }

    /// What this organism spends per tick simply existing — **and moving**.
    ///
    /// **Scales with what it is carrying**, which is the whole point of
    /// reconciling the ledgers: before this, upkeep was a flat milligram and
    /// a body could grow without limit for free. Growing is now a standing
    /// cost, which is what gives *burn or grow* a downside to weigh.
    ///
    /// TD7 adds the second half: rent prices what a body *does*, not only what
    /// it weighs. The three numbers here are all the body plan's own —
    /// [`Self::biomass_mg`], [`Self::actuator_span`], and
    /// [`Self::mass_ceiling_mg`] — so the trophic asymmetry between a plant and
    /// an animal is anatomy rather than an authored constant. See
    /// [`ecology::upkeep_for_body`].
    ///
    /// PD2's third term is the same kind of number: the toxin this body's
    /// allocation holds. A gland is standing machinery, so carrying one costs
    /// every tick whether or not anything bites — which is what makes
    /// installing one a decision rather than a free upgrade.
    pub fn upkeep_mg(&self) -> u64 {
        ecology::upkeep_for_body(
            self.biomass_mg(),
            self.actuator_span(),
            self.mass_ceiling_mg(),
            self.phenotype.secretory_mg(),
        )
    }

    /// Whether the budget holds fewer than `ticks` ticks of upkeep.
    ///
    /// **The one shape every hunger question in the game takes.** Hunger is
    /// never a milligram count — a large body burns through the same number
    /// faster — so it is asked in the only unit that means the same thing to
    /// every body: how long this one could go on doing nothing. The callers
    /// pick their own horizon and say why: the ecology wanders at
    /// `HUNGRY_UPKEEP_TICKS` (nearly out), and a meal routes at
    /// `STARVED_UPKEEP_TICKS` (out soon enough to matter).
    pub fn budget_below(&self, ticks: u64) -> bool {
        self.energy_mg < self.upkeep_mg().saturating_mul(ticks)
    }

    /// Pays one tick of upkeep: from the budget first, then from the body.
    ///
    /// Eating to burn buys survival directly; when there is nothing left to
    /// spend, a creature consumes itself.
    ///
    /// Reports which account each milligram came out of, because the flow record
    /// has to say so: rent paid out of a reserve and rent paid out of a body are
    /// the same transfer to the ground and different transfers out of the
    /// creature. `unpaid_mg` is what it still could not cover, which is
    /// starvation.
    pub fn pay_upkeep(&mut self) -> Upkeep {
        let owed = self.upkeep_mg();
        let reserve_mg = self.energy_mg.min(owed);
        self.energy_mg -= reserve_mg;
        let unpaid_mg = self.spend_mass(owed - reserve_mg);
        Upkeep {
            reserve_mg,
            substance_mg: owed - reserve_mg - unpaid_mg,
            unpaid_mg,
        }
    }
}

/// One tick of rent, and which account it came out of.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Upkeep {
    pub reserve_mg: u64,
    pub substance_mg: u64,
    /// What the body could not cover. This is starvation.
    pub unpaid_mg: u64,
}
