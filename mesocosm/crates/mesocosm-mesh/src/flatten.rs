// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Composing a whole body into one occupancy grid.
//!
//! The live renderer wants quads, one mesh per part, kept separate so limbs
//! can move. A **sprite baker wants the opposite**: a single grid it can
//! project voxel by voxel with depth sorting, because a baked sprite has no
//! moving parts.
//!
//! Both derive from the same body document, which is the actual shared organ.
//! An earlier note in the plan claimed Isometry's baker wanted the mesher's
//! quads; it does not. `isometry-voxel::bake_facing` takes an occupancy grid,
//! so this is the adapter that lane needs, and Isometry's baker requires no
//! change.

use mesocosm_core::{BodyDocument, Yaw};

use crate::{MeshError, Volume, VolumeSource, place_point};

/// A body composed into one grid, plus where that grid sits in body space.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flattened {
    pub volume: Volume,
    /// Body-space coordinate of the grid's `(0, 0, 0)` cell. Body space has
    /// negative coordinates; a grid does not.
    pub origin: [i32; 3],
}

impl Flattened {
    /// Reads a cell by body-space coordinate.
    pub fn at(&self, body_space: [i32; 3]) -> u8 {
        let local = [
            body_space[0] - self.origin[0],
            body_space[1] - self.origin[1],
            body_space[2] - self.origin[2],
        ];
        if local.iter().any(|c| *c < 0) {
            return 0;
        }
        self.volume
            .get(local[0] as u32, local[1] as u32, local[2] as u32)
    }
}

/// Composes every part of a body into a single occupancy grid.
///
/// Parts are written in document order, so a later part overwrites an earlier
/// one where they overlap. That is the right behaviour for a sprite: the grid
/// records what is solid, and the baker decides what is visible.
pub fn flatten(body: &BodyDocument, source: &impl VolumeSource) -> Result<Flattened, MeshError> {
    flatten_attributed(body, source).map(|(flattened, _)| flattened)
}

/// Flattens a body and also records **which part wrote each cell**.
///
/// The second grid is parallel to the first, in the same cell order, holding
/// `part index + 1` with `0` for empty. It exists because flattening otherwise
/// discards history: the occupancy grid records materials, and a reader that
/// wants to tint a sprite by where a limb came from has no way back to the
/// document without this map. See [`crate::profile`].
///
/// Overwrite order is shared with [`flatten`] by construction — the same loop
/// writes both grids — so attribution can never disagree with occupancy about
/// which part won a contested cell.
pub fn flatten_attributed(
    body: &BodyDocument,
    source: &impl VolumeSource,
) -> Result<(Flattened, Vec<u16>), MeshError> {
    // Two passes: find the extent, then fill. A body's parts can sit anywhere,
    // and a grid sized to the union avoids both clipping and waste.
    let mut min = [i32::MAX; 3];
    let mut max = [i32::MIN; 3];
    let mut seen = false;

    for part in body.living() {
        let (pivot_at, pivot, yaw, volume) = resolve(body, source, part.id)?;
        for corner in volume_corners(volume) {
            let placed = place_point(corner, yaw, pivot, pivot_at);
            for axis in 0..3 {
                min[axis] = min[axis].min(placed[axis]);
                max[axis] = max[axis].max(placed[axis]);
            }
            seen = true;
        }
    }

    if !seen {
        return Ok((
            Flattened {
                volume: Volume::empty([1, 1, 1]),
                origin: [0, 0, 0],
            },
            vec![0; 1],
        ));
    }

    let size = [
        (max[0] - min[0] + 1) as u32,
        (max[1] - min[1] + 1) as u32,
        (max[2] - min[2] + 1) as u32,
    ];
    let mut grid = Volume::empty(size);
    let cells = size.iter().map(|d| *d as usize).product::<usize>();
    let mut attribution = vec![0u16; cells];

    for part in body.living() {
        let slot = part.id.0 as usize;
        let (pivot_at, pivot, yaw, volume) = resolve(body, source, part.id)?;
        // Slot + 1, so `0` can mean empty. A body with more parts than a u16
        // can name is far outside anything growth produces, but the artifact
        // this feeds is portable, so the ceiling is checked rather than
        // assumed.
        let tag = u16::try_from(slot + 1).map_err(|_| MeshError::TooManyParts {
            parts: body.parts.len(),
        })?;

        for z in 0..volume.size[2] {
            for y in 0..volume.size[1] {
                for x in 0..volume.size[0] {
                    let material = volume.get(x, y, z);
                    if material == 0 {
                        continue;
                    }
                    let placed = place_point([x as i32, y as i32, z as i32], yaw, pivot, pivot_at);
                    let cell = [
                        (placed[0] - min[0]) as u32,
                        (placed[1] - min[1]) as u32,
                        (placed[2] - min[2]) as u32,
                    ];
                    grid.set(cell[0], cell[1], cell[2], material);
                    let index =
                        (cell[0] + cell[1] * size[0] + cell[2] * size[0] * size[1]) as usize;
                    attribution[index] = tag;
                }
            }
        }
    }

    Ok((
        Flattened {
            volume: grid,
            origin: min,
        },
        attribution,
    ))
}

/// A part resolved for flattening: where its pivot sits, the pivot itself,
/// its orientation, and the voxels to write.
type Placed<'a> = ([i32; 3], [i32; 3], Yaw, &'a Volume);

