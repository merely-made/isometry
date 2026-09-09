// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Turning placed quads into vertices.
//!
//! This is where integers become floats, and it is the only direction that
//! conversion runs. Geometry comes from the mesher in exact voxel units;
//! nothing computed here ever goes back to the core.

use bytemuck::{Pod, Zeroable};
use mesocosm_mesh::{BodyMesh, place_point};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x3,
            },
        ],
    };
}

/// Maps a material id to a base colour. A placeholder until materials become
/// real content; deliberately deterministic so a body looks the same each run.
pub fn material_colour(material: u8) -> [f32; 3] {
    // Generated surfaces carry structural contrast. The host's existing guise
    // tint supplies organism colour; a plate does not assert active fixation.
    let tone = match material {
        240 => Some(0.85),
        241 => Some(0.65),
        242 => Some(0.95),
        243 => Some(0.12),
        244 => Some(0.35),
        245 => Some(1.0),
        _ => None,
    };
    if let Some(tone) = tone {
        return [tone; 3];
    }
    // Cheap integer hash spread across a pleasant range, so distinct materials
    // are distinguishable without an authored palette.
    let h = material.wrapping_mul(97).wrapping_add(31);
    let r = 0.35 + ((h & 0b111) as f32 / 7.0) * 0.55;
    let g = 0.35 + (((h >> 3) & 0b111) as f32 / 7.0) * 0.55;
    let b = 0.35 + (((h >> 5) & 0b111) as f32 / 7.0) * 0.55;
    [r, g, b]
}

/// The colour of a way of making a living.
///
/// Colour by **guise**, never by kingdom: a thing is drawn as what it presents
/// as, so a simulacrum reads as the thing it is pretending to be. The picture
/// is allowed to be lied to.
///
/// This replaces hashing the material id, which produced a legible *palette*
/// and an illegible *world*. The simulation had plants, grazers, and corpses
/// in it; the render had confetti.
pub fn kingdom_colour(guise: u8) -> [f32; 3] {
    match guise {
        // Producer: green, the colour of something making its own living.
        0 => [0.42, 0.68, 0.34],
        // Consumer: warm ochre, the colour of something that eats.
        1 => [0.78, 0.47, 0.28],
        // Decomposer: pale violet, the colour of something working the dead.
        _ => [0.60, 0.52, 0.70],
    }
}

/// Drains a colour toward grey. Used for the dead.
pub fn deadened(colour: [f32; 3]) -> [f32; 3] {
    let grey = (colour[0] + colour[1] + colour[2]) / 3.0 * 0.55;
    [
        colour[0] * 0.25 + grey,
        colour[1] * 0.25 + grey,
        colour[2] * 0.25 + grey,
    ]
}

/// Shifts a colour toward a warning: hot, saturated, conspicuous.
///
/// A signal has to be *seen* to mean anything. This is the visual half of
/// signalling and counter-signalling — and because a bluffer wears the same
/// colours as something genuinely armed, the picture alone cannot tell you
/// which is which. That is the point.
pub fn warning_colour(base: [f32; 3]) -> [f32; 3] {
    [
        (base[0] * 0.4 + 0.85).min(1.0),
        (base[1] * 0.35 + 0.35).min(1.0),
        base[2] * 0.25,
    ]
}

/// How much a face is lit, by which way it points. Top bright, sides mid,
/// bottom dark: the convention that makes untextured voxels read as solid.
pub fn face_shade(axis: u8, positive: bool) -> f32 {
    match (axis, positive) {
        (1, true) => 1.0,
        (1, false) => 0.45,
        (0, true) => 0.78,
        (0, false) => 0.66,
        (_, true) => 0.86,
        (_, false) => 0.58,
    }
}

/// One meshed body placed in world space.
#[derive(Clone, Copy, Debug)]
pub struct SceneItem<'a> {
    pub mesh: &'a BodyMesh,
    /// Where this body's origin sits in the world, in voxel units.
    pub origin: [i32; 3],
    /// Multiplies the body's colour. Used to dim things the critter cannot
    /// reach, so reach reads without a UI element.
    pub tint: f32,
    /// Whether this thing is advertising danger. A claim, not a fact.
    pub warns: bool,
    /// Replaces fixture material colour. Generated structural tones multiply
    /// this colour, matching the live body's guise-tinted presentation.
    pub recolour: Option<[f32; 3]>,
    /// Scales the body about its own origin. Growth you can see.
    pub scale: f32,
}

