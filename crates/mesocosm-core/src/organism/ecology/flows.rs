// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where the tick's matter goes, and the record it leaves.
//!
//! Split out of `ecology.rs` on 2026-09-01 when PE0's flow record pushed that
//! file at the six-hundred-line ceiling — the same split-before-adding move that
//! put `breeding`, `movement` and `rates` in files of their own. What lives here
//! is the two routing decisions every income and every death goes through, now
//! that each of them also has to say what it did.

use crate::flow::{Account, FlowEvent, Process, Records, Subject};
use crate::places::Soil;

use super::{Organism, STARVED_UPKEEP_TICKS};

/// Where a body put what it just took in.
///
/// The three destinations sum to what was offered, which is what makes the
/// ledger reconcile: the caller has somewhere real to put every milligram, and
/// nothing is left to evaporate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Landed {
    pub reserve_mg: u64,
    pub substance_mg: u64,
    /// What the body could not hold. Callers clamp their draw to
    /// [`Organism::intake_room_mg`], so in practice this is zero; it is returned
    /// anyway because a leak here is a leak nothing else would catch.
    pub spilled_mg: u64,
}

/// Feeding income, routed by the body — **the same rule for every kingdom**.
///
/// TD4 gave the played meal one question: is this body inside
/// [`STARVED_UPKEEP_TICKS`] of empty? If so the meal burns, refilling the
/// budget; if not it builds. TD5 asks it of every organism instead, because
/// before this every non-played gain built biomass only and `energy_mg` was a
/// birth endowment that never refilled — so an NPC crossed every hunger
/// threshold within its first few hundred ticks and lived off its own body
/// thereafter, and a decomposer could never bank a corpse against the gap to
/// the next one.
///
/// Called after [`Organism::pay_upkeep`], so the reserve it reads is this
/// tick's, post-rent. TD6's ceilings bound both halves — substance at the body
/// plan's adult mass, reserve at the same number.
pub(super) fn earn(organism: &mut Organism, mg: u64) -> Landed {
    let ceiling = organism.mass_ceiling_mg();
    if organism.budget_below(STARVED_UPKEEP_TICKS) {
        let reserve_mg = mg.min(ceiling.saturating_sub(organism.energy_mg));
        organism.energy_mg += reserve_mg;
        let spilled_mg = organism.gain_mass(mg - reserve_mg);
        Landed {
            reserve_mg,
            substance_mg: mg - reserve_mg - spilled_mg,
            spilled_mg,
        }
    } else {
        let over = organism.gain_mass(mg);
        let reserve_mg = over.min(ceiling.saturating_sub(organism.energy_mg));
        organism.energy_mg += reserve_mg;
        Landed {
            reserve_mg,
            substance_mg: mg - over,
            spilled_mg: over - reserve_mg,
        }
    }
}

/// Records where a body put what it just earned.
///
/// One call for the two ways matter arrives — out of the ground, or out of
/// another body — because [`earn`] routes both by the same rule, and a ledger
/// that split them would have to agree with that rule twice.
///
/// The spill is deliberately not recorded here. A producer's overflow goes back
/// into the soil it was drawn from, so it never left that account and a record
/// would be a transfer to itself; a meal's overflow *did* leave a body, and the
/// caller that knows whose records it.
pub(super) fn record_intake(
    records: &mut Records<'_>,
    at: [i32; 3],
    from: Option<Subject>,
    to: Subject,
    landed: &Landed,
) {
    for (into, mg) in [
        (Account::Reserve, landed.reserve_mg),
        (Account::Substance, landed.substance_mg),
    ] {
        let flow = match from {
            Some(from) => {
                FlowEvent::between(Process::Feeding, from, Account::Substance, to, into, mg)
            },
            None => FlowEvent::uptake(to, into, mg),
        };
        records.flow(at, flow);
    }
}

/// Travel, paid in substance into the ground it was covered over.
///
/// **The trail, not a sink** (TD6): what a step costs comes out of the walker
/// and lands in the column it started from, so a body that wandered the
/// enclosure has left its cost lying behind it. Charged at the origin because
/// that is where the ground was covered.
pub(super) fn pay_travel(
    organism: &mut Organism,
    soil: &mut Soil,
    records: &mut Records<'_>,
    from: [i32; 3],
    distance: u64,
) {
    let owed = distance.max(1);
    let unpaid = organism.spend_mass(owed);
    let column = soil.column_at(from);
    soil.deposit(column, owed - unpaid);
    records.flow(
        from,
        FlowEvent::returned(
            Process::Travel,
            Subject::of(organism),
            Account::Substance,
            owed - unpaid,
        ),
    );
}

/// A body's remains go back to the ground where it lies: what it was still
/// carrying as reserve, released the moment it stops being able to hold it.
///
/// Substance is returned separately and slowly, by [`Stage::Carrion`] decay
/// and by whatever eats the corpse.
///
/// [`Stage::Carrion`]: crate::organism::Stage::Carrion
/// **One death, and the only one there is.**
///
/// Split out of the tick's own life-history pass for DT3, which needs a body's
/// life ended *now* and must not have a second death written for it: the dev
/// tools plan's stop rule is that a dev-caused death reads as a natural one.
/// The tick calls this when a body starves or ages out; `Intent::Kill` calls it
/// when a hand asks. Neither can tell afterwards which it was, because the
/// corpse and the record carry nothing that says.
///
/// What it does is exactly what dying is here: the body becomes carrion holding
/// the substance it had, its reserve goes back into the column under it, the
/// record gets [`Event::Died`], and its gestation clock is cleared. Counting
/// the death is the caller's, because only the tick keeps a [`Tally`].
///
/// [`Event::Died`]: crate::history::Event::Died
/// [`Tally`]: crate::organism::Tally
pub fn perish(organism: &mut Organism, soil: &mut Soil, records: &mut Records<'_>) {
    organism.stage = crate::organism::Stage::Carrion;
    release_reserve(organism, soil, records);
    records.event(
        organism.position,
        crate::history::Event::Died {
            organism: organism.id,
            species: organism.species,
        },
    );
    organism.since_offspring = 0;
}

pub(super) fn release_reserve(organism: &mut Organism, soil: &mut Soil, records: &mut Records<'_>) {
    let column = soil.column_at(organism.position);
    soil.deposit(column, organism.energy_mg);
    records.flow(
        organism.position,
        FlowEvent::returned(
            Process::Death,
            Subject::of(organism),
            Account::Reserve,
            organism.energy_mg,
        ),
    );
    organism.energy_mg = 0;
}
