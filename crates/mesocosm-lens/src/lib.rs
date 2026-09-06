// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The biosphere lens.
//!
//! Two passes project presentation data supplied by a caller. The **march**
//! renders a heightfield and optional SDF/capsule body; the **grade** applies
//! fog, palette quantisation, and ordered dither. Simulation authority stays
//! outside this crate.
//!
//! [`Lens::encode`] is the live boundary: retained resources, a caller-owned
//! encoder, and a caller-owned target. [`Lens::capture`] is the receipt adapter
//! over that same path.

mod body;
pub mod bricks;
pub mod critter;
pub mod maps;
mod renderer;
mod scene;
mod tracer;

#[cfg(test)]
mod netrender_tests;
#[cfg(test)]
#[path = "tracer_tests/terrain.rs"]
mod terrain_tests;
#[cfg(test)]
mod tracer_tests;

pub use body::{BodyLensProjection, BodyPlacement, BodyProjectionError, BodyRevision, LensPart};
pub use bricks::{BrickMap, BrickMapError, BrickProjectionRevision};
pub use renderer::{
    Capture, DirtyRect, FRAME_FORMAT, FrameDiagnostics, FrameInput, Lens, LensError, MapChange,
    MapRevision,
};
pub use scene::{LensScene, SceneCodecError};
pub use tracer::{
    BrickCapture, BrickChange, BrickDiagnostics, BrickFrameInput, BrickRevision, BrickTraceError,
    BrickTracer, LeasedAtlas, SlabWall, TraceCamera,
};

/// Hard admission limit imposed by the baseline uniform layout.
///
/// **256, and the number is the budget rather than the hardware.** A pose costs
/// `bounds 16 + tint_count 16 + eyes 32 + 32 · capsules` bytes, so this pose is
/// 8 256 B and the whole of [`BrickTracer`]'s frame uniform is 8 464 B —
/// **51.7% of the 16 384 B downlevel WebGL2 binding** the tracer has to fit. The
/// previous 96 spent 21% and was never a hardware ceiling; the headroom runs to
/// roughly 500. Raised for DC3 so a played body carrying an authored archetype
/// plus a long career of meals stays on screen.
pub const MAX_CAPSULES: usize = 256;

/// Bodies the brick tracer's pose roster admits beside the single
/// [`BrickFrameInput::pose`](crate::BrickFrameInput). Members past this are
/// dropped; the caller culls to what its camera shows first.
///
/// **The arithmetic.** The tracer is fragment-only for downlevel reach —
/// WebGL2 has neither compute nor storage buffers — so the roster has to be a
/// uniform, and the binding it has to fit is the *downlevel* one:
/// `Limits::downlevel_webgl2_defaults().max_uniform_buffer_binding_size` is
/// **16 384 bytes**, a quarter of the 65 536 desktop default. `g2_glprobe`
/// requests exactly those limits, so this is a live ceiling, not a hypothesis.
///
/// A pose costs `bounds 16 + tint_count 16 + 32 · capsules` bytes:
///
/// - at [`MAX_CAPSULES`] a pose is 8 256 B (eyes included), so a
///   full-fidelity roster would admit `(16384 − 16) / 8256 = 1` member. One is
///   not an ecology.
/// - roster members are background silhouettes rather than the played body,
///   so they drop the eyes and carry [`MAX_ROSTER_CAPSULES`]: 384 B each.
/// - the binding is then `16 + 40 · 384 = 15 376` B, 93.8% of the downlevel
///   limit (23.5% of the desktop one). That is all 40 members can be given:
///   the budget is `M × (C + 1) ≤ 511` and `40 × 12 = 480`.
///
/// Why 40 and 11 rather than some other split of the same bytes: a genesis
/// enclosure of 60 organisms puts 26–34 of them inside the section's slab
/// (measured across seeds), and a developed body carries a median of 8–19
/// living parts. So the member cap clears observed occupancy with margin, and
/// the capsule budget covers a median body. A slab that ever holds more than
/// 40 truncates, and says so through `BrickDiagnostics::roster_dropped`.
///
/// The played critter is not a roster member. It keeps the single pose and
/// its full [`MAX_CAPSULES`], which is why the split exists at all.
pub const MAX_ROSTER: usize = 40;

