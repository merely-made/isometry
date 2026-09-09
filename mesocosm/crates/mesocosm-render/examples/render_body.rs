// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Renders a grown critter to a PNG, for the judgment a test cannot make.
//!
//! The visual tests assert that a part is *visible*. Whether the result is
//! *legible* is for a person looking at it, so this writes a frame to disk.
//!
//! ```text
//! cargo run -p mesocosm-render --example render_body -- <output-dir>
//! ```

use std::path::{Path, PathBuf};

use mesocosm_core::{Intent, OrganismId, Outcome, PartId, Placement, VolumeRef, World, Yaw};
use mesocosm_mesh::{Volume, VolumeMap, mesh_body};
use mesocosm_render::{Camera, Renderer};

const SIZE: u32 = 512;
const SEED: u64 = 0x00A7_7AC4;

fn volumes() -> VolumeMap {
    let mut map = VolumeMap::new();
    // The founding body: a chunky core.
    map.insert(VolumeRef::from_tag(1), Volume::solid([5, 5, 5], 1));
    // Organism volumes, varied so incorporated parts read as different things.
    for tag in 16..24u8 {
        let size = match tag % 4 {
            0 => [3, 2, 2],
            1 => [2, 4, 2],
            2 => [2, 2, 5],
            _ => [3, 3, 2],
        };
        map.insert(VolumeRef::from_tag(tag), Volume::solid(size, tag));
    }
    map.insert(VolumeRef::from_tag(64), Volume::solid([1, 1, 1], 5));
    map
}

/// The founding body's extent, in voxels. Parts are placed flush against it.
const CORE: i32 = 5;

/// Eats whatever is in reach, placing each new part flush against a different
/// face so the body grows into something with a silhouette.
///
/// Offsets are computed from the eaten volume's size rather than written as
/// constants, because **a part's local origin is its lowest corner, not a
/// pivot**. Attaching flush therefore means knowing how big the part is. See
/// the plan's findings: this is the attachment-frame convention question, and
/// it is the reason the first render of this example had limbs floating in
/// space next to the body instead of joined to it.
fn grow(world: &mut World, meals: usize) -> usize {
    let mut eaten = 0;
    for meal in 0..meals {
        let Some(target) = reachable(world) else {
            break;
        };

        let size = world
            .organisms
            .iter()
            .find(|m| m.id == target)
            .map(|m| organism_extent(m.volume()))
            .unwrap_or([2, 2, 2]);

        // Flush against one of the six faces, cycling.
        let offset = match meal % 6 {
            0 => [CORE, 1, 1],
            1 => [-size[0], 1, 1],
            2 => [1, 1, CORE],
            3 => [1, 1, -size[2]],
            4 => [1, CORE, 1],
            _ => [1, -size[1], 1],
        };

        // Yaw stays zero here for the same reason: rotation turns a part about
        // its corner, so a rotated limb swings off the joint it was flush to.
        if let Outcome::Incorporated { .. } = world.apply(Intent::Metabolize {
            organism: target,
            placement: Placement::Explicit {
                parent: PartId(0),
                offset,
                yaw: Yaw::Zero,
            },
        }) {
            eaten += 1;
        }
    }
    eaten
}

/// Mirrors the sizes in [`volumes`], since the world carries only references.
fn organism_extent(reference: VolumeRef) -> [i32; 3] {
    let tag = reference.0[0];
    match tag % 4 {
        0 => [3, 2, 2],
        1 => [2, 4, 2],
        2 => [2, 2, 5],
        _ => [3, 3, 2],
    }
}

fn reachable(world: &World) -> Option<OrganismId> {
    world
        .organisms
        .iter()
        .filter(|m| (0..3).all(|a| (m.position[a] - world.position().unwrap()[a]).abs() <= 8))
        .map(|m| m.id)
        .min()
}

fn main() {
    let out_dir: PathBuf = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".".to_string())
        .into();
    std::fs::create_dir_all(&out_dir).expect("output directory");

    let volumes = volumes();
    let renderer = match Renderer::headless(SIZE, SIZE) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("no renderer available: {e:?}");
            std::process::exit(1);
        },
    };

    let mut world = World::new(SEED, 60);

    // Frame one: the founding body, before it has eaten anything.
    write_frame(&renderer, &world, &volumes, &out_dir, "01_founding.png");

    let eaten = grow(&mut world, 6);
    println!("incorporated {eaten} parts");

    // Frame two: the same critter after incorporation.
    write_frame(&renderer, &world, &volumes, &out_dir, "02_grown.png");

    println!("body parts:   {}", world.body().unwrap().len());
    println!("total mass:   {} mg", world.total_mass_mg());
    println!(
        "centre of mass: {:?}",
        world.body().unwrap().centre_of_mass()
    );
    for part in world.body().unwrap().incorporated() {
        println!(
            "  part {:?} came from {:?}",
            part.id, part.provenance.origin
        );
    }
}

fn write_frame(renderer: &Renderer, world: &World, volumes: &VolumeMap, dir: &Path, name: &str) {
    let mesh = mesh_body(world.body().unwrap(), volumes).expect("every part resolves");
    let (min, max) = mesh.bounds().expect("the body has geometry");
    let camera = Camera::framing(min, max, 1.0);
    let frame = renderer.render(&mesh, &camera).expect("render");

    let path = dir.join(name);
    let file = std::fs::File::create(&path).expect("create png");
    let mut encoder = png::Encoder::new(file, frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .expect("png header")
        .write_image_data(&frame.pixels)
        .expect("png data");

    println!(
        "wrote {} ({} px drawn of {})",
        path.display(),
        frame.covered(),
        frame.width * frame.height
    );
}
