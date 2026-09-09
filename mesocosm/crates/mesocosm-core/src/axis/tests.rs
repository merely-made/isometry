// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tests for the axial recipe.
//!
//! Split out of `axis.rs` on 2026-08-29 when TD7's bigger producer draw pushed
//! that file past this repo's six-hundred-line ceiling — the same
//! split-before-adding move that put `ecology::tests` in a file of its own.

use super::catalogue::*;
use super::*;

#[test]
fn one_method_reaches_the_catalogue() {
    // The claim under test: real body plans are parameter sets, not
    // special cases. Each of these is the same four rules.
    // Totals include the head's feelers, so the trunk is checked directly.
    assert_eq!(
        centipede(40).tagmata[1].appendage_count(),
        40,
        "a limb per segment"
    );
    assert_eq!(
        millipede(40).tagmata[1].appendage_count(),
        80,
        "two per fused segment"
    );
    assert_eq!(centipede(40).appendages(), 41, "plus the head");
    assert_eq!(
        insect().appendages(),
        3 + 2 + 1 + 1,
        "legs, wings, feelers, mouth"
    );
    assert_eq!(spider().appendages(), 4 + 1);
    assert_eq!(tetrapod(20).appendages(), 2 + 1, "two girdles and a head");
    assert_eq!(snake(120).appendages(), 1, "the head keeps its feelers");
}

#[test]
fn a_snake_is_a_tetrapod_with_two_edits() {
    // Suppress the girdles, lengthen the trunk. Nothing else differs,
    // which is the whole point of an axial recipe.
    let walker = tetrapod(20);
    let crawler = snake(120);

    assert_eq!(walker.tagmata.len(), crawler.tagmata.len(), "same regions");
    assert!(crawler.segments() > walker.segments(), "longer");
    assert_eq!(crawler.appendages(), 1, "and legless");
}

#[test]
fn a_millipede_is_a_centipede_with_one_field_changed() {
    let mut plan = centipede(30);
    plan.tagmata[1].per_segment = 2;
    assert_eq!(plan, millipede(30));
}

#[test]
fn dividing_a_worm_is_how_a_body_becomes_regional() {
    // A one-stretch creature growing a head: tagmatization, which is the
    // mutation that makes every other plan reachable.
    let mut worm = Recipe::founding(12);
    assert_eq!(worm.tagmata.len(), 1);

    let tail = worm.divide(0, 3).unwrap();
    assert_eq!(tail, 1);
    assert_eq!(worm.tagmata[0].segments, 3);
    assert_eq!(worm.tagmata[1].segments, 9);
    assert_eq!(worm.segments(), 12, "division moves boundaries, not mass");
}

#[test]
fn a_line_cannot_say_a_word_it_has_not_eaten() {
    // The acquisition rule: kleptoplasty teaches vocabulary, and a plan
    // refuses to express what the lineage has never incorporated.
    let mut worm = Recipe::founding(8);
    assert!(!worm.can_express(Appendage::Limb));
    assert_eq!(
        worm.assign(0, Appendage::Limb),
        Err(Unspeakable::NotInLexicon(Appendage::Limb))
    );

    let before = worm.complexity();
    assert!(
        worm.acquire(Appendage::Limb),
        "the first one is a discovery"
    );
    assert!(
        worm.complexity() > before,
        "learning a word is coming further"
    );
    assert!(!worm.acquire(Appendage::Limb), "the second is a meal");
    assert!(worm.assign(0, Appendage::Limb).is_ok());
    assert_eq!(worm.appendages(), 8);
}

#[test]
fn homeosis_is_one_field() {
    // Antennapedia: legs where feelers belong. A real Hox mutant, and one
    // assignment here.
    let mut fly = insect();
    assert_eq!(fly.tagmata[0].appendage, Appendage::Feeler);
    fly.assign(0, Appendage::Limb).unwrap();
    assert_eq!(fly.tagmata[0].appendage, Appendage::Limb);
    assert_eq!(
        fly.appendages(),
        insect().appendages(),
        "the count is unchanged"
    );
}

#[test]
fn complexity_counts_kinds_not_just_length() {
    // The frontier's new axis: a long worm is not elaborate, and a short
    // creature with several appendage kinds is.
    let worm = centipede(60);
    let bug = insect();
    assert!(
        worm.segments() > bug.segments(),
        "the worm is three times longer"
    );
    assert!(
        bug.complexity() > worm.complexity(),
        "the insect is more elaborate"
    );

    // And a legless snake, longer still, stays simpler than both.
    let crawler = snake(120);
    assert!(crawler.segments() > worm.segments());
    assert!(
        crawler.complexity() < bug.complexity(),
        "length is not intricacy"
    );
}

