// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the panel says, in the exact words a player reads.
//!
//! Split out of `vitals.rs` at the 600-line ceiling when PE2 gave the panel a
//! discovery to report. These are string assertions on purpose: the readings
//! contract rules what the panel may and may not say — a fact with its window
//! on it, never a bare percentage, and never the population instrument's test
//! verdicts — and only the literal words can hold that.

use super::*;

#[test]
fn a_world_with_nobody_in_it_reads_dead() {
    let vitals = reading(None, 1_000, Some("not enough energy"), None);
    assert!(vitals.is_dead());
    assert_eq!(vitals.fullness, 0.0);
    // The death state stands alone: a notice from the same batch is not
    // shown beside "dead", because a dead critter refused nothing.
    assert_eq!(vitals.notice, None);
}

#[test]
fn the_bar_measures_against_the_session_high_water() {
    let world = World::new(0x00A7_7AC4, 8);
    let energy = world.energy_mg().expect("the world starts embodied");
    let full = vitals_of(&world, energy, None, None);
    assert_eq!(full.energy_mg, Some(energy));
    assert!((full.fullness - 1.0).abs() < f32::EPSILON);
    let half = vitals_of(&world, energy * 2, None, None);
    assert!((half.fullness - 0.5).abs() < 0.001);
}

/// The playtest's three silences, each with a word for it.
#[test]
fn the_refusals_the_playtest_hit_have_plain_words() {
    assert_eq!(
        notice_in(&[Outcome::Rejected(Rejection::InsufficientMass)]),
        Some("not enough energy")
    );
    assert_eq!(
        notice_in(&[Outcome::Rejected(Rejection::Disembodied)]),
        Some("no body")
    );
    assert_eq!(notice_in(&[Outcome::Moved, Outcome::Idled]), None);
}

/// The warning says what moved and over what window, or says nothing.
#[test]
fn a_warning_carries_its_evidence_and_only_arrives_when_it_is_true() {
    let quiet = Trend {
        replacement_ticks: 240,
        matured: 4,
        died: 2,
        stand_ticks: 60,
        stand_change_mg: 900,
        grazed_mg: 300,
        shortfall_ticks: 0,
    };
    assert_eq!(warning_words(&quiet), None);
    assert_eq!(replacement_words(&quiet), "4 matured, 2 died in 240 ticks");

    let short = Trend {
        stand_change_mg: -7_930,
        grazed_mg: 15_771,
        shortfall_ticks: mesocosm_core::WARN_AFTER_TICKS,
        ..quiet
    };
    let words = warning_words(&short).expect("a real shortfall says so");
    assert!(words.contains("7930 mg lost over the last 60"));
    assert!(words.contains("mouths took 15771 mg in the same window"));
    assert!(words.contains("ticks"), "and the window it moved over");
    assert!(
        !words.contains('%'),
        "never an unexplained percentage: {words}"
    );
    for verdict in ["breathes", "thins", "boils", "collapses"] {
        assert!(
            !words.contains(verdict),
            "the instrument's verdicts are not player language: {words}"
        );
    }
}

/// PD2's four states, in the words a player reads. Each says what is true
/// and, when something is not working, what would change it.
#[test]
fn the_gland_reads_differently_in_each_of_its_four_states() {
    let part = mesocosm_core::PartId(3);
    let allocated = Gland {
        sites: vec![(part, 5)],
        cells: 5,
        potency_mg: 115,
        ground_mg: 206,
        charged: true,
        rent_mg: 2,
        lost: Vec::new(),
    };
    let words = gland_words(&allocated);
    assert_eq!(words.tissue, "5 cells of part 3", "located, and how much");
    assert_eq!(words.sting, "115 mg a bite", "and it is working");
    assert_eq!(words.rent, "2 mg a tick", "and this is what it costs");

    // Dormant: the two numbers side by side, because their difference is
    // the thing a player can do something about.
    let dry = Gland {
        ground_mg: 100,
        charged: false,
        ..allocated.clone()
    };
    let words = gland_words(&dry);
    assert_eq!(
        words.sting,
        "dry: this ground holds 100 mg, the gland needs 115"
    );
    assert_eq!(words.rent, "2 mg a tick", "and it is still being paid for");

    // Severed: the consequence is gone, and the branch still says what it
    // used to do.
    let lost = Gland {
        sites: Vec::new(),
        cells: 0,
        potency_mg: 0,
        rent_mg: 0,
        lost: vec![part],
        ..allocated
    };
    let words = gland_words(&lost);
    assert_eq!(words.tissue, "gone with part 3");
    assert_eq!(words.sting, "nothing left to sting with");
    assert_eq!(words.rent, "0 mg a tick");
}

