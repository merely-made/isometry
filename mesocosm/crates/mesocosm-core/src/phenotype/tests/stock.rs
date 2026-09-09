// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::matter::{Material, Stock};

#[test]
fn seed_and_scalar_wrappers_keep_untyped_stock() {
    let (mut phenotype, [root, limb, _]) = critter();
    assert_eq!(
        phenotype.part_stock(root),
        Some(&Stock::single(Material::Untyped, 1_000))
    );
    assert!(phenotype.gain_root_mass(25));
    assert_eq!(
        phenotype.part_stock(root),
        Some(&Stock::single(Material::Untyped, 1_025))
    );
    assert_eq!(phenotype.take_part_mass(limb), 200);
    assert_eq!(phenotype.part_stock(limb), Some(&Stock::EMPTY));
    assert!(phenotype.conserves());
}

#[test]
fn typed_stock_moves_as_a_real_mixture() {
    let (mut phenotype, [root, limb, frond]) = critter();
    phenotype
        .replace_part_stock(root, Stock::from_amounts([100, 200, 300, 400]))
        .expect("the root still weighs a gram");
    phenotype
        .gain_root_stock(Stock::from_amounts([10, 20, 30, 40]))
        .expect("typed growth fits");
    assert_eq!(
        phenotype.part_stock(root).copied(),
        Some(Stock::from_amounts([110, 220, 330, 440]))
    );

    let paid = phenotype.spend_stock(500);
    assert_eq!(paid.total(), 500);
    assert!(paid.amount(Material::Producer) > 0);
    assert!(paid.amount(Material::Consumer) > 0);
    assert!(phenotype.conserves());

    let taken = phenotype.take_part_stock(frond);
    assert_eq!(taken, Stock::single(Material::Untyped, 200));
    assert_eq!(phenotype.part_stock(frond), Some(&Stock::EMPTY));
    assert_eq!(phenotype.total_stock().unwrap().total(), 800);
    assert_eq!(phenotype.body().total_mass_mg(), 800);
    assert_eq!(phenotype.part_stock(limb).unwrap().total(), 200);
}

#[test]
fn typed_growth_refuses_overflow_without_a_partial_write() {
    let (mut phenotype, [root, ..]) = critter();
    phenotype
        .replace_part_stock(root, Stock::single(Material::Untyped, u64::MAX))
        .expect_err("a gram root cannot be replaced by max stock");
    let before = phenotype.clone();
    phenotype
        .gain_root_stock(Stock::single(Material::Untyped, u64::MAX))
        .expect_err("the scalar part mass cannot represent the combined stock");
    assert_eq!(phenotype, before);

    let attached = phenotype
        .attach_stock(
            VolumeRef::from_tag(4),
            10,
            Stock::single(Material::Producer, 9),
            [1, 1, 1],
            Attachment {
                parent: root,
                offset: [0, 0, -3],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect_err("a mismatched attachment is rejected before it changes anatomy");
    assert!(matches!(attached, AttachStockError::Mass(_)));
    assert_eq!(phenotype, before);
}

#[test]
fn malformed_phenotype_stock_or_alignment_does_not_restore() {
    let (phenotype, _) = critter();
    let mut mismatched = serde_json::to_value(&phenotype).unwrap();
    mismatched["mosaics"][0]["scruple"] = serde_json::json!([999, 0, 0, 0]);
    assert!(serde_json::from_value::<BodyPhenotype>(mismatched).is_err());

    let mut unpaired = serde_json::to_value(&phenotype).unwrap();
    unpaired["mosaics"].as_array_mut().unwrap().pop();
    assert!(serde_json::from_value::<BodyPhenotype>(unpaired).is_err());
}
