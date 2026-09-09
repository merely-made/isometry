// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A playable family origin in one authored clearing and roofed burrow.

use crate::development::{DevelopmentError, PartPalette};
use crate::places::{BODY_VOXELS_PER_GROUND_VOXEL, WalkerShape};

use super::{Founding, TerrariumHabitat, World};

const CLEARING_RADIUS: i32 = 6;
const CLEARING_HEADROOM: i32 = 5;
const CAVE_TUNNEL_STEPS: i32 = 7;
const CAVE_RADIUS: i32 = 2;
const DESCENT_STEPS: i32 = 5;
const APPROACH_STEPS: i32 = 1;

impl World {
    /// Founds the family origin in a clearing with a reachable roofed chamber.
    ///
    /// The biology and recorded origin are exactly [`Self::family_practice`].
    /// This layer only authors a readable terrain treatment around that origin.
    /// `reserve_assisted` is a paired diagnosis switch: both variants have the
    /// same bodies, history, places, and Ground; the lean variant returns the
    /// parent's excess setup reserve to the soil under its finished stance.
    pub fn family_clearing(
        seed: u64,
        founding: Founding,
        palette: PartPalette,
        reserve_assisted: bool,
    ) -> Result<Self, DevelopmentError> {
        let mut world = Self::family_practice(seed, founding, palette)?;
        let habitat = world.family_clearing_habitat();
        author_family_habitat(&mut world, &habitat);

        if !reserve_assisted {
            let parent = world.controlled_id().expect("family origin is embodied");
            let parent = world
                .organisms
                .iter_mut()
                .find(|organism| organism.id == parent)
                .expect("family parent remains in the origin");
            // The parent was explicitly provisioned so its two ordinary births
            // can establish the authored origin. Once that origin exists, a
            // normal founding body's own mass is the retained reserve ceiling.
            let cap = parent.biomass_mg();
            let excess = parent.energy_mg.saturating_sub(cap);
            parent.energy_mg -= excess;
            let column = world.soil.column_at(parent.position);
            world.soil.deposit(column, excess);
        }

        let _ = world.ground.drain_dirty();
        Ok(world)
    }

    /// Reads the fixed physical framing of [`Self::family_clearing`].
    pub fn family_clearing_habitat(&self) -> TerrariumHabitat {
        let mut habitat = self.terrarium_habitat();
        let clearing = habitat.clearing;
        let next = habitat
            .route
            .get(1)
            .copied()
            .expect("terrarium route has an approach");
        let direction = [
            (next[0] - clearing[0]).signum(),
            (next[2] - clearing[2]).signum(),
        ];
        habitat.route.clear();
        for step in 0..=APPROACH_STEPS + DESCENT_STEPS + CAVE_TUNNEL_STEPS {
            habitat.route.push([
                clearing[0] + direction[0] * step,
                clearing[1] - (step - APPROACH_STEPS).max(0).min(DESCENT_STEPS),
                clearing[2] + direction[1] * step,
            ]);
        }
        habitat.chamber = *habitat.route.last().expect("extended cave endpoint");
        for axis in [0, 2] {
            habitat.bounds.min[axis] =
                habitat.bounds.min[axis].min(habitat.chamber[axis] - CAVE_RADIUS);
            habitat.bounds.max[axis] =
                habitat.bounds.max[axis].max(habitat.chamber[axis] + CAVE_RADIUS);
        }
        for organism in &self.organisms {
            expand_bounds(&mut habitat, organism.position, organism.body().aabb());
        }
        // Reading a frame must never rehearse an opening on the current world:
        // after play, tissue or reserves may no longer admit that sequence.
        habitat.bounds.max[1] = habitat.bounds.max[1].max(clearing[1] + CLEARING_HEADROOM);
        habitat
    }
}

fn expand_bounds(habitat: &mut TerrariumHabitat, position: [i32; 3], body: crate::body::Aabb) {
    for axis in 0..3 {
        let floor = if axis == 1 { body.min[axis] } else { 0 };
        let min =
            position[axis] + (body.min[axis] - floor).div_euclid(BODY_VOXELS_PER_GROUND_VOXEL);
        let max =
            position[axis] - (-(body.max[axis] - floor)).div_euclid(BODY_VOXELS_PER_GROUND_VOXEL);
        habitat.bounds.min[axis] = habitat.bounds.min[axis].min(min);
        habitat.bounds.max[axis] = habitat.bounds.max[axis].max(max);
    }
}

fn clearance_shapes(world: &World) -> Vec<WalkerShape> {
    let mut shapes: Vec<_> = world
        .organisms
        .iter()
        .map(|organism| organism.walker_shape())
        .collect();
    let mut opening = world.clone();
    if let Some(intents) = opening.family_practice_opening() {
        for intent in intents {
            let outcome = opening.apply(intent);
            assert!(
                !matches!(outcome, crate::Outcome::Rejected(_)),
                "the authored family opening must admit every terrain-clearance shape: {outcome:?}"
            );
            let shape = opening
                .controlled()
                .expect("opening keeps its parent embodied")
                .walker_shape();
            shapes.push(shape);
        }
    }
    shapes
}

