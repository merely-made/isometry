// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One room, carved into a hillside of a grown world.
//!
//! Nothing here invents terrain. `Places::grown` decides the relief,
//! `Ground::grow` raises the bricks, and the room is one `carve` into rock
//! the world already had. The hunt below only picks *where*: the first
//! buried spot, in a deterministic outward scan, whose whole footprint is
//! solid and whose thinnest overburden still leaves a roof.

use mesocosm_core::places::{Ground, Places, WALKER_HEIGHT};

/// The world this probe plays in. One seed, so the room is the same room on
/// every machine and in every replay.
pub const SEED: u64 = 4_242;
/// Enclosure half-extent, in voxels. The value mesocosm's own place tests use.
pub const EXTENT: i32 = 64;
/// Place-graph side. Four by four regions over the enclosure.
pub const SIDE: u16 = 4;
/// Half-extent of the carved room: a nine-cubed chamber.
pub const ROOM_RADIUS: i32 = 4;
/// Voxels of rock the room must keep between its ceiling and the open air.
pub const ROOF: i32 = 2;
/// How far out the hillside hunt scans, in rings of columns.
const SEARCH_RINGS: i32 = 48;
/// The thinnest overburden a hillside may have and still hold this room:
/// floor clearance above bedrock, the room itself, and the roof.
const MIN_HILL: i32 = 2 * ROOM_RADIUS + ROOF + 2;

