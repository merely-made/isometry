// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Writes the proof-pair fixtures: one critter somebody played, one nobody
//! did, in the same schema.
//!
//! These are the outbound half of the loop. Isometry reads both, cannot tell
//! them apart, gives each a roster slot and some history, and writes one back
//! — see `tests/homecoming.rs` for the return.
//!
//! Regenerate with:
//!
//! ```text
//! cargo run -p mesocosm-core --example emit_chronicles
//! ```
//!
//! then copy `fixtures/*.chronicle` into isometry's
//! `crates/isometry-campaign/tests/fixtures/`.

use std::{fs, path::PathBuf};

use mesocosm_core::{Chronicle, Intent, OrganismId, Placement, World, generate};

fn main() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    fs::create_dir_all(&dir).expect("the fixture directory is writable");

    for (name, chronicle) in [("played", played()), ("rng", generate(99, 7))] {
        let bytes = chronicle
            .to_bytes()
            .expect("a chronicle is always encodable");
        let file = dir.join(format!("{name}.chronicle"));
        fs::write(&file, &bytes).expect("the fixture is writable");
        println!(
            "wrote {} ({} bytes) - species {}, {} parts, {} incorporated",
            file.display(),
            bytes.len(),
            chronicle.species,
            chronicle.parts.len(),
            chronicle.incorporated_parts(),
        );
    }
}

/// A critter driven through the world until it has eaten. The same hunt
/// `tests/proof_pair.rs` runs, kept here so the fixture is play output rather
/// than a hand-built struct.
fn played() -> Chronicle {
    let mut world = World::new(4_242, 24);

    for _ in 0..400 {
        let Some((prey, at)) = world
            .organisms
            .iter()
            .filter(|organism| {
                organism.biomass_mg() > 0 && Some(organism.id) != world.controlled_id()
            })
            .map(|organism| (organism.id, organism.position))
            .min_by_key(|(_, at): &(OrganismId, [i32; 3])| {
                (0..3)
                    .map(|axis| (at[axis] - world.position().unwrap()[axis]).abs())
                    .max()
                    .unwrap_or(0)
            })
        else {
            break;
        };

        let step = [0, 1, 2].map(|axis| (at[axis] - world.position().unwrap()[axis]).signum());
        if step == [0, 0, 0] {
            world.apply(Intent::Metabolize {
                organism: prey,
                placement: Placement::Planned,
            });
        } else {
            world.apply(Intent::Move { delta: step });
        }
    }

    assert!(
        world.body().unwrap().len() > 1,
        "the played critter actually grew"
    );
    Chronicle::of(world.body().unwrap())
}
