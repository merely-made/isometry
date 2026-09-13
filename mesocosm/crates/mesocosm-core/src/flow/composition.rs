// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Material evidence emitted by an accepted material mutation.
//! Absence means the route still reports scalar amounts, not untyped matter.

use super::FlowEvent;
use crate::matter::{Material, Stock};
use serde::{Deserialize, Serialize};

pub use crate::matter::receipt::Conversion;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Composition {
    pub input: Stock,
    pub output: Stock,
    pub conversion: Option<Conversion>,
}

impl Composition {
    pub(super) fn untyped(amount_mg: u64) -> Self {
        let stock = Stock::single(Material::Untyped, amount_mg);
        Self {
            input: stock,
            output: stock,
            conversion: None,
        }
    }
}

impl FlowEvent {
    /// A producer turns untyped soil into its own provenance-bearing tissue.
    pub fn synthesized(mut self, output: Stock) -> Self {
        assert_eq!(self.source, super::Account::Soil);
        assert_eq!(self.destination, super::Account::Substance);
        assert_eq!(output.total(), u128::from(self.amount_mg));
        assert_eq!(output.amount(Material::Untyped), 0);
        assert_eq!(output.amount(Material::Consumer), 0);
        assert_eq!(output.amount(Material::Decomposer), 0);
        self.composition = Some(Composition {
            input: Stock::single(Material::Untyped, self.amount_mg),
            output,
            conversion: Some(Conversion::Synthesis),
        });
        self
    }

    /// An unchanged lot moved between the named accounts.
    pub fn with_stock(mut self, stock: Stock) -> Self {
        assert_eq!(stock.total(), u128::from(self.amount_mg));
        self.composition = Some(Composition {
            input: stock,
            output: stock,
            conversion: None,
        });
        self
    }

    /// Completed return to soil, preserving the exact consumed input in the record.
    pub fn mineralized(mut self, stock: Stock) -> Self {
        assert_eq!(stock.total(), u128::from(self.amount_mg));
        if stock.amounts()[1..].iter().all(|amount| *amount == 0) {
            return self.with_stock(stock);
        }
        assert_eq!(self.destination, super::Account::Soil);
        assert!(matches!(
            self.source,
            super::Account::Substance | super::Account::Soil
        ));
        self.composition = Some(Composition {
            input: stock,
            output: Stock::single(Material::Untyped, self.amount_mg),
            conversion: Some(Conversion::Mineralization),
        });
        self
    }

    /// The accepted reserve credit, with its actual consumed tissue.
    pub fn digested(mut self, stock: Stock) -> Self {
        assert_eq!(stock.total(), u128::from(self.amount_mg));
        if stock.amounts()[1..].iter().all(|amount| *amount == 0) {
            return self.with_stock(stock);
        }
        assert_eq!(self.source, super::Account::Substance);
        assert_eq!(self.destination, super::Account::Reserve);
        self.composition = Some(Composition {
            input: stock,
            output: Stock::single(Material::Untyped, self.amount_mg),
            conversion: Some(Conversion::Digestion),
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matter::receipt::{Address, Book, Receipt, reconcile};
    use crate::{OrganismId, PartId};

    #[test]
    fn mixed_reserve_conversion_reconciles_with_the_existing_typed_book() {
        let source = Address::Part(OrganismId(1), PartId(0));
        let target = Address::Reserve(OrganismId(2));
        let input = Stock::from_amounts([3, 5, 7, 11]);
        let output = Stock::single(Material::Untyped, 26);
        let before = Book::from([(source, input)]);
        let after = Book::from([(source, Stock::EMPTY), (target, output)]);
        let receipt = Receipt::Conversion {
            kind: Conversion::Digestion,
            from: source,
            to: target,
            input,
            output,
        };
        assert_eq!(reconcile(&before, &after, &[receipt]), Ok(()));
    }

    #[test]
    fn synthesis_reconciles_untyped_soil_as_producer_tissue() {
        let soil = Address::Soil([0, 0, 0]);
        let tissue = Address::Part(OrganismId(2), PartId(0));
        let before = Book::from([(soil, Stock::single(Material::Untyped, 26))]);
        let after = Book::from([
            (soil, Stock::EMPTY),
            (tissue, Stock::single(Material::Producer, 26)),
        ]);
        let receipt = Receipt::Conversion {
            kind: Conversion::Synthesis,
            from: soil,
            to: tissue,
            input: Stock::single(Material::Untyped, 26),
            output: Stock::single(Material::Producer, 26),
        };
        assert_eq!(reconcile(&before, &after, &[receipt]), Ok(()));
    }
}
