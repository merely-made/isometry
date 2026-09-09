// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The deed log: who did what, when, to or for whom. Append-only.
//!
//! Nothing in this crate stores a relationship as an authored number. Standing
//! is folded out of this log (see [`crate::relation`]), so an answer can always
//! name the deeds it rests on, and a deed the player never saw can never move
//! anyone.
//!
//! Deeds are ordinary history and agreement transitions alike. A standing
//! agreement being formed, exercised, renegotiated, or ended is a thing someone
//! did to someone, so it lands here with everything else.

use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::agreement::AgreementId;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct DeedId(pub u64);

/// What was done. Small on purpose: a vocabulary a player can hold in their
/// head is a vocabulary they can use to predict an answer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeedKind {
    /// Stayed when leaving was the safe choice.
    StoodBy,
    /// Left when staying was the promise.
    Abandoned,
    /// Gave away something scarce.
    Shared,
    /// Took what was not offered.
    Took,
    OfferAccepted,
    OfferRefused,
    OfferCountered,
    AgreementFormed(AgreementId),
    PerformedUnderAgreement(AgreementId),
    AgreementRenegotiated(AgreementId),
    AgreementEnded(AgreementId),
}

impl DeedKind {
    /// What this is worth to the person it was done to or for, as trust and
    /// affinity. The doer acts; the recipient's standing toward the doer moves.
    ///
    /// [`DeedKind::OfferRefused`] weighs nothing in either column, and that is
    /// a ruling rather than an oversight: refusal is a legitimate answer here,
    /// so it costs the refuser nothing.
    pub fn weight(&self) -> (i16, i16) {
        match self {
            Self::StoodBy => (3, 2),
            Self::Abandoned => (-5, -3),
            Self::Shared => (1, 2),
            Self::Took => (-2, -2),
            Self::OfferAccepted => (1, 0),
            Self::OfferRefused => (0, 0),
            Self::OfferCountered => (0, 0),
            Self::AgreementFormed(_) => (1, 1),
            Self::PerformedUnderAgreement(_) => (2, 1),
            Self::AgreementRenegotiated(_) => (0, 0),
            Self::AgreementEnded(_) => (0, -1),
        }
    }

    /// How the recipient would put it. The legibility surface is built from
    /// these, so they are written from the second person outward.
    pub fn phrase(&self) -> &'static str {
        match self {
            Self::StoodBy => "stood by me",
            Self::Abandoned => "left me",
            Self::Shared => "shared with me",
            Self::Took => "took from me",
            Self::OfferAccepted => "took up my offer",
            Self::OfferRefused => "turned my offer down",
            Self::OfferCountered => "answered my offer with terms of their own",
            Self::AgreementFormed(_) => "agreed a standing arrangement with me",
            Self::PerformedUnderAgreement(_) => "did the agreed work",
            Self::AgreementRenegotiated(_) => "changed our terms",
            Self::AgreementEnded(_) => "ended our arrangement",
        }
    }
}

/// One entry. `toward` is optional because some deeds are done in the world
/// rather than to a person, and those move nobody's standing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Deed {
    pub id: DeedId,
    pub at: Tick,
    pub doer: SubjectId,
    pub toward: Option<SubjectId>,
    pub kind: DeedKind,
}

/// Append-only by construction: there is no removal, no edit, and no
/// interior mutation of an entry once recorded.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeedLog {
    deeds: Vec<Deed>,
    next: u64,
}

impl DeedLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(
        &mut self,
        at: Tick,
        doer: SubjectId,
        toward: Option<SubjectId>,
        kind: DeedKind,
    ) -> DeedId {
        let id = DeedId(self.next);
        self.next += 1;
        self.deeds.push(Deed {
            id,
            at,
            doer,
            toward,
            kind,
        });
        id
    }

    pub fn deeds(&self) -> &[Deed] {
        &self.deeds
    }

    pub fn get(&self, id: DeedId) -> Option<&Deed> {
        self.deeds.iter().find(|deed| deed.id == id)
    }

    pub fn len(&self) -> usize {
        self.deeds.len()
    }

    pub fn is_empty(&self) -> bool {
        self.deeds.is_empty()
    }

    /// Everything `toward` did to or for `holder`, oldest first. The evidence
    /// behind one person's read of another.
    pub fn done_to(&self, holder: SubjectId, by: SubjectId) -> Vec<&Deed> {
        self.deeds
            .iter()
            .filter(|deed| deed.doer == by && deed.toward == Some(holder))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_refusal_costs_the_refuser_nothing() {
        assert_eq!(DeedKind::OfferRefused.weight(), (0, 0));
    }

    #[test]
    fn the_log_only_grows() {
        let mut log = DeedLog::new();
        let first = log.record(Tick(1), SubjectId(1), Some(SubjectId(2)), DeedKind::StoodBy);
        let second = log.record(Tick(2), SubjectId(1), None, DeedKind::Took);
        assert_ne!(first, second);
        assert_eq!(log.len(), 2);
        assert_eq!(log.get(first).unwrap().kind, DeedKind::StoodBy);
        assert_eq!(log.done_to(SubjectId(2), SubjectId(1)).len(), 1);
    }
}
