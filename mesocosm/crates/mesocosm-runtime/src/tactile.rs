// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The tactile projection: Conatus advice beside the integer record.
//!
//! T1's adapter (engine review §5). [`TactileWorld`] keeps a
//! [`conatus::BodyWorld`] holding the ground's occupied cells as one voxel
//! collider and each named critter as capsule colliders, synchronized from
//! [`GroundVoxelProfile`] updates under the same stale/regressed-revision
//! refusals the profile itself enforces. Queries answer with world cells
//! and critter keys; Rapier stays private inside `conatus`, and nothing
//! here writes into the record — tactile answers are advice.

use std::{collections::BTreeMap, error::Error, fmt};

use conatus::{
    BodyDesc, BodyError, BodyId, BodyKind, BodyWorld, ColliderDesc, ColliderShape, SpatialFilter,
    Transform, VoxelEdit,
};
use mesocosm_core::places::{AIR, BRICK};
use mesocosm_core::voxel_profile::{GroundChunkChange, GroundVoxelProfile, GroundVoxelUpdate};

/// One capsule of a critter's presented pose, world coordinates. A tapered
/// pose capsule is carried at its mean radius; taper is a recorded
/// approximation of this advice tier, not a lost fact.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TactileCapsule {
    pub a: [f32; 3],
    pub b: [f32; 3],
    pub ra: f32,
    pub rb: f32,
}

/// What a tactile ray found.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TactilePick {
    /// A ground voxel, named by its world cell.
    Ground { cell: [i32; 3] },
    /// A critter, named by the key its capsules were set under.
    Critter { key: u64 },
}

/// A pick with its geometric evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TactileHit {
    pub pick: TactilePick,
    pub distance: f32,
    pub point: [f32; 3],
    pub normal: [f32; 3],
}

#[derive(Debug)]
pub enum TactileError {
    /// The update was lowered from a profile revision this world is not at.
    StaleSource {
        expected: u64,
        actual: u64,
    },
    /// The ground held no occupied cell to project.
    EmptyGround,
    Body(BodyError),
}

impl fmt::Display for TactileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StaleSource { expected, actual } => write!(
                formatter,
                "tactile world is at source revision {actual}, not {expected}"
            ),
            Self::EmptyGround => write!(formatter, "the ground profile has no occupied cell"),
            Self::Body(source) => write!(formatter, "tactile body world refused: {source}"),
        }
    }
}

impl Error for TactileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Body(source) => Some(source),
            _ => None,
        }
    }
}

impl From<BodyError> for TactileError {
    fn from(source: BodyError) -> Self {
        Self::Body(source)
    }
}

/// The Conatus tactile world derived from one ground profile.
pub struct TactileWorld {
    world: BodyWorld,
    terrain_body: BodyId,
    terrain_collider: conatus::ColliderId,
    critters: BTreeMap<u64, BodyId>,
    source_revision: u64,
}

impl TactileWorld {
    /// Project a profile's occupied cells into one fixed voxel collider.
    pub fn from_profile(profile: &GroundVoxelProfile) -> Result<Self, TactileError> {
        let mut occupied = Vec::new();
        for (key, chunk) in profile.chunks() {
            for cell in chunk.occupied_cells(|material| *material != AIR) {
                occupied.push(world_cell(key, cell));
            }
        }
        if occupied.is_empty() {
            return Err(TactileError::EmptyGround);
        }
        // Gravity is irrelevant: everything here is fixed and stepped never,
        // queried always.
        let mut world = BodyWorld::try_new([0.0, 0.0, 0.0])?;
        let terrain_body = world.spawn(BodyDesc::new(BodyKind::Fixed).with_collider(
            ColliderDesc::new(ColliderShape::VoxelGrid {
                cell_size: [1.0, 1.0, 1.0],
                occupied,
            }),
        ))?;
        let terrain_collider = world
            .collider_ids(terrain_body)?
            .into_iter()
            .next()
            .expect("the terrain body was spawned with one collider");
        let mut tactile = Self {
            world,
            terrain_body,
            terrain_collider,
            critters: BTreeMap::new(),
            source_revision: profile.source_revision(),
        };
        tactile.settle()?;
        Ok(tactile)
    }

