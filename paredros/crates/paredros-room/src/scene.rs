// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Room and body as triangles, and where the camera stands.
//!
//! The bricks the walker collides against are the bricks that get drawn:
//! `mesh_volume` over `Ground::brick_materials`, exactly as mesocosm-mesh's
//! own ground-projection receipt builds them. The body is drawn as the
//! volume it actually occupies, one voxel by `WALKER_HEIGHT`, so what you
//! see is what `stands` was asked about.

use mesocosm_core::VolumeRef;
use mesocosm_core::places::{BRICK, Ground, WALKER_HEIGHT};
#[cfg(feature = "r1-proof")]
use mesocosm_lens::{CritterPose, TraceCamera, critter::Capsule};
use mesocosm_mesh::{BodyMesh, Volume};
use mesocosm_render::geometry::{SceneItem, Vertex, build_scene_vertices};
use netrender::Scene;
use renderling::glam::{Mat4, Vec3};

/// How far around the room to mesh, in voxels.
pub const MESH_REACH: i32 = 20;
/// The body's colour. Warm against the room's cold rock, so the second
/// person has something to be beside.
const BODY_COLOUR: [f32; 3] = [0.92, 0.62, 0.28];

/// A brick's materials as a mesh volume.
///
/// The brick is y-major (y, z, x) and `Volume` indexes x + y*sx + z*sx*sy,
/// so this walks the brick in the volume's order rather than assuming the
/// two layouts agree.
fn volume_of(ground: &Ground, key: [i16; 3]) -> Option<(Volume, [i32; 3])> {
    let (brick, origin) = ground.brick_materials(key)?;
    let side = BRICK as u32;
    let mut voxels = Vec::with_capacity((side * side * side) as usize);
    for z in 0..side as i32 {
        for y in 0..side as i32 {
            for x in 0..side as i32 {
                voxels.push(brick.get([x, y, z]));
            }
        }
    }
    Volume::new([side, side, side], voxels)
        .ok()
        .map(|volume| (volume, origin))
}

/// The room's rock, as triangles in world voxel space.
pub fn room_vertices(room: &crate::Room) -> Vec<Vertex> {
    let placed: Vec<(BodyMesh, [i32; 3])> = room
        .nearby_bricks(MESH_REACH)
        .into_iter()
        .filter_map(|key| {
            let (volume, origin) = volume_of(&room.ground, key)?;
            (!volume.is_empty())
                .then(|| (BodyMesh::single(VolumeRef::from_tag(1), &volume), origin))
        })
        .collect();

    let items: Vec<SceneItem> = placed
        .iter()
        .map(|(mesh, origin)| SceneItem::new(mesh, *origin))
        .collect();
    build_scene_vertices(&items)
}

/// The body, as the volume `stands` reasons about.
pub fn body_vertices(at: [i32; 3]) -> Vec<Vertex> {
    let volume = Volume::solid([1, WALKER_HEIGHT as u32, 1], 1);
    let mesh = BodyMesh::single(VolumeRef::from_tag(2), &volume);
    build_scene_vertices(&[SceneItem::creature(&mesh, at, 1.0, false, BODY_COLOUR, 1.0)])
}

/// Where the camera stands, and what it can see from there.
#[derive(Clone, Copy, Debug)]
pub struct Camera {
    pub projection: Mat4,
    pub view: Mat4,
    /// The eye in world voxels. Doubles as the torch position.
    pub eye: Vec3,
    /// The Paredros-owned focus point. The DDA profile consumes this policy;
    /// it does not derive or replace it.
    pub target: Vec3,
}

impl Camera {
    #[cfg(feature = "r1-proof")]
    pub fn trace(self, aspect: f32) -> TraceCamera {
        TraceCamera::perspective(
            self.eye.to_array(),
            self.target.to_array(),
            Vec3::Y.to_array(),
            std::f32::consts::FRAC_PI_3,
            aspect,
            200.0,
        )
        .expect("Paredros room camera is valid")
    }
}

