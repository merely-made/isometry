// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The small authored habitat used to inspect one clearing and burrow.

use crate::body::{Attachment, PartId, Provenance, Yaw};
use crate::development::{DevelopmentError, PartPalette};
use crate::flow::Envelope;
use crate::history::Event;
use crate::organism::{Kingdom, Organism, OrganismId, Stage};
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

/// The authored carcass in [`World::graft_practice`].
///
/// It is deliberately outside the founded id range, so a host can identify the
/// one practice source without inferring a generated creature's identity.
pub const GRAFT_PRACTICE_DONOR: OrganismId = OrganismId(9_701);
/// The non-root branch root on [`GRAFT_PRACTICE_DONOR`].
pub const GRAFT_PRACTICE_BRANCH: PartId = PartId(1);
const GRAFT_PRACTICE_ROOT_MG: u64 = 600;
const GRAFT_PRACTICE_BRANCH_MG: u64 = 120;
const GRAFT_PRACTICE_TIP_MG: u64 = 80;
const GRAFT_PRACTICE_CLEARING_RADIUS: i32 = 7;
const GRAFT_PRACTICE_CLEARING_HEIGHT: i32 = 16;

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

    /// Founds the CP1 terrain with one compact played recipient and one nearby
    /// compatible carcass carrying a two-part branch.
    ///
    /// This is an authored practice fixture, not a claim about a natural
    /// encounter. It uses the caller's admitted palette, the normal body and
    /// phenotype attachment APIs, and a same-line carrion donor so either
    /// normal graft preview has a deterministic affinity answer. The source
    /// branch is paid out of the donor root before it is attached, leaving the
    /// fixture's matter accounting whole before a graft ever occurs.
    pub fn graft_practice(
        seed: u64,
        founding: Founding,
        palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        let mut world = Self::terrarium(seed, founding, palette)?;
        let recipient_id = world.controlled_id().expect("a terrarium is embodied");
        let recipient_index = world
            .organisms
            .iter()
            .position(|organism| organism.id == recipient_id)
            .expect("the embodied terrarium organism exists");
        let recipient_at = world.organisms[recipient_index].position;
        let species = world.organisms[recipient_index].species;
        let bulk = palette.mass.default;

        // A normal generated body can leave no plan-resolved landing site for
        // a compact branch. This recipient is deliberately small, while still
        // using the normal consumer constructor and the caller's vocabulary.
        let recipient = Organism::founding(
            recipient_id,
            species,
            Kingdom::Consumer,
            bulk.volume,
            bulk.half_extent,
            recipient_at,
            1_000,
        )
        .matured();
        world.ground.author_walker_stance(
            recipient_at,
            GRAFT_PRACTICE_CLEARING_RADIUS,
            GRAFT_PRACTICE_CLEARING_HEIGHT,
            [0, 0],
        );
        assert!(recipient.walker_shape().stands(&world.ground, recipient_at));
        world.organisms[recipient_index] = recipient;

        // The clearing opened by `terrarium` holds this one-cell offset at the
        // same grounded height. Keep the corpse close enough for the compact
        // body's ordinary reach; its own footing remains part of the fixture.
        let donor_at = [recipient_at[0] + 1, recipient_at[1], recipient_at[2]];
        let mut donor = Organism {
            stage: Stage::Carrion,
            ..Organism::founding(
                GRAFT_PRACTICE_DONOR,
                species,
                Kingdom::Decomposer,
                bulk.volume,
                bulk.half_extent,
                donor_at,
                GRAFT_PRACTICE_ROOT_MG + GRAFT_PRACTICE_BRANCH_MG + GRAFT_PRACTICE_TIP_MG,
            )
        };
        world.ground.author_walker_stance(
            donor_at,
            GRAFT_PRACTICE_CLEARING_RADIUS,
            GRAFT_PRACTICE_CLEARING_HEIGHT,
            [0, 0],
        );
        assert!(donor.walker_shape().stands(&world.ground, donor_at));
        assert_eq!(
            donor
                .phenotype
                .spend_mass(GRAFT_PRACTICE_BRANCH_MG + GRAFT_PRACTICE_TIP_MG),
            0,
            "the authored donor funds its branch before it is attached"
        );
        let branch_shape = palette.sensor.default;
        let root = donor.body().root;
        let branch = donor
            .phenotype
            .attach(
                branch_shape.volume,
                GRAFT_PRACTICE_BRANCH_MG,
                branch_shape.half_extent,
                Attachment {
                    parent: root,
                    offset: [
                        0,
                        bulk.half_extent[1].abs() + branch_shape.half_extent[1],
                        0,
                    ],
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .expect("the authored branch attaches to its donor root");
        assert_eq!(branch, GRAFT_PRACTICE_BRANCH);
        donor
            .phenotype
            .attach(
                branch_shape.volume,
                GRAFT_PRACTICE_TIP_MG,
                branch_shape.half_extent,
                Attachment {
                    parent: branch,
                    offset: [branch_shape.half_extent[0] * 2, 0, 0],
                    yaw: Yaw::Quarter,
                },
                Provenance::founding(),
            )
            .expect("the authored branch carries a second part");
        assert!(
            donor.walker_shape().stands(&world.ground, donor_at),
            "the authored donor stays grounded with its branch"
        );
        world.organisms.push(donor);
        // The wider graft clearing can remove support beneath a neighbour's
        // footprint. Place affected founders on the finished terrain.
        for organism in &mut world.organisms {
            let shape = organism.walker_shape();
            if !shape.stands(&world.ground, organism.position) {
                let direction = match organism.kingdom() {
                    Kingdom::Producer => [1, 1],
                    Kingdom::Consumer => [-1, -1],
                    Kingdom::Decomposer => [1, -1],
                };
                organism.position = stance_near(&world.ground, shape, recipient_at, direction);
            }
        }
        world.next_organism = world
            .next_organism
            .max(GRAFT_PRACTICE_DONOR.0.saturating_add(1));
        world.frontier = world
            .controlled()
            .map(|organism| world.intricacy(organism))
            .unwrap_or(0);
        // This scene begins at the menu with no fabricated births obscuring
        // the first genuine graft record.
        world.pending.clear();
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
    use crate::body::Aabb;
    use crate::graft::Crossing;
    use crate::places::{BODY_VOXELS_PER_GROUND_VOXEL, WalkerShape, route_step_for};
    use crate::world::{Intent, Outcome};

    fn world() -> World {
        let founding = Founding::SpacedRoster;
        World::terrarium(41, founding, founding.palette()).expect("CP1 fixture")
    }

    fn world_aabb(aabb: Aabb, position: [i32; 3]) -> Aabb {
        let origin = position.map(|axis| axis * BODY_VOXELS_PER_GROUND_VOXEL);
        Aabb {
            min: [0, 1, 2].map(|axis| aabb.min[axis] + origin[axis]),
            max: [0, 1, 2].map(|axis| aabb.max[axis] + origin[axis]),
        }
    }

    fn disjoint(left: Aabb, right: Aabb) -> bool {
        (0..3).any(|axis| left.max[axis] <= right.min[axis] || right.max[axis] <= left.min[axis])
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

    #[test]
    fn graft_practice_transfers_its_authored_branch_without_moving_matter() {
        let founding = Founding::SpacedRoster;
        let mut world = World::graft_practice(41, founding, founding.palette())
            .expect("graft practice fixture");
        assert!(
            world.next_organism > GRAFT_PRACTICE_DONOR.0,
            "a later birth cannot reuse the authored donor id"
        );
        let before = world.total_matter_mg();
        let sources = world.graft_sources();
        assert_eq!(
            sources.first(),
            Some(&(GRAFT_PRACTICE_DONOR, GRAFT_PRACTICE_BRANCH)),
            "the branch root is the stable first menu source"
        );
        let recipient_at = world.position().expect("embodied");
        for crossing in [Crossing::Carry, Crossing::Regrow] {
            let preview = world
                .preview_graft(GRAFT_PRACTICE_DONOR, GRAFT_PRACTICE_BRANCH, crossing)
                .expect("the practice candidate is accepted");
            assert!(
                WalkerShape::from_aabb(preview.phenotype.body().aabb())
                    .stands(world.ground(), recipient_at),
                "the {crossing:?} candidate has a legal stance"
            );
        }
        let outcome = world.apply(Intent::Graft {
            organism: GRAFT_PRACTICE_DONOR,
            part: GRAFT_PRACTICE_BRANCH,
            crossing: Crossing::Carry,
        });
        assert!(
            matches!(outcome, Outcome::Grafted { parts: 2, .. }),
            "{outcome:?}"
        );
        let recipient = world
            .controlled()
            .expect("the graft leaves the body embodied");
        assert!(
            recipient
                .walker_shape()
                .stands(world.ground(), recipient.position),
            "the published graft remains grounded"
        );
        assert_eq!(world.total_matter_mg(), before, "grafting conserves matter");
    }

    #[test]
    fn graft_practice_preserves_every_founders_footing() {
        for seed in [7, 41] {
            let founding = Founding::SpacedRoster;
            let world = World::graft_practice(seed, founding, founding.palette()).unwrap();
            for organism in &world.organisms {
                assert!(
                    organism
                        .walker_shape()
                        .stands(world.ground(), organism.position),
                    "seed {seed}: {:?} lost its footing",
                    organism.id
                );
            }
        }
    }

    #[test]
    fn graft_practice_keeps_its_donor_clear_of_each_checked_candidate() {
        let founding = Founding::SpacedRoster;
        let world = World::graft_practice(41, founding, founding.palette())
            .expect("graft practice fixture");
        let recipient = world.controlled().expect("embodied");
        let donor = world
            .organisms
            .iter()
            .find(|organism| organism.id == GRAFT_PRACTICE_DONOR)
            .expect("authored donor");
        let donor_aabb = world_aabb(donor.body().aabb(), donor.position);
        assert!(
            disjoint(
                world_aabb(recipient.body().aabb(), recipient.position),
                donor_aabb
            ),
            "the authored bodies start apart"
        );
        for crossing in [Crossing::Carry, Crossing::Regrow] {
            let preview = world
                .preview_graft(GRAFT_PRACTICE_DONOR, GRAFT_PRACTICE_BRANCH, crossing)
                .expect("the authored candidate is valid");
            assert!(
                disjoint(
                    world_aabb(preview.phenotype.body().aabb(), recipient.position),
                    donor_aabb,
                ),
                "{crossing:?} candidate intersects the donor"
            );
        }
    }
}
