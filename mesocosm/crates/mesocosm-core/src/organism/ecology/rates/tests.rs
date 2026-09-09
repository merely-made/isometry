// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The life-history allometry's own receipts.
//!
//! Split out of `rates.rs` on 2026-09-01, at the ceiling, when PD2 gave rent a
//! third term to price. The same split-before-adding move that put these
//! numbers in a file of their own in the first place.

use super::*;
use crate::organism::Kingdom;

#[test]
fn rent_prices_the_body_plan_not_only_the_mass() {
    // TD7's asymmetry, and that it is a reading rather than a constant.
    // Same mass, same adult ceiling, different build.
    let mass = 1_000;
    let ceiling = 2_000;
    let sessile = upkeep_for_body(mass, 0, ceiling, 0);

    assert_eq!(
        sessile,
        upkeep_for_body(mass, 0, 1, 0),
        "a body with no actuator pays the mass rent whatever its ceiling"
    );
    assert_eq!(
        sessile,
        UPKEEP_BASE_MG + three_quarter_power(mass) / UPKEEP_SCALE,
        "and pays exactly what it paid before TD7"
    );

    // Four palette limbs: half-extent [4,1,1], so they swing 4 apiece.
    let motile = upkeep_for_body(mass, 4 * 4, ceiling, 0);
    assert!(
        motile > sessile,
        "moving cost nothing: {motile} against {sessile}"
    );
    assert!(
        upkeep_for_body(mass, 4 * 8, ceiling, 0) > motile,
        "twice the swing did not cost more"
    );
    // Bounded by construction: a limb swings 4 and holds a 64 mg ceiling,
    // so a body of nothing but limbs — the most motile anatomy the palette
    // can express — reads 4 * 100 / 64 swing per reference segment and
    // tops out near 7x however many of them it grows.
    let all_limbs = upkeep_for_body(mass, 4 * 100, 64 * 100, 0);
    assert!(
        all_limbs <= sessile * 8,
        "the surcharge outran the bound the body plan puts on it: \
         {all_limbs} against {sessile}"
    );
}

#[test]
fn the_horizon_reads_the_same_build_the_rent_does() {
    // TD11's symmetry, stated the way TD7's and TD9's are: a blind plan
    // reads *exactly* the base it was handed, and sensory build buys
    // horizon at the one build multiple.
    let ceiling = 2_000;
    assert_eq!(
        sight_for_body(8, 0, ceiling),
        8,
        "a body with no sense organ sees exactly what it always did"
    );
    assert_eq!(
        sight_for_body(8, 0, 1),
        8,
        "whatever its ceiling, and with a severed plan's ceiling of nothing"
    );

    let sensing = sight_for_body(8, 4, ceiling);
    assert!(sensing > 8, "sensory anatomy bought no horizon: {sensing}");
    assert!(
        sight_for_body(8, 8, ceiling) > sensing,
        "twice the sensory span did not see further"
    );
    // Scale-free: doubling the plan and its sense organs together leaves
    // the horizon where it was, so it reads build rather than size.
    assert_eq!(
        sight_for_body(8, 8, 2 * ceiling),
        sensing,
        "a bigger body with proportionally the same senses saw differently"
    );
    // Bounded by construction: a body of nothing but the palette's
    // [1,1,1] sensor reads 121/21 and tops out at 46 voxels.
    assert_eq!(
        sight_for_body(8, 100, 21 * 100),
        46,
        "no anatomy may see the enclosure"
    );
}

#[test]
fn the_bite_reads_the_same_build_the_rent_does() {
    // TD9's symmetry, stated the way TD7's is: the income side is the same
    // allometric base times the same multiple, so a body that pays a
    // motility surcharge earns a return on it.
    let mass = 1_000;
    let ceiling = 2_000;
    let sessile = feeding_rate_for_body(mass, 0, ceiling);

    assert_eq!(
        sessile,
        feeding_rate_for_body(mass, 0, 1),
        "a body with no actuator bites its mass whatever its ceiling"
    );
    assert_eq!(
        sessile,
        allometric_rate(GRAZES_BASE_MG, mass),
        "and bites exactly what it bit before TD9"
    );
    assert_eq!(
        decay_rate_for_body(mass, 0, ceiling),
        allometric_rate(DECAYS_BASE_MG, mass),
        "the scavenger's sessile draw moved"
    );

    // Four palette limbs, the fixture TD7's rent test uses.
    let motile = feeding_rate_for_body(mass, 4 * 4, ceiling);
    assert!(motile > sessile, "limbs earned nothing: {motile}/{sessile}");
    assert!(
        feeding_rate_for_body(mass, 4 * 8, ceiling) > motile,
        "twice the swing did not earn more"
    );

    // And the symmetry is one function rather than two agreeing comments:
    // rent and income both scale by `build_multiple` of the same body, and
    // that is the whole of what "symmetric with TD7" means here. (The two
    // *rates* still round separately, at their own scales — rent divides by
    // `UPKEEP_SCALE` and income does not — so the shared thing to assert is
    // the multiple, not a ratio of two floors.)
    assert_eq!(
        build_multiple(4 * 4, ceiling),
        (ceiling + 4 * 4 * REFERENCE_SEGMENT_MG, ceiling)
    );
    assert_eq!(
        motile,
        GRAZES_BASE_MG * three_quarter_power(mass) * (ceiling + 1_600)
            / (three_quarter_power(REFERENCE_MASS_MG) * ceiling),
        "the bite is not the formula the doc states"
    );
}

