// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a caller hands the tracer and what it gets back: camera, frame
//! input, diagnostics, capture, errors.

use bytemuck::{Pod, Zeroable};
use modulus::BrickMap;

use super::LeasedAtlas;
use crate::{CritterPose, Flight, Grade, TerrainAppearance};

const PERSPECTIVE: u32 = 0;
const ORTHOGRAPHIC: u32 = 1;
const LEGACY_ASPECT: f32 = 16.0 / 9.0;

/// The rays a camera contributes to the brick traversal.
///
/// Camera policy stays with the vessel. The tracer needs only an origin
/// plane, a forward direction, and the horizontal and vertical ray spans.
/// Perspective and orthographic projections therefore share the exact same
/// map bindings and DDA implementation.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, serde::Serialize, serde::Deserialize)]
pub struct TraceCamera {
    origin: [f32; 3],
    projection: u32,
    forward: [f32; 3],
    far: f32,
    right: [f32; 3],
    _right_pad: f32,
    up: [f32; 3],
    _up_pad: f32,
    /// [`SlabWall::seed`] with a pad, read only by the orthographic branch.
    wall: [f32; 4],
}

/// Where an orthographic slab's rays begin: its **world-vertical front wall**.
///
/// A slab's near plane is perpendicular to the view. Level, that plane is
/// already vertical and is the section's glass wall. Tilt the view and the
/// plane tilts with it, and the plane then cuts the terrain on a slope: rays
/// along the bottom of the frame begin below the surface, inside solid ground,
/// where the DDA has no face to report and hands back its seeded `+y` normal.
/// The slope is drawn as a lit top face, and a face parallel to the screen
/// under a camera that is not is exactly what reads as broken perspective.
///
/// So the wall is kept vertical in the world whatever the camera does: a plane
/// with a **horizontal** normal, standing at the slab's near depth, that every
/// ray is slid along its own direction onto before it marches. What the ray
/// meets there is a vertical section through the terrain under any camera, and
/// the region of the wall that stands inside solid ground is the section's cut
/// — drawn as the wall, never as a face.
///
/// # Level sections are untouched by construction
///
/// A view with no vertical component leans nowhere, so the horizontal normal
/// *is* the forward, the near plane *is* the wall, and [`Self::new`] returns
/// exactly zero advance and exactly the depth it was handed. The arithmetic is
/// skipped rather than merely cancelling, so no rounding can reach a level
/// frame and every capture the tree holds of one is unchanged to the byte.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlabWall {
    /// How far along forward a ray moves to reach the wall, as
    /// `seed[0] * ndc.x + seed[1] * ndc.y + seed[2]`. Negative moves back
    /// toward the viewer.
    pub seed: [f32; 3],
    /// The ray interval that crosses the full slab depth once the near and
    /// far cuts are both vertical. A tilted view walks further than `depth`
    /// to cross the same horizontal thickness.
    pub far: f32,
    /// Half the extent along the camera's forward that the seeded slab
    /// actually reaches, measured from the slab centre. Conservative — it
    /// bounds the slab rather than tracing its corners — and exactly
    /// `depth / 2` for a level view, which is the number it replaces.
    pub reach: f32,
}

impl SlabWall {
    /// The wall a slab of `depth` looking down `forward` stands on, where
    /// `vertical` is the world's up.
    pub fn new(
        forward: [f32; 3],
        vertical: [f32; 3],
        half_height: f32,
        aspect: f32,
        depth: f32,
    ) -> Option<Self> {
        let forward = normalize(forward)?;
        let vertical = normalize(vertical)?;
        let lean = dot(forward, vertical);
        if lean == 0.0 {
            // Level: the near plane already is the wall.
            return Some(Self {
                seed: [0.0; 3],
                far: depth,
                reach: depth * 0.5,
            });
        }
        let normal = normalize(sub(forward, scale(vertical, lean)))?;
        let level = dot(forward, normal);
        // Straight down there is no horizontal normal to stand a wall on, and
        // no section either. The caller keeps its untilted slab.
        if level <= 1e-3 {
            return None;
        }
        let right = normalize(cross(forward, vertical))?;
        let up = normalize(cross(right, forward))?;
        let seed = [
            -dot(scale(right, half_height * aspect), normal) / level,
            -dot(scale(up, half_height), normal) / level,
            depth * 0.5 * (level - 1.0) / level,
        ];
        let far = depth / level;
        let spread = seed[0].abs() + seed[1].abs() + seed[2].abs();
        Some(Self {
            seed,
            far,
            reach: spread + far - depth * 0.5,
        })
    }
}

