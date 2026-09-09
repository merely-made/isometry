// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn metabolize_grows_mass_and_collision() {
    let mut world = World::new(99, 24);
    let target = near_organism(&mut world);
    let eaten = world
        .organisms
        .iter()
        .find(|organism| organism.id == target)
        .unwrap()
        .clone();
    // Far enough out to be outside whatever envelope this seed's body drew,
    // rather than a literal that happened to clear the body it used to draw.
    // (Bodies moved at DC1.5: a consumer's head now bears a jaw or a crop.)
    let out = world.body().unwrap().aabb().extent()[0] + eaten.half_extent()[0].abs() + 1;
    let mut preview = world.body().unwrap().clone();
    preview
        .attach(
            eaten.volume(),
            eaten.biomass_mg(),
            eaten.half_extent(),
            Attachment {
                parent: preview.root,
                offset: [out, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    let stance = crate::places::surface_stance_for(
        world.ground(),
        crate::places::WalkerShape::from_aabb(preview.aabb()),
        world.position().unwrap(),
    )
    .expect("the generated surface has room for the previewed body");
    world.controlled_mut().unwrap().position = stance;
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == target)
        .unwrap()
        .position = stance;
    let mass_before = world.total_mass_mg();
    let box_before = world.collision().unwrap();

    let outcome = world.apply(Intent::Metabolize {
        organism: target,
        placement: Placement::Explicit {
            parent: world.body().unwrap().root,
            offset: [out, 0, 0],
            yaw: Yaw::Zero,
        },
    });

    assert!(matches!(outcome, Outcome::Incorporated { .. }));
    assert!(world.total_mass_mg() > mass_before);
    assert!(world.collision().unwrap().extent()[0] > box_before.extent()[0]);
}

#[test]
fn metabolize_records_where_the_part_came_from() {
    let mut world = World::new(7, 24);
    let controlled = world.controlled_id().expect("the fixture is embodied");
    admit_producers(
        world
            .organisms
            .iter_mut()
            .find(|organism| organism.id == controlled)
            .expect("the controlled organism exists"),
    );
    let target = near_organism(&mut world);
    let eaten_species = world
        .organisms
        .iter()
        .find(|m| m.id == target)
        .map(|m| m.species)
        .unwrap();

    let Outcome::Incorporated { part } = world.apply(Intent::Metabolize {
        organism: target,
        placement: Placement::Explicit {
            parent: world.body().unwrap().root,
            offset: [4, 0, 0],
            yaw: Yaw::Zero,
        },
    }) else {
        panic!("expected incorporation");
    };

    assert_eq!(
        world.body().unwrap().part(part).unwrap().provenance.origin,
        Origin::Incorporated {
            from_species: eaten_species,
            from_part: PartId(0)
        }
    );
}

#[test]
fn incorporation_fits_on_the_surface_and_refuses_inside_the_same_burrow() {
    const SEED: u64 = 0;
    const PREY: OrganismId = OrganismId(900);
    let route = generated_nest_entry(SEED);
    let setup = |eater_at, prey_at| {
        let mut world = World::new(SEED, 0);
        world.organisms = vec![
            Organism::founding(
                OrganismId(0),
                SpeciesId(1),
                Kingdom::Consumer,
                VolumeRef::from_tag(1),
                [1, 1, 1],
                eater_at,
                300,
            ),
            Organism::founding(
                PREY,
                SpeciesId(3),
                Kingdom::Producer,
                VolumeRef::from_tag(2),
                [3, 1, 3],
                prey_at,
                100,
            ),
        ];
        world
    };
    let meal = Intent::Metabolize {
        organism: PREY,
        placement: Placement::Explicit {
            parent: PartId(0),
            offset: [0, 2, 0],
            yaw: Yaw::Zero,
        },
    };

    let mut surface = setup(route[0], route[1]);
    assert!(matches!(
        surface.apply(meal.clone()),
        Outcome::Incorporated { .. }
    ));
    assert_eq!(surface.controlled().unwrap().walker_shape().radius(), 1);
    assert!(
        surface
            .controlled()
            .unwrap()
            .walker_shape()
            .stands(surface.ground(), route[0])
    );

    let mut inside = setup(route[1], route[2]);
    let parts_before = inside.body().unwrap().parts.len();
    let hash_before = crate::snapshot::state_hash(&inside);
    assert_eq!(
        inside.apply(meal),
        Outcome::Rejected(Rejection::NoRoom),
        "the one-voxel interior accepted anatomy with a three-voxel cross-section"
    );
    assert_eq!(inside.body().unwrap().parts.len(), parts_before);
    assert_eq!(inside.controlled().unwrap().walker_shape().radius(), 0);
    assert!(inside.organisms.iter().any(|organism| organism.id == PREY));
    assert_ne!(
        crate::snapshot::state_hash(&inside),
        hash_before,
        "a rejected intent still advances the ordered world"
    );
}

#[test]
fn a_bigger_body_costs_more_to_carry() {
    let world = World::new(3, 2);
    let small = world.controlled().unwrap().upkeep_mg();
    let mut grown = world.clone();
    let me = grown.controlled_id().unwrap();
    grown
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .gain_mass(5_000);

    assert!(
        grown.controlled().unwrap().upkeep_mg() > small,
        "a heavier body pays more rent: {} vs {}",
        grown.controlled().unwrap().upkeep_mg(),
        small
    );
}

#[test]
fn deposit_returns_matter_to_the_enclosure() {
    // It used to spawn a carcass; since TD6 it enriches the ground the
    // depositor is standing on, which is what "returns matter to the
    // enclosure" always meant. The column, not the roster, is what moves.
    let mut world = World::new(3, 2);
    let count = world.organisms.len();
    let before = world.soil().total_mg();

    let outcome = world.apply(Intent::Deposit { mass_mg: 200 });

    assert!(matches!(outcome, Outcome::Deposited { .. }));
    assert_eq!(world.organisms.len(), count, "no carcass is minted");
    // Loose lower bound: the tick that follows also pays rent into the ground
    // and lets any producer draw out of it, so the deposit is most of this
    // move but not all of it.
    assert!(
        world.soil().total_mg() >= before + 150,
        "the ground should be richer for it: {} -> {}",
        before,
        world.soil().total_mg()
    );
}
