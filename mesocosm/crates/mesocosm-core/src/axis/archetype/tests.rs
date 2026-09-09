// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the authored bodies have to be. Split out of `archetype.rs` at the
//! 600-line ceiling when the roster went from one body to eight.

use super::*;
use crate::body::{BodyDocument, SpeciesId};
use crate::development::develop_body;
use crate::organism::{Kingdom, ecology};
use crate::plan::{Role, classify};
use crate::process::{FeedingMode, Process};

/// The adult mass carving B implies, and the mass DC2's column is taken at.
const BROWSER_CEILING_MG: u64 = 1_284;

fn develop(recipe: &Recipe, mass_mg: u64, seed: u64) -> BodyDocument {
    let soma = crate::axis::Soma::develop(recipe, seed);
    develop_body(SpeciesId(2), recipe, &soma, mass_mg, palette())
        .expect("an archetype develops at its own adult mass")
}

/// The body the recipe names, before any individual varies off it: every
/// stretch at its authored segment count and nothing absent. This is what an
/// archetype *is*; `Soma::develop` is what a member of its line is.
fn authored(recipe: &Recipe, mass_mg: u64) -> BodyDocument {
    let soma = crate::axis::Soma {
        segments: recipe.tagmata.iter().map(|tagma| tagma.segments).collect(),
        absent: Vec::new(),
    };
    develop_body(SpeciesId(2), recipe, &soma, mass_mg, palette())
        .expect("an archetype develops at its own adult mass")
}

/// A body as it would actually be born: anatomy with its allocation seeded.
/// Since PD2 the kingdom readings ask what tissue is doing, so a bare document
/// is not something the ecology ever sees.
fn grown(body: &BodyDocument) -> crate::phenotype::BodyPhenotype {
    crate::phenotype::BodyPhenotype::seed(body.clone())
}

fn ceiling(body: &BodyDocument) -> u64 {
    body.living()
        .map(|part| ecology::part_ceiling_mg(part.half_extent))
        .sum()
}

fn span(body: &BodyDocument, process: Process) -> u32 {
    body.living()
        .filter(|part| body.processes(part.id).contains(&process))
        .map(|part| {
            part.half_extent
                .iter()
                .map(|v| v.unsigned_abs())
                .max()
                .unwrap_or(0)
        })
        .sum()
}

