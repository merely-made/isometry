// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The landscape under the graph: an integer heightfield.
//!
//! G0's top-down half (place-graph engine plan §3): continental relief by
//! diamond-square, deterministic from its own seed, integer throughout.
//! The graph derives adjacency from this field, so the field is world
//! truth's input rather than a rendering asset. Bottom-up detail happens
//! per region at G1, inside bricks; this stays coarse on purpose.

use serde::{Deserialize, Serialize};

use crate::rng::Rng;

/// Grid side: 2^6 + 1. Coarse is the point; regions carry local detail.
const SIDE: usize = 65;
/// First displacement half-range. Halves each octave, integer floor 1.
const FIRST_SWING: i32 = 96;
/// The band heights settle into after clamping.
pub const CEILING: i32 = 255;

/// An integer heightfield over the whole enclosure.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Relief {
    heights: Vec<i32>,
    /// Below this, water. A percentile of the field, so seas vary by world.
    pub sea: i32,
}

impl Relief {
    /// Deterministic from `seed` alone; consumes no caller stream.
    pub fn generate(seed: u64) -> Self {
        let mut rng = Rng::from_seed(seed ^ 0x5EA5_CA9E);
        let mut heights = vec![0i32; SIDE * SIDE];
        let at = |row: usize, col: usize| row * SIDE + col;

        for (row, col) in [(0, 0), (0, SIDE - 1), (SIDE - 1, 0), (SIDE - 1, SIDE - 1)] {
            heights[at(row, col)] = 64 + rng.range_i32(0, 127);
        }

        let mut step = SIDE - 1;
        let mut swing = FIRST_SWING;
        while step > 1 {
            let half = step / 2;
            // Diamond: centres from their four corners.
            for row in (half..SIDE).step_by(step) {
                for col in (half..SIDE).step_by(step) {
                    let mean = (heights[at(row - half, col - half)]
                        + heights[at(row - half, col + half)]
                        + heights[at(row + half, col - half)]
                        + heights[at(row + half, col + half)])
                        / 4;
                    heights[at(row, col)] = (mean + rng.range_i32(-swing, swing)).clamp(0, CEILING);
                }
            }
            // Square: edge midpoints from their present neighbours.
            for row in (0..SIDE).step_by(half) {
                let offset = if (row / half).is_multiple_of(2) {
                    half
                } else {
                    0
                };
                for col in (offset..SIDE).step_by(step) {
                    let mut sum = 0i32;
                    let mut count = 0i32;
                    if row >= half {
                        sum += heights[at(row - half, col)];
                        count += 1;
                    }
                    if row + half < SIDE {
                        sum += heights[at(row + half, col)];
                        count += 1;
                    }
                    if col >= half {
                        sum += heights[at(row, col - half)];
                        count += 1;
                    }
                    if col + half < SIDE {
                        sum += heights[at(row, col + half)];
                        count += 1;
                    }
                    heights[at(row, col)] =
                        (sum / count + rng.range_i32(-swing, swing)).clamp(0, CEILING);
                }
            }
            step = half;
            swing = (swing / 2).max(1);
        }

        // Sea at the 25th percentile: most worlds get water, none drown.
        let mut sorted = heights.clone();
        sorted.sort_unstable();
        let sea = sorted[sorted.len() / 4];
        Self { heights, sea }
    }

    /// Height at a world position, nearest-sample. `extent` is the world's
    /// half-width: positions run [-extent, extent] on x and z.
    pub fn sample(&self, extent: i32, x: i32, z: i32) -> i32 {
        let side = (SIDE - 1) as i64;
        let map = |v: i32| -> usize {
            let span = 2 * extent.max(1) as i64;
            let offset = (v as i64 + extent as i64).clamp(0, span);
            (offset * side / span) as usize
        };
        self.heights[map(z) * SIDE + map(x)]
    }

    /// Local ruggedness around a position: max minus min over a small
    /// neighbourhood. The nest heuristic reads this, so pockets are
    /// landscape facts rather than dice rolls.
    pub fn ruggedness(&self, extent: i32, x: i32, z: i32) -> i32 {
        // Two relief cells per step, so the window spans real structure
        // rather than one bilinear patch.
        let reach = (4 * extent / (SIDE as i32 - 1)).max(1);
        let (mut lo, mut hi) = (i32::MAX, i32::MIN);
        for dz in -2..=2 {
            for dx in -2..=2 {
                let h = self.sample(extent, x + dx * reach, z + dz * reach);
                lo = lo.min(h);
                hi = hi.max(h);
            }
        }
        hi - lo
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_seed_raises_the_same_land() {
        assert_eq!(Relief::generate(7), Relief::generate(7));
        assert_ne!(Relief::generate(7), Relief::generate(8));
    }

    #[test]
    fn heights_stay_in_band_and_vary() {
        let relief = Relief::generate(4_242);
        let mut distinct = std::collections::BTreeSet::new();
        for z in (-512..=512).step_by(64) {
            for x in (-512..=512).step_by(64) {
                let h = relief.sample(512, x, z);
                assert!((0..=CEILING).contains(&h));
                distinct.insert(h);
            }
        }
        assert!(distinct.len() > 8, "a landscape, not a plate: {distinct:?}");
    }

    #[test]
    fn some_of_the_world_is_wet_and_most_is_not() {
        for seed in [1u64, 99, 4_242] {
            let relief = Relief::generate(seed);
            let mut wet = 0;
            let mut total = 0;
            for z in (-512..=512).step_by(32) {
                for x in (-512..=512).step_by(32) {
                    total += 1;
                    if relief.sample(512, x, z) < relief.sea {
                        wet += 1;
                    }
                }
            }
            assert!(wet > 0, "seed {seed}: no water at all");
            assert!(wet * 2 < total, "seed {seed}: mostly ocean");
        }
    }
}
