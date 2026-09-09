// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    flow::Account,
    matter::{Material, Stock},
    snapshot,
};

#[test]
fn mixed_soil_survives_ecology_and_snapshot_replay_without_becoming_food() {
    let mut world = World::new(41, 60);
    let column = world.soil.column_at([0, 0, 0]);
    // Test-only initial material. Current ecology still emits untyped returns;
    // it must not consume or relabel these pending typed stocks.
    world
        .soil
        .deposit_stock(column, Stock::from_amounts([23, 101, 103, 107]))
        .unwrap();
    let initial = world.total_matter_mg();
    let typed = world.soil.total_stock().amounts();
    let mut resumed = None;
    for tick in 0..120 {
        let before = world.soil.total_stock();
        world.apply(Intent::Idle);
        let mut expected_untyped = i128::from(before.amount(Material::Untyped));
        for flow in world.drain_flows() {
            let event = flow.record;
            if event.source == Account::Soil {
                expected_untyped -= i128::from(event.amount_mg);
            }
            if event.destination == Account::Soil {
                expected_untyped += i128::from(event.amount_mg);
            }
        }
        let after = world.soil.total_stock();
        assert_eq!(
            i128::from(after.amount(Material::Untyped)),
            expected_untyped
        );
        assert_eq!(&after.amounts()[1..], &typed[1..]);
        assert_eq!(world.total_matter_mg(), initial);
        if let Some(other) = &mut resumed {
            World::apply(other, Intent::Idle);
            assert_eq!(snapshot::state_hash(&world), snapshot::state_hash(other));
        }
        if tick == 59 {
            resumed = Some(
                snapshot::restore_under(&snapshot::snapshot(&world).unwrap(), world.admitted())
                    .unwrap(),
            );
        }
    }
}

#[test]
fn soil_composition_changes_world_hash_even_when_total_mass_matches() {
    let mut producer = World::new(3, 0);
    let mut consumer = producer.clone();
    let column = producer.soil.column_at([0, 0, 0]);
    producer
        .soil
        .deposit_stock(column, Stock::single(Material::Producer, 7))
        .unwrap();
    consumer
        .soil
        .deposit_stock(column, Stock::single(Material::Consumer, 7))
        .unwrap();
    assert_eq!(producer.total_matter_mg(), consumer.total_matter_mg());
    assert_ne!(
        snapshot::state_hash(&producer),
        snapshot::state_hash(&consumer)
    );
}

#[test]
fn pre_typed_soil_rules_are_refused_at_snapshot_admission() {
    let mut world = World::new(3, 0);
    world.rules.trophic_grammar = 1;
    assert!(matches!(
        snapshot::restore_under(&snapshot::snapshot(&world).unwrap(), world.admitted()),
        Err(snapshot::SnapshotError::Rules { .. })
    ));
}
