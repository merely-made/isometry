// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Typed NPC feeding receipts.

use super::*;

use crate::body::{SpeciesId, VolumeRef};
use crate::development::PartPalette;
use crate::flow::{Account, Ledger, Process, Records};
use crate::matter::Stock;
use crate::organism::{Kingdom, OrganismId, step};
use crate::places::Soil;
use crate::process::{IntakePort, NisKind};
use crate::rng::Rng;
use crate::species::Lineages;

fn eater(mass_mg: u64) -> Organism {
    Organism::founding(
        OrganismId(1),
        SpeciesId(1),
        Kingdom::Consumer,
        VolumeRef::from_tag(1),
        [5, 5, 5],
        [0, 0, 0],
        mass_mg,
    )
}

#[test]
fn fed_body_keeps_an_exact_mixed_lot() {
    let mut organism = eater(100);
    organism.energy_mg = organism.mass_ceiling_mg();
    let meal = Stock::from_amounts([1, 2, 3, 4]);

    let landed = earn_stock(&mut organism, meal);

    assert_eq!(landed.body_stock, meal);
    assert_eq!(landed.reserve_stock, Stock::EMPTY);
    assert_eq!(
        organism.phenotype.part_stock(organism.body().root),
        Some(&Stock::from_amounts([100, 2, 3, 4]))
    );
}

#[test]
fn hungry_body_marks_its_exact_reserve_intake_as_digested() {
    let mut organism = eater(100);
    organism.energy_mg = 0;
    let meal = Stock::from_amounts([0, 3, 3, 3]);

    let landed = earn_stock(&mut organism, meal);

    assert_eq!(landed.reserve_stock, meal);
    assert_eq!(landed.body_stock, Stock::EMPTY);
    assert_eq!(landed.spilled_stock, Stock::EMPTY);
    assert_eq!(organism.energy_mg, 9);
}

#[test]
fn full_body_returns_the_exact_overflow_mixture() {
    let mut organism = eater(100);
    let ceiling = organism.mass_ceiling_mg();
    let fill = organism.gain_mass(ceiling - organism.biomass_mg());
    assert_eq!(fill, 0);
    organism.energy_mg = ceiling;
    let meal = Stock::from_amounts([1, 2, 3, 4]);

    let landed = earn_stock(&mut organism, meal);

    assert_eq!(landed.body_stock, Stock::EMPTY);
    assert_eq!(landed.reserve_stock, Stock::EMPTY);
    assert_eq!(landed.spilled_stock, meal);
}

#[test]
fn npc_bite_records_the_mixed_lot_it_preserves() {
    let mut eater = Organism::founding(
        OrganismId(1),
        SpeciesId(1),
        Kingdom::Consumer,
        VolumeRef::from_tag(1),
        [8, 2, 2],
        [0, 0, 0],
        300,
    );
    eater.energy_mg = eater.mass_ceiling_mg();
    let jaw = eater.body().mouth_part().expect("the consumer has a mouth");
    let support = eater
        .phenotype
        .part_port(jaw)
        .expect("the seeded mouth is active")
        .support()
        .expect("the seeded mouth has support");
    assert!(eater.phenotype.declare_port(
        jaw,
        IntakePort::live(NisKind::Producer).supported_by(support),
    ));
    let mut prey = Organism::founding(
        OrganismId(2),
        SpeciesId(2),
        Kingdom::Producer,
        VolumeRef::from_tag(2),
        [4, 4, 4],
        [1, 0, 0],
        300,
    );
    let prey_root = prey.body().root;
    prey.phenotype
        .replace_part_stock(prey_root, Stock::from_amounts([0, 100, 100, 99]))
        .expect("the mixed root retains its scalar mass");
    let mut world = vec![eater, prey];
    let mut lineages = Lineages::new();
    lineages.found(SpeciesId(1));
    lineages.found(SpeciesId(2));
    let mut events = Vec::new();
    let mut flows = Ledger::default();
    flows.open(0);
    let mut rng = Rng::from_seed(7);
    let mut next_id = 3;
    let mut soil = Soil::seeded(8, 100_000);

    step(
        &mut world,
        &mut next_id,
        &mut rng,
        &mut Records::new(0, None, &mut events, &mut flows),
        &lineages,
        PartPalette::primitive(),
        &mut soil,
    );

    let feeding = flows
        .records()
        .iter()
        .map(|record| record.record)
        .find(|flow| {
            flow.process == Process::Feeding
                && flow.source == Account::Substance
                && flow.destination == Account::Substance
        })
        .expect("the nearby NPCs exchange a body bite");
    let composition = feeding.composition.expect("a body bite retains its lot");
    assert_eq!(composition.input, composition.output);
    assert!(composition.input.amounts()[1..].iter().any(|mg| *mg > 0));
}
