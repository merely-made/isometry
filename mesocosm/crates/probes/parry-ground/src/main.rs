// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use mesocosm_core::Places;
use mesocosm_core::places::{AIR, BRICK, Ground};
use parry3d::math::{IVector, Pose, Vector};
use parry3d::query::{
    ContactManifold, DefaultQueryDispatcher, PersistentQueryDispatcher, PointQuery, Ray, RayCast,
};
use parry3d::shape::{Ball, Voxels};

const SEED: u64 = 0xC011_1DE3;
const SIDE: u16 = 3;
const EXTENT: i32 = 24;
const CARVE_RADIUS: i32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
struct QueryReceipt {
    occupancy: bool,
    ray_toi_bits: u32,
    point_inside: bool,
    contacts: usize,
    minimum_contact_bits: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExactReceipt {
    compared_voxels: usize,
    occupied_voxels: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeltaReceipt {
    source_revision: u64,
    committed_revision: u64,
    regions_scanned: Vec<[i16; 3]>,
    voxels_compared: usize,
    voxels_changed: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum ProjectionError {
    SourceRevision { projection: u64, provided: u64 },
    TargetRevision { expected: u64, ground: u64 },
    MissingBrick([i16; 3]),
    UnsupportedContact,
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceRevision {
                projection,
                provided,
            } => write!(
                f,
                "collision projection is at revision {projection}, not supplied source {provided}"
            ),
            Self::TargetRevision { expected, ground } => write!(
                f,
                "collision delta expected Ground revision {expected}, found {ground}"
            ),
            Self::MissingBrick(key) => write!(f, "dirty Ground brick {key:?} is missing"),
            Self::UnsupportedContact => write!(f, "Parry refused the Voxels/Ball contact query"),
        }
    }
}

impl Error for ProjectionError {}

struct GroundCollision {
    revision: u64,
    occupied: usize,
    voxels: Voxels,
}

impl GroundCollision {
    fn from_ground(ground: &Ground) -> Self {
        let mut occupied = Vec::new();
        for key in ground.keys() {
            let (brick, base) = ground
                .brick_materials(key)
                .expect("a public Ground key resolves to its brick");
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        if brick.get([x, y, z]) != AIR {
                            occupied.push(ivec([base[0] + x, base[1] + y, base[2] + z]));
                        }
                    }
                }
            }
        }

        Self {
            revision: ground.revision(),
            occupied: occupied.len(),
            voxels: Voxels::new(Vector::new(1.0, 1.0, 1.0), &occupied),
        }
    }

    fn apply_delta(
        &mut self,
        ground: &Ground,
        source_revision: u64,
        dirty: &[[i16; 3]],
    ) -> Result<DeltaReceipt, ProjectionError> {
        if source_revision != self.revision {
            return Err(ProjectionError::SourceRevision {
                projection: self.revision,
                provided: source_revision,
            });
        }

        let expected_target = source_revision + 1;
        if ground.revision() != expected_target {
            return Err(ProjectionError::TargetRevision {
                expected: expected_target,
                ground: ground.revision(),
            });
        }

        let regions_scanned = dirty.iter().copied().collect::<BTreeSet<_>>();
        let mut updates = Vec::new();
        for key in &regions_scanned {
            let (brick, base) = ground
                .brick_materials(*key)
                .ok_or(ProjectionError::MissingBrick(*key))?;
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        let at = [base[0] + x, base[1] + y, base[2] + z];
                        let filled = brick.get([x, y, z]) != AIR;
                        if parry_filled(&self.voxels, at) != filled {
                            updates.push((at, filled));
                        }
                    }
                }
            }
        }

        for (at, filled) in &updates {
            let previous = self.voxels.set_voxel(ivec(*at), *filled);
            debug_assert_eq!(!previous.is_empty(), !filled);
            if *filled {
                self.occupied += 1;
            } else {
                self.occupied -= 1;
            }
        }
        self.revision = ground.revision();

        Ok(DeltaReceipt {
            source_revision,
            committed_revision: self.revision,
            regions_scanned: regions_scanned.into_iter().collect(),
            voxels_compared: dirty.len() * (BRICK * BRICK * BRICK) as usize,
            voxels_changed: updates.len(),
        })
    }

    fn query(&self, ground: &Ground, target: [i32; 3]) -> Result<QueryReceipt, ProjectionError> {
        let point = voxel_center(target);
        let ray_origin = Vector::new(point.x, point.y + 3.0, point.z);
        let ray = Ray::new(ray_origin, Vector::new(0.0, -1.0, 0.0));
        let hit = self
            .voxels
            .cast_local_ray_and_get_normal(&ray, 64.0, true)
            .expect("stored ground lies below the query ray");
        let expected_toi = ground_ray_toi(ground, ray_origin, target[0], target[2]);
        assert_eq!(
            hit.time_of_impact.to_bits(),
            expected_toi.to_bits(),
            "Parry ray must hit the same stored Ground voxel"
        );

        let ball_center = Vector::new(point.x, target[1] as f32 + 0.9, point.z);
        let (contacts, minimum_contact_bits) = contact_receipt(&self.voxels, ball_center)?;

        Ok(QueryReceipt {
            occupancy: parry_filled(&self.voxels, target),
            ray_toi_bits: hit.time_of_impact.to_bits(),
            point_inside: self.voxels.contains_local_point(point),
            contacts,
            minimum_contact_bits,
        })
    }

    fn assert_exact(&self, ground: &Ground) -> ExactReceipt {
        let mut compared = 0;
        let mut occupied = 0;
        for key in ground.keys() {
            let (brick, base) = ground
                .brick_materials(key)
                .expect("a public Ground key resolves to its brick");
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        let at = [base[0] + x, base[1] + y, base[2] + z];
                        let expected = brick.get([x, y, z]) != AIR;
                        assert_eq!(
                            parry_filled(&self.voxels, at),
                            expected,
                            "occupancy differs at {at:?}"
                        );
                        compared += 1;
                        occupied += expected as usize;
                    }
                }
            }
        }

        assert_eq!(occupied, self.occupied);
        let parry_occupied = self
            .voxels
            .voxels()
            .filter(|voxel| !voxel.state.is_empty())
            .inspect(|voxel| {
                let at = [
                    voxel.grid_coords.x,
                    voxel.grid_coords.y,
                    voxel.grid_coords.z,
                ];
                assert!(at[1] >= 0, "implicit bedrock is not materialized");
                assert!(ground.solid(at), "Parry contains an extra voxel at {at:?}");
            })
            .count();
        assert_eq!(parry_occupied, occupied);

        ExactReceipt {
            compared_voxels: compared,
            occupied_voxels: occupied,
        }
    }
}

