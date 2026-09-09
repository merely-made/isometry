// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The terrain probe receipt: the same painted world in both souls, along a
//! flight path, so the dither can be judged in motion rather than from one
//! still.
//!
//! ```text
//! cargo run -p mesocosm-lens --example flyover -- <out_dir>
//! ```

use mesocosm_lens::{Flight, Grade, Lens, maps};

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

    // A flight: forward motion with a slow turn, three frames per soul.
    // Consecutive frames are what the dither-in-motion judgement needs.
    let terrain_at = |x: f32, z: f32| -> f32 {
        let i = (z as u32 % world.side) * world.side + (x as u32 % world.side);
        world.height[i as usize] as f32
    };

    for (name, look) in [
        ("retro", Grade::retro(world.palette.len() as u32)),
        ("clay", Grade::clay()),
    ] {
        for frame in 0..3 {
            let t = frame as f32;
            let (x, z) = (430.0 + t * 8.0, 350.0 + t * 5.0);
            let flight = Flight {
                // High enough to see over the ridge into the next biomes,
                // pitched down so fog depth and borders both read.
                eye: [x, terrain_at(x, z) + 90.0, z],
                yaw: 0.85 + t * 0.03,
                pitch: -0.26,
                fov: 1.15,
                far: 800.0,
            };
            let pixels = lens.render(&world, &flight, &look);
            let path = out.join(format!("16_lens_{name}_{frame}.png"));
            write_png(&path, width, height, &pixels);
            println!("captured {}", path.display());
        }

        // The vista: above the peaks, pitched down, so rolling biomes and
        // fog depth read together. The No Man's Sky moment, if it exists.
        let vista = Flight {
            eye: [430.0, 330.0, 350.0],
            yaw: 0.85,
            pitch: -0.42,
            fov: 1.2,
            far: 900.0,
        };
        let pixels = lens.render(&world, &vista, &look);
        let path = out.join(format!("16_lens_{name}_vista.png"));
        write_png(&path, width, height, &pixels);
        println!("captured {}", path.display());
    }
}
