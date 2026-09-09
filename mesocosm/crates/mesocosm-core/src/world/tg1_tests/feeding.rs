// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::flow::Account;

fn mixed_target(world: &mut World, target: OrganismId) -> crate::matter::Stock {
    let donor = world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == target)
        .expect("the target is present");
    let root = donor.body().root;
    let mass = donor
        .body()
        .part(root)
        .expect("the target has a root")
        .mass_mg;
    let stock = crate::matter::Stock::from_amounts([mass - 6, 1, 2, 3]);
    donor.phenotype.replace_part_stock(root, stock).unwrap();
    donor.phenotype.total_stock().unwrap()
}

fn admit_live_producer(world: &mut World) {
    let support = mouth_support(world);
    declare_mouth(
        world,
        IntakePort::live(NisKind::Producer).supported_by(support),
    );
    world.controlled_mut().unwrap().energy_mg = 1_000;
}

fn add_stocks(a: crate::matter::Stock, b: crate::matter::Stock) -> crate::matter::Stock {
    a.checked_add(b).unwrap()
}

#[test]
fn whole_mixed_meal_planned_attachment_preserves_stock_and_receipt() {
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    admit_live_producer(&mut world);
    let stock = mixed_target(&mut world, target);

    let outcome = world.metabolize(target, Placement::Planned);
    let parts = match outcome {
        Outcome::Incorporated { part } => vec![part],
        Outcome::IncorporatedPair { part, mirror } => vec![part, mirror],
        other => panic!("planned mixed meal refused: {other:?}"),
    };
    let attached = parts
        .into_iter()
        .fold(crate::matter::Stock::EMPTY, |total, part| {
            add_stocks(
                total,
                *world
                    .controlled()
                    .unwrap()
                    .phenotype
                    .part_stock(part)
                    .unwrap(),
            )
        });
    let feeding = world
        .flows()
        .iter()
        .find(|record| {
            record.record.process == crate::flow::Process::Feeding
                && record.record.destination == Account::Substance
        })
        .expect("the body landing has a feeding flow");
    let body_stock = feeding.record.composition.unwrap().output;
    let spill_stock = world
        .flows()
        .iter()
        .find(|record| {
            record.record.process == crate::flow::Process::Spill
                && record.record.source == Account::Substance
        })
        .and_then(|record| record.record.composition)
        .map_or(crate::matter::Stock::EMPTY, |composition| {
            composition.output
        });
    assert_eq!(attached, body_stock);
    assert_eq!(add_stocks(body_stock, spill_stock), stock);
}

#[test]
fn whole_mixed_meal_explicit_attachment_preserves_stock() {
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    admit_live_producer(&mut world);
    let stock = mixed_target(&mut world, target);
    let parent = world.body().unwrap().root;

    let outcome = world.metabolize(
        target,
        Placement::Explicit {
            parent,
            offset: [4, 0, 0],
            yaw: crate::body::Yaw::Zero,
        },
    );
    let part = match outcome {
        Outcome::Incorporated { part } => part,
        other => panic!("explicit mixed meal refused: {other:?}"),
    };
    assert_eq!(
        world.controlled().unwrap().phenotype.part_stock(part),
        Some(&stock)
    );
}

#[test]
fn whole_mixed_meal_burn_records_digestion_and_untyped_output() {
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    admit_live_producer(&mut world);
    let stock = mixed_target(&mut world, target);
    world.controlled_mut().unwrap().energy_mg = 0;
    let amount = stock.total() as u64;

    assert!(matches!(
        world.metabolize(target, Placement::Planned),
        Outcome::Burned { .. }
    ));
    let flow = world
        .flows()
        .iter()
        .find(|record| {
            record.record.process == crate::flow::Process::Feeding
                && record.record.destination == Account::Reserve
        })
        .expect("burning emits a reserve feeding flow");
    let composition = flow.record.composition.unwrap();
    assert_eq!(flow.record.amount_mg, amount);
    assert_eq!(composition.input, stock);
    assert_eq!(
        composition.output,
        crate::matter::Stock::single(crate::matter::Material::Untyped, amount)
    );
    assert_eq!(
        composition.conversion,
        Some(crate::flow::Conversion::Digestion)
    );
}

