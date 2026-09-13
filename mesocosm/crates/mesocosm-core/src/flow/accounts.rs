// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reducing the flow record into what an account did. (DT2, DT3)
//!
//! Split out of `flow.rs` at the six-hundred-line ceiling, the same
//! split-before-adding move `world/` and `phenotype/` already made, and kept
//! growing there rather than back in `flow.rs` for the same reason.
//!
//! The dev lane's inspector needs what a *body* earned and spent, and
//! [`Score`](crate::Score) already separates exactly that for a *line*. The
//! split is written down once, here, so the two cannot come to disagree about
//! which side of the ledger a transfer is on. [`Account`]'s own two questions —
//! whether an account belongs to a body at all, and what the dev source has
//! issued — are reductions of the same kind and live beside it.

use serde::{Deserialize, Serialize};

use crate::organism::OrganismId;

use super::{Account, Process, RecordedFlow};

impl Account {
    /// Whether this account belongs to a body.
    ///
    /// The soil and the dev source do not: they are the two ends matter enters
    /// and leaves the roster through, and neither names a
    /// [`Subject`](super::Subject).
    pub fn is_body(self) -> bool {
        matches!(self, Self::Substance | Self::Reserve)
    }

    /// What the dev source issued over a stream of flows. (DT3)
    ///
    /// A run's conserved quantity is the enclosure's total *less* this, so a
    /// conservation check subtracts it rather than tolerating it. Written here
    /// so the check and any panel reading the same stream cannot come to
    /// disagree about what counts.
    pub fn issued_mg(flows: &[RecordedFlow]) -> u64 {
        flows
            .iter()
            .filter(|flow| flow.record.source == Account::Dev)
            .map(|flow| flow.record.amount_mg)
            .sum()
    }
}

/// What one body's three accounts did over a window of ticks. (DT2)
///
/// **The same split [`Score`](crate::Score) makes, asked of a body instead of a
/// line.** Income is everything that reached this organism; rent is the flow
/// record's own [`Process::Upkeep`] leaving it; outflow is everything else that
/// left it. The rule lives here, once, so a lineage's score and a body's
/// accounts cannot come to disagree about which side of the ledger a transfer
/// is on.
///
/// Figures with their window on them, never a verdict — [`Self::ticks`] is
/// stated for the reason every other reading states one. Reduced from the flow
/// record, which is `serde(skip)` and outside equality, so nothing here can
/// reach a snapshot or a state hash.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Accounts {
    /// Ticks the figures cover.
    pub ticks: u64,
    /// Matter that reached this body: what it drew out of the ground, what its
    /// mouth took, and what a parent handed it.
    pub income_mg: u64,
    /// Rent: what it spent simply existing.
    pub rent_mg: u64,
    /// Everything else that left it — travel, developing an organ, spill, and
    /// what its death returned to the ground.
    pub outflow_mg: u64,
}

impl Accounts {
    /// Reduces one tick of the flow record for one body, adding it to the
    /// window.
    ///
    /// A tick with nothing in it for this body is still a tick: the window has
    /// to grow whether or not the body did anything, or the figures would be
    /// stated over a length they were not measured across.
    pub fn absorb(&mut self, organism: OrganismId, flows: &[RecordedFlow]) {
        self.ticks = self.ticks.saturating_add(1);
        for flow in flows {
            let record = &flow.record;
            if record.is_internal() {
                continue;
            }
            if record.to.is_some_and(|to| to.organism == organism) {
                self.income_mg = self.income_mg.saturating_add(record.amount_mg);
            }
            if record.from.is_some_and(|from| from.organism == organism) {
                match record.process {
                    Process::Upkeep => {
                        self.rent_mg = self.rent_mg.saturating_add(record.amount_mg);
                    },
                    _ => self.outflow_mg = self.outflow_mg.saturating_add(record.amount_mg),
                }
            }
        }
    }

    /// Income against rent, the one ordering [`Score`](crate::Score) uses.
    pub fn net_mg(self) -> i128 {
        i128::from(self.income_mg) - i128::from(self.rent_mg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::SpeciesId;
    use crate::flow::{Envelope, FlowEvent, Subject};
    use crate::organism::Kingdom;

    const A: OrganismId = OrganismId(1);

    /// The split a body's accounts make, and the window they state. (DT2)
    #[test]
    fn a_bodys_accounts_separate_income_rent_and_everything_else() {
        let me = Subject {
            organism: A,
            lineage: SpeciesId(3),
            kingdom: Kingdom::Consumer,
        };
        let other = Subject {
            organism: OrganismId(2),
            lineage: SpeciesId(3),
            kingdom: Kingdom::Producer,
        };
        let tick = vec![
            // Reached me: income, whichever account it landed in.
            Envelope::new(1, None, FlowEvent::uptake(me, Account::Reserve, 50)),
            Envelope::new(
                1,
                None,
                FlowEvent::between(
                    Process::Feeding,
                    other,
                    Account::Substance,
                    me,
                    Account::Substance,
                    30,
                ),
            ),
            // Left me as rent.
            Envelope::new(
                1,
                None,
                FlowEvent::returned(Process::Upkeep, me, Account::Reserve, 7),
            ),
            // Left me any other way.
            Envelope::new(
                1,
                None,
                FlowEvent::returned(Process::Travel, me, Account::Substance, 4),
            ),
            Envelope::new(
                1,
                None,
                FlowEvent::returned(Process::Develop, me, Account::Reserve, 9),
            ),
            // Nothing to do with me at all.
            Envelope::new(1, None, FlowEvent::uptake(other, Account::Substance, 900)),
            // Digestion within this body changes accounts, not its income.
            Envelope::new(
                1,
                None,
                FlowEvent::between(
                    Process::Uptake,
                    me,
                    Account::Substance,
                    me,
                    Account::Reserve,
                    20,
                ),
            ),
        ];

        let mut accounts = Accounts::default();
        accounts.absorb(A, &tick);
        assert_eq!(accounts.ticks, 1);
        assert_eq!(accounts.income_mg, 80);
        assert_eq!(accounts.rent_mg, 7);
        assert_eq!(accounts.outflow_mg, 13);
        assert_eq!(accounts.net_mg(), 73);

        // A tick with nothing in it still widens the window, or the figures
        // would be stated over a length they were not measured across.
        accounts.absorb(A, &[]);
        assert_eq!(accounts.ticks, 2);
        assert_eq!(accounts.income_mg, 80);
    }
}
