// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The loop closing: a critter that went to Isometry and came back.
//!
//! `fixtures/returned.chronicle` is real Isometry output — `cargo run -p
//! isometry-campaign --example emit_return`, committed unchanged. It began as
//! `played.chronicle`, which this game wrote; over there it took a name, joined
//! a faction, held a ford, got sung about, and lost something.
//!
//! Everything asserted here is about what survives a real round trip rather
//! than about types agreeing in the abstract. `proof_pair.rs` covers the
//! keystone against synthetic deeds; this covers it against bytes another
//! codebase actually produced.

use mesocosm_core::{
    BodyDocument, Chronicle, Consequence, Origin, PartPalette, axis::catalogue,
    chronicle::LOST_PART,
};

/// A critter this game wrote, that Isometry has since had for a while.
const RETURNED: &[u8] = include_bytes!("../fixtures/returned.chronicle");

/// The same critter before it left.
const DEPARTED: &[u8] = include_bytes!("../fixtures/played.chronicle");

fn regrow(chronicle: &Chronicle, palette: PartPalette) -> BodyDocument {
    chronicle
        .found(&catalogue::centipede(1), 17, 2_000, palette)
        .expect("the returned lineage regrows under local rules")
}

#[test]
fn a_creature_comes_home_still_itself() {
    let there = Chronicle::from_bytes(RETURNED).expect("Isometry wrote a chronicle we can read");
    let here = Chronicle::from_bytes(DEPARTED).unwrap();

    assert_eq!(
        there.species, here.species,
        "the lineage is the same lineage"
    );
    assert_eq!(
        there.parts, here.parts,
        "another game did not rewrite our anatomy"
    );
    assert!(
        !there.deeds.is_empty(),
        "and it did not come back empty-handed"
    );
}

#[test]
fn it_came_back_with_history_we_did_not_write() {
    let returned = Chronicle::from_bytes(RETURNED).unwrap();
    let departed = Chronicle::from_bytes(DEPARTED).unwrap();

    assert!(departed.deeds.is_empty(), "it left with no history");
    assert!(
        returned.deeds.len() >= 4,
        "it came back with several facts appended, in somebody else's words"
    );
    assert!(
        returned.deeds.iter().all(|deed| deed.vessel == "isometry"),
        "every one of them says who wrote it"
    );
}

#[test]
fn facts_this_game_cannot_read_are_carried_rather_than_dropped() {
    // Opaque preservation, against a foreign writer. Isometry's own verbs mean
    // nothing here, and its payloads are a format this game has never heard of.
    // Both survive, because dropping them is how a pipeline starts feeling fake.
    let returned = Chronicle::from_bytes(RETURNED).unwrap();
    let unread: Vec<_> = returned.unread().collect();

    assert!(
        unread.len() >= 3,
        "most of what happened over there is opaque here"
    );
    assert!(
        unread.iter().any(|deed| deed.verb == "held-the-ford"),
        "including the one the campaign cared most about"
    );
    assert!(
        unread.iter().all(|deed| !deed.detail.is_empty()),
        "payloads came through whole, not blanked"
    );

    // And they survive us handling them, which is the property that matters
    // over a lineage rather than over one hop.
    let again = Chronicle::from_bytes(&returned.to_bytes().unwrap()).unwrap();
    assert_eq!(again, returned, "we re-emit a foreign record byte for byte");
}

#[test]
fn the_one_verb_we_share_is_the_one_we_act_on() {
    // Deferred interpretation, end to end. Isometry recorded a loss in the
    // shared vocabulary; only Mesocosm decides what losing a part does to a
    // body, and it decides it here.
    let returned = Chronicle::from_bytes(RETURNED).unwrap();

    let acted: Vec<_> = returned
        .read()
        .filter_map(|(deed, consequence)| match consequence {
            Consequence::LostPart { part } => Some((deed.verb.clone(), part)),
            Consequence::Unread => None,
        })
        .collect();

    assert_eq!(
        acted.len(),
        1,
        "exactly one fact was in a vocabulary we share"
    );
    assert_eq!(acted[0].0, LOST_PART);
}

#[test]
fn narrating_a_loss_is_not_the_same_as_claiming_one() {
    // The distinction the round trip forced into the open. Isometry writes
    // prose about a lost arm through its own vocabulary and a part index
    // through the shared one, and only the second is a claim about anatomy.
    // A game that guessed from the prose would be inventing consequences for
    // another game's fiction.
    let returned = Chronicle::from_bytes(RETURNED).unwrap();

    let shared = returned
        .deeds
        .iter()
        .find(|deed| deed.verb == LOST_PART)
        .expect("the shared-vocabulary deed is present");

    assert_eq!(
        shared.detail.len(),
        4,
        "the shared verb carries the agreed payload"
    );
    assert!(
        returned
            .deeds
            .iter()
            .filter(|deed| deed.verb == LOST_PART)
            .count()
            == 1,
        "and it is the only one making that claim"
    );
}

#[test]
fn the_descendant_is_founded_under_this_games_rules() {
    // The end of the arrow. Isometry said what happened; Mesocosm decides what
    // it means for a body, regrows the lineage, and carries the whole record
    // forward including the parts it could not read.
    let returned = Chronicle::from_bytes(RETURNED).unwrap();
    let departed = Chronicle::from_bytes(DEPARTED).unwrap();
    let whole = regrow(&departed, PartPalette::primitive());
    let descendant = regrow(&returned, PartPalette::primitive());

    assert_eq!(
        descendant.species.0, returned.species,
        "the lineage continues"
    );
    assert!(
        descendant.living().count() < whole.living().count(),
        "the local descendant expresses the inherited loss"
    );

    // Every surviving part keeps the history it had. The root founded the
    // lineage and the rest were eaten, so after losing one incorporated part
    // the descendant carries one fewer than it arrived with.
    let living_incorporated = descendant
        .living()
        .filter(|part| matches!(part.provenance.origin, Origin::Incorporated { .. }))
        .count();
    let whole_incorporated = whole
        .living()
        .filter(|part| matches!(part.provenance.origin, Origin::Incorporated { .. }))
        .count();
    assert_eq!(
        living_incorporated,
        whole_incorporated - 1,
        "the descendant expresses one fewer historical origin after the loss"
    );
    assert!(
        returned.incorporated_parts() > living_incorporated,
        "origins without a local site remain in the chronicle rather than forcing geometry"
    );

    // The record is not consumed by being acted on: the next game sees
    // everything this one saw.
    assert_eq!(
        returned.unread().count(),
        3,
        "and the foreign facts are still there"
    );
}

#[test]
fn geometry_did_not_travel_and_that_is_the_law() {
    // Law A, visible as an absence. A chronicle carries provenance and history
    // and no coordinates, so what came home is a record rather than a model.
    // The body is regrown here, which is what makes the round trip cheap and
    // what keeps another game from dictating this one's anatomy.
    let returned = Chronicle::from_bytes(RETURNED).unwrap();
    let a = regrow(&returned, PartPalette::primitive());
    let mut other_world = PartPalette::primitive();
    other_world.mass.default.half_extent = [3, 3, 3];
    let b = regrow(&returned, other_world);

    assert_eq!(a.len(), b.len(), "the same record founds the same lineage");
    assert_ne!(
        a.parts[0].half_extent, b.parts[0].half_extent,
        "but this game chose the geometry, both times"
    );
}
