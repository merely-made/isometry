// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The near tier's floor: kinematic movement over brick truth, and the
//! tier boundary itself.
//!
//! Advisor-tier locomotion per plan ruling §0.10: owned move-and-slide
//! over [`Ground`] occupancy, integer in and integer out, no physics
//! engine and nothing persisted. Rain World's lesson is that physicality
//! is mostly *constraint*: walls stop you, ledges are climbed, gravity
//! is a fact. All of that is queries against the one truth.
//!
//! The tier boundary lives here too because it is a place-graph fact:
//! places near the played body run embodied agents, far places run the
//! statistical ecology, and the line between them moves with hysteresis
//! so a critter pacing the border does not flicker between minds.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::body::Aabb;

use super::Places;
use super::bricks::Ground;

/// How tall a walker is, in voxels, for occupancy checks.
pub const WALKER_HEIGHT: i32 = 2;
/// Body-space voxels represented by one Ground voxel for locomotion.
///
/// The primitive developmental palette makes one mass segment four body
/// voxels across. Treating that as one Ground voxel keeps a root segment at
/// the old one-column scale while still letting lateral anatomy widen the
/// passage a body needs.
pub const BODY_VOXELS_PER_GROUND_VOXEL: i32 = 4;
/// One step may climb this many voxels of ledge.
pub const CLIMB: i32 = 1;
/// A fall taller than this is not walked off voluntarily by prey-seekers.
pub const COMFORT_DROP: i32 = 4;

/// The turning cross-section a body presents to Ground.
///
/// Movement does not yet retain facing, so the shorter horizontal axis is the
/// honest passage width: an elongated body may align itself with a corridor,
/// but broad anatomy still cannot. Height and width are derived readings of
/// [`Aabb`], never a second body shape stored on an organism.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WalkerShape {
    radius: i32,
    height: i32,
}

impl WalkerShape {
    pub const STANDARD: Self = Self {
        radius: 0,
        height: WALKER_HEIGHT,
    };

    pub fn from_aabb(box_: Aabb) -> Self {
        let extent = box_.extent().map(i32::abs);
        let cross_section = extent[0].min(extent[2]);
        let scale = BODY_VOXELS_PER_GROUND_VOXEL;
        let radius = cross_section
            .saturating_sub(scale)
            .saturating_add(2 * scale - 1)
            / (2 * scale);
        let height = extent[1].saturating_add(scale - 1).div_euclid(scale).max(1);
        Self { radius, height }
    }

    pub fn radius(self) -> i32 {
        self.radius
    }

    pub fn height(self) -> i32 {
        self.height
    }

    /// The top-centre point this body uses for terrain sight.
    ///
    /// A one-voxel body keeps the established head-height ray one voxel above
    /// its stance. Anatomy taller than the old two-voxel walker lifts the ray;
    /// the visual horizon itself does not grow.
    pub fn sight_point(self, stance: [i32; 3]) -> [i32; 3] {
        [
            stance[0],
            stance[1] + self.height.max(WALKER_HEIGHT) - 1,
            stance[2],
        ]
    }

    /// Whether the whole cross-section is clear and at least one column has
    /// footing. Wide bodies may bridge a small hollow; they may not intersect
    /// its rim.
    pub fn stands(self, ground: &Ground, at: [i32; 3]) -> bool {
        self.clear(ground, at) && self.supported(ground, at)
    }

    fn clear(self, ground: &Ground, at: [i32; 3]) -> bool {
        (-self.radius..=self.radius).all(|dx| {
            (-self.radius..=self.radius).all(|dz| {
                (0..self.height).all(|dy| !ground.solid([at[0] + dx, at[1] + dy, at[2] + dz]))
            })
        })
    }

    fn supported(self, ground: &Ground, at: [i32; 3]) -> bool {
        (-self.radius..=self.radius).any(|dx| {
            (-self.radius..=self.radius).any(|dz| ground.solid([at[0] + dx, at[1] - 1, at[2] + dz]))
        })
    }
}

/// One kinematic step toward `toward`: slide on block, climb one ledge,
/// then settle under gravity. Returns where the walker ends up; the
/// result is always a standable voxel, and never more than one voxel of
/// horizontal travel from `from`.
pub fn step(ground: &Ground, from: [i32; 3], toward: [i32; 3]) -> [i32; 3] {
    step_for(ground, WalkerShape::STANDARD, from, toward)
}

