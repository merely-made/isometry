// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Total matter is conserved.** TD6's load-bearing invariant.
//!
//! The enclosure has a finite matter budget: producers draw out of the soil
//! column they stand on, rent and travel and decay put it back where the body
//! is, a death releases the reserve it was carrying, and the player's deposit
//! enriches the ground. Light is the one open input, and light is not matter.
//! So the sum of soil, living substance, carrion, and banked reserves is a
//! constant of a run, and mass cannot run away because it has to be somewhere.
//!
//! # Exceptions
//!
//! None. There is no documented sink or source in the tick or in any intent —
//! that is what the checks below assert, milligram-exact, tick by tick, rather
//! than within a tolerance that would hide one.
//!
//! Two things deliberately outside the account, because they are not matter:
//!
//! - **Light.** A producer's income is drawn from soil; the free energy that
//!   powers the drawing never enters the ledger. This is the ruling.
//! - **Ground bricks.** [`Ground`](mesocosm_core::places::Ground) is what is
//!   solid and walkable; the soil store is what can be eaten out of the floor.
//!   Carving air changes the first and not the second, which is why
//!   `Intent::Carve` moves no matter and is checked here saying so.
//!
//! # The dev source is an account, not an exception (DT3)
//!
//! `Intent::PlaceMatter` is the one route by which the enclosure's total
//! changes, and it is a recorded transfer out of
//! [`Account::Dev`](mesocosm_core::flow::Account::Dev). So the conserved
//! quantity is *the enclosure's total less what that account issued*, read off
//! the flow record and subtracted exactly — never a tolerance, and the control
//! below shows what happens to a placement nobody counted.
//!
//! # The instrument is proved, not assumed
//!
//! An absence is evidence only beside a positive control in the same run, so
//! [`conserved`] — the exact check the long runs use — is also handed a world
//! with a conjured milligram and a leaked one, and must report both.

use mesocosm_core::{Crossing, Intent, OrganismId, Placement, World};

/// The ledger, or what is wrong with it. `Ok` is silence; `Err` is the message
/// a failing conservation assertion prints.
///
/// One function, used by the runs that must pass **and** by the controls that
/// must fail, because a check that only ever gets shown conserved worlds has
/// not been shown to detect anything.
fn conserved(world: &World, expected_mg: u64, at: &str) -> Result<(), String> {
    let actual = world.total_matter_mg();
    if actual == expected_mg {
        return Ok(());
    }
    let soil = world.soil().total_mg();
    Err(format!(
        "matter is not conserved {at}: {actual} mg against {expected_mg} mg at genesis \
         ({} mg {}); soil {soil} mg, {} bodies holding {} mg",
        actual.abs_diff(expected_mg),
        if actual > expected_mg {
            "conjured"
        } else {
            "leaked"
        },
        world.organisms.len(),
        actual - soil.min(actual),
    ))
}

#[test]
fn matter_is_conserved_across_a_long_run() {
    // Four seeds across four thousand idle ticks each: births, deaths,
    // grazing, predation, scavenging, decay, dispersal, and the founding
    // cohort dying of old age all happen inside this window.
    //
    // Sixty founders rather than the world's own `FOUNDERS`, deliberately:
    // conservation is a property of the **seams** — one birth, one meal, one
    // death, one step — and this run buys seam coverage by length, which is
    // what four thousand ticks are for. The shipping cohort is proved
    // separately below, where it costs a short run rather than a fifteen-fold
    // one. (2026-08-29 S1)
    for seed in [1u64, 4, 7, 4_242] {
        let mut world = World::new(seed, 60);
        let opening = world.total_matter_mg();
        assert!(opening > 0, "seed {seed} founded an empty enclosure");

        for tick in 1..=4_000 {
            world.apply(Intent::Idle);
            if let Err(why) = conserved(&world, opening, &format!("on tick {tick} of seed {seed}"))
            {
                panic!("{why}");
            }
        }
    }
}

// The same invariant at the size the world actually ships. S1 widened the
// enclosure 16 -> 64 and scaled the founding cohort with its area, so the run
// above — honest about seams — is no longer honest about scale: a cycle that
// closes over 61 bodies and 1,089 columns has not been shown to close over 917
// and 16,641. Short by design, because the seam coverage is the long run's job
// and this one's is the size. (2026-08-29 S1)
#[test]
fn matter_is_conserved_at_the_shipping_cohort() {
    let mut world = World::new(1, mesocosm_core::world::FOUNDERS);
    let opening = world.total_matter_mg();
    assert!(opening > 0, "the wide enclosure founded empty");

    for tick in 1..=200 {
        world.apply(Intent::Idle);
        conserved(
            &world,
            opening,
            &format!("on tick {tick} of the wide enclosure"),
        )
        .expect("conserved");
    }
}