impl<'a> SceneItem<'a> {
    pub fn new(mesh: &'a BodyMesh, origin: [i32; 3]) -> Self {
        Self {
            mesh,
            origin,
            tint: 1.0,
            warns: false,
            recolour: None,
            scale: 1.0,
        }
    }

    pub fn tinted(mesh: &'a BodyMesh, origin: [i32; 3], tint: f32) -> Self {
        Self {
            tint,
            ..Self::new(mesh, origin)
        }
    }

    /// A living thing, drawn as what it appears to be and how big it has got.
    pub fn creature(
        mesh: &'a BodyMesh,
        origin: [i32; 3],
        tint: f32,
        warns: bool,
        colour: [f32; 3],
        scale: f32,
    ) -> Self {
        Self {
            mesh,
            origin,
            tint,
            warns,
            recolour: Some(colour),
            scale,
        }
    }
}

/// Builds the triangle list for a whole scene: several bodies, each placed.
pub fn build_scene_vertices(items: &[SceneItem]) -> Vec<Vertex> {
    let mut out = Vec::new();
    for item in items {
        append_body(&mut out, item);
    }
    out
}

/// Builds the triangle list for a meshed body.
///
/// Two triangles per quad, wound so the mesher's outward-facing corner order
/// survives. Returns an empty list for a body with no geometry, which the
/// renderer draws as an empty frame rather than treating as an error.
pub fn build_vertices(mesh: &BodyMesh) -> Vec<Vertex> {
    let mut out = Vec::new();
    append_body(&mut out, &SceneItem::new(mesh, [0, 0, 0]));
    out
}

fn append_body(out: &mut Vec<Vertex>, item: &SceneItem) {
    let SceneItem {
        mesh,
        origin,
        tint,
        warns,
        recolour,
        scale,
    } = *item;
    for placement in &mesh.placements {
        let Some(part_mesh) = mesh.mesh_for(placement.volume) else {
            continue;
        };
        for quad in &part_mesh.quads {
            let shade = face_shade(quad.axis, quad.positive);
            let material = material_colour(quad.material);
            let base = match recolour {
                Some(colour) if (240..=245).contains(&quad.material) => {
                    [0, 1, 2].map(|axis| colour[axis] * material[axis])
                },
                Some(colour) => colour,
                None => material,
            };
            let base = if warns { warning_colour(base) } else { base };
            let colour = [base[0] * shade, base[1] * shade, base[2] * shade];

            let colour = [colour[0] * tint, colour[1] * tint, colour[2] * tint];

            let corners = quad.corners().map(|corner| {
                let placed =
                    place_point(corner, placement.yaw, placement.pivot, placement.pivot_at);
                [
                    placed[0] as f32 * scale + origin[0] as f32,
                    placed[1] as f32 * scale + origin[1] as f32,
                    placed[2] as f32 * scale + origin[2] as f32,
                ]
            });

            for index in [0usize, 1, 2, 0, 2, 3] {
                out.push(Vertex {
                    position: corners[index],
                    color: colour,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::{BodyDocument, SpeciesId, VolumeRef};
    use mesocosm_mesh::{Volume, VolumeMap, mesh_body};

    fn source() -> VolumeMap {
        let mut map = VolumeMap::new();
        map.insert(VolumeRef::from_tag(1), Volume::solid([2, 2, 2], 1));
        map
    }

    #[test]
    fn a_cube_becomes_six_quads_of_triangles() {
        let body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1]);
        let mesh = mesh_body(&body, &source()).unwrap();
        let vertices = build_vertices(&mesh);
        assert_eq!(vertices.len(), 6 * 6, "six faces, two triangles each");
    }

    #[test]
    fn an_unresolvable_body_yields_no_vertices() {
        let mesh = BodyMesh::default();
        assert!(build_vertices(&mesh).is_empty());
    }

    #[test]
    fn faces_are_shaded_by_direction() {
        assert!(
            face_shade(1, true) > face_shade(1, false),
            "top beats bottom"
        );
        assert!(face_shade(1, true) > face_shade(0, true), "top beats sides");
    }

    #[test]
    fn materials_get_distinct_colours() {
        assert_ne!(material_colour(1), material_colour(2));
        assert_eq!(material_colour(7), material_colour(7));
    }

    #[test]
    fn vertex_is_tightly_packed() {
        assert_eq!(size_of::<Vertex>(), 24);
    }
}
