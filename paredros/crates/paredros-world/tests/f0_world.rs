// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use paredros_identity::{SubjectId, Tick};
use paredros_world::{
    GENERATOR_VERSION, HistoryFactId, Layer, SiteKind, SiteSource, SlotId, World, WorldConfig,
    WorldError, WorldIntent,
};

const SEED: u64 = 4_242;

fn world() -> World {
    World::generate(SEED, WorldConfig::default()).expect("F0 receipt world")
}

fn solid_at_site(world: &World, slot: SlotId) -> [i32; 3] {
    let [x, z] = world
        .grown()
        .places
        .get(slot.place)
        .expect("slot host")
        .centre;
    let y = world.ground().surface(x, z).expect("solid column");
    [x, y, z]
}

#[test]
fn fresh_runs_share_world_facts_and_seed_changes_topology() {
    let first = world();
    let second = world();
    assert_eq!(first, second);
    assert_eq!(first.state_hash().unwrap(), second.state_hash().unwrap());

    let other = World::generate(31_337, WorldConfig::default()).unwrap();
    let shape = |world: &World| {
        world
            .grown()
            .places
            .all()
            .map(|place| world.grown().places.neighbours(place.id).to_vec())
            .collect::<Vec<_>>()
    };
    assert_ne!(shape(&first), shape(&other));
}

#[test]
fn generated_structure_connects_the_foundation_places() {
    let world = world();
    for kind in [
        SiteKind::Settlement,
        SiteKind::Ruin,
        SiteKind::Encounter,
        SiteKind::Dungeon,
    ] {
        assert!(world.map().slots_of_kind(kind).next().is_some(), "{kind:?}");
    }

    let underground = world
        .map()
        .slots_of_kind(SiteKind::Dungeon)
        .next()
        .expect("this seed has a generated underground");
    let ruin = world.map().slots_of_kind(SiteKind::Ruin).next().unwrap();
    let settlement = world
        .map()
        .slots_of_kind(SiteKind::Settlement)
        .next()
        .unwrap();
    let mut route = world.map().route(underground, ruin).unwrap();
    route.extend(
        world
            .map()
            .route(ruin, settlement)
            .unwrap()
            .into_iter()
            .skip(1),
    );
    let layers = route.iter().map(|slot| slot.layer).collect::<BTreeSet<_>>();
    let kinds = route
        .iter()
        .map(|slot| world.map().site(*slot).unwrap().kind)
        .collect::<Vec<_>>();
    assert_eq!(layers, BTreeSet::from([Layer::Surface, Layer::Underground]));
    assert!(kinds.contains(&SiteKind::Ruin));
    assert!(kinds.contains(&SiteKind::Settlement));
    for pair in route.windows(2) {
        assert!(world.map().neighbours(pair[0]).unwrap().contains(&pair[1]));
    }
}

#[test]
fn inherited_history_reuses_the_generated_slot() {
    let mut world = world();
    let slot = world.map().slots_of_kind(SiteKind::Ruin).next().unwrap();
    let neighbours = world.map().neighbours(slot).unwrap().clone();
    let parent = world.map().site(slot).unwrap().parent;

    world
        .apply(WorldIntent::InheritSite {
            tick: Tick(7),
            slot,
            kind: SiteKind::Settlement,
            fact: HistoryFactId(91),
        })
        .unwrap();

    let inherited = world.map().site(slot).unwrap();
    assert_eq!(inherited.slot, slot);
    assert_eq!(inherited.kind, SiteKind::Settlement);
    assert_eq!(inherited.source, SiteSource::Inherited(HistoryFactId(91)));
    assert_eq!(inherited.parent, parent);
    assert_eq!(world.map().neighbours(slot).unwrap(), &neighbours);
}

#[test]
fn player_and_non_player_edits_survive_regrow_and_replay() {
    let mut world = world();
    let settlement = world
        .map()
        .slots_of_kind(SiteKind::Settlement)
        .next()
        .unwrap();
    let ruin = world.map().slots_of_kind(SiteKind::Ruin).next().unwrap();
    let player_cut = solid_at_site(&world, settlement);
    let neighbour_cut = solid_at_site(&world, ruin);

    for intent in [
        WorldIntent::Carve {
            tick: Tick(10),
            by: SubjectId(1),
            centre: player_cut,
            radius: 1,
        },
        WorldIntent::Carve {
            tick: Tick(11),
            by: SubjectId(44),
            centre: neighbour_cut,
            radius: 1,
        },
    ] {
        world.apply(intent).unwrap();
    }
    assert_eq!(world.ground().revision(), 2);
    assert!(world.events().iter().all(|event| match event {
        paredros_world::WorldEvent::Carved { removed, .. } => *removed > 0,
        _ => false,
    }));

    let saved = world.save().unwrap();
    let restored = World::restore(&saved).unwrap();
    assert_eq!(restored, world);
    assert_eq!(restored.state_hash().unwrap(), world.state_hash().unwrap());
    assert!(!restored.ground().solid(player_cut));
    assert!(!restored.ground().solid(neighbour_cut));
}

#[test]
fn restore_refuses_drift_and_invalid_intents_do_not_enter_the_log() {
    let world = world();
    let mut version_drift = world.save_record();
    version_drift.generator_version = GENERATOR_VERSION + 1;
    assert_eq!(
        World::restore_record(version_drift),
        Err(WorldError::GeneratorDiverged {
            saved: GENERATOR_VERSION + 1,
            current: GENERATOR_VERSION,
        })
    );

    let mut base_drift = world.save_record();
    base_drift.base_hash ^= 1;
    assert!(matches!(
        World::restore_record(base_drift),
        Err(WorldError::BaseDiverged { .. })
    ));

    let mut world = world;
    world
        .apply(WorldIntent::Carve {
            tick: Tick(5),
            by: SubjectId(1),
            centre: [0, 1, 0],
            radius: 0,
        })
        .unwrap();
    let count = world.intents().len();
    assert!(matches!(
        world.apply(WorldIntent::Carve {
            tick: Tick(4),
            by: SubjectId(1),
            centre: [0, 1, 0],
            radius: 0,
        }),
        Err(WorldError::OutOfOrder { .. })
    ));
    assert_eq!(world.intents().len(), count);
}
