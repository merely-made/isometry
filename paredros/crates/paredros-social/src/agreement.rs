// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Standing agreements, with the whole lifecycle present.
//!
//! The charter's shape: routine work inside a trusted role should be as easy as
//! assigning it, with the social truth that the role was agreed earlier. So an
//! agreement carries the terms, the work it covers, and its own history, and
//! every transition is also a deed in the log.
//!
//! It is still not a command. A holder may decline under a standing agreement
//! when its premises have changed, which is [`crate::response::RulingKind`]'s
//! `Declined`, and the charter names the cases: danger past what was agreed,
//! capability lost, trust collapsed.

use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::deed::DeedId;
use crate::offer::{Terms, Work};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct AgreementId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EndReason {
    /// The asker no longer wants it.
    Withdrawn,
    /// The holder no longer wants it.
    Resigned,
    /// What it was for is done.
    WorkDone,
    /// What it rested on is no longer true.
    PremisesChanged,
}

impl EndReason {
    pub fn phrase(&self) -> &'static str {
        match self {
            Self::Withdrawn => "withdrawn",
            Self::Resigned => "resigned",
            Self::WorkDone => "the work is done",
            Self::PremisesChanged => "what it rested on changed",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgreementState {
    Standing,
    Ended { at: Tick, why: EndReason },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgreementChange {
    Formed,
    Exercised,
    Renegotiated { from: Terms, to: Terms },
    Ended(EndReason),
}

/// One transition, and the deed that recorded it. The deed id is the join
/// between an agreement's own history and the community's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgreementEvent {
    pub at: Tick,
    pub deed: DeedId,
    pub what: AgreementChange,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Agreement {
    pub id: AgreementId,
    pub asker: SubjectId,
    pub holder: SubjectId,
    /// The work it was made for. Craft and grade bound what it covers.
    pub work: Work,
    pub terms: Terms,
    pub formed_at: Tick,
    pub state: AgreementState,
    pub history: Vec<AgreementEvent>,
}

impl Agreement {
    pub fn standing(&self) -> bool {
        matches!(self.state, AgreementState::Standing)
    }

    pub fn party(&self, subject: SubjectId) -> bool {
        subject == self.asker || subject == self.holder
    }

    /// The other name on it.
    pub fn other(&self, subject: SubjectId) -> SubjectId {
        if subject == self.holder {
            self.asker
        } else {
            self.holder
        }
    }

    /// Whether a piece of work falls inside what was agreed: the same craft, no
    /// harder than the agreed grade, no more dangerous than the agreed cap.
    pub fn covers(&self, work: &Work) -> bool {
        work.craft == self.work.craft
            && work.grade <= self.work.grade
            && work.danger <= self.terms.danger_cap
    }

    pub fn append(&mut self, at: Tick, deed: DeedId, what: AgreementChange) {
        self.history.push(AgreementEvent { at, deed, what });
    }

    /// How many times it has been exercised. The measure of a routine.
    pub fn exercises(&self) -> usize {
        self.history
            .iter()
            .filter(|event| event.what == AgreementChange::Exercised)
            .count()
    }
}
