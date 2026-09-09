// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The S1 refusal scene, as data.
//!
//! Three companions, one offer, three answers. The scene lives in the library
//! rather than in a test so the test and the printed run are provably the same
//! run, the way S0's fixed trace does it.
//!
//! Aud, Bram, Odris and Sela are fixture names for this scene. They are not
//! lore and nothing else in the repository refers to them.
//!
//! The differences are only in history, nerve, and craft:
//!
//! - **Bram** was stood by, twice, and is the better scout. Accepts.
//! - **Odris** was shared with once and then left. Is the *best* scout of the
//!   three, and refuses anyway: the model does not confuse capability with
//!   willingness.
//! - **Sela** was shared with twice and stood by once, likes Aud most of the
//!   three, and is the most careful. The ask is fine and the danger is not, so
//!   she counteroffers.

use paredros_identity::{SubjectId, Tick};

use crate::agreement::EndReason;
use crate::companion::{Companion, Craft, Skill};
use crate::deed::DeedKind;
use crate::offer::{Offer, Terms, Work};
use crate::response::{Response, Ruling, RulingKind};
use crate::society::{SocialError, Society};

pub const AUD: SubjectId = SubjectId(1);
pub const BRAM: SubjectId = SubjectId(2);
pub const ODRIS: SubjectId = SubjectId(3);
pub const SELA: SubjectId = SubjectId(4);

/// Asked in this order, always.
pub const THE_THREE: [SubjectId; 3] = [BRAM, ODRIS, SELA];

/// Scouting ahead of a sortie, at a grade three of them can meet and a danger
/// none of them can ignore.
pub const THE_WORK: Work = Work {
    craft: Craft::Scouting,
    grade: 3,
    danger: 4,
};

pub const THE_TERMS: Terms = Terms {
    share: 2,
    danger_cap: 5,
};

pub const ASKED_AT: Tick = Tick(8);

/// The history that makes the three different, and nothing else. Every deed
/// here is Aud's, done to one of them.
pub fn society() -> Society {
    let mut society = Society::new();
    society.admit(Companion::new(
        AUD,
        "Aud",
        vec![Skill::new(Craft::Scouting, 2)],
        0,
    ));
    society.admit(Companion::new(
        BRAM,
        "Bram",
        vec![Skill::new(Craft::Scouting, 4)],
        0,
    ));
    society.admit(Companion::new(
        ODRIS,
        "Odris",
        vec![
            Skill::new(Craft::Scouting, 5),
            Skill::new(Craft::Watching, 3),
        ],
        1,
    ));
    society.admit(Companion::new(
        SELA,
        "Sela",
        vec![
            Skill::new(Craft::Scouting, 3),
            Skill::new(Craft::Healing, 4),
        ],
        3,
    ));

    for (tick, toward, kind) in [
        (1, BRAM, DeedKind::StoodBy),
        (2, BRAM, DeedKind::StoodBy),
        (3, ODRIS, DeedKind::Shared),
        (4, ODRIS, DeedKind::Abandoned),
        (5, SELA, DeedKind::Shared),
        (6, SELA, DeedKind::Shared),
        (7, SELA, DeedKind::StoodBy),
    ] {
        society.record(Tick(tick), AUD, Some(toward), kind);
    }
    society
}

pub fn offer_to(subject: SubjectId) -> Offer {
    Offer::new(AUD, subject, THE_WORK, THE_TERMS)
}

/// The same offer put to each of the three, in order.
pub fn answers(society: &mut Society) -> Result<Vec<Response>, SocialError> {
    THE_THREE
        .iter()
        .map(|subject| society.consider(&offer_to(*subject), ASKED_AT))
        .collect()
}

/// The standing agreement, whole: formed with the one who accepted, exercised
/// once without renegotiation, renegotiated once, asked again past the new term
/// to show the change bit, and ended.
///
/// A form that was declined returns on its own, so the caller sees one ruling
/// instead of five rather than a lifecycle built on an arrangement nobody made.
pub fn lifecycle(society: &mut Society) -> Result<Vec<Ruling>, SocialError> {
    let formed = society.form(&offer_to(BRAM), Tick(9))?;
    let RulingKind::Formed(id) = formed.kind else {
        return Ok(vec![formed]);
    };
    let performed = society.exercise(id, &THE_WORK, Tick(10))?;
    let changed = society.propose_change(id, BRAM, Terms::new(3, 3), Tick(11))?;
    let past_it = society.exercise(id, &THE_WORK, Tick(12))?;
    let ended = society.end(id, AUD, EndReason::WorkDone, Tick(13))?;
    Ok(vec![formed, performed, changed, past_it, ended])
}
