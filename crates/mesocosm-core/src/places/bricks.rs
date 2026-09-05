// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Brick truth: the ground as voxels, in the one coordinate space.
//!
//! G1's container (place-graph engine plan §3). The relief decides every
//! column's surface; nests carve real burrows under their hosts, so an
//! interior is a hole in the same ground everything walks on, never a
//! scene. Dense 8³ bricks in an ordered map: hashable, diffable, and
//! serialized flat. Carves mark their bricks dirty and bump one revision,
//! which is the fact a renderer's upload discipline consumes.
//!
//! Additive like [`super::grown`]: nothing here is wired into `World`
//! yet. Carve becomes an ordered intent at adoption time.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::grown::{Grown, Nest};

/// Voxels per brick edge.
pub const BRICK: i32 = 8;
/// The world-height band relief maps onto. Three bricks of headroom.
pub const SURFACE_BAND: i32 = 24;

/// Materials. Zero is air; the rest are palette indices for projections.
pub const AIR: u8 = 0;
pub const ROCK: u8 = 2;
pub const SOIL: u8 = 3;

/// One dense 8³ brick, y-major then z then x.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brick {
    materials: Vec<u8>,
}

impl Brick {
    fn empty() -> Self {
        Self {
            materials: vec![AIR; (BRICK * BRICK * BRICK) as usize],
        }
    }

    fn index(local: [i32; 3]) -> usize {
        ((local[1] * BRICK + local[2]) * BRICK + local[0]) as usize
    }

    pub fn get(&self, local: [i32; 3]) -> u8 {
        self.materials[Self::index(local)]
    }

    fn set(&mut self, local: [i32; 3], material: u8) {
        self.materials[Self::index(local)] = material;
    }

    pub fn is_empty(&self) -> bool {
        self.materials.iter().all(|m| *m == AIR)
    }
}

/// The ground: every solid voxel the world owns, plus the lifecycle facts
/// a projection needs (revision, dirty bricks).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Ground {
    extent: i32,
    /// World y of the water line, derived from the relief's sea.
    pub sea_level: i32,
    bricks: BTreeMap<[i16; 3], Brick>,
    revision: u64,
    /// Projection work queue, not world authority. Hosts may drain this at
    /// different frame rates, so it must never alter snapshots or replay
    /// hashes; the revision and brick bytes carry the authoritative change.
    #[serde(skip)]
    dirty: BTreeSet<[i16; 3]>,
}

impl PartialEq for Ground {
    fn eq(&self, other: &Self) -> bool {
        self.extent == other.extent
            && self.sea_level == other.sea_level
            && self.bricks == other.bricks
            && self.revision == other.revision
    }
}

impl Eq for Ground {}

fn brick_of(at: [i32; 3]) -> [i16; 3] {
    [
        at[0].div_euclid(BRICK) as i16,
        at[1].div_euclid(BRICK) as i16,
        at[2].div_euclid(BRICK) as i16,
    ]
}

fn local_of(at: [i32; 3]) -> [i32; 3] {
    [
        at[0].rem_euclid(BRICK),
        at[1].rem_euclid(BRICK),
        at[2].rem_euclid(BRICK),
    ]
}

/// The generated, embodied route into one nest. It is derived from graph
/// facts, never stored beside the brick truth: Ground owns the resulting
/// voxels, while tests and generation share the one construction rule.
#[derive(Clone, Debug)]
pub struct NestEntry {
    /// Surface column at the mouth of the entry.
    pub anchor: [i32; 2],
    /// Floor height of the roofed room at the inner end.
    pub floor: i32,
    /// Ordered standable voxels from surface mouth to inner room.
    pub route: Vec<[i32; 3]>,
}

