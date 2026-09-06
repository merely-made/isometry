// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An append-only, observer-scoped record of claims about accepted deeds.
//!
//! [`DeedLog`] owns objective events. This record can only observe an existing
//! deed at or after its tick. Entry order, including within one tick, is
//! authoritative and named by [`EpistemicId`].

use std::collections::BTreeSet;

use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::{DeedId, DeedLog};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct EpistemicId(pub u64);
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ObservationId(pub u64);
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ClaimId(pub u64);
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ReportId(pub u64);

/// Support can be a direct sighting or an addressed transmission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Evidence {
    Observation(ObservationId),
    Report(ReportId),
}

/// The deliberately small F3a vocabulary for a reading of one deed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Proposition {
    pub event: DeedId,
    pub reading: EventReading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventReading {
    Helped,
    Betrayed,
}

/// A claimant's current derived reading and its pointable revision entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Belief {
    pub holder: SubjectId,
    pub claim: ClaimId,
    pub revision: EpistemicId,
    pub proposition: Proposition,
    pub supports: Vec<Evidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpistemicEntry {
    pub id: EpistemicId,
    pub at: Tick,
    pub record: EpistemicRecord,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EpistemicRecord {
    Observation {
        observation: ObservationId,
        observer: SubjectId,
        event: DeedId,
    },
    Claim {
        claim: ClaimId,
        claimant: SubjectId,
        proposition: Proposition,
        supports: Vec<Evidence>,
    },
    Report {
        report: ReportId,
        reporter: SubjectId,
        hearer: SubjectId,
        claim: ClaimId,
        proposition: Proposition,
    },
    Correction {
        corrector: SubjectId,
        corrects: ClaimId,
        replacement: Proposition,
        supports: Vec<Evidence>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimHistory {
    pub claim: ClaimId,
    pub reports: Vec<ReportId>,
    pub corrections: Vec<EpistemicId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpistemicError {
    UnknownEvent(DeedId),
    ObservationBeforeEvent {
        event: DeedId,
        deed_at: Tick,
        observation_at: Tick,
    },
    OutOfOrder {
        previous: Tick,
        next: Tick,
    },
    MissingSupport,
    ForeignEvidence(Evidence),
    MismatchedSupport(Evidence),
    UnknownObservation(ObservationId),
    UnknownReport(ReportId),
    UnknownClaim(ClaimId),
    CannotReport {
        reporter: SubjectId,
        claim: ClaimId,
        proposition: Proposition,
    },
    NotClaimant {
        corrector: SubjectId,
        claim: ClaimId,
    },
    CorrectionChangesEvent {
        claim: ClaimId,
        expected: DeedId,
        found: DeedId,
    },
    UnchangedCorrection(ClaimId),
    ReplayEntryId {
        expected: EpistemicId,
        found: EpistemicId,
    },
    ReplayObservationId {
        expected: ObservationId,
        found: ObservationId,
    },
    ReplayClaimId {
        expected: ClaimId,
        found: ClaimId,
    },
    ReplayReportId {
        expected: ReportId,
        found: ReportId,
    },
    ReplayRecordMismatch(EpistemicId),
}

/// Deterministic admitted history. It intentionally does not implement
/// `Deserialize`: decode `Vec<EpistemicEntry>` and pass it to [`Self::replay`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EpistemicLog {
    entries: Vec<EpistemicEntry>,
    next_entry: u64,
    next_observation: u64,
    next_claim: u64,
    next_report: u64,
}

impl EpistemicLog {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn entries(&self) -> &[EpistemicEntry] {
        &self.entries
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Record that `observer` saw a deed already accepted by `deeds`.
    pub fn observe(
        &mut self,
        deeds: &DeedLog,
        at: Tick,
        observer: SubjectId,
        event: DeedId,
    ) -> Result<ObservationId, EpistemicError> {
        let deed = deeds
            .get(event)
            .ok_or(EpistemicError::UnknownEvent(event))?;
        if at < deed.at {
            return Err(EpistemicError::ObservationBeforeEvent {
                event,
                deed_at: deed.at,
                observation_at: at,
            });
        }
        let observation = ObservationId(self.next_observation);
        self.append(
            at,
            EpistemicRecord::Observation {
                observation,
                observer,
                event,
            },
        )?;
        self.next_observation += 1;
        Ok(observation)
    }

    pub fn claim(
        &mut self,
        at: Tick,
        claimant: SubjectId,
        proposition: Proposition,
        supports: &[Evidence],
    ) -> Result<ClaimId, EpistemicError> {
        let supports = self.supports_for(claimant, proposition, supports)?;
        let claim = ClaimId(self.next_claim);
        self.append(
            at,
            EpistemicRecord::Claim {
                claim,
                claimant,
                proposition,
                supports,
            },
        )?;
        self.next_claim += 1;
        Ok(claim)
    }

    /// Transmit an exact proposition the reporter owns or was previously told.
    pub fn report(
        &mut self,
        at: Tick,
        reporter: SubjectId,
        hearer: SubjectId,
        claim: ClaimId,
        proposition: Proposition,
    ) -> Result<ReportId, EpistemicError> {
        let (claimant, claimed) = self.claim_details(claim)?;
        let owned_current = claimant == reporter && self.current_revision(claim)?.1 == proposition;
        let received = self.entries.iter().any(|entry| matches!(
            &entry.record,
            EpistemicRecord::Report { hearer: recipient, claim: found_claim, proposition: found_proposition, .. }
                if *recipient == reporter && *found_claim == claim && *found_proposition == proposition
        ));
        if proposition.event != claimed.event || !(owned_current || received) {
            return Err(EpistemicError::CannotReport {
                reporter,
                claim,
                proposition,
            });
        }
        let report = ReportId(self.next_report);
        self.append(
            at,
            EpistemicRecord::Report {
                report,
                reporter,
                hearer,
                claim,
                proposition,
            },
        )?;
        self.next_report += 1;
        Ok(report)
    }

    /// Self-revise a claim, retaining its original entry and all prior revisions.
    pub fn correct(
        &mut self,
        at: Tick,
        corrector: SubjectId,
        corrects: ClaimId,
        replacement: Proposition,
        supports: &[Evidence],
    ) -> Result<EpistemicId, EpistemicError> {
        let (claimant, original) = self.claim_details(corrects)?;
        if claimant != corrector {
            return Err(EpistemicError::NotClaimant {
                corrector,
                claim: corrects,
            });
        }
        if replacement.event != original.event {
            return Err(EpistemicError::CorrectionChangesEvent {
                claim: corrects,
                expected: original.event,
                found: replacement.event,
            });
        }
        if self.current_revision(corrects)?.1 == replacement {
            return Err(EpistemicError::UnchangedCorrection(corrects));
        }
        let supports = self.supports_for(corrector, replacement, supports)?;
        self.append(
            at,
            EpistemicRecord::Correction {
                corrector,
                corrects,
                replacement,
                supports,
            },
        )
    }

    pub fn received_by(&self, hearer: SubjectId) -> impl Iterator<Item = &EpistemicEntry> {
        self.entries.iter().filter(move |entry| matches!(&entry.record, EpistemicRecord::Report { hearer: found, .. } if *found == hearer))
    }

    /// Recompute every claim made by `holder`, folding its latest correction.
    pub fn beliefs_of(&self, holder: SubjectId) -> Vec<Belief> {
        self.entries
            .iter()
            .filter_map(|entry| match &entry.record {
                EpistemicRecord::Claim {
                    claim, claimant, ..
                } if *claimant == holder => {
                    let (revision, proposition, supports) = self.current_revision(*claim).ok()?;
                    Some(Belief {
                        holder,
                        claim: *claim,
                        revision,
                        proposition,
                        supports,
                    })
                },
                _ => None,
            })
            .collect()
    }

    pub fn history_of(&self, claim: ClaimId) -> Result<ClaimHistory, EpistemicError> {
        self.claim_entry(claim)?;
        let mut reports = Vec::new();
        let mut corrections = Vec::new();
        for entry in &self.entries {
            match &entry.record {
                EpistemicRecord::Report {
                    report,
                    claim: found,
                    ..
                } if *found == claim => reports.push(*report),
                EpistemicRecord::Correction { corrects, .. } if *corrects == claim => {
                    corrections.push(entry.id)
                },
                _ => {},
            }
        }
        Ok(ClaimHistory {
            claim,
            reports,
            corrections,
        })
    }

    /// Admit durable entries through the same validation as live operations.
    pub fn replay(deeds: &DeedLog, entries: &[EpistemicEntry]) -> Result<Self, EpistemicError> {
        let mut replayed = Self::new();
        for entry in entries {
            let expected = EpistemicId(replayed.next_entry);
            if entry.id != expected {
                return Err(EpistemicError::ReplayEntryId {
                    expected,
                    found: entry.id,
                });
            }
            match &entry.record {
                EpistemicRecord::Observation {
                    observation,
                    observer,
                    event,
                } => {
                    let found = replayed.observe(deeds, entry.at, *observer, *event)?;
                    if found != *observation {
                        return Err(EpistemicError::ReplayObservationId {
                            expected: found,
                            found: *observation,
                        });
                    }
                },
                EpistemicRecord::Claim {
                    claim,
                    claimant,
                    proposition,
                    supports,
                } => {
                    let found = replayed.claim(entry.at, *claimant, *proposition, supports)?;
                    if found != *claim {
                        return Err(EpistemicError::ReplayClaimId {
                            expected: found,
                            found: *claim,
                        });
                    }
                },
                EpistemicRecord::Report {
                    report,
                    reporter,
                    hearer,
                    claim,
                    proposition,
                } => {
                    let found =
                        replayed.report(entry.at, *reporter, *hearer, *claim, *proposition)?;
                    if found != *report {
                        return Err(EpistemicError::ReplayReportId {
                            expected: found,
                            found: *report,
                        });
                    }
                },
                EpistemicRecord::Correction {
                    corrector,
                    corrects,
                    replacement,
                    supports,
                } => {
                    replayed.correct(entry.at, *corrector, *corrects, *replacement, supports)?;
                },
            }
            if replayed.entries.last() != Some(entry) {
                return Err(EpistemicError::ReplayRecordMismatch(entry.id));
            }
        }
        Ok(replayed)
    }

    fn append(&mut self, at: Tick, record: EpistemicRecord) -> Result<EpistemicId, EpistemicError> {
        if let Some(previous) = self.entries.last().map(|entry| entry.at)
            && at < previous
        {
            return Err(EpistemicError::OutOfOrder { previous, next: at });
        }
        let id = EpistemicId(self.next_entry);
        self.entries.push(EpistemicEntry { id, at, record });
        self.next_entry += 1;
        Ok(id)
    }

    fn supports_for(
        &self,
        actor: SubjectId,
        proposition: Proposition,
        supports: &[Evidence],
    ) -> Result<Vec<Evidence>, EpistemicError> {
        if supports.is_empty() {
            return Err(EpistemicError::MissingSupport);
        }
        let supports: Vec<_> = supports
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        for support in &supports {
            match support {
                Evidence::Observation(id) => {
                    let (observer, event) = self.observation_details(*id)?;
                    if observer != actor {
                        return Err(EpistemicError::ForeignEvidence(*support));
                    }
                    if event != proposition.event {
                        return Err(EpistemicError::MismatchedSupport(*support));
                    }
                },
                Evidence::Report(id) => {
                    let (_, hearer, _, reported) = self.report_details(*id)?;
                    if hearer != actor {
                        return Err(EpistemicError::ForeignEvidence(*support));
                    }
                    if reported != proposition {
                        return Err(EpistemicError::MismatchedSupport(*support));
                    }
                },
            }
        }
        Ok(supports)
    }

    fn current_revision(
        &self,
        claim: ClaimId,
    ) -> Result<(EpistemicId, Proposition, Vec<Evidence>), EpistemicError> {
        let entry = self.claim_entry(claim)?;
        let EpistemicRecord::Claim {
            proposition,
            supports,
            ..
        } = &entry.record
        else {
            unreachable!("claim_entry only returns claims")
        };
        let mut current = (entry.id, *proposition, supports.clone());
        for entry in &self.entries {
            if let EpistemicRecord::Correction {
                corrects,
                replacement,
                supports,
                ..
            } = &entry.record
                && *corrects == claim
            {
                current = (entry.id, *replacement, supports.clone());
            }
        }
        Ok(current)
    }

    fn observation_details(
        &self,
        observation: ObservationId,
    ) -> Result<(SubjectId, DeedId), EpistemicError> {
        match &self.observation_entry(observation)?.record {
            EpistemicRecord::Observation {
                observer, event, ..
            } => Ok((*observer, *event)),
            _ => unreachable!("observation_entry only returns observations"),
        }
    }

    fn claim_details(&self, claim: ClaimId) -> Result<(SubjectId, Proposition), EpistemicError> {
        match &self.claim_entry(claim)?.record {
            EpistemicRecord::Claim {
                claimant,
                proposition,
                ..
            } => Ok((*claimant, *proposition)),
            _ => unreachable!("claim_entry only returns claims"),
        }
    }

    fn report_details(
        &self,
        report: ReportId,
    ) -> Result<(SubjectId, SubjectId, ClaimId, Proposition), EpistemicError> {
        match &self.report_entry(report)?.record {
            EpistemicRecord::Report {
                reporter,
                hearer,
                claim,
                proposition,
                ..
            } => Ok((*reporter, *hearer, *claim, *proposition)),
            _ => unreachable!("report_entry only returns reports"),
        }
    }

    fn observation_entry(
        &self,
        observation: ObservationId,
    ) -> Result<&EpistemicEntry, EpistemicError> {
        self.entries.iter().find(|entry| matches!(&entry.record, EpistemicRecord::Observation { observation: found, .. } if *found == observation)).ok_or(EpistemicError::UnknownObservation(observation))
    }

    fn claim_entry(&self, claim: ClaimId) -> Result<&EpistemicEntry, EpistemicError> {
        self.entries.iter().find(|entry| matches!(&entry.record, EpistemicRecord::Claim { claim: found, .. } if *found == claim)).ok_or(EpistemicError::UnknownClaim(claim))
    }

    fn report_entry(&self, report: ReportId) -> Result<&EpistemicEntry, EpistemicError> {
        self.entries.iter().find(|entry| matches!(&entry.record, EpistemicRecord::Report { report: found, .. } if *found == report)).ok_or(EpistemicError::UnknownReport(report))
    }
}