#[test]
fn refused_whole_meal_leaves_mixed_donor_and_flow_stream_unchanged() {
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    let stock = mixed_target(&mut world, target);
    let before = world.clone();

    assert_eq!(
        world.metabolize(target, Placement::Planned),
        Outcome::Rejected(Rejection::Inedible(target))
    );
    assert_eq!(world, before);
    assert!(world.flows().is_empty());
    let donor = world.organisms.iter().find(|o| o.id == target).unwrap();
    assert_eq!(donor.phenotype.total_stock().unwrap(), stock);
}

#[test]
fn whole_mixed_meal_snapshot_round_trip_preserves_part_and_flow_effect() {
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    admit_live_producer(&mut world);
    let stock = mixed_target(&mut world, target);
    let postcard = crate::snapshot(&world).unwrap();
    let parts = match world.metabolize(target, Placement::Planned) {
        Outcome::Incorporated { part } => vec![part],
        Outcome::IncorporatedPair { part, mirror } => vec![part, mirror],
        other => panic!("mixed meal refused: {other:?}"),
    };
    let mut replayed = crate::snapshot::restore_under(&postcard, world.admitted()).unwrap();
    let replay_parts = match replayed.metabolize(target, Placement::Planned) {
        Outcome::Incorporated { part } => vec![part],
        Outcome::IncorporatedPair { part, mirror } => vec![part, mirror],
        other => panic!("replayed mixed meal refused: {other:?}"),
    };
    assert_eq!(world, replayed);
    assert_eq!(world.flows(), replayed.flows());
    let attached = parts
        .into_iter()
        .fold(crate::matter::Stock::EMPTY, |total, part| {
            add_stocks(
                total,
                *world
                    .controlled()
                    .unwrap()
                    .phenotype
                    .part_stock(part)
                    .unwrap(),
            )
        });
    let replay_attached =
        replay_parts
            .into_iter()
            .fold(crate::matter::Stock::EMPTY, |total, part| {
                add_stocks(
                    total,
                    *replayed
                        .controlled()
                        .unwrap()
                        .phenotype
                        .part_stock(part)
                        .unwrap(),
                )
            });
    assert_eq!(attached, stock);
    assert_eq!(replay_attached, stock);
    let encoded = serde_json::to_vec(world.flows()).unwrap();
    let decoded: Vec<crate::flow::RecordedFlow> = serde_json::from_slice(&encoded).unwrap();
    assert_eq!(decoded, world.flows());
    let encoded = postcard::to_allocvec(world.flows()).unwrap();
    let decoded: Vec<crate::flow::RecordedFlow> = postcard::from_bytes(&encoded).unwrap();
    assert_eq!(decoded, world.flows());
}

#[test]
fn mirrored_odd_meal_preserves_both_halves_and_pending_soil_stock() {
    use crate::matter::{Material, Stock};
    let (mut world, target) = typed_consumer_world(Stage::Mature);
    admit_live_producer(&mut world);
    let at = world.position().unwrap();
    let donor = world.organisms.iter_mut().find(|o| o.id == target).unwrap();
    *donor = Organism::founding(
        target,
        crate::body::SpeciesId(3),
        Kingdom::Producer,
        VolumeRef::from_tag(18),
        [3, 1, 1],
        at,
        301,
    );
    donor
        .phenotype
        .replace_part_stock(donor.body().root, Stock::from_amounts([0, 100, 100, 100]))
        .unwrap();
    donor.energy_mg = 0;
    let stock = donor.phenotype.total_stock().unwrap();
    let column = world.soil.column_at(at);
    let soil_before = world.soil.stock(column);
    let Outcome::IncorporatedPair { part, mirror } = world.metabolize(target, Placement::Planned)
    else {
        panic!("a limb-shaped meal must make a mirrored pair");
    };
    let body = &world.controlled().unwrap().phenotype;
    let first = *body.part_stock(part).unwrap();
    let second = *body.part_stock(mirror).unwrap();
    assert_eq!(first.total(), 150);
    assert_eq!(second.total(), 150);
    let spill = world.soil.stock(column).checked_sub(soil_before).unwrap();
    assert_eq!(spill.total(), 1);
    assert_eq!(add_stocks(add_stocks(first, second), spill), stock);
    assert_eq!(spill.amount(Material::Untyped), 0);
}
