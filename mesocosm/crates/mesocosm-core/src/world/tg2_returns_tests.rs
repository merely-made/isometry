// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use super::*;
use crate::flow::{Account, Conversion, RecordedFlow};
use crate::matter::{Material, Stock};

type Key = (Account, Option<OrganismId>);
type Book = BTreeMap<Key, Stock>;

fn add(book: &mut Book, key: Key, stock: Stock) {
    let next = book.get(&key).copied().unwrap_or(Stock::EMPTY);
    book.insert(key, next.checked_add(stock).unwrap());
}

fn books(world: &World) -> Book {
    let mut book = Book::new();
    add(&mut book, (Account::Soil, None), world.soil.total_stock());
    for organism in &world.organisms {
        add(
            &mut book,
            (Account::Substance, Some(organism.id)),
            organism.phenotype.total_stock().unwrap(),
        );
        add(
            &mut book,
            (Account::Reserve, Some(organism.id)),
            Stock::single(Material::Untyped, organism.energy_mg),
        );
    }
    book
}

fn side_key(account: Account, subject: Option<crate::flow::Subject>) -> Key {
    if account.is_body() {
        (
            account,
            Some(subject.expect("body account has a subject").organism),
        )
    } else {
        (account, None)
    }
}

fn expected(mut before: Book, flows: &[RecordedFlow]) -> Book {
    for envelope in flows {
        let flow = envelope.record;
        let composition = flow
            .composition
            .expect("every live matter flow is composed");
        let source = side_key(flow.source, flow.from);
        let destination = side_key(flow.destination, flow.to);
        let remaining = before
            .get(&source)
            .copied()
            .unwrap_or(Stock::EMPTY)
            .checked_sub(composition.input)
            .unwrap();
        before.insert(source, remaining);
        add(&mut before, destination, composition.output);
    }
    before
}

fn assert_channels(before: &World, after: &World, flows: &[RecordedFlow]) {
    assert!(!flows.is_empty(), "the exercised tick emitted flows");
    let mut claimed = expected(books(before), flows);
    claimed.retain(|_, stock| *stock != Stock::EMPTY);
    let mut actual = books(after);
    actual.retain(|_, stock| *stock != Stock::EMPTY);
    assert_eq!(claimed, actual);
}

fn mixed_body(world: &mut World, id: OrganismId) {
    let organism = world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == id)
        .expect("the selected organism exists");
    let parts: Vec<_> = organism
        .body()
        .living()
        .map(|part| (part.id, part.mass_mg))
        .collect();
    let mut typed = 0;
    for (part, mass) in parts {
        let stock = if mass >= 2 {
            typed += mass - 1;
            Stock::from_amounts([0, mass - 1, 1, 0])
        } else {
            Stock::single(Material::Untyped, mass)
        };
        organism.phenotype.replace_part_stock(part, stock).unwrap();
    }
    assert!(typed > 0, "the fixture has typed tissue to return");
}

fn composed_step(world: &mut World, intent: Intent) -> Vec<RecordedFlow> {
    let before = world.clone();
    world.apply(intent);
    let flows = world.drain_flows();
    assert_channels(&before, world, &flows);
    flows
}

#[test]
fn mixed_tissue_upkeep_reconciles_every_channel() {
    let mut world = World::new(4_242, 24);
    let controlled = world.controlled_id().unwrap();
    mixed_body(&mut world, controlled);
    world.controlled_mut().unwrap().energy_mg = 0;

    let flows = composed_step(&mut world, Intent::Idle);
    assert!(flows.iter().any(|flow| {
        flow.record.process == crate::flow::Process::Upkeep
            && flow.record.source == Account::Substance
            && flow.record.composition.unwrap().input.amounts()[1..]
                .iter()
                .any(|amount| *amount > 0)
    }));
}

