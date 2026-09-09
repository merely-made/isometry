// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The generator's receipt: one method, several animals.
//!
//! Develops catalogue plans into authoritative part graphs, then renders those
//! graphs through the same V2 projection. If a centipede, an insect, and a
//! snake come out visibly different, the axial recipe is doing the work rather
//! than a renderer sculpt.
//!
//! ```text
//! cargo run -p mesocosm-lens --example menagerie -- <out_dir>
//! ```

use mesocosm_core::{
    PartPalette, PartTemplate, Recipe, RoleShapes, Soma, SpeciesId, VolumeRef,
    axis::{archetype, catalogue},
    develop_body,
};
use mesocosm_lens::{BodyLensProjection, BodyPlacement, Flight, Grade, Lens, maps};

fn palette() -> PartPalette {
    PartPalette {
        mass: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(1),
            half_extent: [2, 2, 2],
        }),
        limb: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(2),
            half_extent: [4, 1, 1],
        }),
        plate: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(3),
            half_extent: [4, 4, 1],
        }),
        sensor: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(4),
            half_extent: [1, 1, 1],
        }),
    }
}

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

    // A stable interior anchor over the flattest patch. Presentation lifts it
    // vertically below, but the horizontal frame remains useful when comparing
    // captures across terrain implementations.
    let mut origin = [430.0, 350.0];
    let mut best = f32::MAX;
    for row in (64..(world.side - 112)).step_by(32) {
        for col in (64..(world.side - 112)).step_by(32) {
            let (mut lo, mut hi) = (255u8, 0u8);
            for dz in (0..48).step_by(8) {
                for dx in (0..48).step_by(8) {
                    let h = world.height[((row + dz) * world.side + col + dx) as usize];
                    lo = lo.min(h);
                    hi = hi.max(h);
                }
            }
            if hi >= 250 || lo <= 5 {
                continue;
            }
            if ((hi - lo) as f32) < best {
                best = (hi - lo) as f32;
                origin = [col as f32 + 8.0, row as f32 + 8.0];
            }
        }
    }

    // The catalogue's reference animals, and — since DC2 — the first *authored*
    // body, framed identically so the two are comparable at a glance. The
    // archetype brings its own palette entries; nothing else's shapes move.
    let subjects: [(&str, Recipe, f32, PartPalette, u64); 6] = [
        (
            "18_plan_centipede",
            catalogue::centipede(14),
            1.1,
            palette(),
            9,
        ),
        ("18_plan_insect", catalogue::insect(), 1.5, palette(), 9),
        ("18_plan_spider", catalogue::spider(), 1.6, palette(), 9),
        (
            "18_plan_tetrapod",
            catalogue::tetrapod(6),
            1.6,
            palette(),
            9,
        ),
        ("18_plan_snake", catalogue::snake(16), 1.2, palette(), 9),
        // Development seed 1 rather than 9: the archetype's receipt should show
        // the whole authored body, and seed 9 is one of the individuals a
        // developmental absence takes an appendage pair from.
        (
            "dc2_browser",
            archetype::consumer_browser(),
            1.6,
            archetype::palette(),
            1,
        ),
    ];
    let look = Grade::clay();
    // This receipt compares anatomy, not terrain placement. Lift the subjects
    // above the synthesized world's highest ridge so no camera ray can hide a
    // long body behind an unrelated mountain.
    let presentation_floor = world
        .height
        .iter()
        .copied()
        .max()
        .map_or(16.0, |height| f32::from(height) + 16.0);

    for (species, (name, plan, scale, palette, soma_seed)) in subjects.into_iter().enumerate() {
        let soma = Soma::develop(&plan, soma_seed);
        let body = develop_body(SpeciesId(species as u32 + 1), &plan, &soma, 10_000, palette)
            .expect("catalogue plan develops into a body graph");
        let placed = BodyPlacement {
            ground: [origin[0], presentation_floor, origin[1]],
            scale,
            tint: [0.36, 0.62, 0.42],
        };
        let projected = BodyLensProjection::project(&body, placed)
            .expect("developed body fits the Lens admission limit");
        let pose = projected.pose;

        // Framed to the body's own size at a shallow three-quarter angle so
        // bilateral appendages do not collapse into one silhouette.
        let mid = pose.bounds_centre;
        let off = pose.bounds_radius * 2.6 + 8.0;
        let eye = [
            mid[0] + off * 0.92,
            mid[1] + off * 0.28,
            mid[2] + off * 0.38,
        ];
        let dist = ((mid[0] - eye[0]).powi(2) + (mid[2] - eye[2]).powi(2)).sqrt();
        let flight = Flight {
            eye,
            yaw: f32::atan2(mid[0] - eye[0], mid[2] - eye[2]),
            pitch: f32::atan2(mid[1] - 2.0 - eye[1], dist),
            fov: 0.9,
            far: 700.0,
        };

        let pixels = lens.render_with(&world, &flight, &look, Some(&pose));
        let path = out.join(format!("{name}.png"));
        write_png(&path, width, height, &pixels);
        println!(
            "captured {} ({} parts, {} segments, {} appendages, complexity {})",
            path.display(),
            body.living().count(),
            plan.segments(),
            plan.appendages(),
            plan.complexity()
        );
    }
}