/// A grown world with one room in it, and the stance the body starts from.
#[derive(Clone, Debug)]
pub struct Room {
    pub seed: u64,
    pub ground: Ground,
    /// Centre of the carved chamber, in voxels.
    pub centre: [i32; 3],
    pub radius: i32,
    /// Where the body stands at tick zero: the middle of the room's floor.
    pub stance: [i32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomError {
    /// No column in range carried enough rock to bury a room of this size.
    NoHillside,
}

impl Room {
    /// Grows the world and carves the room. Deterministic from `seed` alone,
    /// which is what makes a save able to name a world by its seed.
    pub fn grow(seed: u64) -> Result<Self, RoomError> {
        let grown = Places::grown(seed, SIDE, EXTENT);
        let mut ground = Ground::grow(&grown, EXTENT);
        let centre = hillside(&ground, ROOM_RADIUS).ok_or(RoomError::NoHillside)?;

        let removed = ground.carve(centre, ROOM_RADIUS);
        assert!(
            removed > 0,
            "the hunt returned {centre:?} as solid rock and the carve removed nothing"
        );

        let stance = [centre[0], centre[1] - ROOM_RADIUS, centre[2]];
        assert!(
            ground.stands(stance, WALKER_HEIGHT),
            "the carved floor is not standable at {stance:?}"
        );

        Ok(Self {
            seed,
            ground,
            centre,
            radius: ROOM_RADIUS,
            stance,
        })
    }

    /// The room's interior bounds, in voxels: lowest air and highest air.
    pub fn interior(&self) -> ([i32; 3], [i32; 3]) {
        let r = self.radius;
        (
            [self.centre[0] - r, self.centre[1] - r, self.centre[2] - r],
            [self.centre[0] + r, self.centre[1] + r, self.centre[2] + r],
        )
    }

    /// The bricks worth meshing: everything within `reach` voxels of the
    /// room. Meshing the whole enclosure would draw a hundred thousand
    /// quads nobody standing in this chamber can see.
    pub fn nearby_bricks(&self, reach: i32) -> Vec<[i16; 3]> {
        let brick = mesocosm_core::places::BRICK;
        self.ground
            .keys()
            .filter(|key| {
                (0..3).all(|axis| {
                    let low = key[axis] as i32 * brick;
                    let high = low + brick - 1;
                    let want = self.centre[axis];
                    high >= want - reach && low <= want + reach
                })
            })
            .collect()
    }
}

/// Finds a buried chamber site: the first column, scanning outward from the
/// origin, whose surroundings carry enough rock overhead and whose footprint
/// is solid all the way through.
fn hillside(ground: &Ground, radius: i32) -> Option<[i32; 3]> {
    let margin = radius + 1;
    for ring in 0..=SEARCH_RINGS {
        for z in -ring..=ring {
            for x in -ring..=ring {
                if x.abs().max(z.abs()) != ring {
                    continue;
                }
                // The thinnest column over the footprint decides the depth,
                // so the roof holds everywhere rather than only at the middle.
                let Some(low) = thinnest(ground, [x, z], margin) else {
                    continue;
                };
                if low < MIN_HILL {
                    continue;
                }
                let centre = [x, low - ROOF - radius, z];
                if centre[1] - radius < 2 {
                    continue;
                }
                if buried(ground, centre, margin) {
                    return Some(centre);
                }
            }
        }
    }
    None
}

/// The lowest surface over a square footprint. `None` if any column in it is
/// off the enclosure entirely.
fn thinnest(ground: &Ground, at: [i32; 2], margin: i32) -> Option<i32> {
    let mut low = i32::MAX;
    for dz in -margin..=margin {
        for dx in -margin..=margin {
            low = low.min(ground.surface(at[0] + dx, at[1] + dz)?);
        }
    }
    Some(low)
}

/// Whether a cube of half-extent `margin` about `centre` is solid throughout.
/// Checked one voxel wider than the carve, so the chamber gets real walls
/// even where a nest already hollowed the hillside next door.
fn buried(ground: &Ground, centre: [i32; 3], margin: i32) -> bool {
    for dy in -margin..=margin {
        for dz in -margin..=margin {
            for dx in -margin..=margin {
                if !ground.solid([centre[0] + dx, centre[1] + dy, centre[2] + dz]) {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_room_is_a_roofed_chamber_with_a_floor_to_stand_on() {
        let room = Room::grow(SEED).expect("this seed has a hillside");
        let (low, high) = room.interior();

        for y in low[1]..=high[1] {
            for z in low[2]..=high[2] {
                for x in low[0]..=high[0] {
                    assert!(
                        !room.ground.solid([x, y, z]),
                        "rock left inside at {x},{y},{z}"
                    );
                }
            }
        }

        // A chamber, not a pit: rock above the ceiling and under the floor.
        let c = room.centre;
        assert!(room.ground.solid([c[0], high[1] + 1, c[2]]), "no roof");
        assert!(room.ground.solid([c[0], low[1] - 1, c[2]]), "no floor");
        assert!(room.ground.stands(room.stance, WALKER_HEIGHT));
    }

    #[test]
    fn the_room_has_walls_the_walker_cannot_cross() {
        let room = Room::grow(SEED).expect("this seed has a hillside");
        let (low, high) = room.interior();
        for y in low[1]..=high[1] {
            assert!(
                room.ground.solid([high[0] + 1, y, room.centre[2]]),
                "east wall"
            );
            assert!(
                room.ground.solid([low[0] - 1, y, room.centre[2]]),
                "west wall"
            );
            assert!(
                room.ground.solid([room.centre[0], y, high[2] + 1]),
                "north wall"
            );
            assert!(
                room.ground.solid([room.centre[0], y, low[2] - 1]),
                "south wall"
            );
        }
    }

    #[test]
    fn the_same_seed_carves_the_same_room() {
        let a = Room::grow(SEED).unwrap();
        let b = Room::grow(SEED).unwrap();
        assert_eq!(a.centre, b.centre);
        assert_eq!(a.stance, b.stance);
        assert_eq!(a.ground, b.ground);
    }

    #[test]
    fn only_the_bricks_around_the_room_are_meshed() {
        let room = Room::grow(SEED).unwrap();
        let near = room.nearby_bricks(16);
        assert!(!near.is_empty(), "the room sits in bricks");
        assert!(
            near.len() < room.ground.brick_count(),
            "the whole enclosure is not the room's neighbourhood"
        );
    }
}
