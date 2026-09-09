// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{MeshVertex, Tenant};
use renderling::{geometry::Vertex, glam::Vec3};

impl Tenant {
    /// Fixture lighting policy. Existing room probes retain their torch shading.
    pub fn set_geometry_with_ambient(
        &self,
        room: &[MeshVertex],
        body: &[MeshVertex],
        eye: Vec3,
        ambient: f32,
    ) {
        self.room.set_vertices(
            self.stage
                .new_vertices(shaded_with_ambient(room, eye, ambient)),
        );
        self.body.set_vertices(
            self.stage
                .new_vertices(shaded_with_ambient(body, eye, ambient)),
        );
    }
}

pub(super) fn shaded(vertices: &[MeshVertex], eye: Vec3) -> Vec<Vertex> {
    shaded_with_ambient(vertices, eye, 0.0)
}

fn shaded_with_ambient(vertices: &[MeshVertex], eye: Vec3, ambient: f32) -> Vec<Vertex> {
    let mut out = Vec::with_capacity(vertices.len());
    for triangle in vertices.chunks_exact(3) {
        let [a, b, c] = [
            triangle[0].position,
            triangle[1].position,
            triangle[2].position,
        ];
        let normal = (Vec3::from(b) - Vec3::from(a))
            .cross(Vec3::from(c) - Vec3::from(a))
            .normalize_or_zero()
            .to_array();
        for vertex in triangle {
            let ambient = if ambient.is_finite() {
                ambient.clamp(0., 1.)
            } else {
                0.
            };
            let lit = ambient + (1.0 - ambient) * crate::scene::torch(eye, vertex.position);
            out.push(
                Vertex::default()
                    .with_position(vertex.position)
                    .with_normal(normal)
                    .with_color([
                        vertex.color[0] * lit,
                        vertex.color[1] * lit,
                        vertex.color[2] * lit,
                        1.0,
                    ]),
            );
        }
    }
    out
}