#[test]
fn producers_creep_and_unlimbed_consumers_do_not() {
    // TD9's second ruling, and the line it must not cross. The exception is
    // written against the feeding mode, so it reaches every producer and no
    // consumer — including the unlimbed consumers TD8 made sessile, who are
    // the free lunch this must not reopen.
    let world = crate::world::World::new(2, 60);
    let mut creeping = 0;
    for organism in world.organisms.iter().filter(|o| o.is_alive()) {
        match organism.kingdom() {
            Kingdom::Producer => {
                assert_eq!(organism.actuator_span(), 0, "a producer grew an actuator");
                assert!(travels(organism), "a producer cannot spread");
                creeping += 1;
            },
            _ if organism.actuator_span() == 0 => panic!(
                "the roster founds nothing sessile that eats: {:?}",
                organism.id
            ),
            _ => assert!(travels(organism)),
        }
    }
    assert!(creeping > 0, "the seed founds a stand");
    // The line the exception must not cross, kept as a fixture rather than
    // as a founding draw: **DC4's roster founds no sessile consumer at
    // all**, every fauna archetype having limbs by the ruling, so the free
    // lunch TD8 closed has to be built to be checked.
    let sessile = crate::organism::Organism::founding(
        crate::organism::OrganismId(1),
        crate::body::SpeciesId(9),
        Kingdom::Consumer,
        crate::body::VolumeRef::from_tag(1),
        [2, 2, 2],
        [0, 0, 0],
        1_000,
    );
    assert_eq!(sessile.actuator_span(), 0, "a crop is not an actuator");
    assert_eq!(sessile.kingdom(), Kingdom::Consumer);
    assert!(
        !travels(&sessile),
        "an unlimbed consumer got the producer's budget"
    );
}

#[test]
fn breeding_asks_the_body_plans_own_adult_mass() {
    // TD8's gate is a share of the ceiling, so two bodies of the same mass
    // and different plans get different answers — which is the whole point
    // of replacing an absolute floor.
    let small = breeding_mass_mg(400);
    let large = breeding_mass_mg(3_200);
    assert!(
        large > small,
        "a bigger plan must have to grow further: {large} against {small}"
    );
    assert_eq!(breeding_mass_mg(0), 0, "a bodiless plan asks for nothing");
    // And it is a share, not a step: doubling the plan doubles the bar.
    assert_eq!(breeding_mass_mg(800), 2 * small);
}

#[test]
fn a_body_with_no_actuator_has_no_dispersal_budget() {
    // TD8: `locomotion` still floors at one for the drive selector, and
    // dispersal no longer reads it. A body that carries nothing contractile
    // gets no steps, hungry or not; anything limbed is unchanged.
    let world = crate::world::World::new(3, 60);
    let (mut sessile, mut motile) = (0, 0);
    for organism in world.organisms.iter().filter(|o| o.is_alive()) {
        if organism.actuator_span() == 0 {
            assert_eq!(
                organism.locomotion(),
                1,
                "the drive selector lost its floor"
            );
            assert_eq!(
                dispersal_for(organism),
                0,
                "a body with no actuator kept a step: {:?}",
                organism.id
            );
            sessile += 1;
        } else {
            assert!(dispersal_for(organism) >= 1);
            motile += 1;
        }
    }
    assert!(sessile > 0 && motile > 0, "the seed founds both kinds");
}

#[test]
fn a_seeded_producer_is_sessile_and_a_seeded_consumer_is_not() {
    // The rent asymmetry only means anything if the bodies the world
    // actually founds differ in the number it reads. They do, by recipe:
    // `axis::seed` gives an unlimbed line no contractile part at all.
    let world = crate::world::World::new(3, 60);
    let (mut sessile_producers, mut motile_consumers) = (0, 0);
    for organism in world.organisms.iter().filter(|o| o.is_alive()) {
        match organism.kingdom() {
            Kingdom::Producer => {
                assert_eq!(
                    organism.actuator_span(),
                    0,
                    "a producer grew an actuator: {:?}",
                    organism.id
                );
                sessile_producers += 1;
            },
            Kingdom::Consumer if organism.actuator_span() > 0 => motile_consumers += 1,
            _ => {},
        }
    }
    assert!(sessile_producers > 0 && motile_consumers > 0);
}
