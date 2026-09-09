// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Greedy meshing of one part's volume.
//!
//! Two reductions, in order. Hidden faces between solid voxels are culled,
//! then coplanar runs of the same material are merged into rectangles. A solid
//! box of any size becomes six quads.
//!
//! Meshing is per part, which is the rendering posture the body pipeline plan
//! chose: parts are rigid and transformed individually, so a part's mesh
//! depends only on its volume and can be built once and reused wherever that
//! part appears. Faces are therefore never merged across a joint, which is
//! correct rather than a shortcut. Merging across parts would weld a body into
//! one mesh and lose the ability to move a limb.

use serde::{Deserialize, Serialize};

use crate::volume::Volume;

/// One merged rectangle of voxel faces, in part-local voxel coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quad {
    /// Lowest corner of the rectangle, on the face plane.
    pub origin: [i32; 3],
    /// Which axis the face is perpendicular to: 0 = x, 1 = y, 2 = z.
    pub axis: u8,
    /// Facing along the axis (`true`) or against it (`false`).
    pub positive: bool,
    /// Extent along the two in-plane axes, in that axis order.
    pub size: [u32; 2],
    pub material: u8,
}

impl Quad {
    /// The two in-plane axes, in the order `size` uses.
    pub fn plane_axes(&self) -> [usize; 2] {
        match self.axis {
            0 => [1, 2],
            1 => [0, 2],
            _ => [0, 1],
        }
    }

    /// The rectangle's four corners, counter-clockwise seen from outside.
    pub fn corners(&self) -> [[i32; 3]; 4] {
        let [u, v] = self.plane_axes();
        let a = self.origin;
        let mut b = self.origin;
        let mut c = self.origin;
        let mut d = self.origin;
        b[u] += self.size[0] as i32;
        c[u] += self.size[0] as i32;
        c[v] += self.size[1] as i32;
        d[v] += self.size[1] as i32;
        if self.positive {
            [a, b, c, d]
        } else {
            [a, d, c, b]
        }
    }

    pub fn area(&self) -> u32 {
        self.size[0] * self.size[1]
    }
}

/// The geometry of one part, in its own voxel space.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartMesh {
    pub quads: Vec<Quad>,
}

impl PartMesh {
    pub fn is_empty(&self) -> bool {
        self.quads.is_empty()
    }

    pub fn len(&self) -> usize {
        self.quads.len()
    }

    /// Total voxel faces covered. Equals the culled face count whether or not
    /// merging ran, which is what makes merging safe to assert against.
    pub fn covered_area(&self) -> u32 {
        self.quads.iter().map(|q| q.area()).sum()
    }
}

/// Meshes a volume, culling hidden faces and merging coplanar runs.
///
/// Output order is fixed by the axis, direction, and slice loops, so the same
/// volume always produces byte-identical geometry.
pub fn mesh_volume(volume: &Volume) -> PartMesh {
    let mut quads = Vec::new();
    for axis in 0..3usize {
        for positive in [false, true] {
            mesh_axis(volume, axis, positive, &mut quads);
        }
    }
    PartMesh { quads }
}

/// Meshes without merging. Kept because it is the honest baseline the merged
/// output is compared against in tests.
pub fn mesh_volume_naive(volume: &Volume) -> PartMesh {
    let mut quads = Vec::new();
    for axis in 0..3usize {
        for positive in [false, true] {
            let (u, v) = plane_of(axis);
            for slice in 0..volume.size[axis] {
                for vv in 0..volume.size[v] {
                    for uu in 0..volume.size[u] {
                        let mut coord = [0u32; 3];
                        coord[axis] = slice;
                        coord[u] = uu;
                        coord[v] = vv;
                        let Some(material) = visible_face(volume, coord, axis, positive) else {
                            continue;
                        };
                        let mut origin = [coord[0] as i32, coord[1] as i32, coord[2] as i32];
                        if positive {
                            origin[axis] += 1;
                        }
                        quads.push(Quad {
                            origin,
                            axis: axis as u8,
                            positive,
                            size: [1, 1],
                            material,
                        });
                    }
                }
            }
        }
    }
    PartMesh { quads }
}

fn plane_of(axis: usize) -> (usize, usize) {
    match axis {
        0 => (1, 2),
        1 => (0, 2),
        _ => (0, 1),
    }
}

/// The material of a face if it is exposed, or `None` if it is hidden or the
/// voxel is empty.
fn visible_face(volume: &Volume, coord: [u32; 3], axis: usize, positive: bool) -> Option<u8> {
    let material = volume.get(coord[0], coord[1], coord[2]);
    if material == 0 {
        return None;
    }
    let mut neighbour = [coord[0] as i64, coord[1] as i64, coord[2] as i64];
    neighbour[axis] += if positive { 1 } else { -1 };
    if volume.get_signed(neighbour) == 0 {
        Some(material)
    } else {
        None
    }
}

