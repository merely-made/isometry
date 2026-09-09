// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The critter probe receipt: a capsule-chain body loping over the marched
//! terrain, in both souls, three gait phases each. The gate it answers:
//! does it read as an animal, not a blob.
//!
//! ```text
//! cargo run -p mesocosm-lens --example lope -- <out_dir>
//! ```

use mesocosm_lens::{CritterPose, Flight, Grade, Lens, critter, maps};

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

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "captures".into());
    let out = std::path::Path::new(&out);

    let (width, height) = (1280, 720);
    let Some(mut lens) = Lens::headless(width, height) else {
        eprintln!("no adapter; the probe needs a GPU");
        std::process::exit(1);
    };
    let world = maps::synthesize(4_242, 1024);

    let ground = |x: f32, z: f32| -> f32 {
        let side = world.side;
        let i = (z.max(0.0) as u32 % side) * side + (x.max(0.0) as u32 % side);
        world.height[i as usize] as f32
    };

    // Find a gentle patch to walk: the flattest 48x48 window on a coarse
    // scan. A chase camera and a cliff face do not mix.
    let mut origin = [430.0, 350.0];
    let mut best = f32::MAX;
    for row in (64..(world.side - 112)).step_by(32) {
        for col in (64..(world.side - 112)).step_by(32) {
            let mut lo = 255u8;
            let mut hi = 0u8;
            for dz in (0..48).step_by(8) {
                for dx in (0..48).step_by(8) {
                    let h = world.height[((row + dz) * world.side + col + dx) as usize];
                    lo = lo.min(h);
                    hi = hi.max(h);
                }
            }
            // A clamped plateau is flat the way a wall is quiet; walk real
            // ground, not the height cap.
            if hi >= 250 || lo <= 5 {
                continue;
            }
            let spread = (hi - lo) as f32;
            if spread < best {
                best = spread;
                // The corner, not the centre: the walk crosses the patch
                // diagonally, so starting at its edge keeps the whole
                // wander on gentle ground.
                origin = [col as f32 + 8.0, row as f32 + 8.0];
            }
        }
    }

    // Walk the critter in along a wander so each captured frame is a real
    // gait phase, not a posed chain.
    let mut body = critter::Body::caterpillar(9, 1.4);
    for step in 0..12 {
        let [x, z] = critter::wander(4_242, origin, step);
        body.step([x, ground(x, z) + 2.2, z], ground);
    }

    // A moss-lineage tint, the golden-angle rule's first entry.
    let tint = [0.36, 0.62, 0.42];

    for (name, look) in [
        ("retro", Grade::retro(world.palette.len() as u32)),
        ("clay", Grade::clay()),
    ] {
        for frame in 0..3 {
            // Advance the walk between frames so consecutive captures are
            // consecutive gait phases: the motion-legibility evidence.
            for step in 0..2 {
                let [x, z] = critter::wander(4_242, origin, 12 + frame * 2 + step);
                body.step([x, ground(x, z) + 2.2, z], ground);
            }
            let pose = CritterPose::from_body(&body, ground, tint);

            // Side-on, slightly above: the pose that shows a gait. The
            // camera stands off the body's flank and looks at its middle.
            let head = body.chain.segments[0].at;
            let tail = body.chain.segments.last().unwrap().at;
            let mid = [
                (head[0] + tail[0]) * 0.5,
                (head[1] + tail[1]) * 0.5,
                (head[2] + tail[2]) * 0.5,
            ];
            let heading = f32::atan2(head[0] - tail[0], head[2] - tail[2]);
            let flank = heading + std::f32::consts::FRAC_PI_2;
            let off = 40.0;
            let mut eye = [
                mid[0] + flank.sin() * off,
                mid[1] + 13.0,
                mid[2] + flank.cos() * off,
            ];
            eye[1] = eye[1].max(ground(eye[0], eye[2]) + 4.0);
            let to_mid = f32::atan2(mid[0] - eye[0], mid[2] - eye[2]);
            let dist = ((mid[0] - eye[0]).powi(2) + (mid[2] - eye[2]).powi(2)).sqrt();
            let flight = Flight {
                eye,
                yaw: to_mid,
                // Aim below the body's midline so the ground it stands on is
                // in frame; a creature without visible footing floats.
                pitch: f32::atan2(mid[1] - 2.0 - eye[1], dist),
                fov: 0.9,
                far: 700.0,
            };

            let pixels = lens.render_with(&world, &flight, &look, Some(&pose));
            let path = out.join(format!("17_critter_{name}_{frame}.png"));
            write_png(&path, width, height, &pixels);
            println!("captured {}", path.display());
        }
    }
}