#[test]
fn matter_is_conserved_through_the_played_verbs() {
    // The tick is not the only thing that moves matter. Every acting intent
    // that touches a ledger is exercised here: a meal (burned or built, the
    // body decides), a deposit into the ground, movement paid in substance,
    // and a carve, which must move none at all.
    let mut world = World::new(11, 40);
    let opening = world.total_matter_mg();

    let mut trace = vec![
        Intent::Deposit { mass_mg: 60 },
        Intent::Move { delta: [1, 0, 0] },
        Intent::Move { delta: [-1, 0, 1] },
        Intent::Carve {
            at: world.position().expect("a played critter"),
            radius: 1,
        },
    ];
    // Whatever the played critter can reach, in id order, so the meal is a
    // real one rather than a rejection.
    let me = world.controlled_id().expect("a played critter");
    for organism in world.living().map(|o| o.id).collect::<Vec<_>>() {
        if organism != me {
            trace.push(Intent::Metabolize {
                organism,
                placement: Placement::Planned,
            });
        }
    }

    for (step, intent) in trace.into_iter().enumerate() {
        world.apply(intent);
        conserved(&world, opening, &format!("after played step {step}")).expect("conserved");
    }
}

#[test]
fn the_check_catches_income_conjured_the_way_it_used_to_be() {
    // **The broken control.** Before TD6 a producer's income was a number the
    // world minted: `earn` credited the budget or the body with
    // `producer_income_for_mass(...)` and nothing anywhere was debited. Replay
    // exactly that on top of a conserved tick — one producer, one tick's worth
    // of the old free income — and the check must say so.
    let mut world = World::new(1, 60);
    let opening = world.total_matter_mg();
    world.apply(Intent::Idle);
    conserved(&world, opening, "before the control").expect("the run itself conserves");

    let producer = world
        .living()
        .find(|o| o.kingdom() == mesocosm_core::Kingdom::Producer)
        .map(|o| o.id)
        .expect("seed 1 founds producers");
    let conjured = 11; // a ~300 mg producer's old per-tick fixing income
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == producer)
        .expect("still on the roster")
        .energy_mg += conjured;

    let complaint = conserved(&world, opening, "after conjuring a producer's old income")
        .expect_err("a conjured milligram must not pass");
    assert!(
        complaint.contains("conjured"),
        "the check must name the direction: {complaint}"
    );
    assert!(
        complaint.contains(&conjured.to_string()),
        "and the size of the discrepancy: {complaint}"
    );
}

#[test]
fn the_check_catches_a_leak() {
    // The other direction, and the one the round actually had to close in
    // several places: matter spent and never put anywhere.
    let mut world = World::new(1, 60);
    let opening = world.total_matter_mg();
    world.apply(Intent::Idle);

    world.organisms[0].spend_mass(25);

    let complaint = conserved(&world, opening, "after burning mass into nothing")
        .expect_err("a leaked milligram must not pass");
    assert!(
        complaint.contains("leaked"),
        "the check must name the direction: {complaint}"
    );
}

/// **The four dev intents conserve matter too** (DT3), and the one that brings
/// matter in from outside says so through its own account.
///
/// A forced birth is a transfer out of a parent, a dev kill releases a reserve
/// into the ground and leaves a corpse holding the rest, a demanded epoch
/// boundary moves nothing at all, and a placement moves exactly what the dev
/// source issued. The expected total is raised by that issue and by nothing
/// else, milligram-exact.
#[test]
fn matter_is_conserved_through_the_dev_verbs() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);
    let opening = world.total_matter_mg();
    let mut issued = 0u64;

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

    let trace = vec![
        Intent::ForceBirth { organism: parent },
        Intent::Kill { organism: doomed },
        Intent::EndEpoch,
        Intent::PlaceMatter {
            at: here,
            mass_mg: 900,
        },
        // The refusals, because a refusal that leaked would be the worse
        // defect: over the bound, off the grid, and a body that is already
        // dead.
        Intent::PlaceMatter {
            at: here,
            mass_mg: u64::MAX,
        },
        Intent::PlaceMatter {
            at: [10_000, 0, 0],
            mass_mg: 100,
        },
        Intent::Kill { organism: doomed },
    ];

    for (step, intent) in trace.into_iter().enumerate() {
        world.apply(intent);
        // The ledger holds one tick, reopened at the top of every one, so this
        // reads exactly what this tick issued and cannot double-count.
        issued += mesocosm_core::flow::Account::issued_mg(world.flows());
        conserved(&world, opening + issued, &format!("after dev step {step}")).expect("conserved");
    }
    assert_eq!(
        issued, 900,
        "one accepted placement, and nothing else issued"
    );

    // And the ticks after them: a corpse decaying, a newborn growing, and a
    // fresh epoch running.
    for tick in 1..=200 {
        world.apply(Intent::Idle);
        issued += mesocosm_core::flow::Account::issued_mg(world.flows());
        conserved(&world, opening + issued, &format!("on dev tick {tick}")).expect("conserved");
    }
    assert_eq!(issued, 900, "and an ordinary tick issues nothing");
}

