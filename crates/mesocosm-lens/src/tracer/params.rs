// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The two uniform blocks the trace pass reads: the frame's own parameters
//! (camera, space, grade, single pose) and the pose roster beside them.

use bytemuck::{Pod, Zeroable};
use modulus::BrickTraceSpace;

use super::types::{BrickChange, BrickFrameInput, BrickTraceError};
use crate::{CritterPose, MAX_CAPSULES, MAX_ROSTER, MAX_ROSTER_CAPSULES};

pub(super) const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct TraceParams {
    pub camera: super::TraceCamera,
    pub space: BrickTraceSpace,
    pub fog: [f32; 4],
    pub look: [f32; 4],
    /// Column-major, identity when the frame carries no depth join.
    pub clip_from_world: [[f32; 4]; 4],
    pub critter: CritterParams,
}

/// The frame's single full-fidelity pose. Unchanged since the tracer's first
/// cut, and left alone by the roster: `MAX_CAPSULES` capsules, one tint, one
/// bounds sphere, two eyes.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct CritterParams {
    pub bounds: [f32; 4],
    pub tint_count: [f32; 4],
    pub eyes: [[f32; 4]; 2],
    pub pairs: [[f32; 4]; MAX_CAPSULES * 2],
}

/// One roster member: a background silhouette, so no eyes and the reduced
/// capsule budget the cap arithmetic in [`crate::MAX_ROSTER`] buys.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub(super) struct RosterPose {
    pub bounds: [f32; 4],
    pub tint_count: [f32; 4],
    pub pairs: [[f32; 4]; MAX_ROSTER_CAPSULES * 2],
}

/// Bytes of roster header before the first member. `count.x` members follow.
pub(super) const ROSTER_HEADER_BYTES: u64 = 16;

/// The whole roster binding, headroom included.
pub(super) const ROSTER_BUFFER_BYTES: u64 =
    ROSTER_HEADER_BYTES + (MAX_ROSTER * size_of::<RosterPose>()) as u64;

impl TraceParams {
    pub(super) fn from_input(input: BrickFrameInput<'_>) -> Self {
        Self {
            camera: input.camera,
            space: BrickTraceSpace::from_map(input.map),
            fog: [
                input.grade.fog[0],
                input.grade.fog[1],
                input.grade.fog[2],
                input.grade.fog_start,
            ],
            look: [
                input.grade.dither,
                input.grade.fog_bands,
                input.grade.palette_len as f32,
                0.0,
            ],
            clip_from_world: input.clip_from_world.unwrap_or(IDENTITY),
            critter: CritterParams::from_pose(input.pose),
        }
    }
}

impl CritterParams {
    fn from_pose(pose: Option<&CritterPose>) -> Self {
        let Some(pose) = pose else {
            return Self::zeroed();
        };
        let mut pairs = [[0.0; 4]; MAX_CAPSULES * 2];
        for (index, capsule) in pose.capsules.iter().enumerate() {
            pairs[index * 2] = [capsule.a[0], capsule.a[1], capsule.a[2], capsule.ra];
            pairs[index * 2 + 1] = [capsule.b[0], capsule.b[1], capsule.b[2], capsule.rb];
        }
        Self {
            bounds: [
                pose.bounds_centre[0],
                pose.bounds_centre[1],
                pose.bounds_centre[2],
                pose.bounds_radius,
            ],
            tint_count: [
                pose.tint[0],
                pose.tint[1],
                pose.tint[2],
                pose.capsules.len() as f32,
            ],
            eyes: pose.eyes,
            pairs,
        }
    }
}

impl RosterPose {
    /// Capsules past [`crate::MAX_ROSTER_CAPSULES`] are dropped rather than
    /// refused, and the ones kept are the **largest by extent**.
    ///
    /// Document order is the axial chain from the root outward, so taking the
    /// first N kept a body's head end rather than its silhouette — and a
    /// silhouette is the whole job of a background member. Ties keep the
    /// earlier capsule, so the choice is deterministic at equal extent.
    pub(super) fn from_pose(pose: &CritterPose) -> Self {
        let mut pairs = [[0.0; 4]; MAX_ROSTER_CAPSULES * 2];
        let count = pose.capsules.len().min(MAX_ROSTER_CAPSULES);
        for (index, kept) in widest(&pose.capsules).into_iter().enumerate() {
            let capsule = &pose.capsules[kept];
            pairs[index * 2] = [capsule.a[0], capsule.a[1], capsule.a[2], capsule.ra];
            pairs[index * 2 + 1] = [capsule.b[0], capsule.b[1], capsule.b[2], capsule.rb];
        }
        Self {
            bounds: [
                pose.bounds_centre[0],
                pose.bounds_centre[1],
                pose.bounds_centre[2],
                pose.bounds_radius,
            ],
            tint_count: [pose.tint[0], pose.tint[1], pose.tint[2], count as f32],
            pairs,
        }
    }
}

