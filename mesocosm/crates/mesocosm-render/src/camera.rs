// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The camera.
//!
//! Orthographic by default, because voxel bodies read best without
//! perspective foreshortening and because an orthographic frame makes a
//! rendered body's size directly comparable between frames, which the tests
//! rely on.
//!
//! Camera choice is presentation. The plan leaves 2.5D versus 3D open, and
//! nothing here forecloses it: a fixed yaw and pitch give the classic
//! three-quarter voxel view, and the same code renders a free camera later.

use glam::{Mat4, Vec3};

/// An orthographic view of the body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// Point the camera looks at, in voxel units.
    pub target: [f32; 3],
    /// Half-height of the visible region, in voxel units. Larger sees more.
    pub extent: f32,
    /// Rotation about the vertical axis, in radians.
    pub yaw: f32,
    /// Tilt above the horizon, in radians.
    pub pitch: f32,
    /// Frame aspect, width over height.
    pub aspect: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            target: [0.0, 0.0, 0.0],
            extent: 16.0,
            // The three-quarter view voxel art is usually drawn in.
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: 0.6154797,
            aspect: 1.0,
        }
    }
}

impl Camera {
    /// Frames a bounding box with a little margin, so a body fills the view
    /// without touching the edges.
    pub fn framing(min: [i32; 3], max: [i32; 3], aspect: f32) -> Self {
        let centre = [
            (min[0] + max[0]) as f32 / 2.0,
            (min[1] + max[1]) as f32 / 2.0,
            (min[2] + max[2]) as f32 / 2.0,
        ];
        let span = (0..3)
            .map(|axis| (max[axis] - min[axis]) as f32)
            .fold(1.0f32, f32::max);
        Self {
            target: centre,
            // Room for the diagonal, since the view is rotated.
            extent: span * 0.9 + 2.0,
            aspect,
            ..Self::default()
        }
    }

    /// Frames a fixed-size region around a point.
    ///
    /// Distinct from [`Self::framing`], which sizes itself to a body. A world
    /// view must hold its scale as the critter moves, or motion reads as the
    /// camera zooming rather than the critter travelling.
    pub fn following(centre: [i32; 3], extent: f32, aspect: f32) -> Self {
        Self {
            target: [centre[0] as f32, centre[1] as f32, centre[2] as f32],
            extent,
            aspect,
            ..Self::default()
        }
    }

    pub fn view_proj(&self) -> Mat4 {
        let distance = self.extent * 4.0 + 32.0;
        let direction = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        let target = Vec3::from_array(self.target);
        let eye = target + direction * distance;

        let view = Mat4::look_at_rh(eye, target, Vec3::Y);
        let half_h = self.extent;
        let half_w = self.extent * self.aspect;
        // Depth range covers the body from either side of the target.
        let projection = Mat4::orthographic_rh(
            -half_w,
            half_w,
            -half_h,
            half_h,
            0.1,
            distance * 2.0 + self.extent * 4.0,
        );
        projection * view
    }

    pub fn view_proj_array(&self) -> [[f32; 4]; 4] {
        self.view_proj().to_cols_array_2d()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec4;

    fn projects(camera: &Camera, point: [f32; 3]) -> Vec3 {
        let clip = camera.view_proj() * Vec4::new(point[0], point[1], point[2], 1.0);
        Vec3::new(clip.x / clip.w, clip.y / clip.w, clip.z / clip.w)
    }

    #[test]
    fn the_target_lands_in_the_middle_of_the_frame() {
        let camera = Camera {
            target: [3.0, 1.0, -2.0],
            ..Camera::default()
        };
        let ndc = projects(&camera, camera.target);
        assert!(ndc.x.abs() < 1e-4, "x centred, got {}", ndc.x);
        assert!(ndc.y.abs() < 1e-4, "y centred, got {}", ndc.y);
    }

    #[test]
    fn a_framed_box_fits_inside_the_view() {
        let min = [-4, 0, -4];
        let max = [12, 6, 5];
        let camera = Camera::framing(min, max, 1.0);
        for x in [min[0], max[0]] {
            for y in [min[1], max[1]] {
                for z in [min[2], max[2]] {
                    let ndc = projects(&camera, [x as f32, y as f32, z as f32]);
                    assert!(
                        ndc.x.abs() <= 1.0 && ndc.y.abs() <= 1.0,
                        "corner ({x},{y},{z}) fell outside the frame at {ndc:?}"
                    );
                    assert!((0.0..=1.0).contains(&ndc.z), "corner outside depth range");
                }
            }
        }
    }

    #[test]
    fn higher_points_render_higher() {
        let camera = Camera::default();
        let low = projects(&camera, [0.0, 0.0, 0.0]);
        let high = projects(&camera, [0.0, 5.0, 0.0]);
        assert!(high.y > low.y);
    }

    #[test]
    fn a_wider_extent_shrinks_what_is_drawn() {
        let near = Camera {
            extent: 8.0,
            ..Camera::default()
        };
        let far = Camera {
            extent: 32.0,
            ..Camera::default()
        };
        let probe = [6.0, 0.0, 0.0];
        assert!(projects(&near, probe).x.abs() > projects(&far, probe).x.abs());
    }
}
