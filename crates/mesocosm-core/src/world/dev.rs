// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The four world-changing dev intents. (DT3)
//!
//! **None of these has its own physics.** That is the dev tools plan's stop
//! rule and it is the whole design of this file: every one of the four is a
//! validator in front of a transaction that already existed and is already
//! reached by something else.
//!
//! | Intent | The transaction it invokes | Also reached by |
//! | --- | --- | --- |
//! | `EndEpoch` | the boundary block in [`World::apply`] | a spent `Timed` budget |
//! | `ForceBirth` | [`ecology::bear`] | the tick's birth pass |
//! | `Kill` | [`ecology::perish`] | starving, and ageing out |
//! | `PlaceMatter` | `Soil::deposit` plus a recorded transfer | every return to the ground |
//!
//! So there is nothing here that decides what a birth costs, what a corpse
//! weighs, or when a round runs. What is here is the refusals — because a dev
//! tool that failed silently would be worse than no tool — and the one thing
//! that is genuinely new: the account matter enters the enclosure through.
//!
//! # They are ordinary intents in every other respect
//!
//! Applied through [`World::apply`] like every other, recorded in the trace,
//! refused by name into the same `Outcome::Rejected`, and conserving matter to
//! the milligram. A run that used one replays to the same hash as the run that
//! recorded it, which is what makes the receipt's "assisted" label a claim
//! about the *run* rather than about the simulation's honesty.
//!
//! # What is not here
//!
//! No `Event` variant of its own. A forced birth writes the ordinary
//! `Event::Born` and a dev kill the ordinary `Event::Died`, from inside the
//! shared transaction — one record, one writer, the arrangement
//! `Outcome::Revised` already uses. Nothing about the record says a hand was
//! involved, which is the point: the DT2 tile, the succession lane and the
//! flow reconciliation all read a dev-caused death exactly as they read a
//! natural one.
//!
//! [`ecology::bear`]: crate::organism::ecology::bear
//! [`ecology::perish`]: crate::organism::ecology::perish

use crate::flow::{FlowEvent, Records};
use crate::organism::{OrganismId, ecology};

use super::{Outcome, Rejection, World};

/// The most matter one [`Intent::PlaceMatter`] may put into a column. (DT3)
///
/// Ten thousand milligrams: a hundred columns' worth of what genesis seeds the
/// ground with (100 mg each), or ten founding bodies. Wide enough that a dev
/// placement is a thing you can see happen in the terrarium, and narrow enough
/// that one keystroke cannot restock an enclosure. The bound is per intent
/// rather than per run, so what a session placed is read off the receipt's dev
/// count and the flow record rather than enforced by a quota nobody named.
///
/// [`Intent::PlaceMatter`]: super::Intent::PlaceMatter
pub const PLACE_MATTER_MAX_MG: u64 = 10_000;

impl World {
    /// Whether this world's epoch rule lets a hand close the epoch now.
    ///
    /// The rule and its reasoning are
    /// [`EpochRule::admits_demand`](crate::rules::EpochRule::admits_demand);
    /// what happens when it says yes is [`World::apply`]'s boundary block,
    /// unchanged.
    pub(super) fn demand_epoch_end(&self) -> Outcome {
        if !self.rules.epoch.admits_demand() {
            return Outcome::Rejected(Rejection::EpochNotOnDemand(self.rules.epoch));
        }
        Outcome::EpochEnded { epoch: self.epoch }
    }

    /// Bears an offspring from one body now, through the ordinary birth.
    ///
    /// The three refusals are the three ways there is no birth to have: nobody
    /// by that id, a body that is not alive, and a parent that cannot provision
    /// its line's recipe out of a quarter of itself. The last is
    /// [`ecology::bear`]'s own gate and not a second one written here — it is
    /// exactly what a natural birth waits on.
    ///
    /// [`ecology::bear`]: crate::organism::ecology::bear
    pub(super) fn force_birth(&mut self, organism: OrganismId) -> Outcome {
        let Some(index) = self.organisms.iter().position(|o| o.id == organism) else {
            return Outcome::Rejected(Rejection::NoSuchOrganism(organism));
        };
        if !self.organisms[index].is_alive() {
            return Outcome::Rejected(Rejection::NotLiving(organism));
        }

        let mut records = Records::new(
            self.tick,
            Some(&self.places),
            &mut self.pending,
            &mut self.flows,
        );
        let born = ecology::bear(
            &mut self.organisms,
            index,
            &mut self.next_organism,
            &mut self.rng,
            &mut records,
            &self.lineages,
            self.development_palette,
            Some(&self.ground),
        );
        match born {
            Some(child) => {
                let offspring = child.id;
                // It joins the roster where the tick's own newborns do; see
                // the field's note on `World`.
                self.forced_birth = Some(child);
                Outcome::Bore {
                    parent: organism,
                    offspring,
                }
            },
            None => Outcome::Rejected(Rejection::InsufficientMass),
        }
    }

    /// Ends one body's life now, through the ordinary death.
    ///
    /// Killing the critter under the hand is allowed: control ends with the
    /// life it was attached to, in [`World::apply`]'s own `control_lost` check,
    /// exactly as it does when the ecology takes that body instead.
    pub(super) fn kill(&mut self, organism: OrganismId) -> Outcome {
        let Some(index) = self.organisms.iter().position(|o| o.id == organism) else {
            return Outcome::Rejected(Rejection::NoSuchOrganism(organism));
        };
        if !self.organisms[index].is_alive() {
            return Outcome::Rejected(Rejection::NotLiving(organism));
        }
        // Read before the death, because releasing it is what the death does.
        let reserve_mg = self.organisms[index].energy_mg;

        let mut records = Records::new(
            self.tick,
            Some(&self.places),
            &mut self.pending,
            &mut self.flows,
        );
        ecology::perish(&mut self.organisms[index], &mut self.soil, &mut records);
        Outcome::Killed {
            organism,
            substance_mg: self.organisms[index].biomass_mg(),
            reserve_mg,
        }
    }

    /// Puts matter into the ground at a cell, out of the dev source.
    ///
    /// **The one place the enclosure's total changes**, and it says so in the
    /// stream: the transfer names [`Account::Dev`] as its source, so the soil's
    /// gain is claimed and a conservation check subtracts what that account
    /// issued rather than tolerating an unexplained rise.
    ///
    /// Off the grid is refused rather than clamped. `Soil::column_at` clamps as
    /// insurance against a leak at the wall; a dev intent that leaned on it
    /// would quietly pile every mistyped coordinate into one edge column.
    ///
    /// [`Account::Dev`]: crate::flow::Account::Dev
    pub(super) fn place_matter(&mut self, at: [i32; 3], mass_mg: u64) -> Outcome {
        let extent = self.soil.extent();
        if !(-extent..=extent).contains(&at[0]) || !(-extent..=extent).contains(&at[2]) {
            return Outcome::Rejected(Rejection::OffGrid(at));
        }
        if mass_mg > PLACE_MATTER_MAX_MG {
            return Outcome::Rejected(Rejection::OverBound {
                mass_mg,
                max_mg: PLACE_MATTER_MAX_MG,
            });
        }
        // Nothing to place. `Intent::Deposit`'s own answer to a zero, and for
        // its reason: a transfer of nothing is not a transaction.
        if mass_mg == 0 {
            return Outcome::Rejected(Rejection::InsufficientMass);
        }
        let column = self.soil.column_at(at);
        self.soil.deposit(column, mass_mg);
        self.flow(at, FlowEvent::placed(mass_mg));
        Outcome::Placed { at, mass_mg }
    }
}

#[cfg(test)]
mod tests;