/// Indices of the [`MAX_ROSTER_CAPSULES`] largest capsules by **extent**, back
/// in document order so the upload is stable. The common case is a body inside
/// the budget, which costs one comparison and no allocation.
///
/// **Extent, not radius** (ruled 2026-08-31). DC3 ordered by the fatter
/// endpoint radius, and on a stand of producers that kept the round root
/// masses and dropped the tall thin fronds — silhouette *area* delivered,
/// silhouette *reading* not. `radius² × length` is the capsule's own volume up
/// to a constant, so a long thin frond outranks a small fat bead. The length is
/// the capsule's, cap to cap: `|b − a|` alone is **zero** for every ball-shaped
/// part, and the primitive `[2,2,2]` trunk segment is a ball.
fn widest(capsules: &[crate::critter::Capsule]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..capsules.len()).collect();
    if order.len() > MAX_ROSTER_CAPSULES {
        let extent = |index: &usize| {
            let capsule = &capsules[*index];
            let radius = capsule.ra.max(capsule.rb);
            let axis = [0, 1, 2].map(|k| capsule.b[k] - capsule.a[k]);
            let length = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
            radius * radius * (length + 2.0 * radius)
        };
        order.sort_by(|a, b| extent(b).total_cmp(&extent(a)).then(a.cmp(b)));
        order.truncate(MAX_ROSTER_CAPSULES);
        order.sort_unstable();
    }
    order
}

/// Capsules the frame's roster could not carry, over every member it drew.
pub(super) fn roster_capsules_dropped(input: BrickFrameInput<'_>) -> u32 {
    input
        .roster
        .iter()
        .take(MAX_ROSTER)
        .map(|pose| pose.capsules.len().saturating_sub(MAX_ROSTER_CAPSULES))
        .sum::<usize>() as u32
}

/// The roster the frame actually carries, capped and laid out for upload.
pub(super) fn roster_of(input: BrickFrameInput<'_>) -> Vec<RosterPose> {
    input
        .roster
        .iter()
        .take(MAX_ROSTER)
        .map(RosterPose::from_pose)
        .collect()
}

pub(super) fn validates_pose(input: BrickFrameInput<'_>) -> Result<(), BrickTraceError> {
    let actual = input.pose.map_or(0, |pose| pose.capsules.len());
    if actual > MAX_CAPSULES {
        return Err(BrickTraceError::TooManyCapsules {
            actual,
            maximum: MAX_CAPSULES,
        });
    }
    Ok(())
}

