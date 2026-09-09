// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The S3 scene: the settled three go out, and one comes back changed
//! toward another.
//!
//! Continuity all the way down: the world is S0's seed and S0's hillside,
//! the society is S2's settled state (Bram in the gatehouse, Sela in the
//! still-room, Odris living as before), and the sortie departs from the
//! surface above the carved chamber. Aud negotiates Bram's scouting for
//! this expedition as a fresh bounded agreement; Sela's part rides her
//! standing settlement agreement, which also carries the tag-in pact.
//!
//! The site direction is calibrated against the real terrain, not authored
//! terrain: the march has to cross ground whose own law produces a fall
//! past comfort on the way out. The receipt asserts the fall, the tag-in,
//! the tend, and the return, so a regrown world that stopped having that
//! cliff fails loudly rather than silently passing a tamer sortie.

use mesocosm_core::places::Ground;
use paredros_identity::{BodyRevisionId, Control, Facets, SubjectId, Tick};
use paredros_room::room::{Room, SEED};
use paredros_social::companion::Craft;
use paredros_social::offer::{Terms, Work};
use paredros_social::response::Response;
use paredros_social::scene::{AUD, BRAM, ODRIS, SELA};
use paredros_social::settling;
use paredros_social::society::Society;

use crate::march;
use crate::party::{self, Pact};
use crate::sortie::Sortie;

/// Scouting for the expedition: harder and more dangerous than Bram's
/// daily bounds, so it is negotiated fresh rather than assumed covered.
pub const OUTING: Work = Work {
    craft: Craft::Scouting,
    grade: 3,
    danger: 4,
};

pub const OUTING_TERMS: Terms = Terms {
    share: 2,
    danger_cap: 5,
};

/// Tending the fallen: inside Sela's standing settlement agreement.
pub const TEND: Work = Work {
    craft: Craft::Healing,
    grade: 3,
    danger: 3,
};

/// Departure day. The settled scene's last deed is tick 12.
pub const DEPART: Tick = Tick(13);

/// Where the sortie is headed, relative to home, in x and z. Calibrated:
/// this heading crosses a scarp the walker can only descend as a forced
/// drop, which is the hazard the receipt depends on, and the site itself
/// sits in the trench below it.
pub const SITE_OFFSET: [i32; 2] = [-15, -8];

/// The settled society: S2's people, homes formed, twelve deeds deep.
pub fn settled_society() -> Society {
    let mut society = settling::society();
    let mut settlement = settling::settlement();
    settling::answers(&mut society).expect("the settling scene admits everyone");
    settling::housings(&mut society, &mut settlement).expect("the settled offers hold");
    society
}

/// Everyone wears their first body at departure.
pub fn facets() -> Facets {
    let mut facets = Facets::new();
    for subject in [AUD, BRAM, ODRIS, SELA] {
        facets.wears(subject, BodyRevisionId(0));
    }
    facets
}

/// The muster: negotiation first, then the march, built on the world grown
/// from S0's seed. Returns the departure answers alongside the sortie so a
/// receipt can assert on why each part exists.
pub fn muster(society: Society) -> (Vec<Response>, Sortie) {
    let room = Room::grow(SEED).expect("S0's seed has a hillside");
    muster_on(room.ground, society)
}

fn muster_on(ground: Ground, society: Society) -> (Vec<Response>, Sortie) {
    let home = home_stance(&ground);
    let site = site_stance(&ground);
    let way = way_home(home);
    muster_at(ground, society, home, site, way)
}