/// A close camera that keeps its back to the room and its eye on the body.
///
/// An over-the-shoulder rig fails in a nine-voxel chamber: put the body in a
/// corner and the shoulder camera is inside the wall, framing rock. So the
/// eye backs off *toward the middle of the room* instead of straight behind
/// the heading, which keeps it inside, keeps it close, and always looks
/// across the chamber rather than into it. Second person wants the body in
/// frame; the Barony/Delver reference wants the walls in frame; this gets
/// both.
pub fn camera(room: &crate::Room, at: [i32; 3], heading: [i32; 2], aspect: f32) -> Camera {
    let head = Vec3::new(at[0] as f32 + 0.5, at[1] as f32 + 1.4, at[2] as f32 + 0.5);
    let aim = Vec3::new(heading[0] as f32, 0.0, heading[1] as f32);
    let forward = if aim.length_squared() > 0.0 {
        aim.normalize()
    } else {
        Vec3::X
    };

    let (low, high) = room.interior();
    let middle = Vec3::new(
        room.centre[0] as f32 + 0.5,
        head.y,
        room.centre[2] as f32 + 0.5,
    );
    let offset = middle - head;
    let (back, reach) = if offset.length_squared() > 0.25 {
        (offset.normalize(), (offset.length() + 2.2).min(4.4))
    } else {
        (-forward, 2.4)
    };

    let eye = Vec3::new(
        (head.x + back.x * reach).clamp(low[0] as f32 + 0.7, high[0] as f32 + 0.3),
        (head.y + 1.2).clamp(low[1] as f32 + 0.7, high[1] as f32 + 0.3),
        (head.z + back.z * reach).clamp(low[2] as f32 + 0.7, high[2] as f32 + 0.3),
    );

    Camera {
        projection: Mat4::perspective_rh(std::f32::consts::FRAC_PI_3, aspect, 0.05, 200.0),
        view: Mat4::look_at_rh(eye, head, Vec3::Y),
        eye,
        target: head,
    }
}

/// The D1 pillars' colour. Cyan, because nothing else in the room is:
/// terrain is brown and grey, the body warm orange, fog near-black blue,
/// so a cyan-dominant pixel in the receipt can only be a pillar.
#[cfg(feature = "d1-proof")]
pub const PILLAR_COLOUR: [f32; 3] = [0.10, 0.85, 0.95];

/// One renderling occlusion witness for D1: a voxel-aligned box.
#[cfg(feature = "d1-proof")]
#[derive(Clone, Copy, Debug)]
pub struct Pillar {
    /// Why this pillar is in the picture, for the receipt.
    pub role: &'static str,
    /// Lowest voxel corner, world coordinates.
    pub min: [i32; 3],
    /// Extent in voxels.
    pub extent: [i32; 3],
}

/// The three witnesses, placed off the room's centre so a relief change
/// upstream moves them with the chamber. All sit against the final
/// camera's corner-facing frustum: one standing before the wall (raster
/// occludes raymarch), one with its base a voxel under the floor
/// (raymarch occludes raster, with its open span as the positive control),
/// and one entirely under the floor, inside rock the carve's solid
/// footprint guarantees, which must therefore be invisible.
#[cfg(feature = "d1-proof")]
pub fn d1_pillars(room: &crate::Room) -> [Pillar; 3] {
    let c = room.centre;
    let r = room.radius;
    [
        Pillar {
            role: "standing before the wall",
            min: [c[0] + 3, c[1] - r, c[2] - 2],
            extent: [1, 3, 1],
        },
        Pillar {
            role: "base buried a voxel under the floor",
            min: [c[0] + 2, c[1] - r - 1, c[2] - r],
            extent: [1, 3, 1],
        },
        Pillar {
            role: "entirely under the floor",
            min: [c[0] + 4, c[1] - r - 3, c[2] - r],
            extent: [1, 2, 1],
        },
    ]
}

/// The pillars as renderling geometry. Render-only witnesses: the trace
/// never collides with them and the world never learns about them.
#[cfg(feature = "d1-proof")]
pub fn pillar_vertices(pillars: &[Pillar]) -> Vec<Vertex> {
    let items: Vec<(BodyMesh, [i32; 3])> = pillars
        .iter()
        .map(|pillar| {
            let extent = [
                pillar.extent[0] as u32,
                pillar.extent[1] as u32,
                pillar.extent[2] as u32,
            ];
            let volume = Volume::solid(extent, 1);
            (
                BodyMesh::single(VolumeRef::from_tag(3), &volume),
                pillar.min,
            )
        })
        .collect();
    let items: Vec<SceneItem> = items
        .iter()
        .map(|(mesh, min)| SceneItem::creature(mesh, *min, 1.0, false, PILLAR_COLOUR, 1.0))
        .collect();
    build_scene_vertices(&items)
}