#[test]
fn kin_vary_without_diverging() {
    // Individuals of one lineage differ, and stay recognisable.
    let plan = centipede(30);
    let bodies: Vec<Soma> = (0..24).map(|seed| Soma::develop(&plan, seed)).collect();

    let lengths: BTreeSet<u32> = bodies.iter().map(Soma::total_segments).collect();
    assert!(lengths.len() > 1, "no two are exactly the same");
    for body in &bodies {
        let drift = body.total_segments().abs_diff(plan.segments());
        assert!(drift <= plan.variance as u32 * plan.tagmata.len() as u32);
    }
    assert!(
        bodies.iter().any(|b| !b.absent.is_empty()),
        "development is imperfect often enough to notice"
    );
}

#[test]
fn development_is_reproducible() {
    let plan = insect();
    assert_eq!(Soma::develop(&plan, 99), Soma::develop(&plan, 99));
    assert_ne!(Soma::develop(&plan, 1), Soma::develop(&plan, 2));
}

#[test]
fn seeded_recipes_vary_and_stay_buildable() {
    // Worldgen draws creatures, not catalogue entries.
    let recipes: Vec<Recipe> = (0..40)
        .map(|s| seed(&mut Rng::from_seed(s), Kingdom::Consumer))
        .collect();

    let shapes: BTreeSet<String> = recipes.iter().map(|r| format!("{:?}", r.tagmata)).collect();
    assert!(
        shapes.len() > 20,
        "forty draws gave {} shapes",
        shapes.len()
    );

    for recipe in &recipes {
        assert!(recipe.segments() > 0);
        assert!(!recipe.tagmata.is_empty());
        // Everything a seeded recipe assigns, it can say.
        for tagma in &recipe.tagmata {
            assert!(recipe.can_express(tagma.appendage));
        }
    }
    assert!(
        recipes.iter().any(|r| r.appendages() > 1),
        "some seeded lines have appendages beyond a mouth"
    );
}

// A drawn line has to *read* as the kingdom it was drawn for, because that is
// all a kingdom is now (DC1.5). The three draws are checked at the recipe
// level here and against real founded bodies in `world::genesis`.
#[test]
fn each_draw_carries_the_anatomy_of_the_kingdom_it_was_asked_for() {
    for s in 0..40 {
        let producer = seed(&mut Rng::from_seed(s), Kingdom::Producer);
        assert!(
            producer
                .tagmata
                .iter()
                .any(|t| t.appendage == Appendage::Plate),
            "a producer draw with nothing to fix with: {producer:?}"
        );
        assert!(
            !producer
                .tagmata
                .iter()
                .any(|t| t.appendage == Appendage::Mouth
                    || t.appendage == Appendage::Limb
                    || t.appendage == Appendage::Vane),
            "a producer draw that eats or moves: {producer:?}"
        );

        let consumer = seed(&mut Rng::from_seed(s), Kingdom::Consumer);
        assert_eq!(
            consumer.tagmata.first().map(|t| t.appendage),
            Some(Appendage::Mouth),
            "a consumer draw with no mouth on its head: {consumer:?}"
        );
        assert!(
            !consumer
                .tagmata
                .iter()
                .any(|t| t.appendage == Appendage::Plate),
            "a consumer draw that also fixes -- the deferred mixotroph: {consumer:?}"
        );

        let decomposer = seed(&mut Rng::from_seed(s), Kingdom::Decomposer);
        assert!(
            !decomposer
                .tagmata
                .iter()
                .any(|t| matches!(t.appendage, Appendage::Mouth | Appendage::Plate)),
            "a decomposer draw with a feeding organ: {decomposer:?}"
        );
    }
}

// Both mouth geometries have to be drawable, or the grazer/predator split the
// slice was ruled to make is a distinction the generator cannot express.
#[test]
fn consumer_draws_reach_both_a_jaw_and_a_crop() {
    let roles: BTreeSet<Option<Role>> = (0..40)
        .map(|s| seed(&mut Rng::from_seed(s), Kingdom::Consumer))
        .filter_map(|r| r.tagmata.first().copied())
        .map(|head| head.appendage.role(head.appendage_shape))
        .collect();
    assert_eq!(
        roles,
        BTreeSet::from([Some(Role::Mass), Some(Role::Limb)]),
        "forty consumer draws reached {roles:?}"
    );
}

#[test]
fn a_recipe_round_trips() {
    let plan = insect();
    let bytes = crate::snapshot::encode(&plan).unwrap();
    assert_eq!(crate::snapshot::decode::<Recipe>(&bytes).unwrap(), plan);
}