/// **The control for the subtraction.** A placement the check does not count
/// must read as conjured matter, or "subtract what the dev source issued" would
/// be a blanket tolerance wearing an account's name.
#[test]
fn the_check_catches_a_placement_the_dev_source_did_not_account_for() {
    let mut world = World::new(1, 60);
    world.apply(Intent::Idle);
    let opening = world.total_matter_mg();
    let here = world.position().expect("a played critter");

    world.apply(Intent::PlaceMatter {
        at: here,
        mass_mg: 700,
    });
    let issued = mesocosm_core::flow::Account::issued_mg(world.flows());
    assert_eq!(issued, 700, "the account says what it issued");
    conserved(&world, opening + issued, "counting the dev source").expect("conserved");

    let complaint = conserved(&world, opening, "not counting the dev source")
        .expect_err("an uncounted placement must not pass");
    assert!(
        complaint.contains("conjured"),
        "the check must name the direction: {complaint}"
    );
    assert!(
        complaint.contains("700"),
        "and the size of the discrepancy: {complaint}"
    );
}

#[test]
fn matter_is_conserved_through_a_branch_transfer() {
    // P3. A branch leaving one body for another is the seam most able to make
    // matter vanish: severing takes a subtree out of the conservation account,
    // and the whole claim is that it arrives somewhere in the same transaction.
    // Both routes, and a refused one, because a refusal that leaked would be
    // the worse defect.
    for crossing in [Crossing::Regrow, Crossing::Carry] {
        let mut world = World::new(11, 40);
        world.apply(Intent::Idle);
        let opening = world.total_matter_mg();
        let donor = OrganismId(9_700);
        let line = mesocosm_core::SpeciesId(6);
        let here = world.position().expect("a played critter");
        let mut corpse = mesocosm_core::Organism {
            stage: mesocosm_core::Stage::Carrion,
            ..mesocosm_core::Organism::founding(
                donor,
                line,
                mesocosm_core::Kingdom::Producer,
                mesocosm_core::VolumeRef::from_tag(1),
                [2, 2, 2],
                [here[0] + 1, here[1], here[2]],
                900,
            )
        };
        let root = corpse.body().root;
        let frond = corpse
            .phenotype
            .attach(
                mesocosm_core::VolumeRef::from_tag(7),
                400,
                [6, 4, 1],
                mesocosm_core::Attachment {
                    parent: root,
                    offset: [0, 7, 0],
                    yaw: mesocosm_core::Yaw::Zero,
                },
                mesocosm_core::Provenance::founding(),
            )
            .expect("a frond attaches");
        corpse
            .phenotype
            .attach(
                mesocosm_core::VolumeRef::from_tag(9),
                150,
                [7, 1, 1],
                mesocosm_core::Attachment {
                    parent: frond,
                    offset: [13, 0, 0],
                    yaw: mesocosm_core::Yaw::Zero,
                },
                mesocosm_core::Provenance::founding(),
            )
            .expect("and a limb hangs off it");
        world.organisms.push(corpse);
        // A corpse carries matter, so the enclosure's total moved when it was
        // conjured onto the roster; the claim under test is about the transfer.
        let opening = opening.max(world.total_matter_mg());

        // The disfavoured edge first: a refused carry must leave the ledger
        // exactly where it found it.
        let mine = world.controlled().expect("embodied").species;
        {
            let lineages = world.lineages_mut();
            lineages.found(line);
            lineages.set_domain(line, mesocosm_core::Domain(2));
            lineages.set_domain(mine, mesocosm_core::Domain(1));
        }
        world.apply(Intent::Graft {
            organism: donor,
            part: frond,
            crossing: Crossing::Carry,
        });
        conserved(&world, opening, "after a refused carry").expect("conserved");

        // Then the landing one.
        world
            .lineages_mut()
            .set_domain(line, mesocosm_core::Domain(1));
        let outcome = world.apply(Intent::Graft {
            organism: donor,
            part: frond,
            crossing,
        });
        assert!(
            matches!(outcome, mesocosm_core::Outcome::Grafted { .. }),
            "{crossing:?} was refused: {outcome:?}"
        );
        conserved(&world, opening, &format!("after a {crossing:?} transfer")).expect("conserved");

        // And the ticks after it, because a transfer that balanced once and
        // left a body holding tissue nobody accounts for would show up here.
        for tick in 1..=40 {
            world.apply(Intent::Idle);
            conserved(
                &world,
                opening,
                &format!("on tick {tick} after a {crossing:?}"),
            )
            .expect("conserved");
        }
    }
}