    /// Refresh the backend's query acceleration structure.
    ///
    /// Conatus queries answer through Rapier's broad phase, which only
    /// learns about spawned or despawned colliders during a step. This
    /// world never simulates — everything in it is fixed and gravity is
    /// zero — so a minimal step moves nothing and exists purely to settle
    /// the query structures after a topology change. T1 is the first
    /// consumer that queries without ever stepping; if Conatus grows a
    /// query-refresh seam, this is the call it replaces.
    fn settle(&mut self) -> Result<(), TactileError> {
        self.world.step(1e-6)?;
        Ok(())
    }

    pub const fn source_revision(&self) -> u64 {
        self.source_revision
    }

    /// Apply one profile update's occupancy edits to the terrain collider.
    ///
    /// The refusal mirrors the profile's own gate: an update lowered from a
    /// different source revision is refused before any mutation.
    pub fn sync(&mut self, update: &GroundVoxelUpdate) -> Result<usize, TactileError> {
        if update.previous_source_revision != self.source_revision {
            return Err(TactileError::StaleSource {
                expected: update.previous_source_revision,
                actual: self.source_revision,
            });
        }
        let mut edits = Vec::new();
        for change in &update.chunks {
            let (key, occupancy_edits) = match change {
                GroundChunkChange::Inserted {
                    key,
                    occupancy_edits,
                    ..
                }
                | GroundChunkChange::Patched {
                    key,
                    occupancy_edits,
                    ..
                }
                | GroundChunkChange::Removed {
                    key,
                    occupancy_edits,
                    ..
                } => (*key, occupancy_edits),
            };
            edits.extend(occupancy_edits.iter().map(|edit| VoxelEdit {
                cell: world_cell(key, edit.cell),
                filled: edit.filled,
            }));
        }
        let changed = if edits.is_empty() {
            0
        } else {
            self.world
                .edit_voxels(self.terrain_collider, edits)?
                .changed
        };
        if changed > 0 {
            self.settle()?;
        }
        self.source_revision = update.source_revision;
        Ok(changed)
    }

    /// Present one critter's capsules under a caller-owned key, replacing
    /// whatever that key presented before.
    pub fn set_critter(
        &mut self,
        key: u64,
        capsules: &[TactileCapsule],
    ) -> Result<(), TactileError> {
        self.clear_critter(key)?;
        if capsules.is_empty() {
            return Ok(());
        }
        let colliders = capsules
            .iter()
            .map(|capsule| {
                let mut desc = ColliderDesc::new(ColliderShape::CapsuleY {
                    half_height: half_height(capsule),
                    radius: (capsule.ra + capsule.rb) * 0.5,
                });
                desc.local_transform = capsule_transform(capsule);
                desc
            })
            .collect();
        let mut desc = BodyDesc::new(BodyKind::Fixed);
        desc.colliders = colliders;
        let body = self.world.spawn(desc)?;
        self.critters.insert(key, body);
        self.settle()
    }

    /// Remove one critter's presence, if it has any.
    pub fn clear_critter(&mut self, key: u64) -> Result<(), TactileError> {
        if let Some(body) = self.critters.remove(&key) {
            self.world.despawn(body)?;
            self.settle()?;
        }
        Ok(())
    }

    /// Ask what a ray touches first: a critter, a ground voxel, or nothing.
    pub fn pick(
        &self,
        origin: [f32; 3],
        direction: [f32; 3],
        far: f32,
    ) -> Result<Option<TactileHit>, TactileError> {
        let Some(hit) =
            self.world
                .raycast(origin, direction, far, true, SpatialFilter::default())?
        else {
            return Ok(None);
        };
        let body = hit.collider.body();
        let pick = if body == self.terrain_body {
            TactilePick::Ground {
                cell: face_cell(hit.point, hit.normal),
            }
        } else {
            let key = self
                .critters
                .iter()
                .find(|(_, critter)| **critter == body)
                .map(|(key, _)| *key)
                .expect("a tactile hit names either the terrain or a set critter");
            TactilePick::Critter { key }
        };
        Ok(Some(TactileHit {
            pick,
            distance: hit.distance,
            point: hit.point,
            normal: hit.normal,
        }))
    }
}