/// PE2's reading: **evidence and route, not only the thing unlocked.**
/// A player who cannot see what taught them is back in a diet tree whether
/// or not the code is.
#[test]
fn a_discovery_says_what_it_is_how_it_was_come_by_and_what_it_grants() {
    let condition = mesocosm_core::discovery::conditions()
        .into_iter()
        .find(|found| found.name == "mesocosm:endured-hunger")
        .expect("the table holds it");
    let discovery = Discovery {
        tick: 940,
        epoch: 0,
        condition: condition.id(),
        route: mesocosm_core::Input::Endurance,
        evidence: mesocosm_core::Evidence::Endured {
            stress: mesocosm_core::Stress::Hunger,
            ticks: 100,
        },
        candidate: condition.grants,
        source: mesocosm_core::Source::Endured,
        digest: 0x1234,
    };
    let words = discovery_words(&discovery);
    assert_eq!(words.what, "endured hunger");
    assert_eq!(
        words.route, "endured: hunger for 100 ticks",
        "the route and the quantity that carried it"
    );
    assert_eq!(
        words.grants, "secrete on a plate",
        "and where it could go, rather than a claim that it is there"
    );
}

/// Evidence that unlocked nothing is still evidence, and the panel says
/// which condition refused it and why.
#[test]
fn a_meal_that_taught_nothing_still_says_what_it_offered_and_what_refused() {
    let hunger = mesocosm_core::discovery::conditions()
        .into_iter()
        .find(|found| found.name == "mesocosm:endured-hunger")
        .expect("the table holds it");
    let observation = Observation {
        tick: 12,
        route: mesocosm_core::Input::Meal,
        evidence: mesocosm_core::Evidence::Meal {
            donor: mesocosm_core::SpeciesId(4),
            part: mesocosm_core::PartId(0),
            role: mesocosm_core::Role::Mass,
            mass_mg: 260,
        },
        matched: None,
        missed: vec![(hunger.id(), mesocosm_core::Miss::UndeclaredInput)],
    };
    let words = observation_words(&observation).expect("it taught nothing, so it is said");
    assert!(words.contains("bulk part 0 of line 4, 260 mg"), "{words}");
    assert!(
        words.contains("endured hunger: not a question this asks"),
        "the refusal is a different fact from a rule that went unmet: {words}"
    );

    // And when a condition took it, the row is not there at all: the three
    // discovery rows above have already said what it was, and a panel that
    // says a thing twice stops being read.
    let taken = Observation {
        matched: Some(hunger.id()),
        ..observation
    };
    assert_eq!(observation_words(&taken), None);
}

/// A refusal a hand can actually produce is answered in words, not in a
/// variant name. The boundary itself stays in the outcome.
#[test]
fn a_development_that_would_not_validate_says_why_in_plain_words() {
    assert_eq!(
        refusal_words(&Rejection::Refused(Refusal::SiteMismatch {
            part: mesocosm_core::PartId(0),
            process: mesocosm_core::ProcessRef {
                definition: mesocosm_core::DefinitionDigest(1),
            },
        })),
        "that shape does not do that"
    );
    assert_eq!(
        refusal_words(&Rejection::Refused(Refusal::Disconnected(
            mesocosm_core::PartId(3)
        ))),
        "an organ is one piece of tissue"
    );
    assert_eq!(
        notice_in(&[Outcome::Expressed {
            part: mesocosm_core::PartId(3),
            cost_mg: 115,
            revision: 1,
        }]),
        Some("rebuilt")
    );
}

