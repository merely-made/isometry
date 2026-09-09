// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! DC4's receipt: the eight authored bodies, at the framing the game uses.
//!
//! Every earlier archetype capture was a three-quarter beauty shot. This one
//! traces each body through the **ruled section framing** — an orthographic
//! slab of half-height 28 looking along `-z`, which is what
//! `mesocosm-genet`'s terrarium view is — over the same grown Ground a world
//! founds on. If a body does not read as a critter here it does not read as one
//! in the game.
//!
//! ```text
//! cargo run -p mesocosm-lens --release --example dc4_roster -- <out_dir>
//! ```
//!
//! **Two cuts.** The shipping section looks along `-z`, and `develop_body`
//! chains a body's segments along `+z` — so the terrarium view looks straight
//! down every body's own axis and a thirty-part animal draws as a blob with two
//! side stubs. That is a finding rather than a framing mistake, so each
//! archetype is captured both ways: `dc4_<name>.png` broadside (the slab turned
//! a quarter, which is the only cut that shows a body plan) and
//! `dc4_<name>_axial.png` down the axis, as the game draws it today.
//!
//! Also writes `dc4_roster.png`, a two-row contact sheet of the broadside cuts.

use mesocosm_core::places::{Ground, Places};
use mesocosm_core::{Recipe, Soma, SpeciesId, axis::archetype, develop_body};
use mesocosm_lens::{
    BodyLensProjection, BodyPlacement, BrickFrameInput, BrickMap, BrickRevision, BrickTracer,
    Grade, TraceCamera,
};

/// The host's own framing (`mesocosm-genet::section::SLAB_HALF_HEIGHT`, ruled
/// 2026-08-29) and its slice depth, quoted rather than imported so the lens
/// example does not depend on the vessel.
const SLAB_HALF_HEIGHT: f32 = 28.0;
const SLAB_DEPTH: f32 = 16.0;
/// The retro palette depth the section runs at.
const PALETTE: u32 = 3;

const FULL: (u32, u32) = (1280, 720);

fn write_png(path: &std::path::Path, width: u32, height: u32, pixels: &[u8]) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let file = std::fs::File::create(path).expect("capture file");
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("png header")
        .write_image_data(pixels)
        .expect("png data");
}

