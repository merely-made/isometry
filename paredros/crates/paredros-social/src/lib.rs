// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S1, the willingness owner: why a companion says yes, no, or not like that.
//!
//! The execution plan's §2 makes social willingness and combat execution
//! separate owners that never read each other, meeting only at the sortie. This
//! is the social half. It holds deeds, standing, confidence, refusal, standing
//! agreements, and the explanation surface, and it holds no bodies, no world,
//! and no fight.
//!
//! The minimal forms are all present, which was the point of the gate:
//!
//! - [`DeedLog`], append-only, who did what to whom and when;
//! - [`EpistemicLog`], append-only observations, claims, reports, and
//!   claimant-owned corrections over accepted deeds, with belief derived;
//! - [`Standing`], folded out of the log rather than authored, carrying the
//!   ids of the deeds that produced it;
//! - [`Confidence`], a companion's own read of whether work is within their
//!   declared craft;
//! - [`Verdict::Refuse`], an outcome and never an error;
//! - [`Premise`], attached to every answer, naming the deeds, the standing,
//!   the confidence, the thresholds, and the terms that produced it;
//! - [`Agreement`], with formation, exercise, renegotiation, and ending, each
//!   transition recorded as a deed.
//!
//! S2 adds the settlement in its peer-agency form: [`Settlement`], where a
//! tenancy exists only because an agreement formed, and where someone lives is
//! derived from that agreement's state, so moving out *is* the arrangement
//! ending. An accepted agreement is the one thing that changes where a peer
//! lives and what they do daily.
//!
//! **Nothing here commands anyone.** There is no call that makes a companion
//! act. Offers are put; answers come back; a standing agreement makes routine
//! work routine without making it compulsory, and a holder whose premises have
//! changed declines under one. That is the puppeteering canary held at the API
//! rather than in a comment.

pub mod agreement;
pub mod companion;
pub mod deed;
pub mod epistemic;
pub mod offer;
pub mod relation;
pub mod response;
pub mod scene;
pub mod settlement;
pub mod settling;
pub mod society;
pub mod willing;

pub use agreement::{
    Agreement, AgreementChange, AgreementEvent, AgreementId, AgreementState, EndReason,
};
pub use companion::{Companion, Confidence, Craft, Skill};
pub use deed::{Deed, DeedId, DeedKind, DeedLog};
pub use epistemic::{
    Belief, ClaimHistory, ClaimId, EpistemicEntry, EpistemicError, EpistemicId, EpistemicLog,
    EpistemicRecord, EventReading, Evidence, ObservationId, Proposition, ReportId,
};
pub use offer::{Offer, Terms, Work};
pub use relation::{Relations, Standing};
pub use response::{Premise, Response, Ruling, RulingKind, Verdict, cited_deeds};
pub use settlement::{DailyRound, Dwelling, DwellingId, SettleError, Settlement, Tenancy};
pub use society::{SocialError, Society};
