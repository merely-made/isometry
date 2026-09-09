// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The authoritative half of G4's composed run.
//!
//! A played producer is hidden from a near consumer by generated Ground. It
//! carves the blocking doorway, the consumer finds it through the new sight
//! line and crosses legally, and a twin replays the exact ordered trace.

use std::time::Instant;

use mesocosm_core::places::{WALKER_HEIGHT, spot, step};
use mesocosm_core::world::ENCLOSURE;
use mesocosm_core::{
    History, Intent, Kingdom, Organism, OrganismId, Outcome, SpeciesId, VolumeRef, World,
    state_hash,
};

const SEED: u64 = 4_242;

struct Run {
    world: World,
    trace: Vec<Intent>,
    from: [i32; 3],
    doorway: [i32; 3],
    player: [i32; 3],
}

fn main() {
    let mut run = setup();
    let mut twin = setup();
    let mut history = History::new();
    let mut twin_history = History::new();
    let mut outcomes = Vec::with_capacity(run.trace.len());
    let mut dirty_slots = 0usize;
    let mut carve_us = 0u128;

    assert!(
        !spot(run.world.ground(), run.from, run.player, 8),
        "the hunter saw the player before the opening"
    );
    for intent in &run.trace {
        let started = Instant::now();
        let outcome = run.world.apply(intent.clone());
        if matches!(intent, Intent::Carve { .. }) {
            carve_us = started.elapsed().as_micros();
            // The dirty queue is projection state. A host would drain the
            // live queue; this clone observes the same revision work without
            // giving the receipt a mutation back-door into `World`.
            let mut projection = run.world.ground().clone();
            dirty_slots = projection.drain_dirty().len();
        }
        history.record_all(run.world.drain_events());
        let twin_outcome = twin.world.apply(intent.clone());
        twin_history.record_all(twin.world.drain_events());
        assert_eq!(outcome, twin_outcome, "replay changed an outcome");
        outcomes.push(outcome);
    }

    assert!(matches!(outcomes[0], Outcome::Idled));
    assert!(matches!(outcomes[1], Outcome::Carved { removed, .. } if removed > 0));
    assert!(spot(run.world.ground(), run.from, run.player, 8));
    assert_eq!(
        run.world
            .organisms
            .iter()
            .find(|organism| organism.id == OrganismId(900))
            .map(|organism| organism.position),
        Some(run.doorway),
        "the hunter did not cross the opened doorway"
    );
    assert!(run.world.ground().stands(run.doorway, WALKER_HEIGHT));
    assert_eq!(state_hash(&run.world), state_hash(&twin.world));
    assert_eq!(history, twin_history);
    println!(
        "burrow-run receipt: seed={SEED}, player={:?}, hunter_from={:?}, doorway={:?}, \
         carve_us={carve_us}, dirty_bricks={dirty_slots}, state_hash={:?}, history_events={}",
        run.player,
        run.from,
        run.doorway,
        state_hash(&run.world),
        history.log().len(),
    );
}

fn setup() -> Run {
    let mut world = World::new(SEED, 0);
    let directions = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    let (from, doorway, player) = (-ENCLOSURE..=ENCLOSURE)
        .find_map(|z| {
            (-ENCLOSURE..=ENCLOSURE).find_map(|x| {
                let top = world.ground().surface(x, z)?;
                let from = [x, top + 1, z];
                if !world.ground().stands(from, WALKER_HEIGHT) {
                    return None;
                }
                directions.into_iter().find_map(|[dx, dz]| {
                    let doorway = [from[0] + dx, from[1], from[2] + dz];
                    let top = world.ground().surface(doorway[0] + dx, doorway[2] + dz)?;
                    let player = [doorway[0] + dx, top + 1, doorway[2] + dz];
                    let blocked = step(world.ground(), from, doorway) == from
                        && world.ground().solid(doorway)
                        && world
                            .ground()
                            .solid([doorway[0], doorway[1] + 1, doorway[2]])
                        && world
                            .ground()
                            .solid([doorway[0], doorway[1] - 1, doorway[2]]);
                    let close = (0..3).all(|axis| (player[axis] - from[axis]).abs() <= 8);
                    (blocked
                        && close
                        && world.ground().stands(player, WALKER_HEIGHT)
                        && !spot(world.ground(), from, player, 8))
                    .then_some((from, doorway, player))
                })
            })
        })
        .expect("seed-4242 contains an occluded grounded doorway");
    // ID zero remains the controlled organism founded by `World::new`; its
    // body is now the player, while the consumer becomes autonomous.
    world.organisms = vec![
        Organism::founding(
            OrganismId(0),
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            player,
            300,
        ),
        Organism::founding(
            OrganismId(900),
            SpeciesId(2),
            Kingdom::Consumer,
            VolumeRef::from_tag(16),
            [3, 1, 1],
            from,
            300,
        ),
    ];
    Run {
        world,
        trace: vec![
            Intent::Idle,
            Intent::Carve {
                at: [doorway[0], doorway[1] + 1, doorway[2]],
                radius: 1,
            },
        ],
        from,
        doorway,
        player,
    }
}