/// TD4's half of the feedback: the player no longer chooses what a meal
/// becomes, so the panel has to say what it became.
#[test]
fn a_landed_meal_says_which_way_the_body_took_it() {
    assert_eq!(
        notice_in(&[Outcome::Burned {
            organism: mesocosm_core::OrganismId(3),
            energy_mg: 120,
        }]),
        Some("burned")
    );
    assert_eq!(
        notice_in(&[Outcome::Incorporated {
            part: mesocosm_core::PartId(1),
        }]),
        Some("grew")
    );
    // PE2's verb says which thing happened: an organ came off something and
    // onto you, and the player picked it rather than the plan.
    assert_eq!(
        notice_in(&[Outcome::Consumed {
            part: mesocosm_core::PartId(4),
            from: mesocosm_core::OrganismId(9),
            from_part: mesocosm_core::PartId(2),
            mass_mg: 400,
        }]),
        Some("took the organ")
    );
}

/// P3's two rows: whose branch it was, and what it is doing here.
#[test]
fn a_transferred_branch_says_where_it_came_from_and_what_it_is_doing() {
    let carried = Graft {
        tick: 340,
        recipient: mesocosm_core::OrganismId(0),
        donor: mesocosm_core::OrganismId(759),
        donor_line: mesocosm_core::SpeciesId(2),
        donor_part: mesocosm_core::PartId(31),
        root: mesocosm_core::PartId(41),
        parts: vec![mesocosm_core::PartId(41), mesocosm_core::PartId(42)],
        mass_mg: 20,
        crossing: Crossing::Carry,
        verdict: mesocosm_core::Verdict::Adapter,
        cost_mg: 72,
        revision: 1,
    };

    // Provenance first, because it is the fact a graft has that growing does
    // not: the parts, the part they came off, and the line.
    let words = graft_words(&carried, false);
    assert_eq!(words.taken, "2 parts from part 31 of line 2");
    assert_eq!(
        words.terms, "carried on part 41 — needs an adapter, doing nothing yet",
        "the crossing, the verdict, and the consequence in one line"
    );

    // Whether it works is read off the body, not inferred from the verdict:
    // an adapted branch that has since been given an adapter stops calling
    // itself idle without the table changing its mind.
    let repaired = graft_words(&carried, true);
    assert_eq!(
        repaired.terms,
        "carried on part 41 — needs an adapter, working"
    );

    let regrown = graft_words(
        &Graft {
            crossing: Crossing::Regrow,
            verdict: mesocosm_core::Verdict::Refused,
            parts: vec![mesocosm_core::PartId(41)],
            ..carried
        },
        true,
    );
    assert_eq!(regrown.taken, "1 part from part 31 of line 2");
    assert_eq!(regrown.terms, "regrown on part 41 — refused, working");
}

#[test]
fn a_landed_branch_says_which_crossing_it_took() {
    let landed = |crossing| {
        notice_in(&[Outcome::Grafted {
            root: mesocosm_core::PartId(41),
            parts: 2,
            from: mesocosm_core::OrganismId(759),
            from_part: mesocosm_core::PartId(31),
            mass_mg: 20,
            crossing,
            verdict: mesocosm_core::Verdict::Native,
        }])
    };
    assert_eq!(landed(Crossing::Carry), Some("carried the branch"));
    assert_eq!(landed(Crossing::Regrow), Some("regrew the branch"));

    // And the two refusals P3 added, in words that say what would make it
    // possible rather than naming an error.
    assert_eq!(
        refusal_words(&Rejection::WholeBody(mesocosm_core::PartId(0))),
        "that is the whole of it"
    );
    assert_eq!(
        refusal_words(&Rejection::Incompatible {
            from: mesocosm_core::Domain(2),
            into: mesocosm_core::Domain(1),
        }),
        "that tissue will not go in you"
    );
}