/// The middle half of a frame, so the contact sheet crops rather than shrinks:
/// at this framing a body is about a tenth of the frame's height, and halving
/// the pixels leaves nothing to look at.
fn crop_centre(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let (w, h) = (width as usize / 2, height as usize / 2);
    let (ox, oy) = (w / 2, h / 2);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        let src = ((oy + y) * width as usize + ox) * 4;
        out[y * w * 4..(y + 1) * w * 4].copy_from_slice(&pixels[src..src + w * 4]);
    }
    out
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "captures".into());
    let out = std::path::Path::new(&out);

    // The enclosure a world actually founds on, at the shipping seed.
    let grown = Places::grown(0x00A7_7AC4 ^ 0x504C_4143_4553_0001, 3, 64);
    let ground = Ground::grow(&grown, 64);
    let map = BrickMap::from_ground(&ground).expect("atlas capacity");
    let revision = BrickRevision(ground.revision());
    let grade = Grade::retro(PALETTE);

    let Some(mut tracer) = BrickTracer::headless(FULL.0, FULL.1) else {
        eprintln!("no adapter; the receipt needs a GPU");
        std::process::exit(1);
    };
    type Subject = (&'static str, fn() -> Recipe);
    let subjects: [Subject; 8] = [
        ("producer_mat", archetype::producer_mat),
        ("producer_shrub", archetype::producer_shrub),
        ("producer_stalk", archetype::producer_stalk),
        ("consumer_browser", archetype::consumer_browser),
        ("consumer_pursuit", archetype::consumer_pursuit),
        ("consumer_armoured", archetype::consumer_armoured),
        ("decomposer_crust", archetype::decomposer_crust),
        ("decomposer_detritivore", archetype::decomposer_detritivore),
    ];

    // A flat-ish column near the middle, so the slab shows ground rather than a
    // ridge behind every body.
    let (mut site, mut flattest) = ([0i32, 0i32], i32::MAX);
    for z in (-40..40).step_by(4) {
        for x in (-40..40).step_by(4) {
            let mut lo = i32::MAX;
            let mut hi = i32::MIN;
            for dz in -8..=8 {
                for dx in -8..=8 {
                    if let Some(top) = ground.surface(x + dx, z + dz) {
                        lo = lo.min(top);
                        hi = hi.max(top);
                    }
                }
            }
            if lo == i32::MAX || hi - lo >= flattest {
                continue;
            }
            flattest = hi - lo;
            site = [x, z];
        }
    }
    let floor = ground.surface(site[0], site[1]).expect("a grown column") as f32 + 1.0;

    let mut tiles: Vec<Vec<u8>> = Vec::new();
    for (name, recipe) in subjects {
        let recipe = recipe();
        // The authored body rather than an individual: no variance, nothing
        // absent, which is what an archetype *is*.
        let soma = Soma {
            segments: recipe.tagmata.iter().map(|tagma| tagma.segments).collect(),
            absent: Vec::new(),
        };
        let body = develop_body(SpeciesId(2), &recipe, &soma, 10_000, archetype::palette())
            .expect("an archetype develops");
        let placed = BodyPlacement {
            ground: [site[0] as f32, floor, site[1] as f32],
            // One world voxel per body voxel: the scale a founded body is
            // posed at, so the size on screen is the size in the terrarium.
            scale: 1.0,
            tint: [0.36, 0.62, 0.42],
        };
        let pose = BodyLensProjection::project(&body, placed)
            .expect("an archetype fits the lens capsule budget")
            .pose;

        let centre = pose.bounds_centre;
        // Broadside first, then the axial cut the section actually uses.
        for (forward, suffix) in [([-1.0, 0.0, 0.0], ""), ([0.0, 0.0, -1.0], "_axial")] {
            let aspect = FULL.0 as f32 / FULL.1 as f32;
            let camera = TraceCamera::orthographic_slab(
                centre,
                forward,
                [0.0, 1.0, 0.0],
                SLAB_HALF_HEIGHT,
                aspect,
                SLAB_DEPTH,
            )
            .expect("slab camera");
            let frame =
                BrickFrameInput::for_camera(&map, revision, camera, &grade).with_pose(&pose);
            let capture = tracer.capture(frame).expect("frame traces");
            let path = out.join(format!("dc4_{name}{suffix}.png"));
            write_png(&path, FULL.0, FULL.1, &capture.pixels);
            println!(
                "captured {} ({} parts, {} capsules)",
                path.display(),
                body.living().count(),
                pose.capsules.len(),
            );
            if suffix.is_empty() {
                tiles.push(crop_centre(&capture.pixels, FULL.0, FULL.1));
            }
        }
    }

    // Four across, two down, each the middle half of its own frame.
    let (tw, th) = (FULL.0 as usize / 2, FULL.1 as usize / 2);
    let (sheet_w, sheet_h) = (tw * 4, th * 2);
    let mut sheet = vec![0u8; sheet_w * sheet_h * 4];
    for (index, tile) in tiles.iter().enumerate() {
        let (ox, oy) = ((index % 4) * tw, (index / 4) * th);
        for y in 0..th {
            let src = y * tw * 4;
            let dst = ((oy + y) * sheet_w + ox) * 4;
            sheet[dst..dst + tw * 4].copy_from_slice(&tile[src..src + tw * 4]);
        }
    }
    let path = out.join("dc4_roster.png");
    write_png(&path, sheet_w as u32, sheet_h as u32, &sheet);
    println!("captured {}", path.display());
}