#[test]
fn mixed_life_history_replays_with_composed_birth_death_and_decay() {
    let mut world = World::new(11, 40);
    let parent = world
        .living()
        .find(|organism| {
            organism.kingdom() == crate::organism::Kingdom::Producer && organism.biomass_mg() > 400
        })
        .map(|organism| organism.id)
        .expect("a parent can provision an ordinary birth");
    // Isolate decay from scavenging while retaining ordinary birth and ticking.
    world.organisms.retain(|organism| organism.id == parent);
    world.controlled = Some(parent);
    mixed_body(&mut world, parent);
    {
        let parent = world.organisms.iter_mut().find(|o| o.id == parent).unwrap();
        parent.stage = crate::organism::Stage::Mature;
        parent.since_offspring = u32::MAX;
    }
    let postcard = crate::snapshot::snapshot(&world).unwrap();
    let mut replay = crate::snapshot::restore_under(&postcard, world.admitted()).unwrap();

    let before = world.clone();
    let outcome = world.apply(Intent::ForceBirth { organism: parent });
    assert!(matches!(outcome, Outcome::Bore { .. }));
    let birth_flows = world.drain_flows();
    assert_channels(&before, &world, &birth_flows);
    let replay_outcome = replay.apply(Intent::ForceBirth { organism: parent });
    assert_eq!(outcome, replay_outcome);
    let replay_birth_flows = replay.drain_flows();
    assert_eq!(birth_flows, replay_birth_flows);
    assert_eq!(world, replay);

    let before = world.clone();
    let killed = world.apply(Intent::Kill { organism: parent });
    assert!(matches!(killed, Outcome::Killed { .. }), "{killed:?}");
    let death_flows = world.drain_flows();
    assert_channels(&before, &world, &death_flows);
    replay.apply(Intent::Kill { organism: parent });
    let replay_death_flows = replay.drain_flows();
    assert_eq!(death_flows, replay_death_flows);
    assert_eq!(world, replay);

    // Birth can empty an early part, which this tick's untyped root growth
    // refills. Decay spends in part order: allow enough cadence doses to
    // exhaust that untyped stock before requiring a typed return.
    let corpse = world.organisms.iter().find(|o| o.id == parent).unwrap();
    let stock = corpse.phenotype.total_stock().unwrap();
    assert!(stock.amounts()[1..].iter().any(|amount| *amount > 0));
    let decay_ticks = (stock.amount(Material::Untyped) + 1)
        * u64::from(crate::organism::ecology::CARRION_DECAY_TICKS);
    assert!(decay_ticks <= 128, "keep this lifecycle fixture bounded");
    let mut saw_decay = false;
    let mut decay_flows = Vec::new();
    for _ in 0..decay_ticks {
        let before = world.clone();
        world.apply(Intent::Idle);
        let flows = world.drain_flows();
        decay_flows.extend(
            flows
                .iter()
                .filter(|f| f.record.process == crate::flow::Process::Decay)
                .cloned(),
        );
        assert_channels(&before, &world, &flows);
        saw_decay |= flows.iter().any(|flow| {
            flow.record.process == crate::flow::Process::Decay
                && flow.record.composition.unwrap().conversion
                    == Some(crate::flow::Conversion::Mineralization)
        });
        replay.apply(Intent::Idle);
        let replay_flows = replay.drain_flows();
        assert_eq!(flows, replay_flows);
        assert_eq!(world, replay);
    }
    assert!(
        saw_decay,
        "the mixed parent eventually mineralized in carrion: {decay_flows:?}; remaining: {:?}",
        world
            .organisms
            .iter()
            .map(|o| (o.id, o.stage, o.age, o.phenotype.total_stock()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn swapping_equal_total_channels_fails_the_account_book() {
    let mut world = World::new(4_242, 0);
    let id = world.controlled_id().unwrap();
    mixed_body(&mut world, id);
    let before = books(&world);
    world.apply(Intent::Idle);
    let flows = world.drain_flows();
    let mut after = books(&world);
    let key = (Account::Substance, Some(id));
    let original = after.get(&key).copied().unwrap();
    let amounts = original.amounts();
    after.insert(
        key,
        Stock::from_amounts([amounts[0], amounts[1] - 1, amounts[2] + 1, amounts[3]]),
    );
    assert_ne!(expected(before, &flows), after);
}

#[test]
fn every_exercised_flow_has_a_typed_or_explicit_untyped_composition() {
    let mut world = World::new(7, 24);
    let before = world.clone();
    world.apply(Intent::Idle);
    let flows = world.drain_flows();
    assert_channels(&before, &world, &flows);
    for flow in flows {
        let composition = flow.record.composition.unwrap();
        if composition.conversion == Some(Conversion::Mineralization) {
            assert_eq!(composition.output.amounts()[1..], [0, 0, 0]);
        }
    }
}