/// Capsules — living parts — a roster member carries. Extra ones are dropped
/// **widest first kept**, so a truncated member is its silhouette rather than
/// its head end; the member's bounds sphere still covers the whole pose, so a
/// truncated body is smaller than its bounds and never larger.
pub const MAX_ROSTER_CAPSULES: usize = 11;

/// A look, as data. Worldgen can emit one of these per world or per biome
/// the way it emits a heightmap.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Grade {
    pub fog: [f32; 3],
    /// Distance (0..1 of far) where fog begins.
    pub fog_start: f32,
    /// Palette entries used; 0 disables quantisation (clay).
    pub palette_len: u32,
    /// Ordered-dither strength; 0 disables.
    pub dither: f32,
    /// Fog band count; 0 is smooth.
    pub fog_bands: f32,
    /// Internal render scale denominator: 1 = full res, 4 = quarter res
    /// integer-upscaled, which is most of the retro grain.
    pub downscale: u32,
}

/// Optional terrain presentation supplied by the host.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TerrainAppearance {
    pub soil: [f32; 3],
    pub rock: [f32; 3],
    pub unknown: [f32; 3],
    pub sky: [f32; 3],
    pub underground: [f32; 3],
    /// Centre of the fixed vertical plane used to classify no-hit pixels.
    pub section_centre: [f32; 3],
    /// World Y separating the illustrated sky and underground backgrounds.
    pub clearing_y: f32,
}

impl Grade {
    /// The Comanche soul: starved palette, ordered dither, banded fog,
    /// quarter-resolution grain.
    pub fn retro(palette_len: u32) -> Self {
        Self {
            fog: [0.66, 0.66, 0.72],
            fog_start: 0.35,
            palette_len,
            dither: 0.10,
            fog_bands: 6.0,
            downscale: 4,
        }
    }

    /// The clay soul: full resolution, smooth ramp, smooth fog.
    pub fn clay() -> Self {
        Self {
            fog: [0.72, 0.74, 0.80],
            fog_start: 0.45,
            palette_len: 0,
            dither: 0.0,
            fog_bands: 0.0,
            downscale: 1,
        }
    }
}

/// A first-person camera over the heightfield, in map units.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Flight {
    pub eye: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub far: f32,
}

/// The body in frame, if any, as the shader wants it.
#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CritterPose {
    pub capsules: Vec<critter::Capsule>,
    pub eyes: [[f32; 4]; 2],
    pub bounds_centre: [f32; 3],
    pub bounds_radius: f32,
    pub tint: [f32; 3],
}

impl CritterPose {
    pub fn from_capsules(
        capsules: Vec<critter::Capsule>,
        eyes: [[f32; 4]; 2],
        tint: [f32; 3],
    ) -> Self {
        let mut centre = [0.0f32; 3];
        for capsule in &capsules {
            for (axis, value) in centre.iter_mut().enumerate() {
                *value += (capsule.a[axis] + capsule.b[axis]) * 0.5;
            }
        }
        let n = capsules.len().max(1) as f32;
        for value in &mut centre {
            *value /= n;
        }
        let bounds_radius = capsules
            .iter()
            .flat_map(|c| [(c.a, c.ra), (c.b, c.rb)])
            .map(|(at, r)| {
                let d = [at[0] - centre[0], at[1] - centre[1], at[2] - centre[2]];
                (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() + r
            })
            .fold(0.0, f32::max)
            + 1.0;
        Self {
            capsules,
            eyes,
            bounds_centre: centre,
            bounds_radius,
            tint,
        }
    }

    pub fn from_body(
        body: &critter::Body,
        ground: impl Fn(f32, f32) -> f32,
        tint: [f32; 3],
    ) -> Self {
        let capsules = body.capsules(ground);
        Self::from_capsules(capsules, body.eyes(), tint)
    }
}
