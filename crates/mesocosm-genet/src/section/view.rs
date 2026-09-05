// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One set of camera numbers for terrain rays, raster depth and body culling.

use super::{CameraMode, SlabWindow};
use mesocosm_lens::{SlabWall, TraceCamera};
use mesocosm_render::ClipSlab;

#[derive(Clone, Copy)]
pub(super) struct View {
    pub mode: CameraMode,
    pub centre: [f32; 3],
    pub half: f32,
    pub aspect: f32,
    pub depth: f32,
    pub pitch: Option<f32>,
    pub bounds: Option<([f32; 3], [f32; 3])>,
}

impl View {
    pub fn basis(self) -> [[f32; 3]; 3] {
        camera_basis(self.mode, self.pitch)
    }

    fn reach(self) -> f32 {
        if self.pitch.is_none() && self.depth == super::SLAB_DEPTH {
            return self.mode.slab_reach(self.half, self.aspect);
        }
        let [_, up, forward] = self.basis();
        SlabWall::new(forward, up, self.half, self.aspect, self.depth)
            .map_or(self.depth * 0.5, |wall| wall.reach)
    }

    pub fn trace(self) -> Option<TraceCamera> {
        let [_, up, forward] = self.basis();
        TraceCamera::orthographic_slab(self.centre, forward, up, self.half, self.aspect, self.depth)
    }

    pub fn window(self) -> SlabWindow {
        SlabWindow {
            centre: self.centre,
            axes: self.basis(),
            half: [self.half * self.aspect, self.half, self.reach()],
        }
    }

    pub fn matrix(self) -> [[f32; 4]; 4] {
        let [right, up, forward] = self.basis();
        let x = right.map(|v| v / (self.half * self.aspect));
        let y = up.map(|v| v / self.half);
        let z = forward.map(|v| v / (2.0 * (self.reach() + 1.0)));
        [
            [x[0], y[0], z[0], 0.0],
            [x[1], y[1], z[1], 0.0],
            [x[2], y[2], z[2], 0.0],
            [
                -dot(x, self.centre),
                -dot(y, self.centre),
                0.5 - dot(z, self.centre),
                1.0,
            ],
        ]
    }

    pub fn clip(self) -> ClipSlab {
        let forward = if self.pitch.is_none() {
            self.mode.forward()
        } else {
            self.basis()[2]
        };
        let length = (forward[0] * forward[0] + forward[2] * forward[2]).sqrt();
        let normal = [forward[0] / length, 0.0, forward[2] / length];
        let middle = dot(normal, self.centre);
        ClipSlab {
            normal,
            min: middle - self.depth * 0.5,
            max: middle + self.depth * 0.5,
            bounds: self.bounds,
        }
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a.into_iter().zip(b).map(|(a, b)| a * b).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn variable_pitch_and_depth_match_traced_rays_after_every_turn() {
        for pitch in [0.0, 12.0, 45.0] {
            let mut mode = CameraMode::TerrariumEast;
            for _ in 0..4 {
                let view = View {
                    mode,
                    centre: [-8.0, 23.0, 16.0],
                    half: 38.0,
                    aspect: 16.0 / 9.0,
                    depth: 180.0,
                    pitch: Some(pitch),
                    bounds: None,
                };
                let camera = serde_json::to_value(view.trace().unwrap()).unwrap();
                let vector =
                    |name: &str| [0, 1, 2].map(|i| camera[name][i].as_f64().unwrap() as f32);
                let origin = vector("origin");
                let right = vector("right");
                let up = vector("up");
                let direction = vector("forward");
                let wall = vector("wall");
                for uv in [[-0.8, 0.5], [0.0, 0.0], [0.6, -0.7]] {
                    let advance = wall[0] * uv[0] + wall[1] * uv[1] + wall[2];
                    let point = [0, 1, 2].map(|i| {
                        origin[i]
                            + right[i] * uv[0]
                            + up[i] * uv[1]
                            + direction[i] * (advance + 40.0)
                    });
                    let matrix = view.matrix();
                    let projected = [0, 1, 2].map(|row| {
                        matrix[3][row]
                            + (0..3).map(|col| matrix[col][row] * point[col]).sum::<f32>()
                    });
                    assert!((projected[0] - uv[0]).abs() < 1e-4);
                    assert!((projected[1] - uv[1]).abs() < 1e-4);
                }
                mode = mode.quarter_turn(false);
            }
        }
    }
}

pub fn camera_basis(mode: CameraMode, pitch: Option<f32>) -> [[f32; 3]; 3] {
    let [right, _, forward] = mode.basis();
    let Some(pitch) = pitch.filter(|_| mode.is_terrarium()) else {
        return mode.basis();
    };
    let pitch = pitch.to_radians();
    let length = forward[0].hypot(forward[2]);
    let forward = [
        forward[0] / length * pitch.cos(),
        -pitch.sin(),
        forward[2] / length * pitch.cos(),
    ];
    let up = [
        right[1] * forward[2] - right[2] * forward[1],
        right[2] * forward[0] - right[0] * forward[2],
        right[0] * forward[1] - right[1] * forward[0],
    ];
    [right, up, forward]
}
