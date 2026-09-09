// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The four dev intents, read against the ledger. (DT3)
//!
//! Split out of `flows.rs` at the 600-line ceiling, beside `boundary` and
//! `transfers`, and the harness is next door and shared for the reason those
//! two share it: two files reconciling ticks two different ways is how they
//! would come to disagree.
//!
//! What is separately worth showing here is the one transfer in the game that
//! brings matter in from *outside* the enclosure. Every other flow moves a
//! milligram between two of TD6's three compartments; a placement comes out of
//! [`Account::Dev`], and the claim is that naming that source is what keeps the
//! soil's gain claimed rather than unexplained.

use mesocosm_core::flow::{Account, Process};
use mesocosm_core::{Intent, World};

use super::stepped;

/// **The dev verbs are in the stream too** (DT3), and the placement's own
/// account is what keeps the soil's side of it claimed.
///
/// `stepped` reconciles every account against the stream on every tick below,
/// so what this test adds is the narrower claim: the one transfer that brings
/// matter in from outside the enclosure names [`Account::Dev`] as its source,
/// says nothing about a body, and accounts for exactly the soil's gain.
#[test]
fn the_stream_accounts_for_the_dev_verbs_too() {
    let mut world = World::new(11, 40);
    stepped(&mut world, Intent::Idle, "settling");

    let parent = world
        .living()
        .find(|o| Some(o.id) != world.controlled_id() && o.biomass_mg() > 400)
        .expect("somebody has a body to divide")
        .id;
    let doomed = world
        .living()
        .find(|o| Some(o.id) != world.controlled_id() && o.id != parent)
        .expect("somebody else is alive")
        .id;
    let here = world.position().expect("a played critter");

    stepped(
        &mut world,
        Intent::ForceBirth { organism: parent },
        "on the forced birth",
    );
    stepped(&mut world, Intent::Kill { organism: doomed }, "on the kill");
    stepped(&mut world, Intent::EndEpoch, "on the demanded boundary");

    let flows = stepped(
        &mut world,
        Intent::PlaceMatter {
            at: here,
            mass_mg: 900,
        },
        "on the placement",
    );
    let placements: Vec<&mesocosm_core::flow::FlowEvent> = flows
        .iter()
        .map(|flow| &flow.record)
        .filter(|record| record.process == Process::Place)
        .collect();
    assert_eq!(placements.len(), 1, "one placement, one record");
    assert_eq!(placements[0].source, Account::Dev);
    assert_eq!(placements[0].destination, Account::Soil);
    assert_eq!(placements[0].amount_mg, 900);
    assert!(
        placements[0].from.is_none() && placements[0].to.is_none(),
        "neither end of it is a body's account"
    );
    assert_eq!(Account::issued_mg(&flows), 900);

    // A refused one emits nothing, at the same commit point every other verb
    // shares.
    let refused = stepped(
        &mut world,
        Intent::PlaceMatter {
            at: here,
            mass_mg: u64::MAX,
        },
        "on the refused placement",
    );
    assert!(refused.iter().all(|f| f.record.process != Process::Place));
    assert_eq!(Account::issued_mg(&refused), 0);

    // And the ticks after them, because a dev verb that balanced once and left
    // a body holding matter nobody accounts for would show up here.
    for tick in 1..=40 {
        stepped(&mut world, Intent::Idle, &format!("on dev tick {tick}"));
    }
}