fn make_ground() -> Ground {
    let grown = Places::grown(SEED, SIDE, EXTENT);
    Ground::grow(&grown, EXTENT)
}

fn choose_boundary_surface(ground: &Ground) -> [i32; 3] {
    for z in (-EXTENT + 1)..EXTENT {
        if z.rem_euclid(BRICK) != BRICK - 1 {
            continue;
        }
        for x in (-EXTENT + 1)..EXTENT {
            if x.rem_euclid(BRICK) != BRICK - 1 {
                continue;
            }
            let Some(y) = ground.surface(x, z) else {
                continue;
            };
            if y < 3 || ground.solid([x, y + 1, z]) {
                continue;
            }
            let supported = (-1..=1).all(|dz| {
                (-1..=1).all(|dx| {
                    ground.solid([x + dx, y, z + dz]) && ground.solid([x + dx, y - 1, z + dz])
                })
            });
            if supported {
                return [x, y, z];
            }
        }
    }
    panic!("fixture needs an exposed brick-boundary surface with two solid layers");
}

fn contact_receipt(
    voxels: &Voxels,
    ball_center: Vector,
) -> Result<(usize, Option<u32>), ProjectionError> {
    let dispatcher = DefaultQueryDispatcher;
    let ball = Ball::new(0.2);
    let position = Pose::translation(ball_center.x, ball_center.y, ball_center.z);
    let mut manifolds: Vec<ContactManifold<(), ()>> = Vec::new();
    let mut workspace = None;
    dispatcher
        .contact_manifolds(
            &position,
            voxels,
            &ball,
            0.0,
            &mut manifolds,
            &mut workspace,
        )
        .map_err(|_| ProjectionError::UnsupportedContact)?;

    let contacts = manifolds
        .iter()
        .flat_map(|manifold| manifold.points.iter())
        .collect::<Vec<_>>();
    let minimum = contacts
        .iter()
        .map(|contact| contact.dist)
        .min_by(f32::total_cmp)
        .map(f32::to_bits);
    Ok((contacts.len(), minimum))
}

fn ground_ray_toi(ground: &Ground, origin: Vector, x: i32, z: i32) -> f32 {
    let start_y = origin.y.floor() as i32;
    for y in (0..=start_y).rev() {
        if ground.solid([x, y, z]) {
            return origin.y - (y + 1) as f32;
        }
    }
    panic!("stored Ground has no ray target at x={x}, z={z}");
}

fn region_signatures(collision: &GroundCollision, ground: &Ground) -> BTreeMap<[i16; 3], u64> {
    ground
        .keys()
        .map(|key| {
            let (_, base) = ground
                .brick_materials(key)
                .expect("a public Ground key resolves to its brick");
            let mut hash = 0xcbf2_9ce4_8422_2325u64;
            for y in 0..BRICK {
                for z in 0..BRICK {
                    for x in 0..BRICK {
                        hash ^= parry_filled(
                            &collision.voxels,
                            [base[0] + x, base[1] + y, base[2] + z],
                        ) as u64;
                        hash = hash.wrapping_mul(0x100_0000_01b3);
                    }
                }
            }
            (key, hash)
        })
        .collect()
}

fn parry_filled(voxels: &Voxels, at: [i32; 3]) -> bool {
    voxels
        .voxel_state(ivec(at))
        .is_some_and(|state| !state.is_empty())
}

fn ivec(at: [i32; 3]) -> IVector {
    IVector::new(at[0], at[1], at[2])
}

