// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn same_seed_builds_the_same_world() {
    assert_eq!(World::new(1234, 12), World::new(1234, 12));
}

#[test]
fn different_seeds_build_different_worlds() {
    assert_ne!(World::new(1, 12).organisms, World::new(2, 12).organisms);
}

#[test]
fn out_of_reach_organisms_are_refused() {
    let mut world = World::new(5, 4);
    world.organisms.push(Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            OrganismId(900),
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(2),
            [1, 1, 1],
            [500, 0, 0],
            100,
        )
    });
    let outcome = world.apply(Intent::Metabolize {
        organism: OrganismId(900),
        placement: Placement::Explicit {
            parent: world.body().unwrap().root,
            offset: [1, 0, 0],
            yaw: Yaw::Zero,
        },
    });
    assert!(
        matches!(
            outcome,
            Outcome::Rejected(Rejection::OutOfReach(crate::process::Unmet::TooFar {
                distance: 500,
                ..
            }))
        ),
        "got {outcome:?}"
    );
}

#[test]
fn rejected_intents_still_advance_the_tick() {
    let mut world = World::new(11, 2);
    let before = world.tick;
    let outcome = world.apply(Intent::Metabolize {
        organism: OrganismId(4242),
        placement: Placement::Explicit {
            parent: world.body().unwrap().root,
            offset: [0, 0, 0],
            yaw: Yaw::Zero,
        },
    });
    assert_eq!(
        outcome,
        Outcome::Rejected(Rejection::NoSuchOrganism(OrganismId(4242)))
    );
    assert_eq!(world.tick, before + 1);
}

#[test]
fn grounded_population_scale_is_deterministic_and_cohort_conserving() {
    // This is the G3 scale receipt. It deliberately uses the real founding
    // path, so both tiers, actual ground, body-derived drives, reproduction,
    // and the graph boundary are exercised together. Wall-clock cost belongs
    // to the release example; a test only asserts portable facts.
    const POPULATION: u32 = 300;
    // The first pass establishes the tier line and the second exercises the
    // resulting mix. Longer wall-clock sampling belongs in the release probe.
    const TICKS: u32 = 2;
    let run = || {
        let mut world = World::new(4_242, POPULATION - 1);
        assert_eq!(world.organisms.len(), POPULATION as usize);
        for _ in 0..TICKS {
            assert!(matches!(world.apply(Intent::Idle), Outcome::Idled));
            assert_grounded_near(&world);
        }
        world
    };

    let a = run();
    let b = run();
    assert_eq!(
        crate::snapshot::state_hash(&a),
        crate::snapshot::state_hash(&b)
    );

    let (actual_members, actual_biomass, actual_energy) = a
        .organisms
        .iter()
        .filter(|organism| organism.is_alive() && organism.tier == Tier::Far)
        .fold((0u64, 0u64, 0u64), |(count, biomass, energy), organism| {
            (
                count + 1,
                biomass + organism.biomass_mg(),
                energy + organism.energy_mg,
            )
        });
    assert!(
        actual_members > 0,
        "the population never exercised the far tier"
    );
    let cohorts = a.far_cohorts();
    assert_eq!(
        crate::cohort::conserved_totals(&cohorts),
        (actual_members, actual_biomass, actual_energy),
        "cohort formation lost a scalar from the population"
    );
}
