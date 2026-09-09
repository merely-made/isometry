// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Grows a critter headlessly and writes frames, using the host's real
//! placement policy.
//!
//! Exists so the growth policy can be looked at without opening a window,
//! which matters when a window is already open: on Windows a running instance
//! holds the binary and the linker cannot replace it.
//!
//! ```text
//! cargo run -p mesocosm-genet --example grow -- <output-dir> [meals]
//! ```

use mesocosm_core::{Placement, World};
use mesocosm_mesh::{VolumeSource, flatten, mesh_body};
use mesocosm_render::{Camera, Renderer, SceneItem};

use mesocosm_genet::fixture;

const SIZE: u32 = 640;

fn main() {
    let mut args = std::env::args().skip(1);
    let dir: std::path::PathBuf = args.next().unwrap_or_else(|| ".".into()).into();
    let meals: usize = args.next().and_then(|v| v.parse().ok()).unwrap_or(18);
    std::fs::create_dir_all(&dir).expect("output directory");

    let volumes = fixture::volumes();
    let renderer = Renderer::headless(SIZE, SIZE).expect("a GPU adapter");

    let mut world = World::new(0x00A7_7AC4, 90);
    let mut eaten = 0;

    for _ in 0..meals {
        let Some(target) = fixture::reachable(&world) else {
            break;
        };
        let intent = fixture::metabolize(&world, target, &volumes, Placement::Planned);
        if matches!(
            world.apply(intent),
            mesocosm_core::Outcome::Incorporated { .. }
        ) {
            eaten += 1;
        }
    }

    let mesh = mesh_body(world.body().unwrap(), &volumes).expect("every part resolves");
    let (min, max) = mesh.bounds().expect("geometry");
    let camera = Camera::framing(min, max, 1.0);
    let frame = renderer
        .render_scene(&[SceneItem::new(&mesh, [0, 0, 0])], &camera)
        .expect("render");

    let path = dir.join("06_grown.png");
    let file = std::fs::File::create(&path).expect("create png");
    let mut encoder = png::Encoder::new(file, frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("header")
        .write_image_data(&frame.pixels)
        .expect("data");

    // The receipt that matters: no voxel was lost to an overlap.
    let flat = flatten(world.body().unwrap(), &volumes).expect("flatten");
    let expected: usize = world
        .body()
        .unwrap()
        .parts
        .iter()
        .filter_map(|p| volumes.volume(p.volume))
        .map(|v| v.solid_count())
        .sum();

    // Lateral balance is the check that matters for a bilateral plan: a
    // mirrored body keeps its centre of mass on the midline.
    let centre = world.body().unwrap().centre_of_mass();
    let pairs = world
        .body()
        .unwrap()
        .parts
        .iter()
        .filter(|p| {
            let Some(at) = world.body().unwrap().world_offset(p.id) else {
                return false;
            };
            at[0] != 0
        })
        .count();

    // Let the enclosure run on its own for a while, with the critter idle.
    // This is the thing that separates an ecology from a field of pickups:
    // it goes somewhere whether or not anyone is playing.
    let before_alive = world.living().count();
    for _ in 0..600 {
        world.apply(mesocosm_core::Intent::Idle);
    }
    let after_alive = world.living().count();
    println!("left alone for 600 ticks: {before_alive} alive -> {after_alive} alive");

    // What the enclosure did on its own while the critter was eating.
    let alive = world.living().count();
    let carrion = world
        .organisms
        .iter()
        .filter(|o| o.stage == mesocosm_core::Stage::Carrion)
        .count();
    let producers = world
        .living()
        .filter(|o| o.kingdom() == mesocosm_core::Kingdom::Producer)
        .count();
    println!(
        "enclosure: {alive} alive ({producers} producers), {carrion} carrion,          {} total ever minted",
        world.organisms.len()
    );

    // Who is telling the truth. The player never sees this list; it exists to
    // prove the world contains liars in both directions.
    let warning = world
        .living()
        .filter(|o| o.signal == mesocosm_core::Signal::Warning)
        .count();
    let armed = world.living().filter(|o| o.venom_mg > 0).count();
    let bluffers = world
        .living()
        .filter(|o| o.signal == mesocosm_core::Signal::Warning && o.venom_mg == 0)
        .count();
    let traps = world
        .living()
        .filter(|o| o.signal == mesocosm_core::Signal::Plain && o.venom_mg > 0)
        .count();
    println!("signals: {warning} warn, {armed} armed -> {bluffers} bluffing, {traps} trapping");

    println!(
        "ate {eaten}, body has {} parts",
        world.body().unwrap().len()
    );
    // With pivots the root is centred on the origin, so the midline is zero.
    let midline = 0;
    println!(
        "centre of mass {centre:?}; midline x={midline} -> {}",
        if centre[0] == midline {
            "balanced"
        } else {
            "DRIFTING"
        }
    );
    println!("{pairs} parts sit off the midline");
    println!("mass {} mg", world.total_mass_mg());
    println!(
        "voxels: {} placed, {} expected{}",
        flat.volume.solid_count(),
        expected,
        if flat.volume.solid_count() == expected {
            " (no overlap)"
        } else {
            " (OVERLAP: parts are stacking)"
        }
    );
    println!("wrote {} ({} px drawn)", path.display(), frame.covered());
}