impl Grown {
    /// The exact generated access routes embodied by [`Ground::grow`].
    ///
    /// This is a read model over the same construction rule as terrain
    /// generation. Debuggers and projections can name a threshold without
    /// reconstructing one from rendered voxels or storing another authority.
    pub fn nest_entries(&self, extent: i32) -> impl Iterator<Item = (Nest, NestEntry)> + '_ {
        self.nests
            .iter()
            .copied()
            .filter_map(move |nest| nest_entry(self, extent, nest).map(|entry| (nest, entry)))
    }
}

pub(crate) fn nest_entry(grown: &Grown, extent: i32, nest: Nest) -> Option<NestEntry> {
    let host = grown.places.get(nest.host)?;
    let [cx, cz] = host.centre;
    let (mut x, mut z, mut anchor) = (cx, cz, 0);
    for dz in -5..=5 {
        for dx in -5..=5 {
            let (px, pz) = (cx + dx, cz + dz);
            // The mouth stays inside the wall too: a burrow is part of the
            // vessel, not a tunnel through it. (TD2b)
            if px.abs() > extent || pz.abs() > extent {
                continue;
            }
            let surface = surface_from(grown, extent, px, pz);
            if surface > anchor {
                (x, z, anchor) = (px, pz, surface);
            }
        }
    }
    if anchor < 4 {
        return None;
    }
    let [dx, dz] = nest_entry_direction(nest.host.0);
    // How many steps this direction has before the route would leave the
    // resident bound; caps depth so the descent never crosses the wall.
    // Measured at 500 real-enclosure seeds before this cap: routes drifted
    // up to 12 voxels past the wall (probe removed once fixed). (TD2b)
    let room_for_direction = extent - (x * dx + z * dz);
    let depth_cap = ((room_for_direction - 1).max(0)) / 3;
    if depth_cap < 1 {
        return None;
    }
    let depth = ((anchor - 2) / 3)
        .clamp(1, nest.depth as i32)
        .min(depth_cap);
    let floor = (anchor - 3 * depth - 1).max(1);
    let route = (0..=anchor - floor)
        .map(|step| [x + dx * step, anchor + 1 - step, z + dz * step])
        .collect();
    Some(NestEntry {
        anchor: [x, z],
        floor,
        route,
    })
}

fn surface_from(grown: &Grown, extent: i32, x: i32, z: i32) -> i32 {
    1 + grown.relief.sample(extent, x, z) * (SURFACE_BAND - 1) / super::relief::CEILING
}

impl Ground {
    /// Raises the ground a [`Grown`] world described. `extent` must be the
    /// extent the world was grown with.
    pub fn grow(grown: &Grown, extent: i32) -> Self {
        let mut ground = Self {
            extent,
            sea_level: 1 + grown.relief.sea * (SURFACE_BAND - 1) / super::relief::CEILING,
            bricks: BTreeMap::new(),
            revision: 0,
            dirty: BTreeSet::new(),
        };

        for z in -extent..=extent {
            for x in -extent..=extent {
                let surface = surface_from(grown, extent, x, z);
                for y in 0..=surface {
                    let material = if y + 2 > surface { SOIL } else { ROCK };
                    ground.place([x, y, z], material);
                }
            }
        }

        // Burrows: anchored at the highest column near the host, so a
        // low-lying host digs into its own hillside instead of cratering.
        // Rooms scale to the depth the ground actually affords, and every
        // chamber keeps a roof. The entry descends one voxel per horizontal
        // step, because a vertical hollow is a picture of a burrow, not a
        // route `near::step` can actually traverse. Deterministic from the
        // graph alone.
        for nest in &grown.nests {
            let Some(entry) = nest_entry(grown, extent, *nest) else {
                continue;
            };
            let [x, z] = entry.anchor;
            let anchor = entry.route[0][1] - 1;
            let floor = entry.floor;
            let radius = if anchor - floor >= 5 { 2 } else { 1 };
            let [entry_dx, entry_dz] = nest_entry_direction(nest.host.0);
            let drop = anchor - floor;
            let (entry_x, entry_z) = (x + entry_dx * drop, z + entry_dz * drop);
            for room in 0..nest.rooms {
                let spin = (nest.host.0 as i32 * 7 + room as i32 * 5) % 8;
                let (dx, dz) = [
                    (radius + 1, 0),
                    (radius, radius),
                    (0, radius + 1),
                    (-radius, radius),
                    (-radius - 1, 0),
                    (-radius, -radius),
                    (0, -radius - 1),
                    (radius, -radius),
                ][spin as usize];
                let lift = (room as i32) % (anchor - 1 - radius - floor).max(1);
                let centre_y = (floor + lift).min(anchor - 1 - radius).max(1 + radius);
                ground.hollow([entry_x + dx, centre_y, entry_z + dz], radius);
            }
            ground.carve_nest_entry(&entry);
        }

        // Generation is the world's starting fact, not an edit.
        ground.dirty.clear();
        ground.revision = 0;
        ground
    }

