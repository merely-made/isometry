// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Presentation camera orientations and the legacy section's cull window.
//!
//! Side, across and the default oblique view retain their measured Q9
//! geometry. CP1 adds four cardinal terrarium views at a shallow pitch.
//! `view.rs` supplies configurable pitch and one matching terrain/raster
//! transform; `terrarium.rs` clips a fixed habitat volume and applies the
//! interior information policy. Rotation never reaches a world intent.

use mesocosm_lens::SlabWall;

/// **Slice thickness, and deliberately not scaled with the enclosure.** A
/// section shows a cut of fixed depth; widening it to keep the same fraction of
/// a bigger world would only stack more bodies into the same pixels, which is
/// the pile the S1 finding started from.
pub const SLAB_DEPTH: f32 = 16.0;

/// World up. Every mode keeps it: a section whose vertical is not the world's
/// vertical stops presenting the axis the section exists to present.
const UP: [f32; 3] = [0.0, 1.0, 0.0];

/// How far [`CameraMode::Oblique`] turns off the section, in degrees, on each
/// of the two free rotations.
///
/// **Both, and that is the point.** A yaw alone foreshortens the body chain
/// along the screen's horizontal and leaves it there — a weaker `across`, not
/// a diagonal. The pitch is what gives the chain a vertical component, so
/// depth reads as a short diagonal rather than as a short line. Twenty sits in
/// the middle of the 15-25 the ruling asked for.
pub const OBLIQUE_DEGREES: f32 = 20.0;

/// The shallow fixed pitch used by the CP1 terrarium camera prototype.
pub const TERRARIUM_DEGREES: f32 = 12.0;

/// Which way the section looks. Presentation, so it is a host flag and never
/// a world fact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CameraMode {
    /// Straight down `-z`, bodies drawn end-on. The control arm of the Q9
    /// measurement, and bit-for-bit what the tree drew before this module.
    Side,
    /// The section turned a quarter about the world's vertical, looking down
    /// `-x`. Bodies chain along `+z`, so they now chain across the screen at
    /// full length — the most part budget a single frame can show.
    Across,
    /// **The shipped section.** Tilted off both ways by [`OBLIQUE_DEGREES`]:
    /// depth reads as a short diagonal and the vertical structure survives, at
    /// the cost of showing the chain at under half its across length. Ruled
    /// the default on 2026-09-04 — it is the only arm that shows a body's part
    /// budget *and* keeps the section.
    #[default]
    Oblique,
    /// Terrarium views keep a little vertical depth while turning around the
    /// body chain. East is the prototype's first view, looking across `-x`.
    TerrariumEast,
    TerrariumSouth,
    TerrariumWest,
    TerrariumNorth,
}

impl CameraMode {
    /// Every mode, in the order the contact sheet composes them.
    pub const ALL: [CameraMode; 7] = [
        Self::Side,
        Self::Across,
        Self::Oblique,
        Self::TerrariumEast,
        Self::TerrariumSouth,
        Self::TerrariumWest,
        Self::TerrariumNorth,
    ];

    pub const fn is_terrarium(self) -> bool {
        matches!(
            self,
            Self::TerrariumEast | Self::TerrariumSouth | Self::TerrariumWest | Self::TerrariumNorth
        )
    }

    /// Turns one quarter around the terrarium. Legacy modes remain unchanged.
    pub const fn quarter_turn(self, backwards: bool) -> Self {
        use CameraMode::*;
        match (self, backwards) {
            (TerrariumEast, false) => TerrariumSouth,
            (TerrariumSouth, false) => TerrariumWest,
            (TerrariumWest, false) => TerrariumNorth,
            (TerrariumNorth, false) => TerrariumEast,
            (TerrariumEast, true) => TerrariumNorth,
            (TerrariumNorth, true) => TerrariumWest,
            (TerrariumWest, true) => TerrariumSouth,
            (TerrariumSouth, true) => TerrariumEast,
            (mode, _) => mode,
        }
    }

