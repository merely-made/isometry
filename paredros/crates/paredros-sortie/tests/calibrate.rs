// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The calibration probe behind `scene::SITE_OFFSET`, kept ignored.
//!
//! `cargo test -p paredros-sortie --test calibrate -- --ignored --nocapture`
//! walks a spread of candidate site offsets over the real terrain and
//! prints, for each: whether the sortie completed, whether the played body
//! fell past comfort on the way, and how long it all took. The shipped
//! offset is chosen from this table, so the hazard is a fact about the
//! grown world rather than an authored set-piece.

use paredros_social::scene::AUD;
use paredros_sortie::scene;
use paredros_sortie::sortie::SortieEvent;

#[test]
#[ignore = "calibration probe, run by hand"]
fn survey_the_marches() {
    for offset in [
        [40, 8],
        [40, -8],
        [-40, 8],
        [-40, -8],
        [8, 40],
        [8, -40],
        [-8, 40],
        [-8, -40],
        [32, 32],
        [32, -32],
        [-32, 32],
        [-32, -32],
        [48, 0],
        [-48, 0],
        [0, 48],
        [0, -48],
    ] {
        let Some((_, sortie)) = scene::surveyed(offset, &scene::WAY_OFFSETS) else {
            println!("offset {offset:?}: no standing site");
            continue;
        };
        let fell = sortie
            .events
            .iter()
            .any(|event| matches!(event, SortieEvent::Fell { who, .. } if *who == AUD));
        let tagged = sortie
            .events
            .iter()
            .any(|event| matches!(event, SortieEvent::TaggedIn { .. }));
        println!(
            "offset {offset:?}: done={} fell={} tagged={} ticks={} wounds={}",
            sortie.done(),
            fell,
            tagged,
            sortie.tick().0 - scene::DEPART.0,
            sortie.wounds.len(),
        );
    }
}
