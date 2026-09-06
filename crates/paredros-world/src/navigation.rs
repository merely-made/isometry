// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Derived exact-ground route queries.
//!
//! [`Navigation`] expands the same movement transition authoritative bodies
//! use. Its paths are advice: movement saves retain accepted inputs and
//! positions, never a planner's chosen route.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use mesocosm_core::places::{Ground, SURFACE_BAND, WALKER_HEIGHT, step};

use crate::{Layer, SlotId, World};

const DIRECTIONS: [[i32; 2]; 8] = [
    [1, 0],
    [0, 1],
    [-1, 0],
    [0, -1],
    [1, 1],
    [-1, 1],
    [-1, -1],
    [1, -1],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Navigation {
    pub maximum_stances: u32,
    /// Search radius around an underground slot's host centre. Explicit
    /// because place and voxel scales are caller choices.
    pub underground_search_radius: i32,
}

impl Default for Navigation {
    fn default() -> Self {
        Self {
            maximum_stances: 100_000,
            underground_search_radius: 16,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationError {
    EmptyBudget,
    InvalidUndergroundRadius,
    MissingPlace(SlotId),
    MissingSurface(SlotId),
    MissingUnderground(SlotId),
    NoRoute { from: [i32; 3], toward: [i32; 3] },
    SearchLimit { from: [i32; 3], toward: [i32; 3] },
}

impl Navigation {
    /// One deterministic valid stance for a structural slot.
    pub fn stance(&self, world: &World, slot: SlotId) -> Result<[i32; 3], NavigationError> {
        self.validate()?;
        match slot.layer {
            Layer::Surface => self.surface_stance(world, slot),
            Layer::Underground => self
                .underground_stances(world, slot)?
                .into_iter()
                .next()
                .ok_or(NavigationError::MissingUnderground(slot)),
        }
    }

    pub fn surface_stance(&self, world: &World, slot: SlotId) -> Result<[i32; 3], NavigationError> {
        if slot.layer != Layer::Surface {
            return Err(NavigationError::MissingSurface(slot));
        }
        let [x, _, z] = place_hint(world, slot)?;
        let top = world
            .ground()
            .surface(x, z)
            .ok_or(NavigationError::MissingSurface(slot))?;
        let stance = [x, top + 1, z];
        world
            .ground()
            .stands(stance, WALKER_HEIGHT)
            .then_some(stance)
            .ok_or(NavigationError::MissingSurface(slot))
    }

    /// An exact route from `from` to a representative stance in `slot`.
    pub fn route_to_slot(
        &self,
        world: &World,
        from: [i32; 3],
        slot: SlotId,
    ) -> Result<Vec<[i32; 3]>, NavigationError> {
        self.validate()?;
        match slot.layer {
            Layer::Surface => {
                let target = self.surface_stance(world, slot)?;
                exact_route(
                    world.ground(),
                    world.config().extent,
                    from,
                    &BTreeSet::from([target]),
                    target,
                    *self,
                )
            },
            Layer::Underground => {
                let goals = self.underground_stances(world, slot)?;
                if goals.is_empty() {
                    return Err(NavigationError::MissingUnderground(slot));
                }
                exact_route(
                    world.ground(),
                    world.config().extent,
                    from,
                    &goals,
                    place_hint(world, slot)?,
                    *self,
                )
            },
        }
    }

    pub fn route_to_position(
        &self,
        world: &World,
        from: [i32; 3],
        target: [i32; 3],
    ) -> Result<Vec<[i32; 3]>, NavigationError> {
        self.validate()?;
        exact_route(
            world.ground(),
            world.config().extent,
            from,
            &BTreeSet::from([target]),
            target,
            *self,
        )
    }

    fn validate(&self) -> Result<(), NavigationError> {
        if self.maximum_stances == 0 {
            return Err(NavigationError::EmptyBudget);
        }
        if self.underground_search_radius < 1 {
            return Err(NavigationError::InvalidUndergroundRadius);
        }
        Ok(())
    }

    fn underground_stances(
        &self,
        world: &World,
        slot: SlotId,
    ) -> Result<BTreeSet<[i32; 3]>, NavigationError> {
        let extent = world.config().extent;
        let [centre_x, _, centre_z] = place_hint(world, slot)?;
        let radius = self.underground_search_radius;
        let mut stances = BTreeSet::new();
        for z in (centre_z - radius).max(-extent)..=(centre_z + radius).min(extent) {
            for x in (centre_x - radius).max(-extent)..=(centre_x + radius).min(extent) {
                for y in 1..SURFACE_BAND {
                    let at = [x, y, z];
                    if world.ground().stands(at, WALKER_HEIGHT)
                        && world.ground().solid([x, y + WALKER_HEIGHT, z])
                    {
                        stances.insert(at);
                    }
                }
            }
        }
        Ok(stances)
    }
}

fn place_hint(world: &World, slot: SlotId) -> Result<[i32; 3], NavigationError> {
    let [x, z] = world
        .grown()
        .places
        .get(slot.place)
        .ok_or(NavigationError::MissingPlace(slot))?
        .centre;
    Ok([x, 0, z])
}

fn exact_route(
    ground: &Ground,
    extent: i32,
    from: [i32; 3],
    goals: &BTreeSet<[i32; 3]>,
    hint: [i32; 3],
    navigation: Navigation,
) -> Result<Vec<[i32; 3]>, NavigationError> {
    if goals.contains(&from) {
        return Ok(vec![from]);
    }
    let mut frontier = BinaryHeap::from([Reverse((heuristic(from, hint), 0u32, from))]);
    let mut cost = BTreeMap::from([(from, 0u32)]);
    let mut previous = BTreeMap::new();

    while let Some(Reverse((_, walked, at))) = frontier.pop() {
        if cost.get(&at).copied() != Some(walked) {
            continue;
        }
        for [dx, dz] in DIRECTIONS {
            let next = step(ground, at, [at[0] + dx, at[1], at[2] + dz]);
            if next == at
                || next[0].abs() > extent
                || next[2].abs() > extent
                || !ground.stands(next, WALKER_HEIGHT)
            {
                continue;
            }
            let next_cost = walked + 1;
            if cost.get(&next).is_some_and(|old| *old <= next_cost) {
                continue;
            }
            if !cost.contains_key(&next) && cost.len() as u32 >= navigation.maximum_stances {
                return Err(NavigationError::SearchLimit { from, toward: hint });
            }
            cost.insert(next, next_cost);
            previous.insert(next, at);
            if goals.contains(&next) {
                return Ok(reconstruct(from, next, &previous));
            }
            frontier.push(Reverse((
                next_cost.saturating_add(heuristic(next, hint)),
                next_cost,
                next,
            )));
        }
    }
    Err(NavigationError::NoRoute { from, toward: hint })
}

fn heuristic(at: [i32; 3], target: [i32; 3]) -> u32 {
    (at[0] - target[0]).abs().max((at[2] - target[2]).abs()) as u32
}

fn reconstruct(
    from: [i32; 3],
    mut at: [i32; 3],
    previous: &BTreeMap<[i32; 3], [i32; 3]>,
) -> Vec<[i32; 3]> {
    let mut route = vec![at];
    while at != from {
        at = previous[&at];
        route.push(at);
    }
    route.reverse();
    route
}