impl TraceCamera {
    /// A rectilinear perspective camera aimed at `target`.
    pub fn perspective(
        eye: [f32; 3],
        target: [f32; 3],
        up: [f32; 3],
        vertical_fov: f32,
        aspect: f32,
        far: f32,
    ) -> Option<Self> {
        if !(vertical_fov > 0.0 && vertical_fov < std::f32::consts::PI && aspect > 0.0 && far > 0.0)
        {
            return None;
        }
        let forward = normalize(sub(target, eye))?;
        let right_unit = normalize(cross(forward, up))?;
        let up_unit = normalize(cross(right_unit, forward))?;
        let half_height = (vertical_fov * 0.5).tan();
        Some(Self {
            origin: eye,
            projection: PERSPECTIVE,
            forward,
            far,
            right: scale(right_unit, half_height * aspect),
            _right_pad: 0.0,
            up: scale(up_unit, half_height),
            _up_pad: 0.0,
            wall: [0.0; 4],
        })
    }

    /// An orthographic section that cuts `depth` voxels of world between two
    /// **world-vertical** walls.
    ///
    /// `centre` is the middle of the retained slab rather than the eye. This
    /// keeps the near and far cut planes symmetric when the section is moved.
    ///
    /// The cuts stand upright whichever way the section looks: see
    /// [`SlabWall`], which is where the ray interval and the seed advance
    /// come from, and which reduces to the plain near plane and exactly
    /// `depth` for a level view.
    pub fn orthographic_slab(
        centre: [f32; 3],
        forward: [f32; 3],
        up: [f32; 3],
        half_height: f32,
        aspect: f32,
        depth: f32,
    ) -> Option<Self> {
        if !(half_height > 0.0 && aspect > 0.0 && depth > 0.0) {
            return None;
        }
        let forward = normalize(forward)?;
        let right_unit = normalize(cross(forward, up))?;
        let up_unit = normalize(cross(right_unit, forward))?;
        // A view with no wall to stand on — straight down world up — keeps
        // the plain near plane rather than losing its section.
        let wall = SlabWall::new(forward, up, half_height, aspect, depth).unwrap_or(SlabWall {
            seed: [0.0; 3],
            far: depth,
            reach: depth * 0.5,
        });
        Some(Self {
            origin: sub(centre, scale(forward, depth * 0.5)),
            projection: ORTHOGRAPHIC,
            forward,
            far: wall.far,
            right: scale(right_unit, half_height * aspect),
            _right_pad: 0.0,
            up: scale(up_unit, half_height),
            _up_pad: 0.0,
            wall: [wall.seed[0], wall.seed[1], wall.seed[2], 0.0],
        })
    }

    /// The same camera with its front wall taken away: the tilted near plane
    /// the tracer seeded rays on before [`SlabWall`], and the fault that
    /// motivated it. The negative control for the wall receipts.
    #[cfg(test)]
    pub(crate) fn without_front_wall(mut self, depth: f32) -> Self {
        self.wall = [0.0; 4];
        self.far = depth;
        self
    }

    fn from_flight(flight: &Flight) -> Self {
        let forward = [
            flight.yaw.sin() * flight.pitch.cos(),
            flight.pitch.sin(),
            flight.yaw.cos() * flight.pitch.cos(),
        ];
        let target = add(flight.eye, forward);
        let vertical_fov = 2.0 * ((flight.fov * 0.5).tan() / LEGACY_ASPECT).atan();
        Self::perspective(
            flight.eye,
            target,
            [0.0, 1.0, 0.0],
            vertical_fov,
            LEGACY_ASPECT,
            flight.far,
        )
        .expect("Flight carries a valid camera")
    }

