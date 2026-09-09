// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Who comes along, on what standing, and what governs a tag-in.
//!
//! Participation is negotiated, never assigned. A companion's part in the
//! sortie exists only because an agreement stands: a fresh one formed at
//! departure for the expedition itself, or a standing one whose work covers
//! the part. A refusal at departure means the sortie goes without them, and
//! nothing anywhere can put a refused companion on the march.
//!
//! The pact is the synthesis ruling's "a standing agreement governs tag-in"
//! made data: it names the agreement, the successor, and nothing else. The
//! *condition* — the played body going down — lives in the sortie's law, and
//! the pact fires only while its agreement stands, so ending the agreement
//! revokes the tag-in the same way it vacates a home in S2.

use paredros_identity::{SubjectId, Tick};
use paredros_social::agreement::AgreementId;
use paredros_social::offer::{Offer, Terms, Work};
use paredros_social::response::{Response, RulingKind, Verdict};
use paredros_social::society::{SocialError, Society};
use serde::{Deserialize, Serialize};

/// A companion's agreed part on the march. Which driver moves them, and
/// under which agreement they are moving at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Part {
    /// Ranges ahead toward the party's goal, up to a fixed lead.
    Scout { under: AgreementId },
    /// Trails the played body; goes to the downed; tends within reach.
    Healer { under: AgreementId },
}

impl Part {
    pub fn under(&self) -> AgreementId {
        match self {
            Self::Scout { under } | Self::Healer { under } => *under,
        }
    }
}

/// The tag-in pact: under this agreement, this companion takes over when
/// the played body is downed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pact {
    pub under: AgreementId,
    pub successor: SubjectId,
}

/// How the departure went: every answer given, and the parts that hold.
#[derive(Clone, Debug)]
pub struct Departure {
    pub answers: Vec<Response>,
    pub parts: Vec<(SubjectId, Part)>,
    /// The expedition agreement formed at departure, if anyone took it.
    pub outing: Option<AgreementId>,
}

/// Negotiates the scout's part: a fresh offer for this expedition, since a
/// bounded dangerous outing is not anyone's daily round. The answer is
/// theirs; a refusal leaves the sortie without a scout and costs them
/// nothing.
pub fn negotiate_scout(
    society: &mut Society,
    asked_by: SubjectId,
    asked_of: SubjectId,
    work: Work,
    terms: Terms,
    at: Tick,
) -> Result<(Response, Option<Part>), SocialError> {
    let offer = Offer::new(asked_by, asked_of, work, terms);
    let answer = society.consider(&offer, at)?;
    if answer.verdict != Verdict::Accept {
        return Ok((answer, None));
    }
    let ruling = society.form(&offer, at)?;
    let RulingKind::Formed(id) = ruling.kind else {
        return Ok((answer, None));
    };
    Ok((answer, Some(Part::Scout { under: id })))
}

/// The healer's part rides a standing agreement: the settlement's healer
/// heals its people wherever they are. Verified, not formed: the agreement
/// must stand and its work must cover the tending, or there is no healer
/// and no pact on the march.
pub fn healer_part(society: &Society, under: AgreementId, tend: &Work) -> Option<Part> {
    let agreement = society.agreement(under)?;
    (agreement.standing() && agreement.covers(tend)).then_some(Part::Healer { under })
}
