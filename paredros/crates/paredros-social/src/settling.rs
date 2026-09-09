// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The S2 settling scene, as data: the negotiated home.
//!
//! The same four people as [`crate::scene`], the same history, one step on:
//! Aud offers housing and daily work. The asks differ by person because the
//! work does, and the answers differ only through history, nerve, and craft:
//!
//! - **Bram** is offered the gatehouse and the bounds to walk. Accepts.
//! - **Odris** is offered the still-room and the night watch, the one craft
//!   he alone holds, at a grade he exactly meets. Refuses at the trust gate,
//!   citing the abandonment: capability is provably not the reason.
//! - **Sela** is offered the still-room and the sickroom's work at a danger
//!   past what she would carry. Counteroffers; Aud returns with her terms and
//!   the ward kept inside them, and that arrangement forms.
//!
//! Two move in, one does not, and every home traces to the bargain under it.

use paredros_identity::{SubjectId, Tick};

use crate::companion::Craft;
use crate::offer::{Offer, Terms, Work};
use crate::response::Response;
use crate::scene::{AUD, BRAM, ODRIS, SELA};
use crate::settlement::{Dwelling, DwellingId, SettleError, Settlement};
use crate::society::{SocialError, Society};

pub const THE_GATEHOUSE: DwellingId = DwellingId(1);
pub const THE_STILL_ROOM: DwellingId = DwellingId(2);

/// Walking the bounds each day: within Bram's craft, mild danger.
pub const BRAM_ASK: Work = Work {
    craft: Craft::Scouting,
    grade: 2,
    danger: 2,
};

/// The night watch: Odris's own craft, at exactly the grade he holds.
pub const ODRIS_ASK: Work = Work {
    craft: Craft::Watching,
    grade: 3,
    danger: 2,
};

/// The sickroom at its worst. Danger 4 is past the 3 Sela would carry.
pub const SELA_ASK: Work = Work {
    craft: Craft::Healing,
    grade: 3,
    danger: 4,
};

/// What Aud puts up alongside each roof.
pub const THE_KEEP: Terms = Terms {
    share: 2,
    danger_cap: 4,
};

/// The settlement before anyone lives in it.
pub fn settlement() -> Settlement {
    let mut settlement = Settlement::new();
    settlement.found(Dwelling {
        id: THE_GATEHOUSE,
        name: "the gatehouse".into(),
    });
    settlement.found(Dwelling {
        id: THE_STILL_ROOM,
        name: "the still-room".into(),
    });
    settlement
}

/// The society is [`crate::scene::society`]'s: same people, same deeds. S2
/// continues the story rather than restarting it.
pub fn society() -> Society {
    crate::scene::society()
}

pub fn ask_of(subject: SubjectId) -> Offer {
    let work = match subject {
        BRAM => BRAM_ASK,
        ODRIS => ODRIS_ASK,
        _ => SELA_ASK,
    };
    Offer::new(AUD, subject, work, THE_KEEP)
}

/// Each offer put and answered, in order: Bram at 8, Odris at 9, Sela at 10.
pub fn answers(society: &mut Society) -> Result<Vec<Response>, SocialError> {
    [BRAM, ODRIS, SELA]
        .iter()
        .zip(8..)
        .map(|(subject, tick)| society.consider(&ask_of(*subject), Tick(tick)))
        .collect()
}

/// Sela's ask, come back around: her countered terms, and the ward's danger
/// held inside the cap she named. The work softened is the counteroffer
/// having mattered.
pub fn sela_settled_ask() -> Offer {
    Offer::new(
        AUD,
        SELA,
        Work {
            danger: 3,
            ..SELA_ASK
        },
        Terms {
            share: 3,
            danger_cap: 3,
        },
    )
}

/// The homes actually made: Bram into the gatehouse at 11, Sela into the
/// still-room at 12 on her own terms. Odris refused, so no arrangement and
/// no tenancy exists to show for him, which is the point.
pub fn housings(society: &mut Society, settlement: &mut Settlement) -> Result<(), SettleError> {
    settlement.offer_home(society, &ask_of(BRAM), THE_GATEHOUSE, Tick(11))?;
    settlement.offer_home(society, &sela_settled_ask(), THE_STILL_ROOM, Tick(12))?;
    Ok(())
}