fn author_family_habitat(world: &mut World, habitat: &TerrariumHabitat) {
    let shapes = clearance_shapes(world);
    let parent_shape = *shapes
        .last()
        .expect("the authored opening has a final played shape");
    let radius = shapes.iter().map(|shape| shape.radius()).max().unwrap_or(0);
    let height = shapes.iter().map(|shape| shape.height()).max().unwrap_or(1);

    // Descend from the clearing, then continue level into the cave. Widen each
    // tread from bottom to top, preserving a trailing floor edge for every
    // actual body shape in the origin.
    for index in (0..habitat.route.len()).rev() {
        let at = habitat.route[index];
        let support_offset = habitat
            .route
            .get(index + 1)
            .map(|next| {
                [
                    (at[0] - next[0]).signum() * radius,
                    (at[2] - next[2]).signum() * radius,
                ]
            })
            .unwrap_or([0, 0]);
        // One trailing floor per tread. Smaller bodies settle sooner; adding
        // their centre floors would obstruct the enlarged body's next step.
        world
            .ground
            .author_walker_stance(at, radius, height, support_offset);
    }

    // Open the air above the mouth without touching its floor. The generated
    // chamber keeps its own roof and floor; authoring the route above joins it
    // to this clearing through ordinary grounded transitions.
    let clearing = habitat.clearing;
    let _ = world.ground.carve(
        [clearing[0], clearing[1] + CLEARING_HEADROOM, clearing[2]],
        CLEARING_RADIUS,
    );
    let parent = world.controlled_id().expect("family origin is embodied");
    for (index, organism) in world.organisms.iter_mut().enumerate() {
        if organism.id != parent {
            // The ordinary births initially share their parent's compact
            // stance. Set them around the clearing before authoring footing,
            // so a relative cannot replace a descending tunnel tread's floor.
            organism.position = [
                clearing[0] - 3,
                clearing[1],
                clearing[2] + if index % 2 == 0 { -3 } else { 3 },
            ];
        }
    }
    for organism in &world.organisms {
        let (shape, support_offset) = if organism.id == parent {
            // A level approach lets the enlarged body clear this founding
            // centre floor before descending. The small origin still needs it.
            (parent_shape, [0, 0])
        } else {
            (organism.walker_shape(), [0, 0])
        };
        world.ground.author_walker_stance(
            organism.position,
            shape.radius(),
            shape.height(),
            support_offset,
        );
        assert!(shape.stands(&world.ground, organism.position));
    }
    // Make the cave pocket broader than its connecting tunnel. The same
    // authoring primitive provides all its floor cells and clear body volume.
    for dz in -CAVE_RADIUS..=CAVE_RADIUS {
        for dx in -CAVE_RADIUS..=CAVE_RADIUS {
            world.ground.author_walker_stance(
                [
                    habitat.chamber[0] + dx,
                    habitat.chamber[1],
                    habitat.chamber[2] + dz,
                ],
                radius,
                height,
                [0, 0],
            );
        }
    }
    // The descent stays open to the clearing. Start the roof only after its
    // widest body can no longer overlap an upper ramp tread.
    let roof_y = habitat.chamber[1] + height;
    let tunnel_start = (APPROACH_STEPS + DESCENT_STEPS + 2 * radius) as usize;
    for at in habitat.route.iter().skip(tunnel_start) {
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                world.ground.author_walker_stance(
                    [at[0] + dx, roof_y + 1, at[2] + dz],
                    0,
                    1,
                    [0, 0],
                );
            }
        }
    }
    // The chamber uses the tunnel's same low ceiling.
    let cave_roof_y = roof_y;
    for dz in -(CAVE_RADIUS + radius)..=(CAVE_RADIUS + radius) {
        for dx in -(CAVE_RADIUS + radius)..=(CAVE_RADIUS + radius) {
            world.ground.author_walker_stance(
                [
                    habitat.chamber[0] + dx,
                    cave_roof_y + 1,
                    habitat.chamber[2] + dz,
                ],
                0,
                1,
                [0, 0],
            );
        }
    }
    assert!(world.ground.solid([
        habitat.chamber[0],
        habitat.chamber[1] - 1,
        habitat.chamber[2]
    ]));
    assert!(world.ground.solid([
        habitat.chamber[0],
        habitat.chamber[1] + height,
        habitat.chamber[2]
    ]));
    let parent_at = world.position().expect("family parent remains embodied");
    for shape in &shapes {
        assert!(
            shape.stands(&world.ground, parent_at),
            "every admitted opening body stands at the clearing mouth"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::places::route_step_for;
    use crate::world::TERRARIUM_ROUTE_BUDGET;
    use crate::{Founding, Intent, restore, snapshot, state_hash};

    fn fixture(seed: u64, reserve_assisted: bool) -> World {
        let founding = Founding::SpacedRoster;
        World::family_clearing(seed, founding, founding.palette(), reserve_assisted)
            .expect("family clearing fixture")
    }

    #[test]
    fn every_origin_body_stands_and_the_grafted_body_reaches_the_cave() {
        let mut world = fixture(41, true);
        let mut replay = world.clone();
        let habitat = world.family_clearing_habitat();
        for organism in &world.organisms {
            assert!(
                organism
                    .walker_shape()
                    .stands(world.ground(), organism.position),
                "{:?} lacks a finished stance",
                organism.id
            );
        }
        for intent in world.family_practice_opening().unwrap() {
            assert!(!matches!(
                world.apply(intent.clone()),
                crate::Outcome::Rejected(_)
            ));
            replay.apply(intent);
        }
        assert!(world.ground().solid([
            habitat.chamber[0],
            habitat.chamber[1] - 1,
            habitat.chamber[2]
        ]));
        assert!(world.ground().solid([
            habitat.chamber[0],
            habitat.chamber[1] + world.controlled().unwrap().walker_shape().height(),
            habitat.chamber[2]
        ]));
        for (index, target) in habitat.route.iter().enumerate().skip(1) {
            for _ in 0..TERRARIUM_ROUTE_BUDGET {
                if world.position() == Some(*target) {
                    break;
                }
                let body = world.controlled().expect("played body");
                let next = route_step_for(
                    world.ground(),
                    body.walker_shape(),
                    body.position,
                    *target,
                    TERRARIUM_ROUTE_BUDGET,
                )
                .unwrap_or_else(|| {
                    panic!(
                        "route fails at tread {index}: {:?} -> {target:?}",
                        body.position
                    )
                });
                let intent = Intent::Move {
                    delta: [next[0] - body.position[0], 0, next[2] - body.position[2]],
                };
                assert!(matches!(
                    world.apply(intent.clone()),
                    super::super::Outcome::Moved
                ));
                assert!(matches!(replay.apply(intent), super::super::Outcome::Moved));
                assert_eq!(state_hash(&world), state_hash(&replay));
            }
            assert_eq!(world.position(), Some(*target));
        }
        assert_eq!(world.position(), Some(habitat.chamber));
    }

    #[test]
    fn every_founding_variant_has_a_floored_roofed_level_tunnel() {
        for seed in [7, 41] {
            for founding in [Founding::Roster, Founding::SpacedRoster] {
                let world = World::family_clearing(seed, founding, founding.palette(), true)
                    .expect("family clearing fixture");
                let habitat = world.family_clearing_habitat();
                let height = clearance_shapes(&world)
                    .iter()
                    .map(|shape| shape.height())
                    .max()
                    .expect("origin and Carry candidate have shapes");
                let radius = clearance_shapes(&world)
                    .iter()
                    .map(|shape| shape.radius())
                    .max()
                    .expect("origin and Carry candidate have shapes");
                for at in habitat
                    .route
                    .iter()
                    .skip((APPROACH_STEPS + DESCENT_STEPS + 2 * radius) as usize)
                {
                    assert!(world.ground().solid([at[0], at[1] - 1, at[2]]));
                    assert!(
                        world
                            .ground()
                            .solid([at[0], habitat.chamber[1] + height, at[2]])
                    );
                }
                assert!(world.ground().solid([
                    habitat.chamber[0],
                    habitat.chamber[1] + height,
                    habitat.chamber[2]
                ]));
            }
        }
    }

    #[test]
    fn geometry_replays_and_reserve_switch_only_changes_the_parent_account() {
        let assisted = fixture(7, true);
        let lean = fixture(7, false);
        assert_eq!(assisted.ground(), lean.ground());
        assert_eq!(
            assisted.family_clearing_habitat(),
            lean.family_clearing_habitat()
        );
        assert_eq!(assisted.events(), lean.events());
        let assisted_parent = assisted.controlled().unwrap();
        let lean_parent = lean.controlled().unwrap();
        assert_eq!(assisted_parent.body(), lean_parent.body());
        assert!(assisted_parent.energy_mg > lean_parent.energy_mg);
        assert_eq!(lean_parent.energy_mg, lean_parent.biomass_mg());
        assert_eq!(assisted.total_matter_mg(), lean.total_matter_mg());

        let bytes = snapshot(&assisted).expect("snapshot");
        let restored = restore(&bytes).expect("restore");
        assert_eq!(state_hash(&assisted), state_hash(&restored));
        let twin = fixture(7, true);
        assert_eq!(state_hash(&assisted), state_hash(&twin));
    }
}