    /// What `--camera` accepts, and what a receipt and a scenario read back.
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|mode| mode.name() == name.trim().to_ascii_lowercase())
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Side => "side",
            Self::Across => "across",
            Self::Oblique => "oblique",
            Self::TerrariumEast => "terrarium-east",
            Self::TerrariumSouth => "terrarium-south",
            Self::TerrariumWest => "terrarium-west",
            Self::TerrariumNorth => "terrarium-north",
        }
    }

    /// Which way the camera looks, unnormalized where that is exact anyway.
    pub fn forward(self) -> [f32; 3] {
        let (yaw, pitch) = (OBLIQUE_DEGREES.to_radians(), OBLIQUE_DEGREES.to_radians());
        match self {
            Self::Side => [0.0, 0.0, -1.0],
            Self::Across => [-1.0, 0.0, 0.0],
            // Yawed off -z, then pitched down. Both by the same angle, so
            // the tilt reads as one turn of the section rather than two.
            Self::Oblique => [
                -yaw.sin() * pitch.cos(),
                -pitch.sin(),
                -yaw.cos() * pitch.cos(),
            ],
            Self::TerrariumEast => terrarium_forward([-1.0, 0.0, 0.0]),
            Self::TerrariumSouth => terrarium_forward([0.0, 0.0, -1.0]),
            Self::TerrariumWest => terrarium_forward([1.0, 0.0, 0.0]),
            Self::TerrariumNorth => terrarium_forward([0.0, 0.0, 1.0]),
        }
    }

    /// The orthonormal frame the tracer will build from
    /// [`Self::forward`] and world up: right, up, forward, in that order.
    ///
    /// Derived by the **same** construction
    /// [`mesocosm_lens::TraceCamera::orthographic_slab`] uses, because the
    /// cull window and the camera disagreeing about where the slab is would
    /// show critters that are not drawn and drop critters that are.
    pub fn basis(self) -> [[f32; 3]; 3] {
        let forward = normalize(self.forward()).unwrap_or([0.0, 0.0, -1.0]);
        let right = normalize(cross(forward, UP)).unwrap_or([1.0, 0.0, 0.0]);
        let up = normalize(cross(right, forward)).unwrap_or(UP);
        [right, up, forward]
    }

    /// How far above and below its centre this camera actually frames, in
    /// world voxels.
    ///
    /// `half_height` for the two axis-aligned modes, and more than that for
    /// [`Self::Oblique`], whose slab depth leans into the vertical — and
    /// which leans it further than the depth alone, because its rays begin on
    /// the world-vertical front wall rather than on the tilted near plane.
    /// The
    /// follow centre's bedrock clamp reads this rather than `half_height` so
    /// a tilted frame does not dip below the world's floor — the same
    /// companion rule Mark ruled with the half-height on 2026-08-29, told the
    /// truth about a camera that is not axis-aligned.
    pub fn vertical_half(self, half_height: f32) -> f32 {
        let [right, up, forward] = self.basis();
        right[1].abs() * half_height * WIDEST_ASPECT
            + up[1].abs() * half_height
            + forward[1].abs() * self.slab_reach(half_height, WIDEST_ASPECT)
    }

    /// How far along this camera's forward the slab reaches from its centre.
    ///
    /// `SLAB_DEPTH / 2` for a level section, and read straight off
    /// [`SlabWall`] rather than restated here, because the tracer seeds its
    /// rays on that wall and a cull box that disagreed with it would drop
    /// bodies the frame draws. A tilted section reaches further: its rays
    /// begin on an upright wall rather than on the tilted near plane, so
    /// where they enter the slab depends on how high up the frame they sit.
    ///
    /// The level modes take [`SlabWall`]'s own level branch, which returns
    /// exactly the half depth this replaces.
    pub fn slab_reach(self, half_height: f32, aspect: f32) -> f32 {
        SlabWall::new(self.forward(), UP, half_height, aspect, SLAB_DEPTH)
            .map_or(SLAB_DEPTH * 0.5, |wall| wall.reach)
    }
}

/// How a section frames the world: how much of it, and which way.
///
/// One argument rather than two because they are one decision — both are
/// presentation, both are host flags, and a `Section` that took them
/// separately was the eighth argument clippy declines to keep counting.
#[derive(Clone, Copy, Debug, Default)]
pub struct Framing {
    /// Half the height of the orthographic slab, in voxels. Zero means the
    /// ruled default.
    pub half_height: f32,
    pub mode: CameraMode,
}

impl Framing {
    pub fn new(half_height: f32, mode: CameraMode) -> Self {
        Self { half_height, mode }
    }
}

/// The aspect the clamp assumes when it is asked before a surface exists.
///
/// Only the horizontal axis is scaled by aspect, and in all three modes that
/// axis is level (`right[1]` is zero), so this number never actually reaches
/// [`CameraMode::vertical_half`]'s result. It is written down rather than
/// left implicit because a future mode that rolled the camera would make it
/// matter, and a silently wrong clamp is the failure this file exists to
/// avoid.
const WIDEST_ASPECT: f32 = 1.0;

/// The world box the section's slab shows: the camera's own oriented box,
/// in voxels around its centre.
///
/// **Oriented, not axis-aligned** (DC4). The half extents are along the
/// camera's right, up and forward — not along world x, y and z — so the same
/// numbers describe the same slab whichever way the section is turned. For
/// [`CameraMode::Side`] the basis is the identity up to sign and this is
/// exactly the box the tree culled against before.
#[derive(Clone, Copy, Debug)]
pub struct SlabWindow {
    pub centre: [f32; 3],
    /// Right, up and forward, orthonormal — [`CameraMode::basis`]'s output.
    pub axes: [[f32; 3]; 3],
    /// Half extents along those three axes, in the same order.
    pub half: [f32; 3],
}