fn resolve<'a>(
    body: &BodyDocument,
    source: &'a impl VolumeSource,
    part: mesocosm_core::PartId,
) -> Result<Placed<'a>, MeshError> {
    let pivot_at = body
        .world_pivot(part)
        .ok_or(MeshError::Unplaceable { part })?;
    let yaw = body
        .world_yaw(part)
        .ok_or(MeshError::Unplaceable { part })?;
    let entry = body.part(part).ok_or(MeshError::Unplaceable { part })?;
    let volume = source
        .volume(entry.volume)
        .ok_or(MeshError::MissingVolume {
            part,
            volume: entry.volume,
        })?;
    Ok((pivot_at, entry.pivot, yaw, volume))
}

/// The eight corners of a volume's occupied range, in local coordinates.
fn volume_corners(volume: &Volume) -> [[i32; 3]; 8] {
    let hi = [
        volume.size[0].saturating_sub(1) as i32,
        volume.size[1].saturating_sub(1) as i32,
        volume.size[2].saturating_sub(1) as i32,
    ];
    [
        [0, 0, 0],
        [hi[0], 0, 0],
        [0, hi[1], 0],
        [0, 0, hi[2]],
        [hi[0], hi[1], 0],
        [hi[0], 0, hi[2]],
        [0, hi[1], hi[2]],
        [hi[0], hi[1], hi[2]],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VolumeMap;
    use mesocosm_core::{Attachment, PartId, Provenance, SpeciesId, VolumeRef};

    /// Volumes sized to exactly twice their part's half-extent, so the picture
    /// and the physics describe the same box.
    fn source() -> VolumeMap {
        let mut map = VolumeMap::new();
        map.insert(VolumeRef::from_tag(1), Volume::solid([4, 4, 4], 1));
        map.insert(VolumeRef::from_tag(2), Volume::solid([2, 2, 2], 7));
        map
    }

    const CORE_HALF: [i32; 3] = [2, 2, 2];
    const ARM_HALF: [i32; 3] = [1, 1, 1];

    fn body() -> BodyDocument {
        BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, CORE_HALF)
    }

    fn with_arm(offset: [i32; 3], yaw: Yaw) -> BodyDocument {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(2),
            10,
            ARM_HALF,
            Attachment {
                parent: PartId(0),
                offset,
                yaw,
            },
            Provenance::founding(),
        )
        .unwrap();
        body
    }

    #[test]
    fn a_lone_body_is_centred_on_its_pivot() {
        let flat = flatten(&body(), &source()).unwrap();
        assert_eq!(flat.volume.size, [4, 4, 4]);
        // A pivot is the centre, so the grid starts a half-extent below it.
        assert_eq!(flat.origin, [-2, -2, -2]);
        assert_eq!(flat.volume.solid_count(), 64);
        assert_eq!(flat.at([0, 0, 0]), 1, "the core covers the origin");
    }

    #[test]
    fn an_attached_part_lands_where_its_pivot_says() {
        let flat = flatten(&with_arm([3, 0, 0], Yaw::Zero), &source()).unwrap();
        // The arm's pivot sits at x=3, so its two voxels straddle it.
        assert_eq!(flat.at([2, 0, 0]), 7);
        assert_eq!(flat.at([3, 0, 0]), 7);
        assert_eq!(flat.at([0, 0, 0]), 1, "the core is still there");
    }

    #[test]
    fn negative_placements_shift_the_origin_rather_than_clipping() {
        let flat = flatten(&with_arm([-3, 0, 0], Yaw::Zero), &source()).unwrap();
        assert_eq!(flat.origin[0], -4);
        assert_eq!(
            flat.at([-4, 0, 0]),
            7,
            "nothing was clipped off the low side"
        );
        assert_eq!(flat.at([0, 0, 0]), 1);
    }

    /// The pivot's whole point: a turned part stays joined instead of swinging
    /// off its corner.
    #[test]
    fn yaw_turns_a_part_about_its_pivot() {
        let straight = flatten(&with_arm([3, 0, 0], Yaw::Zero), &source()).unwrap();
        let turned = flatten(&with_arm([3, 0, 0], Yaw::Quarter), &source()).unwrap();

        // Same body, same joint, so the same amount of matter in the same box.
        assert_eq!(straight.volume.solid_count(), turned.volume.solid_count());
        assert_eq!(straight.volume.size, turned.volume.size);
        assert_eq!(straight.origin, turned.origin);
    }

    #[test]
    fn every_solid_voxel_survives_when_parts_do_not_overlap() {
        let flat = flatten(&with_arm([3, 0, 0], Yaw::Zero), &source()).unwrap();
        assert_eq!(flat.volume.solid_count(), 64 + 8);
    }

    #[test]
    fn flattening_is_deterministic() {
        let build = || flatten(&with_arm([4, 1, -2], Yaw::Half), &source()).unwrap();
        assert_eq!(build(), build());
    }

    #[test]
    fn a_missing_volume_is_reported() {
        let mut body = body();
        body.attach(
            VolumeRef::from_tag(99),
            10,
            ARM_HALF,
            Attachment {
                parent: PartId(0),
                offset: [3, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
        assert!(matches!(
            flatten(&body, &source()),
            Err(MeshError::MissingVolume { .. })
        ));
    }
}
