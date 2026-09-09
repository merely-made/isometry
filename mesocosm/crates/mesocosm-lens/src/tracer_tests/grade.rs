// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The retro grade's step ladders, on the ground's faces.
//!
//! The grade quantises per channel, which is three independent ramps rather
//! than a palette, so a colour crosses their boundaries at three different
//! brightnesses. For a level section that is invisible: the faces it draws are
//! dark and their channels sit far apart. Tilt the camera and the ground's top
//! faces come into view lit from above and then mixed toward a cool fog, and
//! the ordering breaks — a soil top face walked olive, grey, grey, then
//! pink-violet as the fog bands took it.
//!
//! Ground seen from above now climbs the same ladder along its own colour line
//! instead, so a step is a step in light and never a step in hue. These are the
//! two halves of that claim, rendered rather than reasoned: the tilted section
//! keeps the ground in its materials' families, and the level section is
//! untouched, because the gate wants a downward ray and a level section's rays
//! have no vertical component at all.

use crate::{BrickFrameInput, BrickMap, BrickRevision, BrickTracer, Grade, TraceCamera};

use super::ground;

/// The section's own numbers, so these render what the terrarium renders.
const SLAB_DEPTH: f32 = 16.0;
const HALF_HEIGHT: f32 = 28.0;
const PALETTE: u32 = 3;
/// Twenty degrees off `-z` on both free rotations — the shipped oblique.
const TILT: f32 = 20.0;

/// Renders the ground under an orthographic slab pointed `forward`, graded
/// retro, and hands back the frame as RGB triples.
///
/// **The ordered dither is off**, and only here. It is a per-pixel offset that
/// scatters individual pixels onto neighbouring rungs on purpose — that is the
/// grain, and it is what the section is supposed to look like — but it means a
/// handful of pixels of any frame sit a rung off the colour their surface
/// actually graded to. These tests are about which ladder a surface climbs, so
/// they read the ladder rather than the speckle over it.
fn section(forward: [f32; 3]) -> Option<Vec<[u8; 3]>> {
    let ground = ground();
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let mut tracer = BrickTracer::headless(192, 128)?;
    let top = ground.surface(0, 0).unwrap_or(0) as f32;
    let camera = TraceCamera::orthographic_slab(
        [0.0, top, 0.0],
        forward,
        [0.0, 1.0, 0.0],
        HALF_HEIGHT,
        1.5,
        SLAB_DEPTH,
    )
    .expect("a slab camera");
    let grade = Grade {
        dither: 0.0,
        ..Grade::retro(PALETTE)
    };
    let capture = tracer
        .capture(BrickFrameInput::for_camera(
            &map,
            BrickRevision(ground.revision()),
            camera,
            &grade,
        ))
        .expect("a graded section");
    Some(
        capture
            .pixels
            .chunks_exact(4)
            .map(|texel| [texel[0], texel[1], texel[2]])
            .collect(),
    )
}

/// The tilted section, which is the shipped one.
fn oblique() -> Option<Vec<[u8; 3]>> {
    let (yaw, pitch) = (TILT.to_radians(), TILT.to_radians());
    section([
        -yaw.sin() * pitch.cos(),
        -pitch.sin(),
        -yaw.cos() * pitch.cos(),
    ])
}

/// One rung of the ladder, in the bytes a frame actually holds. The per-channel
/// steps land on 0, 64, 127, 191 and 255, so a whole rung apart is 64.
const RUNG: i32 = 64;

/// **The artefact, as an invariant.** Green below *both* red and blue is the
/// violet signature: no surface in the section is that shape — soil runs red
/// over green over blue, rock runs blue over green over red, and the fog and
/// sky are neutral or cool.
///
/// A little of it is honest and unavoidable, because mixing a warm soil toward
/// a cool fog carries blue past green while red is still ahead of both; a
/// fogged top face is measurably a few counts that way and reads as the warm
/// neutral it is. What is not honest is a whole rung of it, which is what
/// three independent channel ramps produce when they step red and blue and
/// leave green behind — the pink-violet `(191, 127, 191)` the ground's top
/// faces wore the first time the camera tilted is exactly one rung. So the
/// invariant is the size of the split rather than its presence, and half a
/// rung sits between the two by a wide margin either way.
#[test]
fn a_tilted_section_never_splits_the_grounds_green_off_by_a_rung() {
    let Some(pixels) = oblique() else {
        eprintln!("no adapter; skipping the tilted grade receipt");
        return;
    };
    let split = |[r, g, b]: [u8; 3]| i32::from(r).min(i32::from(b)) - i32::from(g);
    let worst = pixels.iter().copied().max_by_key(|texel| split(*texel));
    let worst = worst.expect("a frame");
    assert!(
        split(worst) * 2 < RUNG,
        "{worst:?} splits green {} counts below both neighbours",
        split(worst)
    );
}

/// The positive control the assertion above needs, because an empty frame
/// would pass it. Soil lit from above has to actually reach the screen and
/// read as soil: warm, with red clearly over blue rather than the neutral grey
/// a fully fogged face collapses to.
#[test]
fn a_tilted_section_shows_soil_lit_from_above_as_soil() {
    let Some(pixels) = oblique() else {
        eprintln!("no adapter; skipping the tilted grade control");
        return;
    };
    let warm = pixels
        .iter()
        .filter(|[r, g, b]| r > g && g > b && u32::from(*r) >= u32::from(*b) + 48)
        .count();
    assert!(
        warm * 200 > pixels.len(),
        "only {warm} of {} pixels read as lit soil",
        pixels.len()
    );
}

/// **The no-op, proven rather than asserted.** An orthographic section marches
/// its forward, and a level section's forward has no vertical component, so
/// the second ladder cannot reach a pixel of one — and every channel of a
/// level frame therefore still lands on the per-channel ladder's rungs. The
/// hue ladder does not land on them, so this fails the moment a level frame
/// takes the new path. It is the assertion that caught the first gate, which
/// read the face normal alone and so also caught the fabricated `+y` the DDA
/// hands back for a ray that begins inside solid ground.
#[test]
fn a_level_section_still_lands_on_every_rung_of_the_per_channel_ladder() {
    let Some(pixels) = section([0.0, 0.0, -1.0]) else {
        eprintln!("no adapter; skipping the level grade receipt");
        return;
    };
    // floor(c * 5) / 4 over 0..=1, written out: the only values a per-channel
    // frame can hold. The last rung overflows the range and the texture
    // clamps it.
    let rungs = [0u8, 64, 127, 191, 255];
    let stray = pixels
        .iter()
        .filter(|texel| texel.iter().any(|c| !rungs.contains(c)))
        .copied()
        .collect::<Vec<_>>();
    assert!(
        stray.is_empty(),
        "{} level pixels are off the per-channel ladder, e.g. {:?}",
        stray.len(),
        &stray[..stray.len().min(4)]
    );
}