/// A chunk-local occupancy cell as a world voxel cell. With the terrain
/// body at identity and cell size one, collider grid cells are world cells.
fn world_cell(key: [i16; 3], cell: [i32; 3]) -> [i32; 3] {
    [
        i32::from(key[0]) * BRICK + cell[0],
        i32::from(key[1]) * BRICK + cell[1],
        i32::from(key[2]) * BRICK + cell[2],
    ]
}

/// The cell a face hit names: half a cell against the face normal from the
/// hit point. A ray starting inside a solid reports a zero normal; the hit
/// point itself is then inside the cell.
fn face_cell(point: [f32; 3], normal: [f32; 3]) -> [i32; 3] {
    let mut cell = [0; 3];
    for axis in 0..3 {
        cell[axis] = (point[axis] - normal[axis] * 0.5).floor() as i32;
    }
    cell
}

fn half_height(capsule: &TactileCapsule) -> f32 {
    let axis = [
        capsule.b[0] - capsule.a[0],
        capsule.b[1] - capsule.a[1],
        capsule.b[2] - capsule.a[2],
    ];
    (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt() * 0.5
}

/// Place a Y-aligned capsule along the segment `a..b`: translate to the
/// midpoint and rotate `+Y` onto the segment axis.
fn capsule_transform(capsule: &TactileCapsule) -> Transform {
    let midpoint = [
        (capsule.a[0] + capsule.b[0]) * 0.5,
        (capsule.a[1] + capsule.b[1]) * 0.5,
        (capsule.a[2] + capsule.b[2]) * 0.5,
    ];
    let axis = [
        capsule.b[0] - capsule.a[0],
        capsule.b[1] - capsule.a[1],
        capsule.b[2] - capsule.a[2],
    ];
    let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
    let rotation = if length <= f32::EPSILON {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        let direction = [axis[0] / length, axis[1] / length, axis[2] / length];
        rotation_from_y(direction)
    };
    Transform {
        translation: midpoint,
        rotation,
    }
}

/// The quaternion rotating `[0, 1, 0]` onto a unit direction, `[x, y, z, w]`.
fn rotation_from_y(direction: [f32; 3]) -> [f32; 4] {
    let dot = direction[1];
    if dot >= 1.0 - 1e-6 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    if dot <= -1.0 + 1e-6 {
        // Antiparallel: half a turn about any perpendicular axis.
        return [1.0, 0.0, 0.0, 0.0];
    }
    // axis = normalize(Y × direction), angle = acos(dot).
    let axis = [direction[2], 0.0, -direction[0]];
    let axis_length = (axis[0] * axis[0] + axis[2] * axis[2]).sqrt();
    let half = dot.acos() * 0.5;
    let sin = half.sin() / axis_length;
    [axis[0] * sin, 0.0, axis[2] * sin, half.cos()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::places::{Ground, Places};

    fn ground() -> Ground {
        Ground::grow(&Places::grown(4_242, 4, 64), 64)
    }

    fn down_pick(world: &TactileWorld, x: i32, z: i32) -> TactileHit {
        world
            .pick(
                [x as f32 + 0.5, 200.0, z as f32 + 0.5],
                [0.0, -1.0, 0.0],
                400.0,
            )
            .expect("a vertical pick is a valid query")
            .expect("a grown column has ground under it")
    }

    #[test]
    fn a_vertical_pick_names_the_exact_surface_cell() {
        let ground = ground();
        let profile = GroundVoxelProfile::from_ground(&ground).unwrap();
        let world = TactileWorld::from_profile(&profile).unwrap();

        for (x, z) in [(4, 4), (4, 18), (10, 3)] {
            let top = ground.surface(x, z).expect("a grown column");
            let hit = down_pick(&world, x, z);
            assert_eq!(hit.pick, TactilePick::Ground { cell: [x, top, z] });
            assert!(hit.normal[1] > 0.9, "a top face looks up: {:?}", hit.normal);
        }
    }

    #[test]
    fn a_committed_carve_reaches_the_pick_through_sync() {
        let mut ground = ground();
        let mut profile = GroundVoxelProfile::from_ground(&ground).unwrap();
        let mut world = TactileWorld::from_profile(&profile).unwrap();
        let before = ground.revision();
        let top = ground.surface(4, 4).expect("a grown column");
        assert!(ground.carve([4, top, 4], 0) > 0);

        let update = profile.sync(before, &ground).unwrap();
        let changed = world.sync(&update).unwrap();

        assert!(changed > 0);
        assert_eq!(world.source_revision(), ground.revision());
        let hit = down_pick(&world, 4, 4);
        let after = ground.surface(4, 4).expect("carving one voxel leaves rock");
        assert!(after < top);
        assert_eq!(
            hit.pick,
            TactilePick::Ground {
                cell: [4, after, 4]
            }
        );
    }

    #[test]
    fn a_stale_update_is_refused_before_any_edit() {
        let mut ground = ground();
        let mut profile = GroundVoxelProfile::from_ground(&ground).unwrap();
        let mut world = TactileWorld::from_profile(&profile).unwrap();
        let before = ground.revision();
        let top = ground.surface(4, 4).expect("a grown column");
        assert!(ground.carve([4, top, 4], 0) > 0);
        let update = profile.sync(before, &ground).unwrap();
        world.sync(&update).unwrap();
        let held = down_pick(&world, 4, 4);

        assert!(matches!(
            world.sync(&update),
            Err(TactileError::StaleSource { .. })
        ));
        assert_eq!(world.source_revision(), ground.revision());
        assert_eq!(down_pick(&world, 4, 4), held);
    }

    #[test]
    fn a_critter_capsule_is_picked_before_the_ground_under_it() {
        let ground = ground();
        let profile = GroundVoxelProfile::from_ground(&ground).unwrap();
        let mut world = TactileWorld::from_profile(&profile).unwrap();
        let top = ground.surface(4, 18).expect("the fixture column") as f32;
        // The G2 fixture: one X-aligned tapered capsule hovering over the
        // surface.
        let centre = [4.5, top + 1.15, 18.5];
        world
            .set_critter(
                7,
                &[TactileCapsule {
                    a: [centre[0] - 0.7, centre[1], centre[2]],
                    b: [centre[0] + 0.7, centre[1], centre[2]],
                    ra: 0.65,
                    rb: 0.52,
                }],
            )
            .unwrap();

        // Straight down through the capsule's off-centre length: only a
        // correctly rotated capsule is there to hit.
        let through_end = world
            .pick([centre[0] + 0.6, 200.0, centre[2]], [0.0, -1.0, 0.0], 400.0)
            .unwrap()
            .expect("the capsule end is under this ray");
        assert_eq!(through_end.pick, TactilePick::Critter { key: 7 });

        // Beside the capsule, the same ray reaches the ground.
        let beside = world
            .pick([centre[0] + 2.5, 200.0, centre[2]], [0.0, -1.0, 0.0], 400.0)
            .unwrap()
            .expect("ground continues beside the critter");
        assert!(matches!(beside.pick, TactilePick::Ground { .. }));

        world.clear_critter(7).unwrap();
        let cleared = world
            .pick([centre[0] + 0.6, 200.0, centre[2]], [0.0, -1.0, 0.0], 400.0)
            .unwrap()
            .expect("ground remains after the critter leaves");
        assert!(matches!(cleared.pick, TactilePick::Ground { .. }));
    }
}