fn muster_at(
    ground: Ground,
    mut society: Society,
    home: [i32; 3],
    site: [i32; 3],
    way_home: Vec<[i32; 2]>,
) -> (Vec<Response>, Sortie) {
    // Sela's part and the pact ride her standing settlement agreement.
    let sela_home = society
        .agreements()
        .find(|agreement| agreement.holder == SELA && agreement.standing())
        .map(|agreement| agreement.id)
        .expect("the settled scene housed Sela");
    let healer = party::healer_part(&society, sela_home, &TEND)
        .expect("her standing agreement covers the tending");

    // Bram's part is negotiated fresh, and the answer is his.
    let (answer, scout) =
        party::negotiate_scout(&mut society, AUD, BRAM, OUTING, OUTING_TERMS, DEPART)
            .expect("the muster asks people the society knows");

    let mut parts = Vec::new();
    let mut outing = None;
    if let Some(part) = scout {
        outing = Some(part.under());
        parts.push((BRAM, part));
    }
    parts.push((SELA, healer));

    let sortie = Sortie::muster(
        ground,
        society,
        facets(),
        Control::begin(AUD, DEPART),
        &[AUD, BRAM, ODRIS, SELA],
        parts,
        Some(Pact {
            under: sela_home,
            successor: SELA,
        }),
        outing,
        TEND,
        home,
        site,
        way_home,
        DEPART,
    );
    (vec![answer], sortie)
}

/// The route back, as x/z waypoints ending at home. Calibrated with the
/// site: the scarp the outbound march drops down cannot be climbed back,
/// so the return swings around it before making for home.
pub fn way_home(home: [i32; 3]) -> Vec<[i32; 2]> {
    WAY_OFFSETS
        .iter()
        .map(|offset| [home[0] + offset[0], home[2] + offset[1]])
        .chain([[home[0], home[2]]])
        .collect()
}

/// Detour waypoints of the way home, relative to home. Calibrated with
/// `SITE_OFFSET` by the survey in `tests/calibrate.rs`.
pub const WAY_OFFSETS: [[i32; 2]; 1] = [[-16, 16]];

/// The surface above the carved chamber: the settlement's ground.
pub fn home_stance(ground: &Ground) -> [i32; 3] {
    let room = Room::grow(SEED).expect("S0's seed has a hillside");
    march::stand(ground, room.centre[0], room.centre[2])
        .expect("the hill above the room has a surface to stand on")
}

/// The far site, standing on the real surface at the calibrated offset.
pub fn site_stance(ground: &Ground) -> [i32; 3] {
    let home = home_stance(ground);
    march::stand(ground, home[0] + SITE_OFFSET[0], home[2] + SITE_OFFSET[1])
        .expect("the site column has a surface to stand on")
}

/// The whole run: settled society, muster, march to done.
pub fn played_through() -> (Vec<Response>, Sortie) {
    let (answers, mut sortie) = muster(settled_society());
    sortie.run();
    (answers, sortie)
}

/// A run at an arbitrary site offset: the calibration probe's door. The
/// shipped scene is `played_through`; this exists so `SITE_OFFSET` is
/// chosen from a printed survey of the real terrain rather than by hand.
pub fn surveyed(offset: [i32; 2], way_offsets: &[[i32; 2]]) -> Option<(Vec<Response>, Sortie)> {
    let room = Room::grow(SEED).expect("S0's seed has a hillside");
    let ground = room.ground.clone();
    let home = home_stance(&ground);
    let site = march::stand(&ground, home[0] + offset[0], home[2] + offset[1])?;
    let way = way_offsets
        .iter()
        .map(|way| [home[0] + way[0], home[2] + way[1]])
        .chain([[home[0], home[2]]])
        .collect();
    let (answers, mut sortie) = muster_at(ground, settled_society(), home, site, way);
    sortie.run();
    Some((answers, sortie))
}

/// The canary variant: Aud abandons Bram on the eve, Bram refuses the
/// expedition, and the sortie goes without a scout. Nothing can put him on
/// the march, and the receipt watches him stand still to prove it.
pub fn grudged() -> (Vec<Response>, Sortie) {
    let mut society = settled_society();
    society.record(
        Tick(DEPART.0 - 1),
        AUD,
        Some(BRAM),
        paredros_social::deed::DeedKind::Abandoned,
    );
    let (answers, mut sortie) = muster(society);
    sortie.run();
    (answers, sortie)
}

/// The ask S2 saw counteroffered, put again after the sortie. Same work,
/// same terms, one expedition later.
pub fn ask_again(society: &mut Society, at: Tick) -> Response {
    society
        .consider(&settling::ask_of(SELA), at)
        .expect("Sela is still of the society")
}

/// Odris stayed home and stays home: a subject with no part.
pub fn the_stay_at_home() -> SubjectId {
    ODRIS
}