    #[cfg(test)]
    pub(crate) fn ray_at(self, ndc: [f32; 2]) -> ([f32; 3], [f32; 3]) {
        if self.projection == ORTHOGRAPHIC {
            // The shader's `camera_ray`, in the same order: the near-plane
            // point, then the slide along forward onto the front wall.
            let advance = self.wall[0] * ndc[0] + self.wall[1] * ndc[1] + self.wall[2];
            (
                add(
                    add(
                        self.origin,
                        add(scale(self.right, ndc[0]), scale(self.up, ndc[1])),
                    ),
                    scale(self.forward, advance),
                ),
                self.forward,
            )
        } else {
            (
                self.origin,
                normalize(add(
                    self.forward,
                    add(scale(self.right, ndc[0]), scale(self.up, ndc[1])),
                ))
                .expect("camera rays are nonzero"),
            )
        }
    }
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(value: [f32; 3], amount: f32) -> [f32; 3] {
    [value[0] * amount, value[1] * amount, value[2] * amount]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(value: [f32; 3]) -> Option<[f32; 3]> {
    let length = (value[0] * value[0] + value[1] * value[1] + value[2] * value[2]).sqrt();
    (length > 1e-6).then(|| scale(value, 1.0 / length))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct BrickRevision(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BrickChange<'a> {
    #[default]
    Full,
    Slots(&'a [u32]),
}

#[derive(Clone, Copy)]
pub struct BrickFrameInput<'a> {
    pub map: &'a BrickMap,
    pub revision: BrickRevision,
    pub change: BrickChange<'a>,
    pub camera: TraceCamera,
    pub grade: &'a Grade,
    /// Optional vessel-owned terrain presentation. `None` preserves the
    /// original renderer path and its capture bytes.
    pub terrain_appearance: Option<TerrainAppearance>,
    /// Presentation-only SDF bodies. Their source remains the caller's
    /// projection, never the brick map or world state.
    pub pose: Option<&'a CritterPose>,
    /// Every other body in frame, at the roster's reduced capsule budget.
    ///
    /// Additive: a frame that names none traces exactly what it always did.
    /// The caller culls to what its camera shows; the tracer keeps the first
    /// [`crate::MAX_ROSTER`] and drops the rest.
    pub roster: &'a [CritterPose],
    /// Where the atlas texture's voxels come from this frame. `None`
    /// keeps the CPU upload from [`BrickMap`], which is also the
    /// downlevel path. See [`LeasedAtlas`].
    pub leased_atlas: Option<LeasedAtlas<'a>>,
    /// Column-major world-to-clip matrix for the depth join. Required by
    /// [`super::BrickTracer::encode_with_depth`], which writes
    /// `@builtin(frag_depth)` from it so a raster tenant sharing the same
    /// matrix occludes and is occluded exactly. Ignored by plain `encode`.
    pub clip_from_world: Option<[[f32; 4]; 4]>,
    /// The host schedule epoch this frame expects its leased atlas bytes
    /// to have been made safe at. `Some` turns the lease's `read_epoch`
    /// from a producer promise into a tracer-validated identity: a lease
    /// at any other epoch is refused and the CPU path fills the frame.
    pub expected_read_epoch: Option<u64>,
}

impl<'a> BrickFrameInput<'a> {
    pub fn new(
        map: &'a BrickMap,
        revision: BrickRevision,
        flight: &'a Flight,
        grade: &'a Grade,
    ) -> Self {
        Self::for_camera(map, revision, TraceCamera::from_flight(flight), grade)
    }

    /// Construct a frame from a vessel-owned camera policy.
    pub fn for_camera(
        map: &'a BrickMap,
        revision: BrickRevision,
        camera: TraceCamera,
        grade: &'a Grade,
    ) -> Self {
        Self {
            map,
            revision,
            change: BrickChange::Full,
            camera,
            grade,
            terrain_appearance: None,
            pose: None,
            roster: &[],
            leased_atlas: None,
            clip_from_world: None,
            expected_read_epoch: None,
        }
    }

    pub fn changed(mut self, change: BrickChange<'a>) -> Self {
        self.change = change;
        self
    }

    /// Fill the atlas from a GPU-resident producer this frame instead of
    /// uploading it from the CPU.
    pub fn with_leased_atlas(mut self, leased: LeasedAtlas<'a>) -> Self {
        self.leased_atlas = Some(leased);
        self
    }

    pub fn with_pose(mut self, pose: &'a CritterPose) -> Self {
        self.pose = Some(pose);
        self
    }

    /// Draw every other body in frame beside [`Self::with_pose`]'s.
    pub fn with_roster(mut self, roster: &'a [CritterPose]) -> Self {
        self.roster = roster;
        self
    }

    /// Supply the raster tenant's column-major world-to-clip matrix so the
    /// depth join can write comparable fragment depth.
    pub fn with_clip_from_world(mut self, clip_from_world: [[f32; 4]; 4]) -> Self {
        self.clip_from_world = Some(clip_from_world);
        self
    }

    /// State the host schedule epoch a leased atlas must carry this frame.
    pub fn with_expected_read_epoch(mut self, epoch: u64) -> Self {
        self.expected_read_epoch = Some(epoch);
        self
    }

    pub fn with_terrain_appearance(mut self, appearance: TerrainAppearance) -> Self {
        self.terrain_appearance = Some(appearance);
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BrickDiagnostics {
    pub cpu_prepare_us: u64,
    pub brick_upload_bytes: u64,
    /// Voxel bytes that reached the atlas from a GPU-resident producer
    /// rather than the CPU. These are not counted in
    /// `brick_upload_bytes`, which stays the CPU-upload measure.
    pub leased_atlas_bytes: u64,
    /// Producer schedule epoch observed by the accepted atlas lease.
    pub observed_read_epoch: Option<u64>,
    /// Leases refused because their Ground revision did not match the frame's.
    pub stale_lease_rejections: u32,
    /// Leases refused because they materialized another selected projection.
    pub projection_lease_rejections: u32,
    /// Leases refused because their read epoch was not the one the frame
    /// expected.
    pub epoch_lease_rejections: u32,
    /// Leases refused because their extent did not fit the leased range.
    pub misfit_lease_rejections: u32,
    /// Leases refused because they did not cover every atlas region named by
    /// the frame's change declaration.
    pub incomplete_lease_rejections: u32,
    pub uniform_upload_bytes: u64,
    /// Roster members this frame drew, beside the single pose.
    pub roster_members: u32,
    /// Roster members the frame named past [`crate::MAX_ROSTER`].
    pub roster_dropped: u32,
    /// Capsules the drawn roster members carried past
    /// [`crate::MAX_ROSTER_CAPSULES`]. The widest are kept, so this is how much
    /// of the ecology's detail the budget spent rather than how many bodies
    /// went missing.
    pub roster_capsules_dropped: u32,
    pub resource_creations: u32,
    pub bind_group_rebuilds: u32,
    pub map_recreated: bool,
    /// Existing equal-sized textures were retained and fully republished for a
    /// new selected projection.
    pub projection_replaced: bool,
    pub trace_passes: u32,
    pub readback_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct BrickCapture {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub diagnostics: BrickDiagnostics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BrickTraceError {
    CaptureFormat(wgpu::TextureFormat),
    TooManyCapsules {
        actual: usize,
        maximum: usize,
    },
    UnknownBrickSlot(u32),
    /// The depth join cannot write comparable fragment depth without the
    /// raster tenant's world-to-clip matrix.
    MissingClipFromWorld,
    DevicePoll(String),
    Readback(String),
}

impl std::fmt::Display for BrickTraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaptureFormat(format) => {
                write!(
                    f,
                    "brick capture requires Rgba8Unorm output, not {format:?}"
                )
            },
            Self::TooManyCapsules { actual, maximum } => {
                write!(f, "brick trace has {actual} capsules; maximum is {maximum}")
            },
            Self::UnknownBrickSlot(slot) => {
                write!(f, "brick change names unknown atlas slot {slot}")
            },
            Self::MissingClipFromWorld => {
                write!(
                    f,
                    "the depth join needs a clip_from_world matrix on its frame input"
                )
            },
            Self::DevicePoll(message) => write!(f, "device poll failed: {message}"),
            Self::Readback(message) => write!(f, "readback failed: {message}"),
        }
    }
}

impl std::error::Error for BrickTraceError {}

#[cfg(test)]
mod camera_tests {
    use super::*;

    #[test]
    fn orthographic_rays_share_direction_but_move_across_the_slab() {
        let camera = TraceCamera::orthographic_slab(
            [0.0, 4.0, 8.0],
            [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
            5.0,
            2.0,
            6.0,
        )
        .unwrap();
        let left = camera.ray_at([-1.0, 0.0]);
        let right = camera.ray_at([1.0, 0.0]);
        assert_eq!(left.1, right.1);
        assert_eq!(left.0[0], -10.0);
        assert_eq!(right.0[0], 10.0);
        assert_eq!(left.0[2], 11.0);
    }

    #[test]
    fn perspective_rays_share_an_eye_and_diverge() {
        let camera = TraceCamera::perspective(
            [0.0, 0.0, 0.0],
            [0.0, 0.0, -1.0],
            [0.0, 1.0, 0.0],
            std::f32::consts::FRAC_PI_2,
            1.0,
            20.0,
        )
        .unwrap();
        let left = camera.ray_at([-1.0, 0.0]);
        let right = camera.ray_at([1.0, 0.0]);
        assert_eq!(left.0, right.0);
        assert!(left.1[0] < 0.0);
        assert!(right.1[0] > 0.0);
    }
}
