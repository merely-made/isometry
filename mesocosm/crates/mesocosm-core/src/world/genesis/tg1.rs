// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! TG1 receipts for authored declarations at the worldgen boundary.

use super::*;
use crate::phenotype::BodyPhenotype;
use crate::process::{FeedingMode, NisKind};
use crate::program::{Conditions, Founder};

fn declared_ports(
    phenotype: &BodyPhenotype,
) -> Vec<(crate::body::PartId, crate::process::IntakePort)> {
    phenotype
        .body()
        .living()
        .map(|part| {
            (
                part.id,
                phenotype
                    .mosaic(part.id)
                    .expect("a living part has a mosaic")
                    .port(),
            )
        })
        .collect()
}

#[test]
fn drawn_founding_has_only_geometry_seeded_ports() {
    let world = World::founded(41, FOUNDERS, Founding::Drawn).expect("drawn founding is valid");
    for organism in &world.organisms {
        let seeded = BodyPhenotype::seed(organism.body().clone());
        assert_eq!(
            declared_ports(&organism.phenotype),
            declared_ports(&seeded),
            "drawn founder {:?} received an authored port override",
            organism.id
        );
    }
}

#[test]
fn the_spaced_roster_controlled_browser_is_an_omnivore() {
    let world = World::founded(41, FOUNDERS, Founding::SpacedRoster)
        .expect("the spaced roster is admissible");
    assert_eq!(
        world
            .controlled()
            .expect("a founder is controlled")
            .feeding_mode(),
        FeedingMode::Omnivore
    );
}

#[test]
fn a_forked_controlled_lineage_preview_keeps_its_two_ports() {
    let mut world = World::founded(41, FOUNDERS, Founding::SpacedRoster)
        .expect("the spaced roster is admissible");
    let parent = world.controlled().expect("a founder is controlled");
    let (species, mass_mg, seed) = (parent.species, parent.biomass_mg(), parent.development_seed);
    let child_species = world
        .lineages_mut()
        .fork(species, "browser child".into(), 1)
        .expect("the controlled lineage is registered");
    let preview = world
        .lineages()
        .get(child_species)
        .expect("the fork is registered")
        .preview(
            world.ruleset(),
            Founder {
                mass_mg,
                palette: world.development_palette(),
                conditions: Conditions {
                    ground_mg: 0,
                    material_mg: 0,
                },
            },
            seed,
        )
        .expect("the forked child preview realizes");

    assert_eq!(
        preview
            .phenotype
            .intake_ports()
            .admits_live(NisKind::Producer),
        true
    );
    assert_eq!(
        preview
            .phenotype
            .intake_ports()
            .admits_live(NisKind::Consumer),
        true
    );
    assert_eq!(FeedingMode::of(&preview.phenotype), FeedingMode::Omnivore);
}
