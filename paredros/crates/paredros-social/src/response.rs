// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Answers, and the premises behind them.
//!
//! This is the legibility surface the charter calls day-one work. Every answer
//! this crate can give carries an ordered list of [`Premise`], and a premise is
//! machine-readable rather than a rendered sentence: it names the deed ids, the
//! standing, the confidence read, and the thresholds that actually produced the
//! answer. A test can assert on them, and
//! [`crate::Society::say`] turns each into a line for the player.
//!
//! Refusal is a [`Verdict`], not an error. Nothing in this module can fail.

use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::agreement::{AgreementId, EndReason};
use crate::companion::Craft;
use crate::deed::{DeedId, DeedKind};
use crate::offer::Terms;

/// How an offer was answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Accept,
    /// A first-class outcome. Recorded, weightless, and theirs.
    Refuse,
    /// Yes, on these terms instead.
    Counteroffer(Terms),
}

impl Verdict {
    pub fn accepted(&self) -> bool {
        matches!(self, Self::Accept)
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Accept => "accepts",
            Self::Refuse => "refuses",
            Self::Counteroffer(_) => "counteroffers",
        }
    }
}

/// One step of the reasoning, as data. The list is ordered: evidence, then what
/// the evidence adds up to, then the checks it was put against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Premise {
    /// A deed being weighed, and what it was worth.
    Deed {
        deed: DeedId,
        at: Tick,
        doer: SubjectId,
        kind: DeedKind,
        trust: i16,
        affinity: i16,
    },
    /// What those deeds add up to.
    Standing {
        toward: SubjectId,
        trust: i16,
        affinity: i16,
        from_deeds: Vec<DeedId>,
    },
    /// Their own read of the work.
    Confidence {
        craft: Craft,
        demanded: u8,
        held: u8,
        margin: i16,
    },
    /// The trust this danger asks for, against the trust they hold.
    TrustAsked {
        danger: u8,
        asked: i16,
        held: i16,
        met: bool,
    },
    /// The danger against what they would bear for this person.
    DangerWeighed {
        danger: u8,
        bearable: i16,
        affinity: i16,
        caution: i16,
        borne: bool,
    },
    /// A standing agreement's terms, and whether they cover the work asked.
    AgreementTerm {
        agreement: AgreementId,
        share: u8,
        danger_cap: u8,
        covers: bool,
    },
    /// A change asked of an existing arrangement.
    TermChange {
        agreement: AgreementId,
        from: Terms,
        to: Terms,
    },
    /// Why an arrangement ended.
    Ending {
        agreement: AgreementId,
        why: EndReason,
    },
}

impl Premise {
    /// The deed ids this premise rests on. The explanation surface is only
    /// honest if it points at real entries, and this is what a test checks.
    pub fn deeds(&self) -> Vec<DeedId> {
        match self {
            Self::Deed { deed, .. } => vec![*deed],
            Self::Standing { from_deeds, .. } => from_deeds.clone(),
            _ => Vec::new(),
        }
    }
}

/// Every deed id cited anywhere in a premise list, oldest first, deduplicated.
pub fn cited_deeds(premises: &[Premise]) -> Vec<DeedId> {
    let mut cited: Vec<DeedId> = Vec::new();
    for premise in premises {
        for deed in premise.deeds() {
            if !cited.contains(&deed) {
                cited.push(deed);
            }
        }
    }
    cited
}

/// An answer to an offer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Response {
    pub by: SubjectId,
    pub at: Tick,
    pub verdict: Verdict,
    pub premises: Vec<Premise>,
    /// Answering is itself a deed, so it is in the log like anything else.
    pub recorded: DeedId,
}

impl Response {
    pub fn cited_deeds(&self) -> Vec<DeedId> {
        cited_deeds(&self.premises)
    }
}

/// What happened to a standing agreement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RulingKind {
    Formed(AgreementId),
    /// Asked under the agreement and done, without renegotiation.
    Performed,
    /// Asked under the agreement and declined anyway, because the premises the
    /// agreement rested on have changed. An agreement is not a command.
    Declined,
    /// The work asked is not the work agreed. A fresh offer is the way through.
    OutsideTerms,
    Renegotiated {
        from: Terms,
        to: Terms,
    },
    Ended(EndReason),
}

impl RulingKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Formed(_) => "forms the arrangement",
            Self::Performed => "does the agreed work",
            Self::Declined => "declines, under the arrangement",
            Self::OutsideTerms => "says that is not what we agreed",
            Self::Renegotiated { .. } => "changes the terms",
            Self::Ended(_) => "ends the arrangement",
        }
    }
}

/// An answer about a standing agreement. Same shape as [`Response`]: an
/// outcome and its premises. `recorded` is `None` when nothing changed, since
/// only transitions are deeds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ruling {
    pub by: SubjectId,
    pub at: Tick,
    pub kind: RulingKind,
    pub premises: Vec<Premise>,
    pub recorded: Option<DeedId>,
}

impl Ruling {
    pub fn cited_deeds(&self) -> Vec<DeedId> {
        cited_deeds(&self.premises)
    }
}