fn mesh_axis(volume: &Volume, axis: usize, positive: bool, out: &mut Vec<Quad>) {
    let (u, v) = plane_of(axis);
    let width = volume.size[u] as usize;
    let height = volume.size[v] as usize;
    if width == 0 || height == 0 {
        return;
    }

    let mut mask = vec![0u8; width * height];

    for slice in 0..volume.size[axis] {
        // Build the visibility mask for this slice.
        for vv in 0..height {
            for uu in 0..width {
                let mut coord = [0u32; 3];
                coord[axis] = slice;
                coord[u] = uu as u32;
                coord[v] = vv as u32;
                mask[vv * width + uu] = visible_face(volume, coord, axis, positive).unwrap_or(0);
            }
        }

        // Merge rectangles out of the mask, consuming as we go.
        for vv in 0..height {
            let mut uu = 0;
            while uu < width {
                let material = mask[vv * width + uu];
                if material == 0 {
                    uu += 1;
                    continue;
                }

                // Extend along u while the material matches.
                let mut run = 1;
                while uu + run < width && mask[vv * width + uu + run] == material {
                    run += 1;
                }

                // Extend along v while the whole run matches.
                let mut span = 1;
                'grow: while vv + span < height {
                    for offset in 0..run {
                        if mask[(vv + span) * width + uu + offset] != material {
                            break 'grow;
                        }
                    }
                    span += 1;
                }

                for row in 0..span {
                    for offset in 0..run {
                        mask[(vv + row) * width + uu + offset] = 0;
                    }
                }

                let mut origin = [0i32; 3];
                origin[axis] = slice as i32 + if positive { 1 } else { 0 };
                origin[u] = uu as i32;
                origin[v] = vv as i32;

                out.push(Quad {
                    origin,
                    axis: axis as u8,
                    positive,
                    size: [run as u32, span as u32],
                    material,
                });

                uu += run;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_voxel_has_six_faces() {
        let mesh = mesh_volume(&Volume::solid([1, 1, 1], 1));
        assert_eq!(mesh.len(), 6);
        assert_eq!(mesh.covered_area(), 6);
    }

    #[test]
    fn a_solid_box_merges_to_six_quads() {
        let mesh = mesh_volume(&Volume::solid([4, 5, 6], 3));
        assert_eq!(mesh.len(), 6, "each face of a box is one rectangle");
        // Two faces per axis pair: 4*5, 4*6, 5*6, doubled.
        assert_eq!(mesh.covered_area(), 2 * (4 * 5 + 4 * 6 + 5 * 6));
    }

    #[test]
    fn an_empty_volume_meshes_to_nothing() {
        assert!(mesh_volume(&Volume::empty([4, 4, 4])).is_empty());
    }

    #[test]
    fn interior_faces_are_culled() {
        // A 3x3x3 block has 27 voxels but only the shell is visible.
        let mesh = mesh_volume(&Volume::solid([3, 3, 3], 1));
        assert_eq!(mesh.covered_area(), 6 * 9);
    }

    #[test]
    fn merging_never_changes_covered_area() {
        let mut volume = Volume::solid([6, 6, 6], 1);
        volume.set(2, 2, 2, 0);
        volume.set(0, 0, 0, 0);
        volume.set(5, 3, 1, 0);

        let merged = mesh_volume(&volume);
        let naive = mesh_volume_naive(&volume);

        assert_eq!(merged.covered_area(), naive.covered_area());
        assert!(
            merged.len() < naive.len(),
            "merging must reduce quad count: {} vs {}",
            merged.len(),
            naive.len()
        );
    }

    #[test]
    fn different_materials_do_not_merge() {
        let mut volume = Volume::solid([2, 1, 1], 1);
        volume.set(1, 0, 0, 2);
        let mesh = mesh_volume(&volume);
        // The +y face spans two voxels of different materials, so it cannot
        // merge into one quad.
        let top: Vec<_> = mesh
            .quads
            .iter()
            .filter(|q| q.axis == 1 && q.positive)
            .collect();
        assert_eq!(top.len(), 2);
        assert_ne!(top[0].material, top[1].material);
    }

    #[test]
    fn meshing_is_deterministic() {
        let mut volume = Volume::solid([5, 5, 5], 4);
        volume.set(1, 1, 1, 0);
        volume.set(3, 0, 2, 9);
        assert_eq!(mesh_volume(&volume), mesh_volume(&volume));
    }

    #[test]
    fn positive_faces_sit_on_the_far_boundary() {
        let mesh = mesh_volume(&Volume::solid([2, 1, 1], 1));
        let plus_x = mesh
            .quads
            .iter()
            .find(|q| q.axis == 0 && q.positive)
            .expect("a +x face exists");
        assert_eq!(
            plus_x.origin[0], 2,
            "the +x face is at the volume's far edge"
        );
        let minus_x = mesh
            .quads
            .iter()
            .find(|q| q.axis == 0 && !q.positive)
            .expect("a -x face exists");
        assert_eq!(minus_x.origin[0], 0);
    }

    #[test]
    fn corners_bound_the_quad() {
        let quad = Quad {
            origin: [1, 2, 3],
            axis: 1,
            positive: true,
            size: [2, 4],
            material: 1,
        };
        let corners = quad.corners();
        let xs: Vec<i32> = corners.iter().map(|c| c[0]).collect();
        let zs: Vec<i32> = corners.iter().map(|c| c[2]).collect();
        assert_eq!(xs.iter().min(), Some(&1));
        assert_eq!(xs.iter().max(), Some(&3));
        assert_eq!(zs.iter().min(), Some(&3));
        assert_eq!(zs.iter().max(), Some(&7));
        assert!(
            corners.iter().all(|c| c[1] == 2),
            "all corners lie on the plane"
        );
    }
}
