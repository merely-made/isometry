// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Writes the interchange fixture: one critter, grown by incorporation,
//! exported as `mesocosm.body/v0` bytes.
//!
//! The literal file protects persisted v0 data. The live drift receipt is
//! `shared/wing-integration`, which invokes this same producer path and feeds
//! the current bytes to the tabletop reader and baker. Keep the fixture as a
//! deliberate compatibility record rather than treating it as that live proof.
//!
//! Regenerate with:
//!
//! ```text
//! cargo run -p mesocosm-mesh --example emit_profile
//! ```
//!
//! then copy `fixtures/critter.body` into isometry's
//! `crates/isometry-voxel/tests/fixtures/` only as an explicit persisted-data
//! compatibility update. The live integration receipt remains independent.

use std::{fs, path::PathBuf};

use mesocosm_core::{
    Attachment, BodyDocument, Origin, PartId, Provenance, SpeciesId, VolumeRef, Yaw,
};
use mesocosm_mesh::{BodyProfile, Volume, VolumeMap};

fn main() {
    let (body, volumes) = grown();
    let profile = BodyProfile::of(&body, &volumes).expect("the fixture body is placeable");
    let bytes = profile.to_bytes().expect("a profile is always encodable");

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures");
    fs::create_dir_all(&path).expect("the fixture directory is writable");
    let file = path.join("critter.body");
    fs::write(&file, &bytes).expect("the fixture is writable");

    println!("wrote {} ({} bytes)", file.display(), bytes.len());
    println!(
        "  species {}, {} parts ({} incorporated), grid {:?} at {:?}",
        profile.species,
        profile.parts.len(),
        profile.parts.iter().filter(|p| p.is_incorporated()).count(),
        profile.size,
        profile.origin,
    );
}

/// A critter that ate two others: a founding trunk, a limb taken from one
/// species and a plate taken from another. Three parts is the smallest body
/// that shows founding and incorporated material side by side and still has a
/// part with a non-zero yaw.
fn grown() -> (BodyDocument, VolumeMap) {
    let mut body = BodyDocument::new(SpeciesId(7), VolumeRef::from_tag(1), 4_000, [2, 3, 2]);

    body.attach(
        VolumeRef::from_tag(2),
        900,
        [1, 1, 3],
        Attachment {
            parent: body.root,
            offset: [3, 0, 0],
            yaw: Yaw::Zero,
        },
        Provenance {
            origin: Origin::Incorporated {
                from_species: SpeciesId(42),
                from_part: PartId(1),
            },
            epoch: 3,
        },
    )
    .expect("the limb attaches");

    body.attach(
        VolumeRef::from_tag(3),
        1_400,
        [2, 1, 1],
        Attachment {
            parent: body.root,
            offset: [0, 4, 0],
            yaw: Yaw::Quarter,
        },
        Provenance {
            origin: Origin::Incorporated {
                from_species: SpeciesId(11),
                from_part: PartId(0),
            },
            epoch: 7,
        },
    )
    .expect("the plate attaches");

    let mut volumes = VolumeMap::default();
    volumes.insert(VolumeRef::from_tag(1), Volume::solid([5, 7, 5], 1));
    volumes.insert(VolumeRef::from_tag(2), Volume::solid([3, 3, 7], 2));
    volumes.insert(VolumeRef::from_tag(3), Volume::solid([5, 3, 3], 3));
    (body, volumes)
}
