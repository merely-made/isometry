// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Captures Ground through the retained brick tracer before and after a carve.
//!
//! ```text
//! cargo run -p mesocosm-lens --example trace_ground --release -- <output-dir>
//! ```

use std::path::{Path, PathBuf};

use mesocosm_core::places::{Ground, Places};
use mesocosm_lens::{
    BrickChange, BrickFrameInput, BrickMap, BrickRevision, BrickTracer, Flight, Grade,
};

const WIDTH: u32 = 512;
const HEIGHT: u32 = 384;

fn main() {
    let out = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| "captures".into());
    std::fs::create_dir_all(&out).expect("output directory");

    let grown = Places::grown(4_242, 4, 64);
    let mut ground = Ground::grow(&grown, 64);
    let mut map = BrickMap::from_ground(&ground).expect("the standard ground fits the atlas");
    let mut tracer = BrickTracer::headless(WIDTH, HEIGHT).expect("GPU adapter");
    let (from, to) = tunnel(&ground);
    let flight = flight(from);
    let grade = Grade::clay();

    let before = tracer
        .capture(BrickFrameInput::new(
            &map,
            BrickRevision(ground.revision()),
            &flight,
            &grade,
        ))
        .expect("initial frame");
    write_png(out.join("g2_ground_before.png"), &before.pixels);

    let removed: u32 = (from[0]..=to[0])
        .map(|x| ground.carve([x, from[1], from[2]], 1))
        .sum();
    assert!(removed > 0, "fixture carve removes material");
    let dirty = ground.drain_dirty();
    let slots = map.refresh(&ground, dirty).expect("unchanged brick shape");
    let after = tracer
        .capture(
            BrickFrameInput::new(&map, BrickRevision(ground.revision()), &flight, &grade)
                .changed(BrickChange::Slots(&slots)),
        )
        .expect("carved frame");
    write_png(out.join("g2_ground_carved.png"), &after.pixels);

    println!(
        "G2 brick trace: wall={from:?}..{to:?}, removed={removed}, dirty_slots={}, first_upload_bytes={}, carved_upload_bytes={}, pixels_changed={}",
        slots.len(),
        before.diagnostics.brick_upload_bytes,
        after.diagnostics.brick_upload_bytes,
        before.pixels != after.pixels,
    );
}

fn flight(from: [i32; 3]) -> Flight {
    Flight {
        eye: [
            from[0] as f32 + 0.5,
            from[1] as f32 + 0.4,
            from[2] as f32 + 0.5,
        ],
        yaw: std::f32::consts::FRAC_PI_2,
        pitch: 0.0,
        fov: 0.72,
        far: 20.0,
    }
}

/// Finds a real hill that blocks a horizontal sight-line, then provides the
/// two stances on either side. The carve below bores exactly that sight-line.
fn tunnel(ground: &Ground) -> ([i32; 3], [i32; 3]) {
    for z in -40..40 {
        for x in -40..30 {
            let (Some(a), Some(b)) = (ground.surface(x, z), ground.surface(x + 8, z)) else {
                continue;
            };
            let eye = a.max(b) + 1;
            let from = [x, eye, z];
            let to = [x + 8, eye, z];
            if ground.stands(from, 1) && ground.stands(to, 1) && ground.solid([x + 4, eye, z]) {
                return (from, to);
            }
        }
    }
    panic!("the seeded ground contains a horizontal hill");
}

fn write_png(path: impl AsRef<Path>, pixels: &[u8]) {
    let file = std::fs::File::create(path).expect("capture file");
    let mut encoder = png::Encoder::new(file, WIDTH, HEIGHT);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("PNG header")
        .write_image_data(pixels)
        .expect("PNG pixels");
}