impl SlabWindow {
    /// The window a camera in `mode` frames at `centre`.
    pub fn new(mode: CameraMode, centre: [f32; 3], half_height: f32, aspect: f32) -> Self {
        Self {
            centre,
            axes: mode.basis(),
            half: [
                half_height * aspect,
                half_height,
                mode.slab_reach(half_height, aspect),
            ],
        }
    }

    /// Whether a voxel position falls inside the window. Position alone, not
    /// the body's extent: a body straddling the cut plane is drawn whole and
    /// the tracer's own ray interval does the trimming.
    pub fn holds(&self, at: [i32; 3]) -> bool {
        let offset = [
            at[0] as f32 - self.centre[0],
            at[1] as f32 - self.centre[1],
            at[2] as f32 - self.centre[2],
        ];
        (0..3).all(|axis| dot(offset, self.axes[axis]).abs() <= self.half[axis])
    }
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn normalize(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = dot(value, value).sqrt();
    (length > 1e-6).then(|| [value[0] / length, value[1] / length, value[2] / length])
}

fn terrarium_forward(horizontal: [f32; 3]) -> [f32; 3] {
    let pitch = TERRARIUM_DEGREES.to_radians();
    [
        horizontal[0] * pitch.cos(),
        -pitch.sin(),
        horizontal[2] * pitch.cos(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mode names itself, and every name round-trips. The receipt and a
    /// scenario's `assert snap camera` both read this string.
    #[test]
    fn every_mode_round_trips_through_its_name() {
        for mode in CameraMode::ALL {
            assert_eq!(CameraMode::parse(mode.name()), Some(mode));
            assert_eq!(CameraMode::parse(&mode.name().to_uppercase()), Some(mode));
        }
        assert_eq!(CameraMode::default(), CameraMode::Oblique);
        assert_eq!(CameraMode::parse("isometric"), None);
        assert!(CameraMode::TerrariumEast.is_terrarium());
        assert!(!CameraMode::Oblique.is_terrarium());
    }

    #[test]
    fn terrarium_turns_four_times_back_to_the_start() {
        let mut mode = CameraMode::TerrariumEast;
        for _ in 0..4 {
            mode = mode.quarter_turn(false);
        }
        assert_eq!(mode, CameraMode::TerrariumEast);
        assert_eq!(
            CameraMode::TerrariumEast.quarter_turn(true),
            CameraMode::TerrariumNorth
        );
        assert_eq!(CameraMode::Side.quarter_turn(false), CameraMode::Side);
    }

    #[test]
    fn terrarium_views_keep_world_up_and_share_a_center() {
        let centre = [13.0, 29.0, -7.0];
        for mode in [
            CameraMode::TerrariumEast,
            CameraMode::TerrariumSouth,
            CameraMode::TerrariumWest,
            CameraMode::TerrariumNorth,
        ] {
            let forward = mode.forward();
            assert!((forward[1] + TERRARIUM_DEGREES.to_radians().sin()).abs() < 1e-5);
            let [right, up, basis_forward] = mode.basis();
            assert!(dot(up, UP) > 0.97, "{} lost world up", mode.name());
            assert!(
                dot(forward, basis_forward) > 0.999,
                "{} changed forward",
                mode.name()
            );
            assert!(SlabWindow::new(mode, centre, 28.0, 1.0).holds([13, 29, -7]));
            assert!(dot(right, right) > 0.99);
        }
        let first = CameraMode::TerrariumEast.forward();
        assert!(
            first[0] < -0.9 && first[2].abs() < 1e-5,
            "east starts across -x"
        );
    }

    /// The frame the cull window tests against has to be the frame the tracer
    /// marches, or the roster shows the wrong critters.
    #[test]
    fn every_basis_is_orthonormal() {
        for mode in CameraMode::ALL {
            let axes = mode.basis();
            for axis in axes {
                assert!(
                    (dot(axis, axis) - 1.0).abs() < 1e-5,
                    "{} has a non-unit axis {axis:?}",
                    mode.name()
                );
            }
            for (a, b) in [(0, 1), (1, 2), (0, 2)] {
                assert!(
                    dot(axes[a], axes[b]).abs() < 1e-5,
                    "{}: axes {a} and {b} are not perpendicular",
                    mode.name()
                );
            }
        }
    }

    /// **The measurement Q9 exists for**, as an assertion rather than a
    /// screenshot: how much of a body's `+z` chain each mode puts on screen.
    ///
    /// `side` projects it to nothing at all, which is the finding on record —
    /// the chain runs straight into the camera. `across` gives it the whole
    /// screen-horizontal. `oblique` gives it a diagonal a little under half
    /// as long, which is the stated cost of keeping the section's read.
    #[test]
    fn the_body_chain_projects_to_nothing_side_on_and_to_a_full_span_across() {
        let chain = [0.0, 0.0, 1.0];
        let screen = |mode: CameraMode| {
            let [right, up, _] = mode.basis();
            (dot(chain, right), dot(chain, up))
        };

        let (x, y) = screen(CameraMode::Side);
        assert!(x.abs() < 1e-5 && y.abs() < 1e-5, "side draws it end-on");

        let (x, y) = screen(CameraMode::Across);
        assert!((x.abs() - 1.0).abs() < 1e-5, "across draws it full length");
        assert!(y.abs() < 1e-5, "and level");

        let (x, y) = screen(CameraMode::Oblique);
        assert!(x.abs() > 0.3 && y.abs() > 0.3, "oblique draws a diagonal");
        let length = (x * x + y * y).sqrt();
        assert!(
            (0.4..0.55).contains(&length),
            "a short one: {length} of the across span"
        );
    }

    /// The bedrock clamp's floor: the two level modes frame exactly their
    /// half-height, and the tilted one frames more because its depth leans
    /// into the vertical.
    #[test]
    fn a_tilted_camera_declares_the_extra_height_it_frames() {
        assert_eq!(CameraMode::Side.vertical_half(28.0), 28.0);
        assert_eq!(CameraMode::Across.vertical_half(28.0), 28.0);
        let tilted = CameraMode::Oblique.vertical_half(28.0);
        assert!(
            (32.5..33.5).contains(&tilted),
            "28 cos20 plus the seeded slab's reach leaning into y: {tilted}"
        );
    }

    /// The two level modes take the wall's level branch, so the number the
    /// cull box and the bedrock clamp read is the *same* half depth they read
    /// before the wall existed — not a value that happens to round to it.
    #[test]
    fn a_level_camera_reaches_exactly_the_half_depth_it_always_did() {
        for mode in [CameraMode::Side, CameraMode::Across] {
            for half in [20.0, 28.0, 48.0] {
                for aspect in [1.0, 1.78] {
                    assert_eq!(
                        mode.slab_reach(half, aspect),
                        SLAB_DEPTH * 0.5,
                        "{} moved its slab reach",
                        mode.name()
                    );
                }
            }
        }
        assert!(
            CameraMode::Oblique.slab_reach(28.0, 1.78) > SLAB_DEPTH * 0.5,
            "a tilted section reaches past its half depth"
        );
    }

    /// The window is the camera's box, so `across` keeps what it can see and
    /// drops what it cannot — the exact inverse of `side` on the same world.
    #[test]
    fn the_cull_window_turns_with_the_camera() {
        let centre = [0.0, 30.0, 0.0];
        let side = SlabWindow::new(CameraMode::Side, centre, 28.0, 1.78);
        let across = SlabWindow::new(CameraMode::Across, centre, 28.0, 1.78);

        // Far along x, on the cut plane in z: in shot side-on, behind the
        // camera's own slab across.
        let along_x = [40, 30, 0];
        assert!(side.holds(along_x));
        assert!(!across.holds(along_x));

        // And the other way about.
        let along_z = [0, 30, 40];
        assert!(!side.holds(along_z));
        assert!(across.holds(along_z));

        // Whoever is being followed is in shot under every mode, which is
        // what makes the three captures comparable at all.
        for mode in CameraMode::ALL {
            let window = SlabWindow::new(mode, centre, 28.0, 1.78);
            assert!(window.holds([0, 30, 0]), "{} lost the centre", mode.name());
        }
    }

    /// `side` is the control arm and has to stay the shipped framing: the
    /// generalized window must agree with the axis-aligned box the tree
    /// culled against before this module existed.
    #[test]
    fn the_side_window_is_the_axis_aligned_box_it_replaces() {
        let (centre, half_height, aspect) = ([3.0, 30.0, -5.0], 28.0, 1.78);
        let window = SlabWindow::new(CameraMode::Side, centre, half_height, aspect);
        let old = [half_height * aspect, half_height, SLAB_DEPTH * 0.5];
        for at in [
            [3, 30, -5],
            [52, 30, -5],
            [53, 30, -5],
            [3, 57, -5],
            [3, 59, -5],
            [3, 30, 2],
            [3, 30, 4],
        ] {
            let axis_aligned =
                (0..3).all(|axis| (at[axis] as f32 - centre[axis]).abs() <= old[axis]);
            assert_eq!(
                window.holds(at),
                axis_aligned,
                "the side window moved at {at:?}"
            );
        }
    }
}
