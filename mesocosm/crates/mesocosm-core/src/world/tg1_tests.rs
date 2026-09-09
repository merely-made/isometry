// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://www.mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::body::VolumeRef;
use crate::organism::{Kingdom, Organism, OrganismId, Stage};
use crate::places::Tier;
use crate::process::{IntakePort, NisKind, Process, Registry};

fn typed_consumer_world(target_stage: Stage) -> (World, OrganismId) {
    let mut world = World::new(4_242, 0);
    let eater_id = world.controlled_id().expect("the fixture is embodied");
    let target_id = OrganismId(900);
    let at = world.position().expect("the fixture has a position");
    let eater = Organism::founding(
        eater_id,
        crate::body::SpeciesId(2),
        Kingdom::Consumer,
        VolumeRef::from_tag(16),
        [3, 1, 1],
        at,
        300,
    );
    let mut target = Organism::founding(
        target_id,
        crate::body::SpeciesId(3),
        Kingdom::Producer,
        VolumeRef::from_tag(18),
        [1, 1, 1],
        at,
        300,
    );
    target.stage = target_stage;
    target.tier = Tier::Near;
    world.organisms = vec![eater, target];
    (world, target_id)
}

fn declare_mouth(world: &mut World, port: IntakePort) {
    let mouth = world
        .body()
        .expect("the eater has a body")
        .mouth_part()
        .expect("the eater has a mouth");
    assert!(
        world
            .controlled_mut()
            .expect("the eater is alive")
            .phenotype
            .declare_port(mouth, port)
    );
}

fn mouth_support(world: &World) -> crate::process::ProcessRef {
    let mouth = world
        .body()
        .expect("the eater has a body")
        .mouth_part()
        .expect("the eater has a mouth");
    world
        .controlled()
        .expect("the eater is embodied")
        .phenotype
        .part_port(mouth)
        .expect("the seeded mouth port is active")
        .support()
        .expect("the seeded mouth has support")
}

#[test]
fn metabolize_refuses_live_meal_outside_typed_port() {
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    let before = world.clone();
    let outcome = world.metabolize(
        target,
        Placement::Explicit {
            parent: world.body().expect("the eater has a body").root,
            offset: [0, 0, 0],
            yaw: crate::body::Yaw::Zero,
        },
    );

    assert_eq!(outcome, Outcome::Rejected(Rejection::Inedible(target)));
    assert_eq!(world, before);
}

#[test]
fn metabolize_refuses_carrion_without_deadstock_port() {
    let (mut world, target) = typed_consumer_world(Stage::Carrion);
    let before = world.clone();
    let outcome = world.metabolize(target, Placement::Planned);

    assert_eq!(outcome, Outcome::Rejected(Rejection::Inedible(target)));
    assert_eq!(world, before);
}

#[test]
fn metabolize_accepts_matching_declared_live_port() {
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    let support = mouth_support(&world);
    declare_mouth(
        &mut world,
        IntakePort::live(NisKind::Producer).supported_by(support),
    );
    world
        .controlled_mut()
        .expect("the eater is alive")
        .energy_mg = 0;

    let outcome = world.metabolize(target, Placement::Planned);

    assert!(matches!(outcome, Outcome::Burned { organism, .. } if organism == target));
}

#[test]
fn inactive_declared_live_port_does_not_admit_a_meal() {
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    let fix = Registry::native().of_native(Process::Fix).reference();
    declare_mouth(
        &mut world,
        IntakePort::live(NisKind::Producer).supported_by(fix),
    );
    let before = world.clone();

    let outcome = world.metabolize(target, Placement::Planned);

    assert_eq!(outcome, Outcome::Rejected(Rejection::Inedible(target)));
    assert_eq!(world, before);
}

#[test]
fn consume_requires_and_accepts_active_deadstock_port() {
    let (mut world, target) = typed_consumer_world(Stage::Carrion);
    let root = world.body().expect("the eater has a body").root;
    let support = mouth_support(&world);
    let before = world.clone();
    let refused = world.consume(target, root);
    assert_eq!(refused, Outcome::Rejected(Rejection::Inedible(target)));
    assert_eq!(world, before);

    declare_mouth(
        &mut world,
        IntakePort::live(NisKind::Consumer)
            .with_deadstock()
            .supported_by(support),
    );
    let accepted = world.consume(target, root);
    assert!(
        matches!(accepted, Outcome::Consumed { from, from_part, .. } if from == target && from_part == root)
    );
}

#[test]
fn a_named_part_meal_preserves_its_donor_mixture() {
    let (mut world, target) = typed_consumer_world(Stage::Carrion);
    let support = mouth_support(&world);
    declare_mouth(
        &mut world,
        IntakePort::live(NisKind::Consumer)
            .with_deadstock()
            .supported_by(support),
    );
    let donor = world.organisms.iter_mut().find(|o| o.id == target).unwrap();
    let root = donor.body().root;
    let mass = donor.body().part(root).unwrap().mass_mg;
    let mix = crate::matter::Stock::from_amounts([mass - 6, 1, 2, 3]);
    donor.phenotype.replace_part_stock(root, mix).unwrap();
    let Outcome::Consumed { part, .. } = world.consume(target, root) else {
        panic!("the fixture part must fit")
    };
    assert_eq!(
        world.controlled().unwrap().phenotype.part_stock(part),
        Some(&mix)
    );
    assert_eq!(
        world
            .organisms
            .iter()
            .find(|o| o.id == target)
            .unwrap()
            .phenotype
            .part_stock(root),
        Some(&crate::matter::Stock::EMPTY)
    );
    let restored =
        crate::snapshot::restore_under(&crate::snapshot(&world).unwrap(), world.admitted())
            .unwrap();
    assert_eq!(world, restored);
}
