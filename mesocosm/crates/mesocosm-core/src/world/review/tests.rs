// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The review's own claims, in a bare world. (PE3b)
//!
//! A world nobody has played holds no discovery, so every row here is the
//! status quo — which is exactly the case worth pinning at this scale: the
//! table is never empty, the figures reproduce, and the preview is the one
//! `Species::preview` answers. What a *candidate* row does needs a line that
//! has come to something, which is an embodied claim and lives in
//! `tests/embodied/review.rs`.

use super::*;
use crate::organism::ecology::OFFSPRING_COST;
use crate::rules::{EpochRule, WorldRules};

/// A world whose scoring window is short enough that a test can afford several.
fn brisk(seed: u64, founders: u32) -> World {
    World::new(seed, founders).with_rules(
        WorldRules::native()
            .ending(EpochRule::Timed { ticks: 8 })
            .scoring_over(4),
    )
}

fn played(world: &World) -> SpeciesId {
    world.controlled().expect("embodied").species
}

#[test]
fn a_review_built_twice_is_the_same_review() {
    // The determinism the whole panel rests on: every figure is a pure function
    // of the world, and the scoring copies are discarded rather than left
    // somewhere the second call could find them.
    let world = brisk(4_242, 24);
    let species = played(&world);
    let once = world.offers(species);
    let twice = world.offers(species);
    assert_eq!(once, twice, "the same world reviews the same way");
    assert_eq!(world.lineage_budget(species), world.lineage_budget(species));
    assert_eq!(world.draw(), world.draw(), "and draws the same number");
}

#[test]
fn a_review_moves_no_world() {
    // It takes `&self`, so it cannot — and the hash says so rather than the
    // signature, because `World::score` grows copies of this world and a copy
    // that leaked back would move it.
    let world = brisk(77, 24);
    let before = crate::snapshot::state_hash(&world);
    let _ = world.offers(played(&world));
    let _ = world.draw();
    assert_eq!(crate::snapshot::state_hash(&world), before);
}

#[test]
fn the_status_quo_is_always_the_first_row() {
    // *Nothing beat what I have* is a reading off the table, never an absence.
    let world = brisk(9, 24);
    let offers = world.offers(played(&world));
    assert!(!offers.is_empty(), "a review is never an empty table");
    assert_eq!(offers[0].candidate, None, "the status quo leads");
    assert!(!offers[0].takeable(), "and committing it is not a thing");
    assert_eq!(
        offers[0].score.ticks, 4,
        "scored over the window the rules name"
    );
    assert_eq!(
        offers[0].price_mg, 0,
        "a founding program declares nothing, so nothing is charged"
    );
    assert_eq!(offers[0].program, 0, "and its digest is the absence of one");
}

#[test]
fn the_preview_row_is_the_one_species_preview_answers() {
    // The row is not a second realization. Same line, same declared inputs,
    // same body — asked here through `Species::preview` directly.
    let world = brisk(3_300, 24);
    let species = played(&world);
    let prospect = world.prospect(species).expect("the line is living");
    let line = world.lineages.get(species).expect("the line");
    let expected = line
        .preview(world.ruleset(), prospect.founder, prospect.seed)
        .expect("a founder realizes");

    let offers = world.offers(species);
    assert_eq!(offers[0].preview, expected.phenotype.digest());
    assert_eq!(offers[0].program, expected.program);
}

#[test]
fn the_budget_is_the_ecology_s_own_provisioning_arithmetic() {
    // No invented currency: what a founder will have banked is what the birth
    // pass would hand it, and the review reports that rather than a pool.
    let world = brisk(555, 24);
    let species = played(&world);
    let parent = world.controlled().expect("embodied");
    let mass_mg = parent.biomass_mg() / OFFSPRING_COST;

    let prospect = world.prospect(species).expect("the line is living");
    assert_eq!(prospect.founder.mass_mg, mass_mg);
    assert_eq!(
        prospect.founder.conditions.material_mg,
        parent.energy_mg.min(mass_mg),
        "a parent hands over what it has, up to what the body costs"
    );
    assert_eq!(prospect.budget_mg, prospect.founder.conditions.material_mg);
    assert_eq!(world.lineage_budget(species), prospect.budget_mg);
}

#[test]
fn a_line_with_nothing_living_has_no_prospect_and_no_budget() {
    // The honest reading of an extinct line, rather than a founder grown from
    // nobody.
    let world = brisk(11, 24);
    let gone = SpeciesId(9_999);
    assert_eq!(world.prospect(gone), None);
    assert_eq!(world.lineage_budget(gone), 0);
    let offers = world.offers(gone);
    assert_eq!(offers.len(), 1, "the status quo, and nothing to weigh");
    assert_eq!(offers[0].why_not, Some(Untakeable::Extinct));
}

#[test]
fn on_uniform_ground_the_quote_is_the_column_under_the_parent() {
    // **The equality half of the neighbourhood ruling** (PE3, ruled by Mark
    // 2026-09-02). A preview declares the poorest column a birth could land on
    // rather than the one its parent stands over, so the quote can never exceed
    // what a birth in reach affords. Where the ground is uniform there is
    // nothing to choose between them and the two readings coincide, which is
    // what keeps the change a ceiling rather than a discount.
    //
    // A world nobody has ticked is exactly that case: `Soil::seeded` gives
    // every column the same milligrams, and no rent, decay or percolation has
    // moved any of them yet.
    let world = brisk(4_242, 24);
    let species = played(&world);
    let parent = world
        .controlled()
        .filter(|organism| organism.species == species)
        .or_else(|| world.living().find(|organism| organism.species == species))
        .expect("a living parent to quote against");

    let soil = world.soil();
    let mut columns = (-BIRTH_SCATTER..=BIRTH_SCATTER)
        .flat_map(|dz| (-BIRTH_SCATTER..=BIRTH_SCATTER).map(move |dx| (dx, dz)));
    let under = soil.matter_mg(soil.column_at(parent.position));
    assert!(
        columns.all(|(dx, dz)| {
            let at = [
                parent.position[0] + dx,
                parent.position[1],
                parent.position[2] + dz,
            ];
            soil.matter_mg(soil.column_at(at)) == under
        }),
        "a world at tick zero has uniform soil, which is the case this pins"
    );
    assert!(under > 0, "and it is holding something");

    let prospect = world.prospect(species).expect("a living line has one");
    assert_eq!(
        prospect.founder.conditions.ground_mg, under,
        "on uniform ground the poorest column in reach is the one underfoot"
    );
}
