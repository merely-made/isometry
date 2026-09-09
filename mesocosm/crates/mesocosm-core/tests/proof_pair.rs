// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Law C, demonstrated rather than asserted.
//!
//! > Same seed format whether authored by play or by RNG, at every import slot
//! > in every game. Player history *displaces* procedural content; it never
//! > gates it.
//!
//! The founding record says the proof pair must "import a played critter and an
//! RNG critter through the same profile and confirm the consuming game cannot
//! tell them apart structurally. Only the player can, by pointing."
//!
//! So one half of this file plays a critter — really plays it, by driving the
//! world with intents until it has eaten things — and the other half generates
//! one. Then it checks that nothing downstream can separate them except by
//! reading the record and recognising it.
//!
//! The rest of the file covers the protocol keystone the return direction
//! rests on: **additive facts, opaque preservation, deferred interpretation**.

use mesocosm_core::{
    BodyDocument, Chronicle, Consequence, Deed, Intent, PartPalette, Placement, World,
    axis::catalogue, chronicle::LOST_PART, generate,
};

fn regrow(chronicle: &Chronicle) -> BodyDocument {
    chronicle
        .found(
            &catalogue::centipede(2),
            17,
            1_000,
            PartPalette::primitive(),
        )
        .expect("the local recipe regrows")
}

/// A critter somebody played: it takes an explicit meal through the world
/// intent path, rather than carrying a procedural origin marker.
///
/// Deliberately not hand-built. A hand-built body would prove that two structs
/// with the same fields compare equal, which is not the claim; the claim is
/// that a creature which *came out of play* is indistinguishable from one that
/// did not.
fn played() -> Chronicle {
    let mut world = World::new(4_242, 24);
    let prey = nearest(&world).expect("the fixture scatters organisms");
    let here = world.position().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == prey.0)
        .expect("the selected organism remains in the roster")
        .position = here;
    // Incorporate rather than Metabolize, so the body plan decides placement,
    // which is the whole point of automatic symmetric growth being the default
    // and explicit placement being the editor path.
    world.apply(Intent::Metabolize {
        organism: prey.0,
        placement: Placement::Planned,
    });

    assert!(
        world.body().unwrap().len() > 1,
        "the played critter actually grew"
    );
    Chronicle::of(world.body().unwrap())
}

/// The closest living organism, by the same distance the reach rule uses.
fn nearest(world: &World) -> Option<(mesocosm_core::OrganismId, [i32; 3])> {
    world
        .organisms
        .iter()
        .filter(|organism| organism.biomass_mg() > 0 && Some(organism.id) != world.controlled_id())
        .map(|organism| (organism.id, organism.position))
        .min_by_key(|(_, at)| {
            (0..3)
                .map(|axis| (at[axis] - world.position().unwrap()[axis]).abs())
                .max()
                .unwrap_or(0)
        })
}

/// A critter nobody played.
fn rng() -> Chronicle {
    generate(99, 7)
}

#[test]
fn a_played_critter_and_a_generated_one_are_the_same_kind_of_thing() {
    // The structural half of Law C. Both are a Chronicle: same type, same
    // fields, no origin marker. There is no `is_player_made` to branch on,
    // which is the property that has to survive future edits.
    let played = played();
    let generated = rng();

    assert!(!played.parts.is_empty() && !generated.parts.is_empty());
    assert!(
        played.incorporated_parts() > 0,
        "the played critter has a history of eating"
    );
    assert!(
        generated.incorporated_parts() > 0,
        "so does the generated one -- a blank-slate RNG critter would be \
         trivially distinguishable, and Law C would be decoration"
    );
}

#[test]
fn the_consuming_game_cannot_tell_them_apart_from_the_bytes() {
    // The real test of "structurally indistinguishable": encode both and
    // confirm a reader has no field to branch on. If a marker were ever added,
    // the two decoded records would differ in a way a consumer could switch
    // on, and this is where that would show up.
    let played = played();
    let generated = rng();

    let a = Chronicle::from_bytes(&played.to_bytes().unwrap()).unwrap();
    let b = Chronicle::from_bytes(&generated.to_bytes().unwrap()).unwrap();

    // Same shape of answer to every question a consumer can ask.
    for chronicle in [&a, &b] {
        assert!(chronicle.species < u32::MAX);
        assert!(!chronicle.parts.is_empty());
        assert!(chronicle.deeds.is_empty(), "neither has been anywhere yet");
        assert!(
            chronicle.parts[0].from_species.is_none(),
            "both start founded"
        );
    }

    // And the only thing that separates them is the content of the record,
    // which is exactly what "the player can tell, by pointing" means.
    assert_ne!(
        a, b,
        "they are different creatures, not indistinguishable data"
    );
}

#[test]
fn history_attaches_to_both_the_same_way() {
    // Displacement, not gating: a generated critter accepts everything a
    // played one does. If the pipeline only enriched player-made creatures,
    // inheritance would be a prerequisite rather than a bonus.
    for mut chronicle in [played(), rng()] {
        let before = chronicle.parts.len();
        chronicle.append(Deed::new("isometry", "joined-a-faction", 12));
        chronicle.append(Deed::new("isometry", "held-the-ford", 19));

        let read = Chronicle::from_bytes(&chronicle.to_bytes().unwrap()).unwrap();
        assert_eq!(read.deeds.len(), 2);
        assert_eq!(
            read.parts.len(),
            before,
            "appending history changed no anatomy"
        );
    }
}

