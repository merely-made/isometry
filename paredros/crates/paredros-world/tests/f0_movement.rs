// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use mesocosm_core::places::{WALKER_HEIGHT, step};
use paredros_identity::{SubjectId, Tick};
use paredros_world::{
    Layer, Movement, MovementError, MovementIntent, Navigation, SiteKind, SlotId, World,
    WorldConfig,
};

const SEED: u64 = 4_242;
const PLAYED: SubjectId = SubjectId(1);

fn itinerary(world: &World) -> Vec<SlotId> {
    let underground = world
        .map()
        .slots_of_kind(SiteKind::Dungeon)
        .next()
        .expect("receipt seed has an underground");
    let parent = world.map().site(underground).unwrap().parent.unwrap();
    let ruin = world.map().slots_of_kind(SiteKind::Ruin).next().unwrap();
    let settlement = world
        .map()
        .slots_of_kind(SiteKind::Settlement)
        .next()
        .unwrap();

    let mut slots = vec![parent, underground];
    slots.extend(
        world
            .map()
            .route(underground, ruin)
            .unwrap()
            .into_iter()
            .skip(1),
    );
    slots.extend(
        world
            .map()
            .route(ruin, settlement)
            .unwrap()
            .into_iter()
            .skip(1),
    );
    slots
}

fn receipt_plan(world: &World) -> (Vec<SlotId>, Vec<[i32; 3]>, Vec<usize>) {
    let navigation = Navigation::default();
    let slots = itinerary(world);
    let start = navigation.surface_stance(world, slots[0]).unwrap();
    let mut path = vec![start];
    let mut waypoint_steps = vec![0];
    for slot in slots.iter().copied().skip(1) {
        let from = *path.last().unwrap();
        let route = navigation.route_to_slot(world, from, slot).unwrap();
        path.extend(route.into_iter().skip(1));
        waypoint_steps.push(path.len() - 1);
    }
    (slots, path, waypoint_steps)
}

fn movement(world: &World, start: [i32; 3]) -> Movement {
    let mut movement = Movement::new();
    movement.spawn(world, PLAYED, start).unwrap();
    movement
}

#[test]
fn generic_movement_carries_the_four_place_receipt_over_exact_ground() {
    let world = World::generate(SEED, WorldConfig::default()).unwrap();
    let (slots, path, waypoint_steps) = receipt_plan(&world);
    let mut movement = movement(&world, path[0]);
    for toward in path.iter().copied().skip(1) {
        movement.step(&world, PLAYED, toward).unwrap();
    }

    assert_eq!(movement.position(PLAYED), path.last().copied());
    assert_eq!(movement.trail(PLAYED).unwrap(), path);
    for pair in movement.trail(PLAYED).unwrap().windows(2) {
        assert_eq!(step(world.ground(), pair[0], pair[1]), pair[1]);
        assert!(world.ground().stands(pair[1], WALKER_HEIGHT));
    }

    let layers = slots.iter().map(|slot| slot.layer).collect::<BTreeSet<_>>();
    let kinds = slots
        .iter()
        .filter_map(|slot| world.map().site(*slot))
        .map(|site| site.kind)
        .collect::<Vec<_>>();
    assert_eq!(layers, BTreeSet::from([Layer::Surface, Layer::Underground]));
    assert!(kinds.contains(&SiteKind::Dungeon));
    assert!(kinds.contains(&SiteKind::Ruin));
    assert!(kinds.contains(&SiteKind::Settlement));

    for (slot, index) in slots.into_iter().zip(waypoint_steps) {
        let at = path[index];
        if slot.layer == Layer::Underground {
            assert!(world.ground().solid([at[0], at[1] + WALKER_HEIGHT, at[2]]));
        } else {
            assert_eq!(world.grown().places.at(at), Some(slot.place));
        }
    }
}

#[test]
fn movement_save_replays_without_persisting_world_or_navigation() {
    let world = World::generate(SEED, WorldConfig::default()).unwrap();
    let (_, path, _) = receipt_plan(&world);
    let halfway = path.len() / 2;
    let mut straight = movement(&world, path[0]);
    for toward in path.iter().copied().take(halfway).skip(1) {
        straight.step(&world, PLAYED, toward).unwrap();
    }
    let world_saved = world.save().unwrap();
    let movement_saved = straight.save().unwrap();
    let restored_world = World::restore(&world_saved).unwrap();
    let mut restored = Movement::restore(&restored_world, &movement_saved).unwrap();
    assert_eq!(restored, straight);

    for toward in path.iter().copied().skip(halfway) {
        straight.step(&world, PLAYED, toward).unwrap();
        restored.step(&restored_world, PLAYED, toward).unwrap();
    }
    assert_eq!(restored, straight);
    assert_eq!(
        restored.state_hash().unwrap(),
        straight.state_hash().unwrap()
    );
}

#[test]
fn navigation_is_deterministic_advice() {
    let first = World::generate(SEED, WorldConfig::default()).unwrap();
    let second = World::generate(SEED, WorldConfig::default()).unwrap();
    let (_, first_path, _) = receipt_plan(&first);
    let (_, second_path, _) = receipt_plan(&second);
    assert_eq!(first_path, second_path);

    let movement = movement(&first, first_path[0]);
    assert_eq!(
        movement.save_record().intents.len(),
        1,
        "recorded spawn only"
    );
}

#[test]
fn blocked_input_is_a_recorded_hold_and_wrong_order_is_refused() {
    let world = World::generate(SEED, WorldConfig::default()).unwrap();
    let (_, path, _) = receipt_plan(&world);
    let mut movement = movement(&world, path[0]);
    let held = movement.step(&world, PLAYED, path[0]).unwrap();
    assert!(matches!(held, paredros_world::MovementEvent::Held { .. }));
    let count = movement.intents().len();
    assert!(matches!(
        movement.apply(
            &world,
            MovementIntent::Step {
                tick: Tick(99),
                subject: PLAYED,
                toward: path[1],
            }
        ),
        Err(MovementError::WrongTick { .. })
    ));
    assert_eq!(movement.intents().len(), count);
}

#[test]
fn the_same_store_moves_played_and_unplayed_subjects() {
    let world = World::generate(SEED, WorldConfig::default()).unwrap();
    let (_, path, _) = receipt_plan(&world);
    let mut movement = Movement::new();
    for subject in [PLAYED, SubjectId(44)] {
        movement.spawn(&world, subject, path[0]).unwrap();
        movement.step(&world, subject, path[1]).unwrap();
    }
    assert_eq!(movement.position(PLAYED), movement.position(SubjectId(44)));
    assert_eq!(movement.positions().count(), 2);
}
