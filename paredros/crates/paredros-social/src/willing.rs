// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The willingness rule.
//!
//! Deliberately small. A player has to be able to hold it in their head and
//! predict an answer from the relationship surface before committing, which the
//! charter ruled on 2026-07-31, and a rule with a dozen coefficients cannot be
//! held that way. Three gates, in this order:
//!
//! 1. **Can I do it?** Confidence, from their own declared craft. Short here
//!    and they refuse, and the refusal is about the work, not the asker.
//! 2. **Would I risk that for you?** Trust against what the danger asks.
//!    Short here and they refuse, and the refusal is about the asker.
//! 3. **Is it more than I would bear?** Danger against what affection and
//!    nerve will carry. Short here and they counteroffer rather than refuse,
//!    because the ask is fine and the terms are not.
//!
//! Every gate reached appends its premise, and gates past the deciding one are
//! not evaluated and so are not claimed.

use paredros_identity::SubjectId;

use crate::companion::Companion;
use crate::deed::DeedLog;
use crate::offer::{Terms, Work};
use crate::relation::Standing;
use crate::response::{Premise, Verdict};

/// The trust a danger asks for. One integer, no curve.
pub fn trust_asked(danger: u8) -> i16 {
    i16::from(danger)
}

/// The danger a companion would bear for a person they feel this way about.
/// Affection buys risk; caution sells it.
pub fn bearable(affinity: i16, caution: i16) -> i16 {
    3 + affinity / 2 - caution
}

/// An answer and the reasoning that produced it, before either becomes a
/// recorded thing.
pub struct Weighed {
    pub verdict: Verdict,
    pub premises: Vec<Premise>,
}

/// The deeds behind a standing, then the standing itself. Always in this
/// order: the evidence, then what it adds up to.
pub fn evidence(log: &DeedLog, standing: &Standing, toward: SubjectId) -> Vec<Premise> {
    let mut premises: Vec<Premise> = standing
        .from_deeds
        .iter()
        .filter_map(|id| log.get(*id))
        .map(|deed| {
            let (trust, affinity) = deed.kind.weight();
            Premise::Deed {
                deed: deed.id,
                at: deed.at,
                doer: deed.doer,
                kind: deed.kind,
                trust,
                affinity,
            }
        })
        .collect();
    premises.push(Premise::Standing {
        toward,
        trust: standing.trust,
        affinity: standing.affinity,
        from_deeds: standing.from_deeds.clone(),
    });
    premises
}

fn confidence_premise(companion: &Companion, work: &Work) -> (bool, Premise) {
    let read = companion.read(work);
    (
        read.sufficient(),
        Premise::Confidence {
            craft: read.craft,
            demanded: read.demanded,
            held: read.held,
            margin: read.margin(),
        },
    )
}

/// The three gates, for a fresh offer.
pub fn weigh(
    companion: &Companion,
    toward: SubjectId,
    standing: &Standing,
    log: &DeedLog,
    work: &Work,
    terms: &Terms,
) -> Weighed {
    let (able, confidence) = confidence_premise(companion, work);
    if !able {
        // Nothing about the asker is in play, so nothing about the asker is
        // cited. The premises say only what decided it.
        return Weighed {
            verdict: Verdict::Refuse,
            premises: vec![confidence],
        };
    }

    let mut premises = evidence(log, standing, toward);
    premises.push(confidence);

    let asked = trust_asked(work.danger);
    let met = standing.trust >= asked;
    premises.push(Premise::TrustAsked {
        danger: work.danger,
        asked,
        held: standing.trust,
        met,
    });
    if !met {
        return Weighed {
            verdict: Verdict::Refuse,
            premises,
        };
    }

    let limit = bearable(standing.affinity, companion.caution);
    let borne = i16::from(work.danger) <= limit;
    premises.push(Premise::DangerWeighed {
        danger: work.danger,
        bearable: limit,
        affinity: standing.affinity,
        caution: companion.caution,
        borne,
    });
    if !borne {
        let counter = Terms::new(
            terms.share.saturating_add(1),
            u8::try_from(limit.clamp(0, 255)).unwrap_or(0),
        );
        return Weighed {
            verdict: Verdict::Counteroffer(counter),
            premises,
        };
    }

    Weighed {
        verdict: Verdict::Accept,
        premises,
    }
}

/// The two gates that still apply inside a standing agreement. Danger was
/// settled when the terms were, so only capability and trust are live, and
/// either of them can still fail: that is how an agreement stays an agreement
/// rather than becoming an order.
pub fn weigh_routine(
    companion: &Companion,
    toward: SubjectId,
    standing: &Standing,
    log: &DeedLog,
    work: &Work,
) -> (bool, Vec<Premise>) {
    let (able, confidence) = confidence_premise(companion, work);
    let mut premises = evidence(log, standing, toward);
    premises.push(confidence);
    if !able {
        return (false, premises);
    }

    let asked = trust_asked(work.danger);
    let met = standing.trust >= asked;
    premises.push(Premise::TrustAsked {
        danger: work.danger,
        asked,
        held: standing.trust,
        met,
    });
    (met, premises)
}
