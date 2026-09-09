// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resolving a body plan into an actual attachment.
//!
//! This is the default path: a player eats, and growth happens by the plan.
//! Explicit placement stays available for an editor, which is the ruling
//! Mark made — automatic and symmetric by default, total control possible but
//! never the resting state.

use crate::body::{Attachment, BodyDocument, PartId, Yaw};
use crate::plan::{Facing, Role, classify};

/// Where a part should go, and whether it grows a mirrored twin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Growth {
    pub parent: PartId,
    pub offset: [i32; 3],
    pub yaw: Yaw,
    pub facing: Facing,
    pub role: Role,
    /// The mirrored attachment, when the plan pairs this facing. One meal
    /// grows two parts and the mass is split between them.
    pub mirror: Option<(PartId, [i32; 3])>,
}

/// Finds where a part of the given size should attach, by the body's plan.
///
/// Candidate sites are generated from existing parts, tried in the plan's
/// facing order, and rejected if they would overlap anything already there.
/// Returns `None` when nothing fits, which the caller reports as a rejection
/// rather than forcing a part into occupied space.
pub fn resolve(body: &BodyDocument, half_extent: [i32; 3]) -> Option<Growth> {
    let role = classify(half_extent);

    for facing in body.plan.candidates(role) {
        // Nearest the root first, so a body grows outward rather than
        // sprouting from whatever it ate last.
        for part in body.living() {
            let anchor = body.world_pivot(part.id)?;
            let offset = flush(part.half_extent, half_extent, facing);
            let at = [
                anchor[0] + offset[0],
                anchor[1] + offset[1],
                anchor[2] + offset[2],
            ];

            if !free(body, at, half_extent) {
                continue;
            }

            // A bilateral plan means what it says: if the mirror will not fit,
            // this is not a site for a pair, so keep looking rather than
            // growing a single lopsided limb. Best-effort mirroring drifts a
            // body sideways one failed pair at a time.
            let mirror = if body.plan.mirrors(facing) {
                let mirrored_offset = flush(part.half_extent, half_extent, facing.mirrored());
                let mirrored_at = [
                    anchor[0] + mirrored_offset[0],
                    anchor[1] + mirrored_offset[1],
                    anchor[2] + mirrored_offset[2],
                ];
                if mirrored_at == at || !free(body, mirrored_at, half_extent) {
                    continue;
                }
                Some((part.id, mirrored_offset))
            } else {
                None
            };

            return Some(Growth {
                parent: part.id,
                offset,
                yaw: Yaw::Zero,
                facing,
                role,
                mirror,
            });
        }
    }

    None
}

/// Pivot-to-pivot displacement that puts a part flush against a host's face.
///
/// **Symmetric, which is the point of pivots.** Both parts are measured from
/// their centres, so the two sides of an axis are exact negations and the
/// offset no longer depends on which face is being used.
fn flush(host_half: [i32; 3], own_half: [i32; 3], facing: Facing) -> [i32; 3] {
    let (axis, sign) = facing.axis();
    let mut offset = [0i32; 3];
    offset[axis] = sign * (host_half[axis].abs() + own_half[axis].abs());
    offset
}

/// Whether a part of `half` centred at `at` clears every existing part.
fn free(body: &BodyDocument, at: [i32; 3], half: [i32; 3]) -> bool {
    for part in body.living() {
        let Some(centre) = body.world_pivot(part.id) else {
            continue;
        };
        let overlaps = (0..3).all(|axis| {
            let gap = (at[axis] - centre[axis]).abs();
            gap < half[axis].abs() + part.half_extent[axis].abs()
        });
        if overlaps {
            return false;
        }
    }
    true
}

/// Builds the attachment a [`Growth`] describes.
pub fn attachment(growth: &Growth) -> Attachment {
    Attachment {
        parent: growth.parent,
        offset: growth.offset,
        yaw: growth.yaw,
    }
}