/// [`step`] with occupancy derived from the body that is moving.
pub fn step_for(ground: &Ground, shape: WalkerShape, from: [i32; 3], toward: [i32; 3]) -> [i32; 3] {
    let want = [
        (toward[0] - from[0]).signum(),
        (toward[2] - from[2]).signum(),
    ];
    let vertical = (toward[1] - from[1]).signum();
    // The enclosure is a wall, not a cliff: no walker, played critter
    // included, may take a horizontal step that leaves Ground's resident
    // bound. Checked per candidate target so a body sitting at the bound
    // can still slide along it or step back in. (TD2b)
    let bound = ground.extent();

    // Try the full move, then each axis alone: the slide.
    let tries = [[want[0], want[1]], [want[0], 0], [0, want[1]]];
    let mut forced_drop = None;
    for try_move in tries {
        if try_move == [0, 0] {
            continue;
        }
        let target = [from[0] + try_move[0], from[1], from[2] + try_move[1]];
        if target[0].abs() > bound || target[2].abs() > bound {
            // Off the resident map: refuse outright, before the doorway
            // branch below can mistake the void beyond it for a shaft.
            continue;
        }
        // Prefer lifts that close the vertical gap, but preference is an
        // ordering, never a refusal: descending the far side of a hill
        // requires climbing the near side first.
        let lifts = if vertical < 0 {
            [0, -1, CLIMB]
        } else {
            [0, CLIMB, -1]
        };
        for lift in lifts {
            let at = [target[0], from[1] + lift, target[2]];
            if shape.stands(ground, at) {
                return settle(ground, shape, at);
            }
        }
        // The doorway case: air ahead but no floor (a shaft, a burrow
        // mouth). Walk in and let gravity decide, if the drop is short
        // or the quarry is below. A deeper drop is remembered: comfort
        // orders choices, and when the drop is the only way off a cliff
        // edge, standing there forever is not an option.
        let ahead = [target[0], from[1], target[2]];
        if !solid_span(ground, shape, ahead) {
            let landing = settle(ground, shape, ahead);
            if from[1] - landing[1] <= COMFORT_DROP || vertical < 0 {
                return landing;
            }
            if forced_drop.is_none() {
                forced_drop = Some(landing);
            }
        }
    }
    if let Some(landing) = forced_drop {
        return landing;
    }
    // Boxed in horizontally; at least settle where we are.
    settle(ground, shape, from)
}

/// The first legal step on a bounded route to `target`. It expands the same
/// owned [`step`] transitions the player and ecology use, so routing cannot
/// invent a second collision representation. `None` means the target lies
/// outside the local budget or no route was found there.
pub fn route_step(
    ground: &Ground,
    from: [i32; 3],
    target: [i32; 3],
    budget: i32,
) -> Option<[i32; 3]> {
    route_step_for(ground, WalkerShape::STANDARD, from, target, budget)
}

/// [`route_step`] over the transition relation of one body's cross-section.
pub fn route_step_for(
    ground: &Ground,
    shape: WalkerShape,
    from: [i32; 3],
    target: [i32; 3],
    budget: i32,
) -> Option<[i32; 3]> {
    if from == target || budget < 1 || chebyshev(from, target) > budget {
        return None;
    }
    const DIRECTIONS: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
    const MAX_ROUTE_STANCES: usize = 256;
    let mut frontier = VecDeque::from([from]);
    let mut previous = BTreeMap::from([(from, from)]);
    while let Some(at) = frontier.pop_front() {
        for [dx, dz] in DIRECTIONS {
            let next = step_for(ground, shape, at, [at[0] + dx, at[1], at[2] + dz]);
            if next == at || chebyshev(from, next) > budget || previous.contains_key(&next) {
                continue;
            }
            if previous.len() >= MAX_ROUTE_STANCES {
                return None;
            }
            previous.insert(next, at);
            if next == target {
                let mut first = target;
                while previous[&first] != from {
                    first = previous[&first];
                }
                return Some(first);
            }
            frontier.push_back(next);
        }
    }
    None
}

