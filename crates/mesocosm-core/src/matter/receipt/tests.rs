// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::matter::{Material, Stock};

fn soil() -> Address {
    Address::Soil([0, 0, 0])
}
fn reserve() -> Address {
    Address::Reserve(crate::OrganismId(1))
}
fn part(id: u16) -> Address {
    Address::Part(crate::OrganismId(1), crate::PartId(u32::from(id)))
}

fn book(entries: impl IntoIterator<Item = (Address, Stock)>) -> Book {
    entries.into_iter().collect()
}

#[test]
fn typed_control_fails_when_scalar_mass_still_matches() {
    let before = book([(soil(), Stock::single(Material::Producer, 10))]);
    let after = book([(soil(), Stock::single(Material::Consumer, 10))]);
    let scalar = [Receipt::Transfer {
        from: soil(),
        to: soil(),
        stock: Stock::single(Material::Untyped, 0),
    }];
    assert!(reconcile(&before, &after, &scalar).is_err());
}

#[test]
fn soil_producer_consumer_decomposer_soil_chain_reconciles() {
    let before = book([
        (soil(), Stock::single(Material::Untyped, 40)),
        (part(1), Stock::EMPTY),
        (part(2), Stock::EMPTY),
        (part(3), Stock::EMPTY),
    ]);
    let after = book([
        (soil(), Stock::single(Material::Untyped, 40)),
        (part(1), Stock::EMPTY),
        (part(2), Stock::EMPTY),
        (part(3), Stock::EMPTY),
    ]);
    let receipts = [
        Receipt::Conversion {
            kind: Conversion::Synthesis,
            from: soil(),
            to: part(1),
            input: Stock::single(Material::Untyped, 10),
            output: Stock::single(Material::Producer, 10),
        },
        Receipt::Conversion {
            kind: Conversion::Digestion,
            from: part(1),
            to: part(2),
            input: Stock::single(Material::Producer, 10),
            output: Stock::single(Material::Consumer, 10),
        },
        Receipt::Conversion {
            kind: Conversion::Digestion,
            from: part(2),
            to: part(3),
            input: Stock::single(Material::Consumer, 10),
            output: Stock::single(Material::Decomposer, 10),
        },
        Receipt::Conversion {
            kind: Conversion::Mineralization,
            from: part(3),
            to: soil(),
            input: Stock::single(Material::Decomposer, 10),
            output: Stock::single(Material::Untyped, 10),
        },
    ];
    assert!(reconcile(&before, &after, &receipts).is_ok());
}

#[test]
fn reserve_and_graft_are_unchanged_typed_transfers() {
    let before = book([
        (reserve(), Stock::single(Material::Untyped, 8)),
        (part(1), Stock::single(Material::Consumer, 12)),
        (part(2), Stock::EMPTY),
    ]);
    let after = book([
        (reserve(), Stock::single(Material::Untyped, 4)),
        (
            part(1),
            Stock::single(Material::Consumer, 2)
                .checked_add(Stock::single(Material::Untyped, 4))
                .unwrap(),
        ),
        (part(2), Stock::single(Material::Consumer, 10)),
    ]);
    let receipts = [
        Receipt::Transfer {
            from: reserve(),
            to: part(1),
            stock: Stock::single(Material::Untyped, 4),
        },
        Receipt::Transfer {
            from: part(1),
            to: part(2),
            stock: Stock::single(Material::Consumer, 10),
        },
    ];
    assert!(reconcile(&before, &after, &receipts).is_ok());
}

#[test]
fn conversion_rejects_untyped_relabel_and_mass_loss() {
    let untyped = Stock::single(Material::Untyped, 10);
    assert!(matches!(
        validate_conversion(
            Conversion::Digestion,
            soil(),
            part(1),
            untyped,
            Stock::single(Material::Consumer, 10)
        ),
        Err(ReceiptError::InvalidConversion(Conversion::Digestion))
    ));
    assert!(matches!(
        validate_conversion(
            Conversion::Synthesis,
            soil(),
            part(1),
            untyped,
            Stock::single(Material::Producer, 9)
        ),
        Err(ReceiptError::ConversionMassMismatch)
    ));
}

#[test]
fn omitted_destination_and_missing_receipt_are_rejected() {
    let before = book([
        (soil(), Stock::single(Material::Untyped, 10)),
        (part(1), Stock::EMPTY),
    ]);
    let after = book([
        (soil(), Stock::single(Material::Untyped, 9)),
        (part(1), Stock::EMPTY),
    ]);
    let receipts = [Receipt::Transfer {
        from: soil(),
        to: part(2),
        stock: Stock::single(Material::Untyped, 1),
    }];
    assert!(reconcile(&before, &after, &receipts).is_err());
    assert!(reconcile(&before, &after, &[]).is_err());
}

#[test]
fn typed_reserve_is_rejected_at_the_intermediate_step() {
    let before = book([
        (part(1), Stock::single(Material::Producer, 5)),
        (reserve(), Stock::EMPTY),
    ]);
    let receipts = [
        Receipt::Transfer {
            from: part(1),
            to: reserve(),
            stock: Stock::single(Material::Producer, 5),
        },
        Receipt::Transfer {
            from: reserve(),
            to: part(1),
            stock: Stock::single(Material::Producer, 5),
        },
    ];
    assert!(matches!(
        reconcile(&before, &before, &receipts),
        Err(ReceiptError::TypedReserve(_))
    ));
}

#[test]
fn mixed_synthesis_is_rejected() {
    let mixed = Stock::single(Material::Untyped, 2)
        .checked_add(Stock::single(Material::Producer, 1))
        .unwrap();
    assert!(matches!(
        validate_conversion(
            Conversion::Synthesis,
            soil(),
            part(1),
            mixed,
            Stock::single(Material::Producer, 3)
        ),
        Err(ReceiptError::InvalidConversion(Conversion::Synthesis))
    ));
}

#[test]
fn stock_overflow_is_rejected() {
    let before = book([
        (soil(), Stock::single(Material::Untyped, 1)),
        (part(1), Stock::single(Material::Untyped, u64::MAX)),
    ]);
    let receipts = [Receipt::Transfer {
        from: soil(),
        to: part(1),
        stock: Stock::single(Material::Untyped, 1),
    }];
    assert!(matches!(
        replay(&before, &receipts),
        Err(ReceiptError::Stock(_))
    ));
}