/// The same played body as a presentation-only SDF for the DDA profile.
#[cfg(feature = "r1-proof")]
pub fn body_pose(at: [i32; 3]) -> CritterPose {
    let centre = [at[0] as f32 + 0.5, at[1] as f32, at[2] as f32 + 0.5];
    CritterPose::from_capsules(
        vec![Capsule {
            a: [centre[0], centre[1] + 0.35, centre[2]],
            ra: 0.34,
            b: [centre[0], centre[1] + 1.65, centre[2]],
            rb: 0.30,
        }],
        [
            [centre[0] - 0.13, centre[1] + 1.52, centre[2] - 0.27, 0.07],
            [centre[0] + 0.13, centre[1] + 1.52, centre[2] - 0.27, 0.07],
        ],
        BODY_COLOUR,
    )
}

/// How far the torch carries, in voxels. Underground, an unlit greedy-meshed
/// wall is one flat colour across its whole quad; falloff from the eye is
/// what turns that back into a surface with a near side and a far side.
pub const TORCH_REACH: f32 = 13.0;
/// How dark the far side of the torch's reach gets.
pub const TORCH_FLOOR: f32 = 0.14;

/// The torch's brightness at a point.
pub fn torch(eye: Vec3, at: [f32; 3]) -> f32 {
    let distance = (Vec3::from(at) - eye).length();
    (1.15 - distance / TORCH_REACH).clamp(TORCH_FLOOR, 1.0)
}

/// The chrome bar netrender paints over the tenant's frame: the vello half
/// of the composed master, and the proof that both halves land in one image.
pub fn chrome(size: [u32; 2], tick: usize, ticks: usize) -> Scene {
    let (width, height) = (size[0] as f32, size[1] as f32);
    let mut scene = Scene::new(size[0], size[1]);

    scene.push_rect(0.0, 0.0, width, 40.0, [0.03, 0.04, 0.06, 0.86]);
    // A trace-progress bar, so the chrome carries a fact rather than a shape.
    let done = (tick as f32 / ticks.max(1) as f32).clamp(0.0, 1.0);
    scene.push_rect(
        16.0,
        14.0,
        16.0 + (width - 32.0) * done,
        26.0,
        [0.94, 0.66, 0.32, 0.95],
    );
    scene.push_rect(16.0, 14.0, width - 16.0, 15.0, [0.55, 0.60, 0.68, 0.60]);

    let edge = [0.94, 0.66, 0.32, 1.0];
    scene.push_rect(0.0, 0.0, width, 2.0, edge);
    scene.push_rect(0.0, height - 2.0, width, height, edge);
    scene.push_rect(0.0, 0.0, 2.0, height, edge);
    scene.push_rect(width - 2.0, 0.0, width, height, edge);
    scene
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Probe;
    use crate::room::SEED;

    #[test]
    fn the_room_and_the_body_both_produce_geometry() {
        let probe = Probe::new(SEED).unwrap();
        let room = room_vertices(probe.room());
        assert!(
            room.len() > 600,
            "a chamber is more than a few faces: {}",
            room.len()
        );
        assert_eq!(room.len() % 3, 0, "triangles come in threes");

        let body = body_vertices(probe.at());
        assert_eq!(body.len(), 6 * 6, "six faces, two triangles each");
    }

    #[test]
    fn the_camera_stays_inside_the_room_all_the_way_through_the_trace() {
        let mut probe = Probe::new(SEED).unwrap();
        let (low, high) = probe.room().interior();
        loop {
            let camera = camera(probe.room(), probe.at(), probe.heading(), 16.0 / 9.0);
            for axis in 0..3 {
                assert!(
                    camera.eye[axis] >= low[axis] as f32
                        && camera.eye[axis] <= high[axis] as f32 + 1.0,
                    "the camera left the room on axis {axis}: {:?}",
                    camera.eye
                );
            }
            // The eye must also be the view's own translation, or the torch
            // would be lighting from somewhere the picture is not.
            let from_view = camera.view.inverse().w_axis.truncate();
            assert!((from_view - camera.eye).length() < 1e-3);
            if !probe.advance() {
                break;
            }
        }
    }

    #[test]
    fn the_torch_falls_off_with_distance() {
        let eye = Vec3::ZERO;
        assert!(torch(eye, [0.0, 0.0, 0.0]) > torch(eye, [6.0, 0.0, 0.0]));
        assert_eq!(torch(eye, [400.0, 0.0, 0.0]), TORCH_FLOOR);
    }
}
