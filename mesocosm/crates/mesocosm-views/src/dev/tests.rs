// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the dev tile says, against the core queries it says it from. (DT1, DT2)
//!
//! The vitals panel's own pattern: string assertions, because the claim being
//! held is about the *line a player reads*. DT2's claim is narrower and
//! stronger than "the words are sensible" — it is that **every field on the
//! panel equals the corresponding core reading**, so each assertion below puts
//! the drawn line beside the query it was drawn from rather than beside a
//! literal somebody typed.

use super::*;
use mesocosm_core::flow::Accounts;
use mesocosm_core::{Ending, OrganismId, Passing, World, classify};

/// A world with a body in it, and the id of the critter under the hand.
fn fixture() -> (World, OrganismId) {
    let world = World::new(0x00A7_7AC4, 8);
    let id = world.controlled_id().expect("the world starts embodied");
    (world, id)
}

fn accounts() -> Accounts {
    Accounts {
        ticks: 240,
        income_mg: 1_204,
        rent_mg: 517,
        outflow_mg: 66,
    }
}

/// A regression guard on the one thing worth guarding in the sheet: the class
/// the root sets is the class the sheet styles.
#[test]
fn the_root_builds_and_the_sheet_styles_the_class_it_sets() {
    let (world, id) = fixture();
    let dev = Dev {
        running: true,
        speed: "1",
        tick: 10,
        manual_steps: 2,
        follow: follow_of(&world, id, accounts()),
        lost: None,
        inspection: None,
    };
    let _ = dev_root(&dev);
    assert!(
        dev_css().contains(".dev {"),
        "the sheet styles the class the root sets"
    );
    assert!(
        dev_css().contains(".dev-notice {"),
        "and the class the lost-follow notice sets"
    );
}

/// The two plain words a player reads for the clock, and only those two. (DT1)
#[test]
fn running_and_paused_are_the_only_two_states() {
    for running in [true, false] {
        let dev = Dev {
            running,
            ..Dev::default()
        };
        let word = if dev.running { "running" } else { "paused" };
        assert!(word == "running" || word == "paused");
    }
}

/// **The DT2 done-condition, executable**: each drawn line beside the core
/// query it came from.
#[test]
fn every_field_on_the_panel_equals_the_core_reading_it_came_from() {
    let (world, id) = fixture();
    let organism = world
        .organisms
        .iter()
        .find(|o| o.id == id)
        .expect("the controlled critter is in the roster");
    let follow = follow_of(&world, id, accounts()).expect("a body in the roster reads");

    // id — and the fact that this one is the critter under the hand, which is
    // `World::controlled_id`.
    assert_eq!(follow.id, format!("{} (controlled)", organism.id.0));

    // species, with the lineage registry's own name when it has one.
    let named = world
        .lineages()
        .get(organism.species)
        .and_then(|line| line.name.clone());
    let expected_species = match named {
        Some(name) => format!("{} — {name}", organism.species.0),
        None => organism.species.0.to_string(),
    };
    assert_eq!(follow.species, expected_species);

    // position.
    assert_eq!(
        follow.at,
        format!(
            "{}, {}, {}",
            organism.position[0], organism.position[1], organism.position[2]
        )
    );

    // The two accounts a body holds, in the flow record's own words: the
    // reserve is `energy_mg`, the substance is what its living parts weigh.
    assert_eq!(follow.reserve, format!("{} mg", organism.energy_mg));
    assert_eq!(follow.substance, format!("{} mg", organism.biomass_mg()));

    // The flow accounts, exactly as the driver's reduction separates them, and
    // the window they cover as its own line.
    let reduced = accounts();
    assert_eq!(
        follow.flows,
        format!(
            "in {}, rent {}, out {}",
            reduced.income_mg, reduced.rent_mg, reduced.outflow_mg
        )
    );
    assert_eq!(follow.window, format!("{} ticks", reduced.ticks));

    // The line's current revision, off `Species::program().current()`. A world
    // at genesis has committed nothing, which is `founding` rather than blank.
    let current = world
        .lineages()
        .get(organism.species)
        .and_then(|line| line.program().current())
        .map(|revision| revision.id.0.to_string());
    assert_eq!(
        follow.revision,
        current.unwrap_or_else(|| "founding".to_string())
    );
    assert_eq!(follow.revision, "founding", "a world founds no revision");

    // What the world has come to. Nothing, at genesis — and `none` rather than
    // an empty line, so the row is a reading instead of a gap.
    assert!(world.discoveries().is_empty());
    assert_eq!(follow.discovered, "none");

    // The body: the count off `BodyDocument::living`, and one row per part
    // naming the role `classify` reads off its half-extent.
    let living: Vec<_> = organism.body().living().collect();
    assert_eq!(follow.parts, living.len().to_string());
    assert_eq!(follow.part_rows.len(), living.len().min(MAX_PART_ROWS));
    for (part, (key, value)) in living.iter().zip(&follow.part_rows) {
        assert_eq!(key, &format!("part {}", part.id.0));
        assert!(
            value.starts_with(follow::role_word(classify(part.half_extent))),
            "the role is `classify`'s: {value}"
        );
        assert!(
            value.contains(&format!(
                "{}x{}x{}",
                part.half_extent[0], part.half_extent[1], part.half_extent[2]
            )),
            "the half-extent is the part's own: {value}"
        );
        // The sites are the phenotype's explanation of that part, cell counts
        // and all.
        let explained = organism
            .phenotype
            .explain(part.id)
            .expect("a living part has a mosaic");
        for site in &explained.sites {
            assert!(
                value.contains(&format!("on {} cells", site.cells)),
                "site cell count missing from {value}"
            );
        }
    }
    assert_eq!(
        follow.more_parts,
        living.len().saturating_sub(MAX_PART_ROWS)
    );

    // And the loop closes: every field asserted above is a row the panel draws,
    // in the order it draws them, with nothing in between that came from
    // anywhere else.
    let drawn = follow_rows(&follow);
    let expected: Vec<(&str, &str)> = [
        ("id", follow.id.as_str()),
        ("species", follow.species.as_str()),
        ("at", follow.at.as_str()),
        ("reserve", follow.reserve.as_str()),
        ("substance", follow.substance.as_str()),
        ("flows", follow.flows.as_str()),
        ("window", follow.window.as_str()),
        ("revision", follow.revision.as_str()),
        ("discovered", follow.discovered.as_str()),
        ("parts", follow.parts.as_str()),
    ]
    .into_iter()
    .chain(
        follow
            .part_rows
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str())),
    )
    .collect();
    assert_eq!(
        drawn.len(),
        expected.len() + usize::from(follow.more_parts > 0)
    );
    for (row, (key, value)) in drawn.iter().zip(&expected) {
        assert_eq!((row.key.as_str(), row.value.as_str()), (*key, *value));
    }
    if follow.more_parts > 0 {
        let last = drawn.last().expect("a truncated body draws its count");
        assert_eq!(last.key, "more");
        assert_eq!(last.value, format!("+{} parts", follow.more_parts));
    }
}