/// Builds the mirrored attachment, when there is one.
pub fn mirror_attachment(growth: &Growth) -> Option<Attachment> {
    growth.mirror.map(|(parent, offset)| Attachment {
        parent,
        offset,
        yaw: growth.yaw,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{Provenance, SpeciesId, VolumeRef};
    use crate::plan::Symmetry;

    fn body() -> BodyDocument {
        BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [3, 3, 3])
    }

    fn grow(body: &mut BodyDocument, half: [i32; 3], mass: u64) -> Option<Growth> {
        let growth = resolve(body, half)?;
        let parts = if growth.mirror.is_some() { 2 } else { 1 };
        let each = mass / parts;
        body.attach(
            VolumeRef::from_tag(9),
            each,
            half,
            attachment(&growth),
            Provenance::founding(),
        )
        .ok()?;
        if let Some(mirrored) = mirror_attachment(&growth) {
            body.attach(
                VolumeRef::from_tag(9),
                each,
                half,
                mirrored,
                Provenance::founding(),
            )
            .ok()?;
        }
        Some(growth)
    }

    #[test]
    fn a_limb_grows_to_the_side_and_brings_its_pair() {
        let mut body = body();
        let growth = grow(&mut body, [4, 1, 1], 400).expect("a limb fits");

        assert_eq!(growth.role, Role::Limb);
        assert!(growth.facing.is_lateral(), "limbs go out to the sides");
        assert!(
            growth.mirror.is_some(),
            "bilateral plans pair lateral growth"
        );
        assert_eq!(body.len(), 3, "one meal, two limbs, plus the core");
    }

    #[test]
    fn a_mirrored_pair_splits_the_mass_it_came_from() {
        let mut body = body();
        let before = body.total_mass_mg();
        grow(&mut body, [4, 1, 1], 400).unwrap();
        assert_eq!(
            body.total_mass_mg(),
            before + 400,
            "the pair together weigh what was eaten"
        );
    }

    #[test]
    fn a_mirrored_body_is_balanced_across_its_lateral_axis() {
        let mut body = body();
        // With pivots, the root is centred on the origin, so the midline is
        // zero and a balanced body says so plainly.
        let midline = body.centre_of_mass()[0];
        assert_eq!(midline, 0);

        grow(&mut body, [4, 1, 1], 400).unwrap();
        assert_eq!(
            body.centre_of_mass()[0],
            midline,
            "a paired limb leaves the centre of mass on the midline"
        );
    }

    /// The regression for best-effort mirroring: a bilateral plan that grows a
    /// single limb whenever the mirror will not fit drifts the whole body
    /// sideways, one failed pair at a time.
    #[test]
    fn a_bilateral_body_never_drifts_off_its_midline() {
        let mut body = body();
        let midline = body.centre_of_mass()[0];

        for _ in 0..10 {
            if grow(&mut body, [4, 1, 1], 300).is_none() {
                break;
            }
        }

        assert!(body.len() >= 5, "the body must actually grow limbs");
        assert_eq!(
            body.centre_of_mass()[0],
            midline,
            "every lateral part is paired, so the body stays balanced"
        );
    }

    #[test]
    fn parts_never_overlap_however_much_is_eaten() {
        let mut body = body();
        for i in 0..12 {
            let half = if i % 3 == 0 { [4, 1, 1] } else { [2, 2, 2] };
            if grow(&mut body, half, 200).is_none() {
                break;
            }
        }
        assert!(body.len() > 6, "the fixture must actually grow");

        // Every pair of parts is disjoint.
        for a in &body.parts {
            for b in &body.parts {
                if a.id >= b.id {
                    continue;
                }
                let (pa, pb) = (
                    body.world_pivot(a.id).unwrap(),
                    body.world_pivot(b.id).unwrap(),
                );
                // Centre-to-centre: two boxes overlap when the gap on every
                // axis is smaller than the sum of their half-extents.
                let overlaps = (0..3).all(|ax| {
                    (pa[ax] - pb[ax]).abs() < a.half_extent[ax].abs() + b.half_extent[ax].abs()
                });
                assert!(!overlaps, "{:?} overlaps {:?}", a.id, b.id);
            }
        }
    }

    #[test]
    fn an_unmirrored_plan_grows_one_at_a_time() {
        let mut body = body();
        body.plan.symmetry = Symmetry::None;
        let growth = grow(&mut body, [4, 1, 1], 400).unwrap();
        assert!(growth.mirror.is_none());
        assert_eq!(body.len(), 2);
    }

    #[test]
    fn the_plan_decides_where_a_role_goes() {
        let mut body = body();
        body.plan.set_preference(Role::Limb, Facing::Above);
        body.plan.tolerance = 0;
        let growth = grow(&mut body, [4, 1, 1], 100).unwrap();
        assert_eq!(growth.facing, Facing::Above);
        assert!(growth.mirror.is_none(), "vertical growth does not pair");
    }

    /// What the pivot bought: a turned part stays joined. Before pivots a
    /// rotation swung a part about its lowest corner, so a limb that had been
    /// placed flush swung off the joint entirely, and every yaw in the game
    /// was pinned to zero to hide it.
    #[test]
    fn a_turned_part_stays_joined_to_its_parent() {
        let mut body = body();
        let arm = body
            .attach(
                VolumeRef::from_tag(9),
                100,
                [4, 1, 1],
                Attachment {
                    parent: body.root,
                    offset: [7, 0, 0],
                    yaw: Yaw::Quarter,
                },
                Provenance::founding(),
            )
            .unwrap();

        // The joint is where the plan put it, whatever the part does about it.
        assert_eq!(body.world_pivot(arm), Some([7, 0, 0]));

        // And the part is still the same size, just pointing elsewhere: its
        // far tip is a half-length from the pivot in the turned direction.
        let tip = body.place(arm, [8, 1, 1]).unwrap();
        let pivot = body.world_pivot(arm).unwrap();
        let reach: i32 = (0..3).map(|a| (tip[a] - pivot[a]).abs()).sum();
        assert_eq!(reach, 4, "a quarter turn moves the tip, not the joint");
    }

    #[test]
    fn growth_is_deterministic() {
        let run = || {
            let mut b = body();
            for _ in 0..6 {
                if grow(&mut b, [4, 1, 1], 200).is_none() {
                    break;
                }
            }
            b
        };
        assert_eq!(run(), run());
    }
}