fn voxel_center(at: [i32; 3]) -> Vector {
    Vector::new(at[0] as f32 + 0.5, at[1] as f32 + 0.5, at[2] as f32 + 0.5)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut ground = make_ground();
    assert_eq!(ground.revision(), 0);
    assert!(ground.drain_dirty().is_empty());
    let target = choose_boundary_surface(&ground);

    let mut collision = GroundCollision::from_ground(&ground);
    let initial_exact = collision.assert_exact(&ground);
    let initial_queries = collision.query(&ground, target)?;
    assert!(initial_queries.occupancy);
    assert!(initial_queries.point_inside);
    assert!(initial_queries.contacts > 0);
    let initial_regions = region_signatures(&collision, &ground);
    let initial_ground = ground.clone();

    let removed = ground.carve(target, CARVE_RADIUS);
    assert!(removed > 0);
    let dirty = ground.drain_dirty();
    assert!(
        dirty.len() >= 4,
        "the boundary carve must cross Ground bricks"
    );

    let refusal = collision
        .apply_delta(&ground, collision.revision + 1, &dirty)
        .expect_err("a stale source revision must be refused");
    assert!(matches!(refusal, ProjectionError::SourceRevision { .. }));
    assert_eq!(collision.revision, 0);
    assert_eq!(
        initial_queries,
        collision.query(&initial_ground, target)?,
        "revision refusal must leave collision state unchanged"
    );

    let mut skipped_ground = initial_ground.clone();
    assert!(skipped_ground.carve(target, CARVE_RADIUS) > 0);
    assert!(
        skipped_ground.carve(target, CARVE_RADIUS + 1) > 0,
        "the larger second carve must create revision two"
    );
    let skipped_dirty = skipped_ground.drain_dirty();
    let skipped = collision
        .apply_delta(&skipped_ground, 0, &skipped_dirty)
        .expect_err("a skipped target revision must be refused");
    assert!(matches!(skipped, ProjectionError::TargetRevision { .. }));
    assert_eq!(
        initial_queries,
        collision.query(&initial_ground, target)?,
        "skipped revision refusal must leave collision state unchanged"
    );

    let delta = collision.apply_delta(&ground, 0, &dirty)?;
    assert_eq!(delta.voxels_changed, removed as usize);
    assert_eq!(delta.regions_scanned.len(), dirty.len());
    let committed_exact = collision.assert_exact(&ground);
    let committed_queries = collision.query(&ground, target)?;
    assert!(!committed_queries.occupancy);
    assert!(!committed_queries.point_inside);
    assert_eq!(committed_queries.contacts, 0);
    assert_ne!(initial_queries.ray_toi_bits, committed_queries.ray_toi_bits);

    let dirty_set = dirty.iter().copied().collect::<BTreeSet<_>>();
    let committed_regions = region_signatures(&collision, &ground);
    for (key, signature) in &initial_regions {
        if !dirty_set.contains(key) {
            assert_eq!(
                committed_regions.get(key),
                Some(signature),
                "unchanged Ground region {key:?} changed occupancy"
            );
        }
    }

    let mut replay_ground = make_ground();
    let replay_target = choose_boundary_surface(&replay_ground);
    assert_eq!(replay_target, target);
    let mut replay_collision = GroundCollision::from_ground(&replay_ground);
    assert_eq!(
        replay_collision.query(&replay_ground, target)?,
        initial_queries
    );
    let replay_removed = replay_ground.carve(target, CARVE_RADIUS);
    let replay_dirty = replay_ground.drain_dirty();
    assert_eq!(replay_removed, removed);
    assert_eq!(replay_dirty, dirty);
    let replay_delta = replay_collision.apply_delta(&replay_ground, 0, &replay_dirty)?;
    assert_eq!(replay_delta, delta);
    assert_eq!(
        replay_collision.query(&replay_ground, target)?,
        committed_queries
    );

    println!(
        "projection receipt: Ground revision 0, {} stored bricks, {} compared cells, {} occupied Parry voxels",
        ground.brick_count(),
        initial_exact.compared_voxels,
        initial_exact.occupied_voxels
    );
    println!(
        "delta receipt: revision {} -> {}, target {target:?}, {removed} removed voxels, {} dirty 8^3 regions, {} cells rescanned",
        delta.source_revision,
        delta.committed_revision,
        delta.regions_scanned.len(),
        delta.voxels_compared
    );
    println!(
        "query receipt: ray {:.1} -> {:.1}, point inside {} -> {}, contacts {} -> {}",
        f32::from_bits(initial_queries.ray_toi_bits),
        f32::from_bits(committed_queries.ray_toi_bits),
        initial_queries.point_inside,
        committed_queries.point_inside,
        initial_queries.contacts,
        committed_queries.contacts
    );
    println!(
        "authority receipt: stale source and skipped target revisions refused before mutation; {} unchanged regions retained occupancy; replay query bits identical",
        initial_regions.len() - dirty_set.len()
    );
    println!(
        "scope receipt: {} committed occupied voxels; implicit y<0 bedrock remains an analytic half-space",
        committed_exact.occupied_voxels
    );

    Ok(())
}
