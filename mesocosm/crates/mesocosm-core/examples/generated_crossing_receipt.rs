// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G4's authoritative generated-threshold and place-boundary receipt.

use std::time::Instant;

use mesocosm_core::organism::FaunaDrive;
use mesocosm_core::places::{Places, WALKER_HEIGHT, step};
use mesocosm_core::world::{ENCLOSURE, PLACE_SALT, PLACE_SIDE};
use mesocosm_core::{
    History, Intent, Kingdom, Organism, OrganismId, Outcome, SpeciesId, VolumeRef, World,
    state_hash,
};

const SEED: u64 = 0;
const PLAYER: OrganismId = OrganismId(0);
const HUNTER: OrganismId = OrganismId(900);

fn main() {
    let grown = Places::grown(SEED ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let (route, boundary_step) = grown
        .nest_entries(ENCLOSURE)
        .find_map(|(_, entry)| {
            let boundary_step = entry
                .route
                .windows(2)
                .position(|pair| grown.places.at(pair[0]) != grown.places.at(pair[1]))?;
            Some((entry.route, boundary_step))
        })
        .expect("seed 0 has a generated entry crossing a place boundary");
    let from_place = grown.places.at(route[boundary_step]).unwrap();
    let to_place = grown.places.at(route[boundary_step + 1]).unwrap();
    assert_ne!(from_place, to_place);

    let mut world = setup(&route);
    let mut twin = setup(&route);
    let mut history = History::new();
    let mut twin_history = History::new();
    let mut max_tick_us = 0;
    let trace = route
        .windows(2)
        .skip(1)
        .map(|pair| Intent::Move {
            delta: [0, 1, 2].map(|axis| pair[1][axis] - pair[0][axis]),
        })
        .collect::<Vec<_>>();

    for (index, intent) in trace.iter().enumerate() {
        let started = Instant::now();
        let outcome = world.apply(intent.clone());
        max_tick_us = max_tick_us.max(started.elapsed().as_micros());
        let twin_outcome = twin.apply(intent.clone());
        history.record_all(world.drain_events());
        twin_history.record_all(twin.drain_events());
        assert_eq!(outcome, twin_outcome);
        assert!(matches!(outcome, Outcome::Moved));
        assert_eq!(position(&world, PLAYER), route[index + 2]);
        assert_eq!(position(&world, HUNTER), route[index + 1]);
        assert!(
            world
                .ground()
                .stands(position(&world, HUNTER), WALKER_HEIGHT)
        );
        assert_eq!(
            world
                .organisms
                .iter()
                .find(|organism| organism.id == HUNTER)
                .and_then(|organism| organism.last_fauna_decision.as_ref())
                .map(|decision| decision.selected_drive),
            Some(FaunaDrive::Pursue)
        );
    }

    assert_eq!(step(world.ground(), route[0], route[1]), route[1]);
    assert_eq!(world.places().at(position(&world, PLAYER)), Some(to_place));
    assert_eq!(world.places().at(position(&world, HUNTER)), Some(to_place));
    assert_eq!(state_hash(&world), state_hash(&twin));
    assert_eq!(history, twin_history);
    println!(
        "generated-crossing receipt: seed={SEED}, route={route:?}, threshold_step=0, \
         boundary_step={boundary_step}, places={from_place:?}->{to_place:?}, intents={}, \
         player={:?}->{:?}, hunter={:?}->{:?}, max_tick_us={max_tick_us}, \
         population=2, state_hash={:?}, history_events={}",
        trace.len(),
        route[1],
        position(&world, PLAYER),
        route[0],
        position(&world, HUNTER),
        state_hash(&world),
        history.log().len(),
    );
}

fn setup(route: &[[i32; 3]]) -> World {
    let mut world = World::new(SEED, 0);
    for pair in route.windows(2) {
        assert_eq!(step(world.ground(), pair[0], pair[1]), pair[1]);
    }
    world.organisms = vec![
        Organism::founding(
            PLAYER,
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            route[1],
            300,
        ),
        Organism::founding(
            HUNTER,
            SpeciesId(2),
            Kingdom::Consumer,
            VolumeRef::from_tag(16),
            [3, 1, 1],
            route[0],
            300,
        ),
    ];
    world
}

fn position(world: &World, id: OrganismId) -> [i32; 3] {
    world
        .organisms
        .iter()
        .find(|organism| organism.id == id)
        .expect("the crossing retains both organisms")
        .position
}
