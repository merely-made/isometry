// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Typed ordinary-birth receipts.

use super::*;

use crate::body::{SpeciesId, VolumeRef};
use crate::development::PartPalette;
use crate::flow::{Ledger, Records};
use crate::matter::{Material, Stock};
use crate::organism::Kingdom;

fn parent() -> Organism {
    Organism::founding(
        OrganismId(1),
        SpeciesId(1),
        Kingdom::Producer,
        VolumeRef::from_tag(1),
        [8, 8, 8],
        [0, 0, 0],
        1_000,
    )
}

fn records<'a>(
    events: &'a mut Vec<crate::flow::RecordedEvent>,
    flows: &'a mut Ledger,
) -> Records<'a> {
    flows.open(0);
    Records::new(0, None, events, flows)
}

#[test]
fn birth_moves_the_parents_mixed_debit_into_the_realized_child() {
    let mut parent = parent();
    parent.energy_mg = 100;
    let root = parent.body().root;
    parent
        .phenotype
        .replace_part_stock(root, Stock::from_amounts([900, 30, 30, 39]))
        .expect("the root retains its scalar mass");
    let parent_before = parent.phenotype.total_stock().unwrap();
    let expected = parent
        .phenotype
        .clone()
        .spend_stock(parent.biomass_mg() / OFFSPRING_COST);
    let mut organisms = vec![parent];
    let mut lineages = Lineages::new();
    lineages.found(SpeciesId(1));
    let mut next_id = 2;
    let mut rng = Rng::from_seed(7);
    let mut events = Vec::new();
    let mut flows = Ledger::default();

    let child = bear(
        &mut organisms,
        0,
        &mut next_id,
        &mut rng,
        &mut records(&mut events, &mut flows),
        &lineages,
        PartPalette::primitive(),
        None,
    )
    .expect("the default lineage realizes from a quarter of this parent");

    assert_eq!(child.phenotype.total_stock().unwrap(), expected);
    assert_eq!(
        organisms[0]
            .phenotype
            .total_stock()
            .unwrap()
            .checked_add(child.phenotype.total_stock().unwrap())
            .unwrap(),
        parent_before,
        "the child receives exactly the parent's mixed body debit"
    );
    let restored: Organism = crate::snapshot::decode(&crate::snapshot::encode(&child).unwrap())
        .expect("a typed child survives its ordinary snapshot");
    assert_eq!(restored.phenotype.total_stock().unwrap(), expected);
    assert!(
        Material::ALL
            .into_iter()
            .all(|material| expected.amount(material) > 0)
    );
    assert_eq!(child.energy_mg, 100);
    assert_eq!(organisms[0].energy_mg, 0);
    let body = flows
        .records()
        .iter()
        .find(|record| record.record.destination == Account::Substance)
        .expect("birth records its body transfer")
        .record;
    assert_eq!(
        body.composition.expect("typed body transfer").input,
        expected
    );
    let reserve = flows
        .records()
        .iter()
        .find(|record| record.record.destination == Account::Reserve)
        .expect("birth records its reserve transfer")
        .record;
    assert_eq!(
        reserve.composition.expect("untyped reserve transfer").input,
        Stock::single(Material::Untyped, 100)
    );
}

#[test]
fn refused_birth_leaves_parent_identity_and_entropy_unchanged() {
    let mut organisms = vec![parent()];
    let before = organisms.clone();
    let mut next_id = 2;
    let mut rng = Rng::from_seed(7);
    let before_rng = rng;
    let mut events = Vec::new();
    let mut flows = Ledger::default();

    assert!(
        bear(
            &mut organisms,
            0,
            &mut next_id,
            &mut rng,
            &mut records(&mut events, &mut flows),
            &Lineages::new(),
            PartPalette::primitive(),
            None,
        )
        .is_none()
    );
    assert_eq!(organisms, before);
    assert_eq!(next_id, 2);
    assert_eq!(rng, before_rng);
    assert!(events.is_empty());
    assert!(flows.records().is_empty());
}

#[test]
fn distribution_preserves_severed_history_and_refuses_without_a_partial_write() {
    let mut organism = parent();
    let root = organism.body().root;
    let frond = organism
        .body()
        .canopy_parts()
        .next()
        .expect("the producer has a frond");
    organism
        .phenotype
        .replace_part_stock(root, Stock::from_amounts([900, 30, 30, 39]))
        .unwrap();
    organism.phenotype.sever(frond);
    let severed = organism.phenotype.part_stock(frond).copied().unwrap();
    let before = organism.phenotype.clone();

    assert!(
        organism
            .phenotype
            .distribute_stock(Stock::single(Material::Untyped, 998))
            .is_err()
    );
    assert_eq!(organism.phenotype, before);

    organism
        .phenotype
        .distribute_stock(Stock::from_amounts([900, 30, 30, 39]))
        .unwrap();
    assert_eq!(organism.phenotype.part_stock(frond), Some(&severed));
}
