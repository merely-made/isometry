// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The carved-doorway fixture retained for G4's interactive judgment harness.

use mesocosm_core::places::{WALKER_HEIGHT, spot, step};
use mesocosm_core::world::ENCLOSURE;
use mesocosm_core::{Kingdom, Organism, OrganismId, SpeciesId, VolumeRef, World};

pub const SEED: u64 = 4_242;
pub const PLAYER_ID: OrganismId = OrganismId(0);
pub const HUNTER_ID: OrganismId = OrganismId(900);

pub struct BurrowFixture {
    pub world: World,
    pub hunter_start: [i32; 3],
    pub doorway: [i32; 3],
    pub player: [i32; 3],
}

pub fn setup() -> BurrowFixture {
    let mut world = World::new(SEED, 0);
    let directions = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    let (hunter_start, doorway, player) = (-ENCLOSURE..=ENCLOSURE)
        .find_map(|z| {
            (-ENCLOSURE..=ENCLOSURE).find_map(|x| {
                let top = world.ground().surface(x, z)?;
                let hunter_start = [x, top + 1, z];
                if !world.ground().stands(hunter_start, WALKER_HEIGHT) {
                    return None;
                }
                directions.into_iter().find_map(|[dx, dz]| {
                    let doorway = [hunter_start[0] + dx, hunter_start[1], hunter_start[2] + dz];
                    let top = world.ground().surface(doorway[0] + dx, doorway[2] + dz)?;
                    let player = [doorway[0] + dx, top + 1, doorway[2] + dz];
                    let blocked = step(world.ground(), hunter_start, doorway) == hunter_start
                        && world.ground().solid(doorway)
                        && world
                            .ground()
                            .solid([doorway[0], doorway[1] + 1, doorway[2]])
                        && world
                            .ground()
                            .solid([doorway[0], doorway[1] - 1, doorway[2]]);
                    let close = (0..3).all(|axis| (player[axis] - hunter_start[axis]).abs() <= 8);
                    (blocked
                        && close
                        && world.ground().stands(player, WALKER_HEIGHT)
                        && !spot(world.ground(), hunter_start, player, 8))
                    .then_some((hunter_start, doorway, player))
                })
            })
        })
        .expect("seed 4242 has an occluded grounded doorway");
    world.organisms = vec![
        Organism::founding(
            PLAYER_ID,
            SpeciesId(3),
            Kingdom::Producer,
            VolumeRef::from_tag(18),
            [1, 1, 1],
            player,
            300,
        ),
        Organism::founding(
            HUNTER_ID,
            SpeciesId(2),
            Kingdom::Consumer,
            VolumeRef::from_tag(16),
            [3, 1, 1],
            hunter_start,
            300,
        ),
    ];
    BurrowFixture {
        world,
        hunter_start,
        doorway,
        player,
    }
}