fn chebyshev(a: [i32; 3], b: [i32; 3]) -> i32 {
    (0..3)
        .map(|axis| (a[axis] - b[axis]).abs())
        .max()
        .unwrap_or(0)
}

fn solid_span(ground: &Ground, shape: WalkerShape, at: [i32; 3]) -> bool {
    !shape.clear(ground, at)
}

/// Fall until footing. Bedrock guarantees termination.
fn settle(ground: &Ground, shape: WalkerShape, at: [i32; 3]) -> [i32; 3] {
    let mut here = at;
    while !shape.supported(ground, here) && here[1] > 0 {
        let below = [here[0], here[1] - 1, here[2]];
        if !shape.clear(ground, below) {
            break;
        }
        here[1] -= 1;
    }
    here
}

/// Places a body on the highest surface under its cross-section.
///
/// This is used when a graph-level position becomes embodied. It does not
/// search for a route or alter Ground; it only realizes that x/z position as
/// a legal stance when its footprint overlaps resident terrain.
pub fn surface_stance_for(
    ground: &Ground,
    shape: WalkerShape,
    position: [i32; 3],
) -> Option<[i32; 3]> {
    let mut surface = None;
    for dx in -shape.radius..=shape.radius {
        for dz in -shape.radius..=shape.radius {
            if let Some(column) = ground.surface(position[0] + dx, position[2] + dz) {
                surface = Some(surface.map_or(column, |top: i32| top.max(column)));
            }
        }
    }
    let at = [position[0], surface? + 1, position[2]];
    shape.stands(ground, at).then_some(at)
}

/// Which mind runs an agent: embodied, or the statistical ecology.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    #[default]
    Near,
    Far,
}

/// The tier boundary with hysteresis. Promote when an agent's place
/// comes within `promote_hops` of the focus; demote only past
/// `demote_hops`. The band between is memory.
#[derive(Clone, Copy, Debug)]
pub struct TierLine {
    pub promote_hops: u32,
    pub demote_hops: u32,
}

impl Default for TierLine {
    fn default() -> Self {
        Self {
            promote_hops: 1,
            // The standard enclosure is a 3x3 graph with diameter two. A
            // threshold of three would make the far tier unreachable in the
            // shipped world, so the outer ring is the demotion boundary.
            demote_hops: 2,
        }
    }
}

impl TierLine {
    /// The next tier, given where the agent and the focus are.
    pub fn tick(&self, places: &Places, current: Tier, agent: [i32; 3], focus: [i32; 3]) -> Tier {
        let (Some(a), Some(f)) = (places.at(agent), places.at(focus)) else {
            return current;
        };
        let hops = places.hops(a, f).unwrap_or(u32::MAX);
        match current {
            Tier::Far if hops <= self.promote_hops => Tier::Near,
            Tier::Near if hops >= self.demote_hops => Tier::Far,
            _ => current,
        }
    }
}

/// Whether `eye` can see `target`: distance-capped line of sight from
/// head height to head height.
pub fn spot(ground: &Ground, eye: [i32; 3], target: [i32; 3], range: i32) -> bool {
    spot_for(
        ground,
        WalkerShape::STANDARD,
        eye,
        WalkerShape::STANDARD,
        target,
        range,
    )
}

/// [`spot`] between sight points derived from both live bodies.
///
/// Range remains stance-to-stance. Becoming taller changes which terrain
/// occludes a body, rather than granting a longer visual horizon.
pub fn spot_for(
    ground: &Ground,
    observer_shape: WalkerShape,
    observer: [i32; 3],
    target_shape: WalkerShape,
    target: [i32; 3],
    range: i32,
) -> bool {
    let dx = (observer[0] - target[0]).abs();
    let dy = (observer[1] - target[1]).abs();
    let dz = (observer[2] - target[2]).abs();
    if dx.max(dy).max(dz) > range {
        return false;
    }
    ground.sees(
        observer_shape.sight_point(observer),
        target_shape.sight_point(target),
    )
}

// Split out at the 600-LOC ceiling (2026-08-29, TD2b): same module, just a
// separate file, per the `organism::ecology` / `ecology::tests` precedent.
#[cfg(test)]
mod tests;
