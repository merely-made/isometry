// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The small authored habitat used to inspect one clearing and burrow.

use crate::development::{DevelopmentError, PartPalette};
use crate::flow::Envelope;
use crate::history::Event;
use crate::organism::Kingdom;
use crate::places::{Ground, Places, surface_stance_for};

use super::{ENCLOSURE, Founding, World};

/// The Ground resident bound retained for existing full-size body recipes.
pub const TERRARIUM_EXTENT: i32 = ENCLOSURE;
/// Enough ordinary grounded transitions to traverse the authored entry.
pub const TERRARIUM_ROUTE_BUDGET: i32 = 96;
const TERRARIUM_TERRAIN_SEED: u64 = 4_242;
const TERRARIUM_FOUNDERS: u32 = 3;
const CLEARING_RADIUS: i32 = 5;
const HABITAT_HALF_WIDTH: i32 = 24;
const HABITAT_ABOVE_CLEARING: i32 = 12;
const HABITAT_BELOW_CHAMBER: i32 = 6;

/// World-aligned, immutable bounds for the CP1 camera and cutaway.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerrariumBounds {
    pub min: [i32; 3],
    pub max: [i32; 3],
}

/// The one physical habitat CP1 frames.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerrariumHabitat {
    pub bounds: TerrariumBounds,
    pub clearing: [i32; 3],
    pub entrance: [i32; 3],
    pub chamber: [i32; 3],
    pub route: Vec<[i32; 3]>,
}

impl World {
    /// Founds a four-critter CP1 scene on a fixed generated Ground habitat.
    ///
    /// The passed vocabulary is still the only body vocabulary. The scene
    /// merely places its existing producer, consumer, and decomposer forms at
    /// legal stances around Ground's own generated burrow route.
    pub fn terrarium(
        seed: u64,
        founding: Founding,
        palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        let mut world = Self::founded_with_palette(seed, TERRARIUM_FOUNDERS, founding, palette)?;
        let grown = Places::grown(TERRARIUM_TERRAIN_SEED, super::PLACE_SIDE, TERRARIUM_EXTENT);
        let entry = entry_from(&grown);
        world.places = grown.places.clone();
        world.ground = Ground::grow(&grown, TERRARIUM_EXTENT);

        let controlled = world.controlled_id().expect("a founded world is embodied");
        let shape = world
            .organisms
            .iter()
            .find(|organism| organism.id == controlled)
            .expect("controlled organism exists")
            .walker_shape();
        widen_route(&mut world.ground, &entry.route, shape);
        let clearing = entry.route[0];
        clear_opening(&mut world.ground, clearing);
        assert!(shape.stands(&world.ground, clearing));
        assert!(shape.stands(
            &world.ground,
            *entry.route.last().expect("entry has chamber")
        ));

        let food = stance_near(
            &world.ground,
            world
                .organisms
                .iter()
                .find(|organism| organism.kingdom() == Kingdom::Producer)
                .expect("three founders cover producer")
                .walker_shape(),
            clearing,
            [1, 1],
        );
        let threat = stance_near(
            &world.ground,
            world
                .organisms
                .iter()
                .find(|organism| {
                    organism.id != controlled && organism.kingdom() == Kingdom::Consumer
                })
                .expect("three founders cover another consumer")
                .walker_shape(),
            clearing,
            [-1, -1],
        );
        let detritivore = stance_near(
            &world.ground,
            world
                .organisms
                .iter()
                .find(|organism| organism.kingdom() == Kingdom::Decomposer)
                .expect("three founders cover decomposer")
                .walker_shape(),
            clearing,
            [1, -1],
        );
        for organism in &mut world.organisms {
            organism.position = if organism.id == controlled {
                clearing
            } else {
                match organism.kingdom() {
                    Kingdom::Producer => food,
                    Kingdom::Consumer => threat,
                    Kingdom::Decomposer => detritivore,
                }
            };
        }
        world.pending = world
            .organisms
            .iter()
            .map(|organism| {
                Envelope::new(
                    0,
                    world.places.at(organism.position),
                    Event::Born {
                        organism: organism.id,
                        species: organism.species,
                        parent: None,
                    },
                )
            })
            .collect();
        world.frontier = world
            .controlled()
            .map(|organism| world.intricacy(organism))
            .unwrap_or(0);
        let _ = world.ground.drain_dirty();
        Ok(world)
    }

    /// Reads CP1's fixed framing from its physical route.
    pub fn terrarium_habitat(&self) -> TerrariumHabitat {
        let route = pinned_entry().route;
        let clearing = route[0];
        let chamber = *route.last().expect("entry has chamber");
        TerrariumHabitat {
            bounds: bounds_for(clearing, chamber),
            clearing,
            entrance: clearing,
            chamber,
            route,
        }
    }
}

fn pinned_entry() -> crate::places::NestEntry {
    let grown = Places::grown(TERRARIUM_TERRAIN_SEED, super::PLACE_SIDE, TERRARIUM_EXTENT);
    entry_from(&grown)
}