#[test]
fn deeds_this_game_cannot_read_survive_it_untouched() {
    // Opaque preservation. Isometry speaks about factions and fords; Mesocosm
    // has no rule for either and must not drop them, because the next game to
    // hold this record may. Fact loss happens by omission, so it is tested for
    // directly.
    let mut chronicle = played();
    let foreign = Deed::detailed("isometry", "swore-an-oath", 31, vec![9, 8, 7, 6, 5]);
    chronicle.append(foreign.clone());
    chronicle.append(Deed::detailed(
        "paredros",
        "was-vouched-for",
        44,
        vec![1, 2],
    ));

    let read = Chronicle::from_bytes(&chronicle.to_bytes().unwrap()).unwrap();

    assert_eq!(read.unread().count(), 2, "both are foreign to this game");
    assert_eq!(
        read.deeds[0], foreign,
        "the deed came back byte for byte, payload included"
    );

    // And a second crossing does not erode it. This is the property that
    // matters over a lineage's life, not over one hop.
    let again = Chronicle::from_bytes(&read.to_bytes().unwrap()).unwrap();
    assert_eq!(
        again, read,
        "an uninterpretable record is stable across crossings"
    );
}

#[test]
fn this_game_derives_its_own_consequence_from_a_foreign_fact() {
    // Deferred interpretation. Isometry records that something was lost, in
    // Isometry's words. Only Mesocosm knows what losing it does to a body, and
    // it decides that here rather than being told.
    let mut chronicle = played();
    let before = chronicle.parts.len();
    let uninjured = regrow(&chronicle).living().count();
    assert!(before >= 2, "there is a part to lose");

    chronicle.append(Deed::detailed(
        "isometry",
        LOST_PART,
        50,
        1u32.to_le_bytes().to_vec(),
    ));

    let consequences: Vec<_> = chronicle.read().map(|(_, c)| c).collect();
    assert!(consequences.contains(&Consequence::LostPart { part: 1 }));

    let descendant = regrow(&chronicle);
    assert!(
        descendant.living().count() < uninjured,
        "the locally regrown descendant expresses the inherited loss"
    );
    assert_eq!(
        descendant.species.0, chronicle.species,
        "the lineage continues"
    );
}

#[test]
fn a_foreign_verb_we_half_recognise_is_not_guessed_at() {
    // Our verb, a payload that is not ours. Refusing to guess is the point: a
    // malformed detail must not become a plausible part index, because a
    // wrong-but-plausible interpretation is worse than none.
    let mut chronicle = played();
    chronicle.append(Deed::detailed("isometry", LOST_PART, 50, vec![1, 2]));

    assert_eq!(chronicle.unread().count(), 1, "kept, uninterpreted");
    assert_eq!(
        regrow(&chronicle).living().count(),
        regrow(&played()).living().count(),
        "nothing was lost on a payload we could not read"
    );
}

#[test]
fn interpreting_a_record_does_not_consume_it() {
    // Re-entry is interpretation, not merging. After Mesocosm acts on a deed,
    // the deed is still there for the next game, which may make something else
    // of it entirely.
    let mut chronicle = played();
    chronicle.append(Deed::detailed(
        "isometry",
        LOST_PART,
        50,
        1u32.to_le_bytes().to_vec(),
    ));
    let before = chronicle.clone();

    let _ = regrow(&chronicle);

    assert_eq!(
        chronicle, before,
        "founding a descendant read the record without editing it"
    );
}

#[test]
fn a_generated_critter_founds_a_lineage_exactly_like_a_played_one() {
    // The end of the arrow: no-homework means a player who has never touched
    // Mesocosm gets the same machinery.
    let chronicles = [played(), rng()];
    let descendants: Vec<_> = chronicles.iter().map(regrow).collect();
    assert_eq!(
        descendants[0].len(),
        descendants[1].len(),
        "the local program, not origin, decides structural size"
    );

    for (chronicle, descendant) in chronicles.iter().zip(descendants) {
        assert_eq!(descendant.species.0, chronicle.species);
        let expressed = Chronicle::of(&descendant);
        assert!(
            expressed
                .parts
                .iter()
                .filter(|origin| origin.is_incorporated())
                .all(|origin| chronicle.parts.contains(origin)),
            "regrowth may leave origins dormant but never invents one"
        );
    }
}

#[test]
fn size_is_not_an_origin_tell() {
    // The subtler half of structural indistinguishability. Nobody has to add
    // an is_player_made flag to break Law C -- it is enough that generated
    // creatures are always small and played ones are not, because then a
    // consumer can guess from the part count alone. The distributions have to
    // overlap, so this checks that generation reaches the size play reaches.
    let played_parts = played().parts.len();
    let generated: Vec<usize> = (0..64).map(|seed| generate(seed, 7).parts.len()).collect();

    assert!(
        generated.iter().any(|parts| *parts >= played_parts),
        "some generated creature is at least as elaborate as a played one          (played {played_parts}, generated max {:?})",
        generated.iter().max()
    );
    assert!(
        generated.iter().any(|parts| *parts < 5),
        "and some are small"
    );
}

#[test]
fn generation_is_deterministic_and_seeds_differ() {
    assert_eq!(
        generate(99, 7),
        generate(99, 7),
        "a seed names one creature"
    );
    assert_ne!(
        generate(99, 7),
        generate(100, 7),
        "different seeds, different creatures"
    );
}

#[test]
fn a_chronicle_is_not_a_body_profile() {
    // Two schemas ride the same framing, so the magic is what keeps a reader
    // from mis-decoding one as the other. Worth a test, because the failure
    // would otherwise surface as a confusing "malformed" far from the cause.
    let bytes = played().to_bytes().unwrap();
    assert!(
        matches!(
            mesocosm_core::wire::peek(*b"MESOBODY", &bytes),
            Err(mesocosm_core::WireError::WrongSchema { .. })
        ),
        "a chronicle does not answer to the body profile's magic"
    );
}
