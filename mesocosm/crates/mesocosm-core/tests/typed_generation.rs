// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! TG2f's cross seam receipt: founding recipes, producer synthesis, and filial
//! provisioning must all agree with the live per-channel account book.

use std::collections::BTreeMap;

use mesocosm_core::flow::{Account, Conversion, RecordedFlow};
use mesocosm_core::matter::{Material, Stock};
use mesocosm_core::snapshot;
use mesocosm_core::{Intent, OrganismId, World};

type Key = (Account, Option<OrganismId>);

fn add(book: &mut BTreeMap<Key, Stock>, key: Key, stock: Stock) {
    let sum = book.get(&key).copied().unwrap_or(Stock::EMPTY);
    book.insert(key, sum.checked_add(stock).expect("account stock fits"));
}

fn accounts(world: &World) -> BTreeMap<Key, Stock> {
    let mut book = BTreeMap::new();
    add(&mut book, (Account::Soil, None), world.soil().total_stock());
    for organism in &world.organisms {
        add(
            &mut book,
            (Account::Substance, Some(organism.id)),
            organism.phenotype.total_stock().expect("body stock fits"),
        );
        add(
            &mut book,
            (Account::Reserve, Some(organism.id)),
            Stock::single(Material::Untyped, organism.energy_mg),
        );
    }
    book
}

fn body_key(account: Account, subject: Option<mesocosm_core::flow::Subject>) -> Key {
    if account.is_body() {
        (
            account,
            Some(subject.expect("body flow has a subject").organism),
        )
    } else {
        (account, None)
    }
}

fn expected(mut before: BTreeMap<Key, Stock>, flows: &[RecordedFlow]) -> BTreeMap<Key, Stock> {
    for envelope in flows {
        let flow = envelope.record;
        let composition = flow.composition.expect("live matter flow is composed");
        let source = body_key(flow.source, flow.from);
        let destination = body_key(flow.destination, flow.to);
        let remainder = before
            .get(&source)
            .copied()
            .unwrap_or(Stock::EMPTY)
            .checked_sub(composition.input)
            .expect("receipt input is held by its source");
        before.insert(source, remainder);
        add(&mut before, destination, composition.output);
    }
    before
}

fn reconcile(before: &World, after: &World, flows: &[RecordedFlow]) {
    assert!(!flows.is_empty(), "the accepted step emitted a flow");
    let mut expected = expected(accounts(before), flows);
    expected.retain(|_, stock| *stock != Stock::EMPTY);
    let mut actual = accounts(after);
    actual.retain(|_, stock| *stock != Stock::EMPTY);
    assert_eq!(
        expected, actual,
        "typed accounts disagree with flow receipt"
    );
}

#[test]
fn generated_founder_synthesis_and_birth_replay_per_channel() {
    let mut world = World::new(7, 24);
    let producer = world
        .living()
        .find(|organism| {
            organism.kingdom() == mesocosm_core::Kingdom::Producer && organism.biomass_mg() > 400
        })
        .expect("the generated stand has a producer");
    let producer_id = producer.id;
    let founding = producer
        .phenotype
        .total_stock()
        .expect("founder stock fits");
    assert!(
        founding.amount(Material::Producer) > 0,
        "a generated producer starts with producer tissue"
    );
    world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == producer_id)
        .expect("producer remains in the roster")
        .energy_mg = 0;

    let mut replay = snapshot::restore_under(
        &snapshot::snapshot(&world).expect("founder snapshot encodes"),
        world.admitted(),
    )
    .expect("founder snapshot restores under admitted rules");

    let mut saw_synthesis = false;
    let mut saw_digestion = false;
    for _ in 0..80 {
        let before = world.clone();
        world.apply(Intent::Idle);
        let flows = world.drain_flows();
        reconcile(&before, &world, &flows);
        replay.apply(Intent::Idle);
        assert_eq!(flows, replay.drain_flows(), "idle receipt replays exactly");
        assert_eq!(world, replay, "typed founder state replays exactly");
        saw_synthesis |= flows.iter().any(|flow| {
            flow.record.process == mesocosm_core::flow::Process::Uptake
                && flow.record.composition.is_some_and(|composition| {
                    composition.conversion == Some(Conversion::Synthesis)
                        && composition.output.amount(Material::Producer) > 0
                })
        });
        saw_digestion |= flows.iter().any(|flow| {
            flow.record.process == mesocosm_core::flow::Process::Uptake
                && flow.record.composition.is_some_and(|composition| {
                    composition.conversion == Some(Conversion::Digestion)
                })
        });
        if saw_synthesis && saw_digestion {
            break;
        }
    }
    assert!(
        saw_synthesis,
        "generated producer records synthesis into tissue"
    );
    assert!(saw_digestion, "producer reserve intake records digestion");

    let parent = world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == producer_id)
        .expect("producer remains alive");
    parent.stage = mesocosm_core::organism::Stage::Mature;
    parent.since_offspring = u32::MAX;
    // Give the birth a deliberately mixed fixture scruple. The account book
    // below proves that the exact lot, rather than its scalar mass, crosses
    // the filial boundary.
    let mass = parent.biomass_mg();
    parent
        .phenotype
        .distribute_stock(Stock::from_amounts([0, mass / 2, mass - mass / 2, 0]))
        .expect("mixed fixture stock fits the body");
    let before = world.clone();
    let birth_snapshot = snapshot::snapshot(&world).expect("mixed founder snapshot encodes");
    let mut birth_replay = snapshot::restore_under(&birth_snapshot, world.admitted())
        .expect("mixed founder snapshot restores");
    let outcome = world.apply(Intent::ForceBirth {
        organism: producer_id,
    });
    assert!(
        matches!(outcome, mesocosm_core::Outcome::Bore { .. }),
        "{outcome:?}"
    );
    let flows = world.drain_flows();
    reconcile(&before, &world, &flows);
    let replay_outcome = birth_replay.apply(Intent::ForceBirth {
        organism: producer_id,
    });
    assert_eq!(outcome, replay_outcome);
    assert_eq!(flows, birth_replay.drain_flows());
    assert_eq!(world, birth_replay);
    assert!(
        flows.iter().any(|flow| {
            flow.record.process == mesocosm_core::flow::Process::Birth
                && flow.record.composition.is_some_and(|composition| {
                    composition.input.amount(Material::Producer) > 0
                        && composition.input.amount(Material::Consumer) > 0
                })
        }),
        "birth preserves the parent's actual typed lot"
    );

    let old = mesocosm_core::rules::WorldRules {
        trophic_grammar: 5,
        ..world.rules()
    };
    let ruleset = world.admitted();
    let old_world = world.with_rules(old);
    assert!(matches!(
        snapshot::restore_under(&snapshot::snapshot(&old_world).unwrap(), ruleset),
        Err(snapshot::SnapshotError::Rules { .. })
    ));
}
