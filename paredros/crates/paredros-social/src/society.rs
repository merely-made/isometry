// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The willingness owner's whole state, and the calls that move it.
//!
//! Ordered containers throughout, integers throughout, no clocks and no
//! randomness: the same society asked the same thing at the same tick gives the
//! same answer with the same premises, which is what the replay receipt checks.
//!
//! Standing is not stored. It is folded out of the deed log on each call, so
//! there is no second copy to fall out of step with the history a player was
//! shown. At this scale the fold is cheaper than the bookkeeping would be.

use std::collections::BTreeMap;

use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::agreement::{Agreement, AgreementChange, AgreementId, AgreementState, EndReason};
use crate::companion::Companion;
use crate::deed::{DeedId, DeedKind, DeedLog};
use crate::offer::{Offer, Terms, Work};
use crate::relation::{Relations, Standing};
use crate::response::{Premise, Response, Ruling, RulingKind, Verdict};
use crate::willing;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SocialError {
    NoSuchCompanion(SubjectId),
    NoSuchAgreement(AgreementId),
    AlreadyEnded(AgreementId),
    /// Someone tried to change an arrangement they are not named on.
    NotAParty(SubjectId),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Society {
    companions: BTreeMap<SubjectId, Companion>,
    log: DeedLog,
    agreements: BTreeMap<AgreementId, Agreement>,
    next_agreement: u64,
}

impl Society {
    pub fn new() -> Self {
        Self::default()
    }

    /// Admits a peer. The played character is admitted the same way as anyone
    /// else, because succession means the distinction is temporary.
    pub fn admit(&mut self, companion: Companion) {
        self.companions.insert(companion.subject, companion);
    }

    pub fn companion(&self, subject: SubjectId) -> Option<&Companion> {
        self.companions.get(&subject)
    }

    pub fn companions(&self) -> impl Iterator<Item = &Companion> {
        self.companions.values()
    }

    pub fn name_of(&self, subject: SubjectId) -> &str {
        self.companions
            .get(&subject)
            .map(|companion| companion.name.as_str())
            .unwrap_or("someone")
    }

    pub fn log(&self) -> &DeedLog {
        &self.log
    }

    /// History happens outside this crate too. Anything that befalls people is
    /// recorded here, and standing follows from it.
    pub fn record(
        &mut self,
        at: Tick,
        doer: SubjectId,
        toward: Option<SubjectId>,
        kind: DeedKind,
    ) -> DeedId {
        self.log.record(at, doer, toward, kind)
    }

    pub fn relations(&self) -> Relations {
        Relations::derive(&self.log)
    }

    pub fn standing(&self, holder: SubjectId, toward: SubjectId) -> Standing {
        self.relations().of(holder, toward)
    }

    pub fn agreement(&self, id: AgreementId) -> Option<&Agreement> {
        self.agreements.get(&id)
    }

    pub fn agreements(&self) -> impl Iterator<Item = &Agreement> {
        self.agreements.values()
    }

    fn known(&self, subject: SubjectId) -> Result<&Companion, SocialError> {
        self.companions
            .get(&subject)
            .ok_or(SocialError::NoSuchCompanion(subject))
    }

    fn live(&self, id: AgreementId) -> Result<&Agreement, SocialError> {
        let agreement = self
            .agreements
            .get(&id)
            .ok_or(SocialError::NoSuchAgreement(id))?;
        if !agreement.standing() {
            return Err(SocialError::AlreadyEnded(id));
        }
        Ok(agreement)
    }

    /// Puts an offer to someone. The answer is theirs; a refusal is an answer
    /// and not an error, and answering is itself a deed.
    pub fn consider(&mut self, offer: &Offer, at: Tick) -> Result<Response, SocialError> {
        self.known(offer.asked_by)?;
        let companion = self.known(offer.asked_of)?.clone();
        let standing = self.standing(offer.asked_of, offer.asked_by);
        let weighed = willing::weigh(
            &companion,
            offer.asked_by,
            &standing,
            &self.log,
            &offer.work,
            &offer.terms,
        );
        let kind = match weighed.verdict {
            Verdict::Accept => DeedKind::OfferAccepted,
            Verdict::Refuse => DeedKind::OfferRefused,
            Verdict::Counteroffer(_) => DeedKind::OfferCountered,
        };
        let recorded = self
            .log
            .record(at, offer.asked_of, Some(offer.asked_by), kind);
        Ok(Response {
            by: offer.asked_of,
            at,
            verdict: weighed.verdict,
            premises: weighed.premises,
            recorded,
        })
    }

    /// Forms a standing agreement on the offer's terms, if the offer is one
    /// they would take. An arrangement nobody accepted cannot be created here:
    /// the weighing runs again and a declined form changes nothing.
    pub fn form(&mut self, offer: &Offer, at: Tick) -> Result<Ruling, SocialError> {
        self.known(offer.asked_by)?;
        let companion = self.known(offer.asked_of)?.clone();
        let standing = self.standing(offer.asked_of, offer.asked_by);
        let weighed = willing::weigh(
            &companion,
            offer.asked_by,
            &standing,
            &self.log,
            &offer.work,
            &offer.terms,
        );
        if !weighed.verdict.accepted() {
            return Ok(Ruling {
                by: offer.asked_of,
                at,
                kind: RulingKind::Declined,
                premises: weighed.premises,
                recorded: None,
            });
        }

        let id = AgreementId(self.next_agreement);
        self.next_agreement += 1;
        let deed = self.log.record(
            at,
            offer.asked_of,
            Some(offer.asked_by),
            DeedKind::AgreementFormed(id),
        );
        let mut agreement = Agreement {
            id,
            asker: offer.asked_by,
            holder: offer.asked_of,
            work: offer.work,
            terms: offer.terms,
            formed_at: at,
            state: AgreementState::Standing,
            history: Vec::new(),
        };
        agreement.append(at, deed, AgreementChange::Formed);
        self.agreements.insert(id, agreement);

        let mut premises = weighed.premises;
        premises.push(Premise::AgreementTerm {
            agreement: id,
            share: offer.terms.share,
            danger_cap: offer.terms.danger_cap,
            covers: true,
        });
        Ok(Ruling {
            by: offer.asked_of,
            at,
            kind: RulingKind::Formed(id),
            premises,
            recorded: Some(deed),
        })
    }

    /// Asks for work under an existing arrangement. Routine when the terms
    /// cover it and the premises still hold; still theirs to decline when they
    /// do not.
    pub fn exercise(
        &mut self,
        id: AgreementId,
        work: &Work,
        at: Tick,
    ) -> Result<Ruling, SocialError> {
        let agreement = self.live(id)?;
        let (asker, holder, terms, covers) = (
            agreement.asker,
            agreement.holder,
            agreement.terms,
            agreement.covers(work),
        );
        let mut premises = vec![Premise::AgreementTerm {
            agreement: id,
            share: terms.share,
            danger_cap: terms.danger_cap,
            covers,
        }];
        if !covers {
            return Ok(Ruling {
                by: holder,
                at,
                kind: RulingKind::OutsideTerms,
                premises,
                recorded: None,
            });
        }

        let companion = self.known(holder)?.clone();
        let standing = self.standing(holder, asker);
        let (willing, weighing) =
            willing::weigh_routine(&companion, asker, &standing, &self.log, work);
        premises.extend(weighing);
        if !willing {
            return Ok(Ruling {
                by: holder,
                at,
                kind: RulingKind::Declined,
                premises,
                recorded: None,
            });
        }

        let deed = self.log.record(
            at,
            holder,
            Some(asker),
            DeedKind::PerformedUnderAgreement(id),
        );
        if let Some(agreement) = self.agreements.get_mut(&id) {
            agreement.append(at, deed, AgreementChange::Exercised);
        }
        Ok(Ruling {
            by: holder,
            at,
            kind: RulingKind::Performed,
            premises,
            recorded: Some(deed),
        })
    }

    /// Proposes different terms. When the asker proposes, the holder weighs
    /// them like any other ask and may decline. When the holder proposes, the
    /// asker takes them: the played side of a bargain is the player's own
    /// choice, and this gate does not model the player refusing themselves.
    pub fn propose_change(
        &mut self,
        id: AgreementId,
        by: SubjectId,
        terms: Terms,
        at: Tick,
    ) -> Result<Ruling, SocialError> {
        let agreement = self.live(id)?;
        if !agreement.party(by) {
            return Err(SocialError::NotAParty(by));
        }
        let (asker, holder, from) = (agreement.asker, agreement.holder, agreement.terms);
        let other = agreement.other(by);
        let mut premises = vec![Premise::TermChange {
            agreement: id,
            from,
            to: terms,
        }];

        if by == asker {
            let companion = self.known(holder)?.clone();
            let standing = self.standing(holder, asker);
            premises.extend(willing::evidence(&self.log, &standing, asker));
            let limit = willing::bearable(standing.affinity, companion.caution);
            let borne = i16::from(terms.danger_cap) <= limit && terms.share >= from.share;
            premises.push(Premise::DangerWeighed {
                danger: terms.danger_cap,
                bearable: limit,
                affinity: standing.affinity,
                caution: companion.caution,
                borne,
            });
            if !borne {
                return Ok(Ruling {
                    by: holder,
                    at,
                    kind: RulingKind::Declined,
                    premises,
                    recorded: None,
                });
            }
        }

        let deed = self
            .log
            .record(at, by, Some(other), DeedKind::AgreementRenegotiated(id));
        if let Some(agreement) = self.agreements.get_mut(&id) {
            agreement.terms = terms;
            agreement.append(at, deed, AgreementChange::Renegotiated { from, to: terms });
        }
        Ok(Ruling {
            by,
            at,
            kind: RulingKind::Renegotiated { from, to: terms },
            premises,
            recorded: Some(deed),
        })
    }

    /// Ends an arrangement. Either party may; the reason and the standing it
    /// was ended from are both recorded.
    pub fn end(
        &mut self,
        id: AgreementId,
        by: SubjectId,
        why: EndReason,
        at: Tick,
    ) -> Result<Ruling, SocialError> {
        let agreement = self.live(id)?;
        if !agreement.party(by) {
            return Err(SocialError::NotAParty(by));
        }
        let other = agreement.other(by);
        let standing = self.standing(by, other);
        let mut premises = willing::evidence(&self.log, &standing, other);
        premises.push(Premise::Ending { agreement: id, why });

        let deed = self
            .log
            .record(at, by, Some(other), DeedKind::AgreementEnded(id));
        if let Some(agreement) = self.agreements.get_mut(&id) {
            agreement.state = AgreementState::Ended { at, why };
            agreement.append(at, deed, AgreementChange::Ended(why));
        }
        Ok(Ruling {
            by,
            at,
            kind: RulingKind::Ended(why),
            premises,
            recorded: Some(deed),
        })
    }

    /// One premise, as a line a player reads. The premise is the fact; this is
    /// only its wording, and the two are kept apart so a test can assert on the
    /// fact without asserting on prose.
    pub fn say(&self, premise: &Premise) -> String {
        match premise {
            Premise::Deed {
                at,
                doer,
                kind,
                trust,
                affinity,
                ..
            } => format!(
                "tick {}: {} {} (trust {:+}, liking {:+})",
                at.0,
                self.name_of(*doer),
                kind.phrase(),
                trust,
                affinity
            ),
            Premise::Standing {
                toward,
                trust,
                affinity,
                from_deeds,
            } => format!(
                "where I stand with {}: trust {}, liking {}, out of {} deeds",
                self.name_of(*toward),
                trust,
                affinity,
                from_deeds.len()
            ),
            Premise::Confidence {
                craft,
                demanded,
                held,
                margin,
            } => format!(
                "{} at grade {demanded}; I hold {held} ({margin:+})",
                craft.name()
            ),
            Premise::TrustAsked {
                danger,
                asked,
                held,
                met,
            } => format!(
                "danger {danger} asks trust {asked}; I hold {held} ({})",
                if *met { "enough" } else { "not enough" }
            ),
            Premise::DangerWeighed {
                danger,
                bearable,
                affinity,
                caution,
                borne,
            } => format!(
                "danger {danger} against the {bearable} I would bear for them \
                 (liking {affinity}, caution {caution}): {}",
                if *borne { "within me" } else { "past me" }
            ),
            Premise::AgreementTerm {
                share,
                danger_cap,
                covers,
                ..
            } => format!(
                "our arrangement: share {share}, up to danger {danger_cap} ({})",
                if *covers {
                    "this fits"
                } else {
                    "this is not that"
                }
            ),
            Premise::TermChange { from, to, .. } => format!(
                "terms: share {} -> {}, danger cap {} -> {}",
                from.share, to.share, from.danger_cap, to.danger_cap
            ),
            Premise::Ending { why, .. } => format!("ending: {}", why.phrase()),
        }
    }

    pub fn explain(&self, premises: &[Premise]) -> Vec<String> {
        premises.iter().map(|premise| self.say(premise)).collect()
    }
}
