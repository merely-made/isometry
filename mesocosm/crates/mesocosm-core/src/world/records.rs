// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What an accepted act writes down.
//!
//! Split out of `act.rs` at the 600-line ceiling when PE0 gave every act a
//! second record to write. Resolving an intent and recording what it did are
//! two jobs, and only the first belongs next to the rules.
//!
//! **One commit point, both records.** A refused intent reaches nothing here,
//! which is the whole of the guarantee that accepted and refused transactions
//! cannot disagree with the stream.

use crate::flow::{Account, FlowEvent, Process, Subject};
use crate::organism::{Organism, OrganismId};

use super::{Outcome, World};

/// The event an outcome amounts to, if any.
///
/// Refusals produce nothing: a history records what happened, and a rejected
/// intent is a thing that did not. The same rule governs the flow stream, and it
/// is the same rule for the same reason — an accepted transaction emits its
/// records at this commit point, a refused one emits neither a resource movement
/// nor a false ecological consequence.
pub(super) fn event_for(
    outcome: &Outcome,
    actor: Option<OrganismId>,
) -> Option<crate::history::Event> {
    use crate::history::Event;
    match *outcome {
        // Burning names the meal, growing names the grower. Both need the
        // actor, because a history is keyed by who a thing happened to and an
        // event citing nobody would fork that creature's line.
        Outcome::Burned { energy_mg, .. } => Some(Event::Burned {
            organism: actor?,
            energy_mg,
        }),
        // Growing is growing, however the part arrived: PE2's organ off a
        // carcass is the same biographical fact as a meal that built. Where it
        // *came from* is on the part's own provenance, which is a durable
        // record rather than an event.
        Outcome::Incorporated { part }
        | Outcome::IncorporatedPair { part, .. }
        | Outcome::Consumed { part, .. } => Some(Event::Grew {
            organism: actor?,
            part,
        }),
        // A branch transfer is not growing: it names both bodies, because the
        // loss and the acquisition are one fact. (P3)
        Outcome::Grafted {
            root,
            parts,
            from,
            from_part,
            ..
        } => Some(Event::Grafted {
            organism: actor?,
            from,
            part: root,
            from_part,
            parts,
        }),
        Outcome::Inhabited { organism } => Some(Event::Inhabited { organism }),
        // Carving air did not happen to anyone; only removed matter is
        // biographical.
        Outcome::Carved { at, removed } if removed > 0 => Some(Event::Carved {
            organism: actor?,
            at,
            removed,
        }),
        Outcome::Carved { .. } => None,
        // Rebuilding an organ is the third way a body becomes different, after
        // growing and losing one, and it is as biographical as either. (PD2)
        Outcome::Expressed { part, cost_mg, .. } => Some(Event::Expressed {
            organism: actor?,
            part,
            cost_mg,
        }),
        Outcome::Speciated {
            species,
            from,
            founder,
        } => Some(Event::Speciated {
            species,
            from,
            founder,
        }),
        // A committed revision writes its own event, inside the transaction
        // that made it, because an unplayed lineage takes that transaction and
        // never reaches this function. One record, one writer. (P4)
        Outcome::Revised { .. } => None,
        // DT3's forced birth and dev kill write their own too, and for exactly
        // the same reason twice over: the transaction is the ecology's, the
        // ecology reaches it without a hand, and the record it leaves must not
        // be able to say which door asked. A `Born` or a `Died` written here
        // as well would be the same fact twice, and a *different* event would
        // be the tell that makes a dev death read as something else.
        Outcome::Bore { .. } | Outcome::Killed { .. } => None,
        // Ending an epoch did not happen to a creature, and matter entering
        // the ground did not either. What the boundary came to is the world
        // record's and `last_round`'s; where the matter went is the flow
        // stream's.
        Outcome::EpochEnded { .. } | Outcome::Placed { .. } => None,
        // Resuming is a decision, and the ordered trace is where decisions are
        // kept. A history records what happened to a creature, and carrying on
        // unchanged did not happen to one.
        Outcome::Moved
        | Outcome::Deposited { .. }
        | Outcome::Idled
        | Outcome::Resumed
        | Outcome::Rejected(_) => None,
    }
}

/// Where a landed meal's mass went: into the budget, or into the body.
///
/// The two are not interchangeable and the difference is the whole verb, but
/// the closed cycle needs one more thing from them than the outcome says —
/// their **sum**, because whatever the meal weighed and neither of them took
/// has to go back into the world. (TD6)
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Landed {
    pub budget_mg: u64,
    pub body_mg: u64,
}

impl World {
    /// Records a matter movement an accepted intent caused, where it happened.
    pub(super) fn flow(&mut self, position: [i32; 3], flow: FlowEvent) {
        let place = self.places.at(position);
        self.flows.record(place, flow);
    }

    /// The meal's ledger: every milligram the transaction moved, out of the
    /// account it left and into the one it reached.
    ///
    /// Five records because the meal has five destinations, and the sum of them
    /// is exactly the eaten body — substance and reserve both — so nothing it
    /// weighed can be visible to the state while absent from the stream.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn record_meal(
        &mut self,
        eaten: &Organism,
        meal: Subject,
        eater: Subject,
        eater_at: [i32; 3],
        landed: &Landed,
        unkept: u64,
        spilled: u64,
    ) {
        for (into, mg) in [
            (Account::Reserve, landed.budget_mg),
            (Account::Substance, landed.body_mg),
        ] {
            self.flow(
                eater_at,
                FlowEvent::between(Process::Feeding, meal, Account::Substance, eater, into, mg),
            );
        }
        let at = eaten.position;
        // What the eater did not keep, what the meal was still carrying, and
        // what bringing up a toxin cost. All three land in the ground, and the
        // last one comes out of the eater rather than the eaten.
        self.flow(
            at,
            FlowEvent::returned(Process::Spill, meal, Account::Substance, unkept),
        );
        self.flow(
            at,
            FlowEvent::returned(Process::Death, meal, Account::Reserve, eaten.energy_mg),
        );
        self.flow(
            at,
            FlowEvent::returned(Process::Spill, eater, Account::Reserve, spilled),
        );
    }
}