    fn place(&mut self, at: [i32; 3], material: u8) {
        let key = brick_of(at);
        self.bricks
            .entry(key)
            .or_insert_with(Brick::empty)
            .set(local_of(at), material);
        self.dirty.insert(key);
    }

    /// Clears a two-voxel walker volume while preserving (or supplying) its
    /// footing. This is generation geometry, not an edit: callers reset the
    /// revision and dirty queue once the initial world has been raised.
    fn make_stance(&mut self, at: [i32; 3]) {
        let floor = [at[0], at[1] - 1, at[2]];
        if !self.solid(floor) {
            self.place(floor, SOIL);
        }
        for dy in 0..2 {
            let air = [at[0], at[1] + dy, at[2]];
            if self.solid(air) {
                self.place(air, AIR);
            }
        }
    }

    /// Authors one wide walker stance into initial terrain geometry.
    ///
    /// This preserves exactly one chosen floor voxel while clearing the
    /// walker's occupied volume. It is for deterministic world construction;
    /// simulation edits continue to use [`Self::carve`].
    pub fn author_walker_stance(
        &mut self,
        at: [i32; 3],
        radius: i32,
        height: i32,
        support_offset: [i32; 2],
    ) {
        let radius = radius.max(0);
        debug_assert!(support_offset[0].abs() <= radius);
        debug_assert!(support_offset[1].abs() <= radius);
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                for dy in 0..height.max(1) {
                    let air = [at[0] + dx, at[1] + dy, at[2] + dz];
                    if self.solid(air) {
                        self.place(air, AIR);
                    }
                }
            }
        }
        self.place(
            [
                at[0] + support_offset[0],
                at[1] - 1,
                at[2] + support_offset[1],
            ],
            SOIL,
        );
    }

    /// Dig the access route for one generated nest. It descends one voxel per
    /// horizontal step to the roofed entry room. Its directness matters: the
    /// current embodied ecology follows a visible target vector, and must not
    /// need an unimplemented pathfinder merely to enter a burrow.
    fn carve_nest_entry(&mut self, entry: &NestEntry) {
        for at in &entry.route {
            self.make_stance(*at);
        }
        let inside = *entry
            .route
            .last()
            .expect("a nest entry has a mouth and a room");
        let roof = [inside[0], inside[1] + 2, inside[2]];
        if !self.solid(roof) {
            self.place(roof, ROCK);
        }
    }

    fn hollow(&mut self, centre: [i32; 3], radius: i32) {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let at = [centre[0] + dx, centre[1] + dy, centre[2] + dz];
                    if at[1] < 1 {
                        continue;
                    }
                    if self.solid(at) {
                        self.place(at, AIR);
                    }
                }
            }
        }
    }

    /// The resident bound: how far `Ground::grow` laid bricks from the
    /// origin on either horizontal axis. The enclosure's wall, not its
    /// floor — `step_for` is what refuses a step past it. (TD2b)
    pub fn extent(&self) -> i32 {
        self.extent
    }

    /// Whether a voxel is solid. Below y = 0 is bedrock, always solid.
    pub fn solid(&self, at: [i32; 3]) -> bool {
        if at[1] < 0 {
            return true;
        }
        self.bricks
            .get(&brick_of(at))
            .map(|brick| brick.get(local_of(at)) != AIR)
            .unwrap_or(false)
    }

    /// Whether a creature of the given height can occupy `at`: solid
    /// footing below, air through the body.
    pub fn stands(&self, at: [i32; 3], height: i32) -> bool {
        self.solid([at[0], at[1] - 1, at[2]])
            && (0..height.max(1)).all(|dy| !self.solid([at[0], at[1] + dy, at[2]]))
    }

    /// Line of sight between voxel centres: no solid voxel strictly
    /// between them. Integer sampling at half-voxel strides, so the walk
    /// is deterministic and never skips a corner.
    pub fn sees(&self, from: [i32; 3], to: [i32; 3]) -> bool {
        let delta = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
        let strides = 2 * delta.iter().map(|d| d.abs()).max().unwrap_or(0);
        let mut last = from;
        for stride in 1..strides {
            let at = [
                from[0] + (delta[0] * stride).div_euclid(strides.max(1)),
                from[1] + (delta[1] * stride).div_euclid(strides.max(1)),
                from[2] + (delta[2] * stride).div_euclid(strides.max(1)),
            ];
            if at == last || at == to {
                continue;
            }
            last = at;
            if self.solid(at) {
                return false;
            }
        }
        true
    }

    /// Carves a cube of air around `at`. One revision per carve, however
    /// many voxels it removed; the dirty set records which bricks changed.
    pub fn carve(&mut self, at: [i32; 3], radius: i32) -> u32 {
        let before = self.dirty.clone();
        let mut removed = 0;
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                for dx in -radius..=radius {
                    let voxel = [at[0] + dx, at[1] + dy, at[2] + dz];
                    if voxel[1] >= 1 && self.solid(voxel) {
                        self.place(voxel, AIR);
                        removed += 1;
                    }
                }
            }
        }
        if removed > 0 {
            self.revision += 1;
        } else {
            self.dirty = before;
        }
        removed
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// The bricks changed since the last drain, for a projection's
    /// region-upload discipline. Draining is a projection act and does not
    /// touch the revision.
    pub fn drain_dirty(&mut self) -> Vec<[i16; 3]> {
        std::mem::take(&mut self.dirty).into_iter().collect()
    }

    pub fn brick_count(&self) -> usize {
        self.bricks.len()
    }

    /// One brick's materials as a mesh-crate volume, for the raster
    /// projection. `None` for a brick that was never touched (all air).
    pub fn brick_materials(&self, key: [i16; 3]) -> Option<(&Brick, [i32; 3])> {
        self.bricks.get(&key).map(|brick| {
            (
                brick,
                [
                    key[0] as i32 * BRICK,
                    key[1] as i32 * BRICK,
                    key[2] as i32 * BRICK,
                ],
            )
        })
    }

    pub fn keys(&self) -> impl Iterator<Item = [i16; 3]> + '_ {
        self.bricks.keys().copied()
    }

    /// The highest solid y at a column, if any solid exists there.
    pub fn surface(&self, x: i32, z: i32) -> Option<i32> {
        (0..SURFACE_BAND + 1).rev().find(|y| self.solid([x, *y, z]))
    }
}

fn nest_entry_direction(host: u16) -> [i32; 2] {
    [[1, 0], [0, 1], [-1, 0], [0, -1]][host as usize % 4]
}

impl Brick {
    /// The brick as raw material bytes, for building a mesh-crate volume.
    pub fn raw(&self) -> &[u8] {
        &self.materials
    }
}

// Split out at the 600-LOC ceiling (2026-08-29, TD2b): same module, just a
// separate file, per the `organism::ecology` / `ecology::tests` precedent.
#[cfg(test)]
mod tests;