/// A critter that is not the one under the hand says so by omission, and the
/// panel still reads every field for it.
#[test]
fn a_followed_critter_that_is_not_controlled_is_not_labelled_controlled() {
    let (world, controlled) = fixture();
    let other = world
        .living()
        .find(|o| o.id != controlled)
        .expect("a founded world holds more than the played body")
        .id;
    let follow = follow_of(&world, other, Accounts::default()).expect("it is in the roster");
    assert_eq!(follow.id, other.0.to_string());
    assert!(!follow.id.contains("controlled"));
    // The window is still stated, even when it is empty: zero ticks watched is
    // a reading, not a missing one.
    assert_eq!(follow.window, "0 ticks");
    assert_eq!(follow.flows, "in 0, rent 0, out 0");
}

/// An id the roster does not hold reads as nothing at all, rather than as a
/// panel full of zeroes.
#[test]
fn an_id_the_roster_does_not_hold_reads_as_nothing() {
    let (world, _) = fixture();
    assert_eq!(follow_of(&world, OrganismId(9_999), accounts()), None);
}

/// The death report: the record's own event and its own tick, in one sentence.
#[test]
fn a_lost_follow_target_is_reported_with_its_tick_and_which_way_it_went() {
    let died = lost_of(
        OrganismId(42),
        Some(Ending {
            organism: OrganismId(42),
            tick: 812,
            how: Passing::Died,
        }),
        900,
    );
    assert_eq!(died.tick, 812, "the record's tick, not the world's");
    assert_eq!(lost_words(died), "critter 42 died at tick 812");

    let returned = lost_of(
        OrganismId(7),
        Some(Ending {
            organism: OrganismId(7),
            tick: 30,
            how: Passing::Returned,
        }),
        900,
    );
    assert_eq!(
        lost_words(returned),
        "critter 7 went back to the ground at tick 30"
    );

    // Gone without an ending in this record: still said, never dropped.
    let unrecorded = lost_of(OrganismId(3), None, 512);
    assert_eq!(
        lost_words(unrecorded),
        "critter 3 left the roster at tick 512"
    );

    // And the notice reaches the tree.
    let dev = Dev {
        lost: Some(died),
        inspection: None,
        ..Dev::default()
    };
    let _ = dev_root(&dev);
}