/// Every archetype, its whole-body part count, and what the world reads it as.
///
/// One table rather than eight tests: the roster's claim is that eight
/// recipes reach eight readings through one path, and a table is what that
/// claim looks like.
/// One row: the identifier, the recipe, the whole body's part count, and the
/// two readings the world takes off it.
type Row = (&'static str, fn() -> Recipe, usize, Kingdom, FeedingMode);

const ROSTER: [Row; 8] = [
    (
        "producer_mat",
        producer_mat,
        41,
        Kingdom::Producer,
        FeedingMode::Producer,
    ),
    (
        "producer_shrub",
        producer_shrub,
        41,
        Kingdom::Producer,
        FeedingMode::Producer,
    ),
    (
        "producer_stalk",
        producer_stalk,
        26,
        Kingdom::Producer,
        FeedingMode::Producer,
    ),
    (
        "consumer_browser",
        consumer_browser,
        33,
        Kingdom::Consumer,
        FeedingMode::Grazer,
    ),
    (
        "consumer_pursuit",
        consumer_pursuit,
        30,
        Kingdom::Consumer,
        FeedingMode::Predator,
    ),
    (
        "consumer_armoured",
        consumer_armoured,
        29,
        Kingdom::Consumer,
        FeedingMode::Grazer,
    ),
    (
        "decomposer_crust",
        decomposer_crust,
        36,
        Kingdom::Decomposer,
        FeedingMode::Scavenger,
    ),
    (
        "decomposer_detritivore",
        decomposer_detritivore,
        31,
        Kingdom::Decomposer,
        FeedingMode::Scavenger,
    ),
];

/// **The roster conserves each tier's scale**, which is the invariant §2.3
/// asks for and the one an authored roster is likeliest to break in silence.
///
/// TD7 made plants big on purpose — a stand whose adult mass sits under the
/// grazer's is the inverted pyramid TD6 measured — and a roster sized by eye
/// rather than against the draw it replaces would undo that without touching a
/// constant. So each tier's mean adult mass is held within a fifth of the mean
/// the transitional draw produces for the same tier. Measured against
/// `axis::seed` itself rather than against numbers copied out of a receipt.
#[test]
fn the_roster_conserves_each_tiers_scale() {
    for (kingdom, roster) in [
        (Kingdom::Producer, &PRODUCERS[..]),
        (Kingdom::Consumer, &CONSUMERS[..]),
        (Kingdom::Decomposer, &DECOMPOSERS[..]),
    ] {
        let mut drawn = Vec::new();
        for species in 2u64..12 {
            for seed in 1u64..=10 {
                let mut stream =
                    crate::rng::Rng::from_seed(seed ^ crate::world::RECIPE_SALT ^ species);
                let recipe = crate::axis::seed(&mut stream, kingdom);
                let soma = crate::axis::Soma::develop(&recipe, seed);
                let body = develop_body(
                    SpeciesId(2),
                    &recipe,
                    &soma,
                    100_000,
                    crate::development::PartPalette::primitive(),
                )
                .expect("a drawn recipe develops");
                drawn.push(ceiling(&body));
            }
        }
        let drawn_mean = drawn.iter().sum::<u64>() / drawn.len() as u64;
        let authored_mean = roster
            .iter()
            .map(|recipe| ceiling_of(&recipe()))
            .sum::<u64>()
            / roster.len() as u64;
        let ratio = authored_mean as f64 / drawn_mean as f64;
        assert!(
            (0.8..=1.2).contains(&ratio),
            "{kingdom:?}: the roster means {authored_mean} mg against the draw's {drawn_mean}"
        );
    }
}
#[test]
fn every_archetype_develops_the_body_it_was_authored_as() {
    for (name, recipe, parts, kingdom, mode) in ROSTER {
        let recipe = recipe();
        let body = authored(&recipe, ceiling_of(&recipe));
        assert_eq!(body.living().count(), parts, "{name}: part count");
        assert_eq!(Kingdom::of(&grown(&body)), kingdom, "{name}: kingdom");
        assert_eq!(FeedingMode::of(&grown(&body)), mode, "{name}: feeding mode");
    }
}

/// The adult mass a recipe implies, developed once at a generous mass and
/// measured. Every reading below is taken at the body's own ceiling, which is
/// what the ecology grows a founder to.
fn ceiling_of(recipe: &Recipe) -> u64 {
    ceiling(&authored(recipe, 100_000))
}

/// **Senses presumed.** The ruling this plan was founded on: bodies have
/// senses, limbs and set roles rather than rolling for them. Every fauna
/// archetype carries a working eye and something that contracts; the three
/// producers carry neither, deliberately, which is also what keeps their build
/// multiple at exactly 1.
#[test]
fn every_fauna_archetype_sees_and_contracts_and_no_producer_does() {
    for (name, recipe, _, kingdom, _) in ROSTER {
        let recipe = recipe();
        let body = authored(&recipe, ceiling_of(&recipe));
        let fauna = kingdom != Kingdom::Producer;
        assert_eq!(
            body.performs(Process::Sense),
            fauna,
            "{name}: Sense against fauna={fauna}"
        );
        assert_eq!(
            body.performs(Process::Contract),
            fauna,
            "{name}: Contract against fauna={fauna}"
        );
        if fauna {
            assert!(span(&body, Process::Sense) > 0, "{name}: sees nothing");
        }
    }
}

/// **The position rule, on the two bodies authored around it.** Both wear
/// plates and both perform `Process::Fix`; neither is a producer, because
/// covering is not a canopy.
#[test]
fn a_covered_body_reads_by_its_mouth_rather_than_as_a_producer() {
    for (name, recipe) in [
        ("consumer_armoured", consumer_armoured as fn() -> Recipe),
        ("decomposer_crust", decomposer_crust),
    ] {
        let recipe = recipe();
        let body = authored(&recipe, ceiling_of(&recipe));
        assert!(body.performs(Process::Fix), "{name}: wears plates");
        assert!(!grown(&body).canopy(), "{name}: none of them is held up");
        assert_ne!(Kingdom::of(&grown(&body)), Kingdom::Producer, "{name}");
    }
}

/// And the other side of the same rule: a producer's fronds are lit, so the
/// three producer archetypes still read Producer under it.
#[test]
fn every_producer_archetype_holds_a_lit_frond() {
    for (name, recipe, _, kingdom, _) in ROSTER {
        if kingdom != Kingdom::Producer {
            continue;
        }
        let recipe = recipe();
        let body = authored(&recipe, ceiling_of(&recipe));
        assert!(grown(&body).canopy(), "{name}");
    }
}

/// **No individual develops out of its line's kingdom.** Development varies
/// segment counts and drops an appendage pair here and there; neither may
/// reach a feeding organ, so the reading is a property of the recipe rather
/// than of the individual.
#[test]
fn development_cannot_change_an_archetypes_kingdom() {
    for (name, recipe, _, kingdom, mode) in ROSTER {
        let recipe = recipe();
        let mass = ceiling_of(&recipe);
        for seed in 0u64..256 {
            let body = develop(&recipe, mass, seed);
            assert_eq!(Kingdom::of(&grown(&body)), kingdom, "{name}, seed {seed}");
            assert_eq!(FeedingMode::of(&grown(&body)), mode, "{name}, seed {seed}");
        }
    }
}

/// **And no individual develops out of its senses or its legs either.** The
/// guard in `Soma::develop` spares a feeding organ and a sense organ; limbs it
/// does not, so a body whose legs sit on one short stretch can be born unable
/// to move. Checked here rather than discovered in the census, because a
/// one-in-a-hundred-and-forty-four founder is exactly what a ten-seed world
/// finds and a bench test misses.
#[test]
fn development_cannot_blind_or_strand_a_fauna_archetype() {
    for (name, recipe, _, kingdom, _) in ROSTER {
        if kingdom == Kingdom::Producer {
            continue;
        }
        let recipe = recipe();
        let mass = ceiling_of(&recipe);
        for seed in 0u64..4096 {
            let body = develop(&recipe, mass, seed);
            assert!(
                body.performs(Process::Sense) && span(&body, Process::Sense) > 0,
                "{name}, seed {seed}: born blind"
            );
            assert!(
                body.performs(Process::Contract),
                "{name}, seed {seed}: born unable to move"
            );
        }
    }
}

/// Kin resemble without cloning: what still differs between individuals is a
/// developmental absence, and at variance 1 the segment counts too.
#[test]
fn kin_resemble_and_are_not_clones() {
    for (name, recipe, whole, _, _) in ROSTER {
        let recipe = recipe();
        let mass = ceiling_of(&recipe);
        let counts: std::collections::BTreeSet<usize> = (0u64..256)
            .map(|seed| develop(&recipe, mass, seed).living().count())
            .collect();
        assert!(counts.len() > 1, "{name}: every individual is identical");
        assert!(
            counts.contains(&whole),
            "{name}: the authored body is not among the ones it develops: {counts:?}"
        );
    }
}

/// The two invariants §2.3 asks an archetype palette to hold, asserted rather
/// than commented: every shape classifies as the role whose bank it sits in,
/// and no `Limb` or `Sensor` is priced past the primitive palette's.
/// `PartPalette::validate` refuses either, so developing at all is the proof.
#[test]
fn the_archetype_palette_is_admissible() {
    assert_eq!(classify(CROP), Role::Mass, "the crop must be bulk");
    assert_eq!(classify(SLIM), Role::Mass);
    assert_eq!(classify(BROAD), Role::Mass);
    assert_eq!(classify(LEG), Role::Limb);
    assert_eq!(classify(EYE), Role::Sensor);
    assert_eq!(classify(SPECK), Role::Sensor);
    assert_eq!(classify(BLADE), Role::Plate);
    assert_eq!(classify(PAD), Role::Plate);
    assert_eq!(classify(SHELL), Role::Plate);
    // Both banks the roster fills are full, which is the budget line the plan
    // asked to be checked rather than assumed.
    let palette = palette();
    assert_eq!(palette.mass.admitted().count(), 4, "Mass bank is full");
    assert_eq!(palette.plate.admitted().count(), 4, "Plate bank is full");
    assert_eq!(palette.limb.admitted().count(), 2);
    assert_eq!(palette.sensor.admitted().count(), 2);
}

/// **The DC2 receipt, unmoved.** Every economy number §2.4 predicted for
/// carving B, still measured off a developed browser after the roster and the
/// position rule landed around it.
#[test]
fn the_browser_still_reads_the_carving_b_column() {
    let body = authored(&consumer_browser(), BROWSER_CEILING_MG);
    assert_eq!(body.living().count(), 33, "part count");
    assert_eq!(ceiling(&body), BROWSER_CEILING_MG, "mass_ceiling_mg");
    assert_eq!(span(&body, Process::Contract), 18, "actuator_span");
    assert_eq!(span(&body, Process::Sense), 2, "sensor_span");

    // build multiple 2.40: kept as the exact rational the economy uses so the
    // assertion is the formula rather than a rounded copy of it.
    assert_eq!(BROWSER_CEILING_MG + 18 * 100, 3_084);

    let rent = ecology::upkeep_for_body(BROWSER_CEILING_MG, 18, BROWSER_CEILING_MG, 0);
    assert_eq!(rent, 9, "rent at adult mass, mg/tick");
    assert_eq!(
        ecology::feeding_rate_for_body(BROWSER_CEILING_MG, 18, BROWSER_CEILING_MG),
        49,
        "bite at adult mass, mg"
    );
    assert_eq!(
        ecology::sight_for_body(8, 2, BROWSER_CEILING_MG),
        9,
        "sight horizon, voxels"
    );
    assert_eq!(BROWSER_CEILING_MG * 33 / 100, 423, "breeding gate");
    assert_eq!(BROWSER_CEILING_MG / rent, 142, "ticks of reserve at full");
}

/// A decorative voxel is not a sense organ. Both are `Sensor`-classified and
/// both perform `Sense`; only the span tells them apart, and the whole sight
/// horizon rests on that.
#[test]
fn a_speck_sees_nothing() {
    let body = authored(&consumer_browser(), BROWSER_CEILING_MG);
    let sensing = body
        .living()
        .filter(|part| body.processes(part.id).contains(&Process::Sense))
        .count();
    assert_eq!(sensing, 4, "two eyes and two specks");
    assert_eq!(span(&body, Process::Sense), 2, "and only the eyes carry it");
}

/// **The roster's economy, measured rather than derived.** No archetype may
/// price itself past the band the whole TD series was tuned in: TD7's bound is
/// that no anatomy reads past ~7x, and it is a property of the palette
/// (`development.rs`), so a roster that broke it would move every rate without
/// moving a constant.
#[test]
fn no_archetype_prices_itself_out_of_the_tuned_band() {
    for (name, recipe, _, _, _) in ROSTER {
        let recipe = recipe();
        let body = authored(&recipe, ceiling_of(&recipe));
        let ceiling = ceiling(&body);
        let build =
            (ceiling + u64::from(span(&body, Process::Contract)) * 100) as f64 / ceiling as f64;
        assert!(
            build <= 6.25 + 1.0,
            "{name} builds at {build:.2}x, past the primitive limb's bound"
        );
        // Fertility, the silent one (§2.5): a body whose mean part falls below
        // five voxels never breeds at all.
        let mean = ceiling / body.living().count() as u64;
        assert!(mean * 5 >= 4 * 5, "{name}: mean part {mean} mg is sterile");
    }
}