pub(super) fn validates_change(input: BrickFrameInput<'_>) -> Result<(), BrickTraceError> {
    if let BrickChange::Slots(slots) = input.change {
        for slot in slots {
            if input.map.pointer_coord(*slot).is_none() {
                return Err(BrickTraceError::UnknownBrickSlot(*slot));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::critter::Capsule;

    /// A frond against a bead: the case the extent key was ruled for.
    ///
    /// The DC3 finding was that ordering by radius alone keeps a producer's
    /// round root masses and drops its tall thin fronds. A capsule twice as
    /// long at three quarters the radius has the larger extent and has to win.
    #[test]
    fn a_long_thin_capsule_outranks_a_short_fat_one() {
        let bead = |index: usize| Capsule {
            a: [index as f32, 0.0, 0.0],
            ra: 1.0,
            b: [index as f32, 0.2, 0.0],
            rb: 1.0,
        };
        let frond = |index: usize| Capsule {
            a: [index as f32, 0.0, 0.0],
            ra: 0.75,
            b: [index as f32, 6.0, 0.0],
            rb: 0.75,
        };
        // Beads first in document order, so only the key can save the fronds.
        let capsules: Vec<Capsule> = (0..MAX_ROSTER_CAPSULES)
            .map(bead)
            .chain((MAX_ROSTER_CAPSULES..MAX_ROSTER_CAPSULES * 2).map(frond))
            .collect();
        let pose = CritterPose::from_capsules(capsules, [[0.0; 4]; 2], [0.0; 3]);
        let uploaded = RosterPose::from_pose(&pose);
        for index in 0..MAX_ROSTER_CAPSULES {
            assert_eq!(
                uploaded.pairs[index * 2][3],
                0.75,
                "capsule {index} is a frond"
            );
        }
    }

    /// And a ball is not free: `|b - a|` is zero for the primitive `[2,2,2]`
    /// segment, so the length the key reads has to be the capsule's own, cap to
    /// cap, or every trunk segment would rank below every sliver.
    #[test]
    fn a_ball_shaped_capsule_still_ranks_by_its_radius() {
        let ball = |index: usize| Capsule {
            a: [index as f32, 0.0, 0.0],
            ra: 2.0,
            b: [index as f32, 0.0, 0.0],
            rb: 2.0,
        };
        let sliver = |index: usize| Capsule {
            a: [index as f32, 0.0, 0.0],
            ra: 0.5,
            b: [index as f32, 8.0, 0.0],
            rb: 0.5,
        };
        let capsules: Vec<Capsule> = (0..MAX_ROSTER_CAPSULES)
            .map(sliver)
            .chain((MAX_ROSTER_CAPSULES..MAX_ROSTER_CAPSULES * 2).map(ball))
            .collect();
        let pose = CritterPose::from_capsules(capsules, [[0.0; 4]; 2], [0.0; 3]);
        let uploaded = RosterPose::from_pose(&pose);
        for index in 0..MAX_ROSTER_CAPSULES {
            assert_eq!(
                uploaded.pairs[index * 2][3],
                2.0,
                "capsule {index} is a ball"
            );
        }
    }

    /// DC-R1, read off the uniform: the budget goes to the silhouette.
    ///
    /// The fat capsules are last in document order on purpose — that is the
    /// case the old `take(10)` got wrong, since document order is the axial
    /// chain from the root outward.
    #[test]
    fn a_truncated_member_keeps_its_widest_capsules() {
        let capsules: Vec<Capsule> = (0..MAX_ROSTER_CAPSULES * 2)
            .map(|index| {
                let radius = if index < MAX_ROSTER_CAPSULES {
                    0.1
                } else {
                    0.6
                };
                Capsule {
                    a: [index as f32, 0.0, 0.0],
                    ra: radius,
                    b: [index as f32, 1.0, 0.0],
                    rb: radius,
                }
            })
            .collect();
        let pose = CritterPose::from_capsules(capsules, [[0.0; 4]; 2], [0.0; 3]);
        let uploaded = RosterPose::from_pose(&pose);
        assert_eq!(uploaded.tint_count[3], MAX_ROSTER_CAPSULES as f32);
        for index in 0..MAX_ROSTER_CAPSULES {
            assert_eq!(uploaded.pairs[index * 2][3], 0.6, "capsule {index} is fat");
        }
        // Kept in document order, so the upload is stable frame to frame.
        let xs: Vec<f32> = (0..MAX_ROSTER_CAPSULES)
            .map(|index| uploaded.pairs[index * 2][0])
            .collect();
        assert!(xs.windows(2).all(|pair| pair[0] < pair[1]));
    }

    /// Equal radii are a real case — a body of one repeated template — and the
    /// tie-break has to be the same every frame or the roster churns.
    #[test]
    fn equal_radii_truncate_to_document_order() {
        let capsules: Vec<Capsule> = (0..MAX_ROSTER_CAPSULES + 5)
            .map(|index| Capsule {
                a: [index as f32, 0.0, 0.0],
                ra: 0.5,
                b: [index as f32, 1.0, 0.0],
                rb: 0.5,
            })
            .collect();
        let pose = CritterPose::from_capsules(capsules, [[0.0; 4]; 2], [0.0; 3]);
        let uploaded = RosterPose::from_pose(&pose);
        for index in 0..MAX_ROSTER_CAPSULES {
            assert_eq!(uploaded.pairs[index * 2][0], index as f32);
        }
    }

    /// The cap arithmetic documented on [`crate::MAX_ROSTER`], asserted
    /// against the limit that actually binds — the downlevel WebGL2 one.
    #[test]
    fn the_roster_binding_fits_the_downlevel_uniform_limit() {
        let limit = wgpu::Limits::downlevel_webgl2_defaults().max_uniform_buffer_binding_size;
        assert_eq!(limit, 16_384);
        // The played pose at DC3's 256: 16 + 16 + 32 + 32 · 256.
        assert_eq!(size_of::<CritterParams>(), 8256);
        // A roster member at DC3's 11: 16 + 16 + 32 · 11.
        assert_eq!(size_of::<RosterPose>(), 384);
        assert_eq!(ROSTER_BUFFER_BYTES, 15_376);
        assert!(ROSTER_BUFFER_BYTES < limit);
        assert!((size_of::<TraceParams>() as u64) < limit);
        // Both bindings are live at once, and each has to fit the same limit
        // on its own. The frame uniform (header 224 B + pose, including the slab wall) spends 51.7%,
        // the roster 93.8%.
        assert_eq!(size_of::<TraceParams>(), 8480);
        assert_eq!(ROSTER_BUFFER_BYTES * 100 / limit, 93);
        // The budget §3 writes as `M × (C + 1) ≤ 511`, checked rather than
        // recited: 40 members at 11 capsules is 480, and one more capsule each
        // would be 520.
        assert_eq!(MAX_ROSTER * (MAX_ROSTER_CAPSULES + 1), 480);
        assert_eq!(MAX_ROSTER * (MAX_ROSTER_CAPSULES + 2), 520);
        // What the reduced budget buys off: at `MAX_CAPSULES` per member the
        // same binding would hold one body.
        assert_eq!(
            (limit - ROSTER_HEADER_BYTES) / size_of::<CritterParams>() as u64,
            1
        );
    }
}
