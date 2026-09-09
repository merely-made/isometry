// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Biome maps from a seed: the world painted as two images.
//!
//! The march consumes a heightmap and a colormap and nothing else, so a
//! biosphere is whatever gets painted into them. This probe synthesiser uses
//! the same [`Places`] partition the simulation runs on: one biome per place,
//! terrain relief from layered value noise shaped per biome, colours from the
//! biome's base tint shaded by height. Worldgen later replaces this with the
//! vello lane painting richer maps; the renderer contract does not change.

use mesocosm_core::{Places, Rng};

/// One biome, derived from a place.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct Biome {
    pub base: [f32; 3],
    /// Terrain amplitude, 0..1 of the height range.
    pub relief: f32,
    /// Base elevation, 0..1.
    pub floor: f32,
}

/// The two images the march eats, plus the palette the grade may use.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BiomeMaps {
    pub side: u32,
    /// R8 heights, one byte per texel.
    pub height: Vec<u8>,
    /// RGBA colours.
    pub color: Vec<u8>,
    /// The palette implied by the biomes: base tints at four shades each,
    /// plus fog and sky entries. What a retro grade quantises against.
    pub palette: Vec<[f32; 3]>,
}

/// Golden-angle biome tint, the same rule the minimap uses for lineages, so a
/// place's identity holds across projections.
fn biome_tint(index: u32) -> [f32; 3] {
    let hue = (index as f32 * 137.507_77) % 360.0;
    let (h, s, l) = (hue, 0.38f32, 0.42f32);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0..60 => (c, x, 0.0),
        60..120 => (x, c, 0.0),
        120..180 => (0.0, c, x),
        180..240 => (0.0, x, c),
        240..300 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    [r + m, g + m, b + m]
}

/// Deterministic value noise: a lattice of seeded values, smoothly
/// interpolated. Float math is fine here; maps are presentation inputs, and
/// the same seed paints the same map on every machine that runs this code.
fn value_noise(seed: u64, x: f32, y: f32) -> f32 {
    let lattice = |ix: i64, iy: i64| -> f32 {
        let mut r = Rng::from_seed(
            seed ^ (ix as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ (iy as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F),
        );
        (r.below(1 << 16) as f32) / (1 << 16) as f32
    };
    let (ix, iy) = (x.floor() as i64, y.floor() as i64);
    let (fx, fy) = (x - x.floor(), y - y.floor());
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let top = lattice(ix, iy) * (1.0 - sx) + lattice(ix + 1, iy) * sx;
    let bottom = lattice(ix, iy + 1) * (1.0 - sx) + lattice(ix + 1, iy + 1) * sx;
    top * (1.0 - sy) + bottom * sy
}

fn fbm(seed: u64, x: f32, y: f32) -> f32 {
    let mut total = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    for octave in 0..5 {
        total += value_noise(seed + octave, x * frequency, y * frequency) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    total
}

/// Paints a biosphere: biomes from a place partition, relief from noise.
pub fn synthesize(seed: u64, side: u32) -> BiomeMaps {
    // A coarser partition than the enclosure's: this is a probe of the
    // renderer at landscape scale, and "much bigger" was the ruling.
    let mut rng = Rng::from_seed(seed ^ 0x4C45_4E53);
    let extent = side as i32 / 2;
    let places = Places::scatter(&mut rng, 4, extent);

    let biomes: Vec<Biome> = (0..places.len() as u32)
        .map(|index| {
            let mut r = Rng::from_seed(seed.wrapping_add(index as u64 * 7919));
            Biome {
                base: biome_tint(index),
                relief: 0.25 + (r.below(60) as f32) / 100.0,
                floor: 0.15 + (r.below(40) as f32) / 100.0,
            }
        })
        .collect();

    let mut height = vec![0u8; (side * side) as usize];
    let mut color = vec![0u8; (side * side * 4) as usize];

    for row in 0..side {
        for col in 0..side {
            let world = [col as i32 - extent, 0, row as i32 - extent];
            let place = places.at(world).expect("the partition is total");
            let biome = biomes[place.0 as usize];

            let (nx, ny) = (col as f32 / 48.0, row as f32 / 48.0);
            // Continental shape shared across biomes so borders meet on a
            // common ground; per-biome relief rides on top.
            let continent = fbm(seed, nx * 0.5, ny * 0.5);
            let local = fbm(seed ^ 0xB10B, nx * 2.0, ny * 2.0);
            let h = (biome.floor + continent * 0.35 + local * biome.relief).clamp(0.0, 1.0);

            let i = (row * side + col) as usize;
            height[i] = (h * 255.0) as u8;

            // Colour: biome base shaded by elevation, with a high-ground
            // wash toward pale so ridges read.
            let shade = 0.55 + h * 0.6;
            let wash = ((h - 0.6).max(0.0) * 1.6).min(0.6);
            let texel = [
                (biome.base[0] * shade * (1.0 - wash) + 0.85 * wash).min(1.0),
                (biome.base[1] * shade * (1.0 - wash) + 0.83 * wash).min(1.0),
                (biome.base[2] * shade * (1.0 - wash) + 0.78 * wash).min(1.0),
            ];
            for (channel, value) in texel.iter().enumerate() {
                color[i * 4 + channel] = (value * 255.0) as u8;
            }
            color[i * 4 + 3] = 255;
        }
    }

    // Palette: every biome at four shades, then fog and sky anchors. Small
    // on purpose; a retro grade is supposed to be starved.
    let mut palette = Vec::new();
    for biome in &biomes {
        for shade in [0.5, 0.72, 0.95, 1.2] {
            palette.push([
                (biome.base[0] * shade).min(1.0),
                (biome.base[1] * shade).min(1.0),
                (biome.base[2] * shade).min(1.0),
            ]);
        }
    }
    palette.push([0.66, 0.66, 0.72]); // fog
    palette.push([0.65, 0.72, 0.80]); // sky low
    palette.push([0.35, 0.45, 0.62]); // sky high
    palette.push([0.88, 0.86, 0.80]); // ridge wash

    BiomeMaps {
        side,
        height,
        color,
        palette,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_paints_the_same_world() {
        let a = synthesize(77, 128);
        let b = synthesize(77, 128);
        assert_eq!(a.height, b.height);
        assert_eq!(a.color, b.color);
    }

    #[test]
    fn different_seeds_paint_different_worlds() {
        assert_ne!(synthesize(1, 128).height, synthesize(2, 128).height);
    }

    #[test]
    fn biomes_are_regions_not_noise() {
        // Neighbouring texels mostly share a biome: sample colours at close
        // pairs and expect the overwhelming majority identical, which noise
        // would not give.
        let maps = synthesize(9, 256);
        let mut same = 0;
        let mut total = 0;
        for row in (8..248).step_by(16) {
            for col in (8..248).step_by(16) {
                let a = (row * 256 + col) as usize * 4;
                let b = (row * 256 + col + 1) as usize * 4;
                total += 1;
                // Hue proximity: same biome shades along elevation, so
                // compare channel ordering rather than exact bytes.
                let orders = |i: usize| {
                    let (r, g, bl) = (maps.color[i], maps.color[i + 1], maps.color[i + 2]);
                    (r > g, g > bl, r > bl)
                };
                if orders(a) == orders(b) {
                    same += 1;
                }
            }
        }
        assert!(
            same * 10 > total * 9,
            "{same}/{total} neighbour pairs shared a biome"
        );
    }

    #[test]
    fn the_palette_is_starved() {
        let maps = synthesize(3, 128);
        assert!(maps.palette.len() < 80, "a retro palette stays small");
    }
}