fn entry_from(grown: &crate::places::Grown) -> crate::places::NestEntry {
    grown
        .nest_entries(TERRARIUM_EXTENT)
        .min_by_key(|(_, entry)| {
            let [x, _, z] = entry.route[0];
            x * x + z * z
        })
        .expect("the pinned CP1 terrain has an entry")
        .1
}

fn clear_opening(ground: &mut Ground, at: [i32; 3]) {
    ground.carve([at[0], at[1] + CLEARING_RADIUS, at[2]], CLEARING_RADIUS);
}

fn widen_route(ground: &mut Ground, route: &[[i32; 3]], shape: crate::places::WalkerShape) {
    // Work from the chamber back to the surface. A descending next stance
    // clears at the prior tread's floor height, so that tread keeps its one
    // support voxel on the trailing edge, outside the next footprint.
    for index in (0..route.len()).rev() {
        let at = route[index];
        let support_offset = route
            .get(index + 1)
            .map(|next| {
                [
                    (at[0] - next[0]).signum() * shape.radius(),
                    (at[2] - next[2]).signum() * shape.radius(),
                ]
            })
            .unwrap_or([0, 0]);
        ground.author_walker_stance(at, shape.radius(), shape.height(), support_offset);
    }
}

fn stance_near(
    ground: &Ground,
    shape: crate::places::WalkerShape,
    at: [i32; 3],
    direction: [i32; 2],
) -> [i32; 3] {
    for distance in 8..=24 {
        let candidate = [
            at[0] + direction[0] * distance,
            at[1],
            at[2] + direction[1] * distance,
        ];
        if let Some(stance) = surface_stance_for(ground, shape, candidate) {
            return stance;
        }
    }
    panic!("the pinned CP1 clearing has a stance for every founded form")
}

fn bounds_for(clearing: [i32; 3], chamber: [i32; 3]) -> TerrariumBounds {
    TerrariumBounds {
        min: [
            clearing[0] - HABITAT_HALF_WIDTH,
            chamber[1] - HABITAT_BELOW_CHAMBER,
            clearing[2] - HABITAT_HALF_WIDTH,
        ],
        max: [
            clearing[0] + HABITAT_HALF_WIDTH,
            clearing[1] + HABITAT_ABOVE_CLEARING,
            clearing[2] + HABITAT_HALF_WIDTH,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::places::{BODY_VOXELS_PER_GROUND_VOXEL, route_step_for};
    use crate::world::{Intent, Outcome};

    fn world() -> World {
        let founding = Founding::SpacedRoster;
        World::terrarium(41, founding, founding.palette()).expect("CP1 fixture")
    }

    #[test]
    fn founded_critters_stand_inside_the_authored_habitat() {
        let world = world();
        let habitat = world.terrarium_habitat();
        eprintln!("CP1 habitat: {habitat:?}");
        assert!(habitat.route.len() > 1);
        for organism in &world.organisms {
            assert!(
                organism
                    .walker_shape()
                    .stands(world.ground(), organism.position),
                "{:?} does not stand at {:?}",
                organism.id,
                organism.position
            );
        }
        assert!(habitat.bounds.min[0] <= habitat.chamber[0]);
        assert!(habitat.bounds.max[0] >= habitat.chamber[0]);
        assert!(habitat.bounds.min[1] <= habitat.chamber[1]);
        assert!(habitat.bounds.max[1] >= habitat.clearing[1]);
        for organism in &world.organisms {
            let body = organism.body().aabb();
            for axis in 0..3 {
                let min = organism.position[axis]
                    + body.min[axis].div_euclid(BODY_VOXELS_PER_GROUND_VOXEL);
                let max = organism.position[axis]
                    - (-body.max[axis]).div_euclid(BODY_VOXELS_PER_GROUND_VOXEL);
                assert!(
                    min >= habitat.bounds.min[axis],
                    "body escapes minimum on axis {axis}"
                );
                assert!(
                    max <= habitat.bounds.max[axis],
                    "body escapes maximum on axis {axis}"
                );
            }
        }
    }

    #[test]
    fn ordinary_grounded_moves_reach_the_chamber() {
        let mut world = world();
        let chamber = world.terrarium_habitat().chamber;
        for _ in 0..TERRARIUM_ROUTE_BUDGET {
            let organism = world.controlled().expect("embodied");
            if organism.position == chamber {
                break;
            }
            let next = route_step_for(
                world.ground(),
                organism.walker_shape(),
                organism.position,
                chamber,
                TERRARIUM_ROUTE_BUDGET,
            )
            .expect("the generated entry has a grounded route");
            let delta = [
                next[0] - organism.position[0],
                0,
                next[2] - organism.position[2],
            ];
            assert_eq!(world.apply(Intent::Move { delta }), Outcome::Moved);
        }
        assert_eq!(world.position(), Some(chamber));
    }
}
