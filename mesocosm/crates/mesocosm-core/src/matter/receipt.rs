// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Typed, host-independent stock receipts (TG2a).
//!
//! This ledger is deliberately separate from `World`. It checks a supplied
//! before/after book and a typed receipt stream, so scalar account tests cannot
//! accidentally certify a channel substitution.
//!
//! These are candidate accounting rules, not the live world's metabolism:
//! synthesis moves pure untyped soil to producer tissue; digestion declares
//! a tissue conversion or an untyped metabolized reserve; mineralization
//! completes a return to soil. Recipes and admission remain the caller's
//! responsibility. In particular, a meal need not convert its entire stock:
//! retained incoming nis travels by Transfer, preserving the future scruple.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::{Stock, StockError};
use crate::{OrganismId, PartId};

/// A bounded account address for typed stock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Address {
    Soil([i32; 3]),
    Reserve(OrganismId),
    Part(OrganismId, PartId),
    Dev,
}

/// A named, admitted channel conversion. There is no generic relabel route.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Conversion {
    Synthesis,
    Digestion,
    Mineralization,
}

/// One typed matter movement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Receipt {
    Transfer {
        from: Address,
        to: Address,
        stock: Stock,
    },
    Conversion {
        kind: Conversion,
        from: Address,
        to: Address,
        input: Stock,
        output: Stock,
    },
}

/// A typed account book.
pub type Book = BTreeMap<Address, Stock>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReceiptError {
    MissingAccount(Address),
    BalanceMismatch(Address),
    Stock(StockError),
    InvalidConversion(Conversion),
    ConversionMassMismatch,
    TypedReserve(Address),
}

/// Reconciles independent typed before/after balances against every receipt.
pub fn reconcile(before: &Book, after: &Book, receipts: &[Receipt]) -> Result<(), ReceiptError> {
    let expected = replay(before, receipts)?;
    validate_book(after)?;
    let addresses = before
        .keys()
        .chain(after.keys())
        .chain(expected.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    for address in addresses {
        let expected_stock = expected.get(&address).copied().unwrap_or(Stock::EMPTY);
        let actual_stock = after.get(&address).copied().unwrap_or(Stock::EMPTY);
        if expected_stock != actual_stock {
            return Err(ReceiptError::BalanceMismatch(address));
        }
    }
    Ok(())
}

/// Applies a receipt stream to a cloned book. A failed stream never mutates
/// the caller's book, and every intermediate book is checked as well.
pub fn replay(before: &Book, receipts: &[Receipt]) -> Result<Book, ReceiptError> {
    validate_book(before)?;
    let mut book = before.clone();
    for receipt in receipts {
        match receipt {
            Receipt::Transfer { from, to, stock } => move_stock(&mut book, *from, *to, *stock)?,
            Receipt::Conversion {
                kind,
                from,
                to,
                input,
                output,
            } => {
                validate_conversion(*kind, *from, *to, *input, *output)?;
                take_from(&mut book, *from, *input)?;
                add_to(&mut book, *to, *output)?;
            },
        }
        validate_book(&book)?;
    }
    Ok(book)
}

fn validate_book(book: &Book) -> Result<(), ReceiptError> {
    for (address, stock) in book {
        if matches!(address, Address::Reserve(_))
            && stock.amounts()[1..].iter().any(|amount| *amount > 0)
        {
            return Err(ReceiptError::TypedReserve(*address));
        }
    }
    Ok(())
}

fn move_stock(
    book: &mut Book,
    from: Address,
    to: Address,
    stock: Stock,
) -> Result<(), ReceiptError> {
    take_from(book, from, stock)?;
    add_to(book, to, stock)
}

fn take_from(book: &mut Book, address: Address, stock: Stock) -> Result<(), ReceiptError> {
    let current = book
        .get(&address)
        .copied()
        .ok_or(ReceiptError::MissingAccount(address))?;
    let remainder = current.checked_sub(stock).map_err(ReceiptError::Stock)?;
    book.insert(address, remainder);
    Ok(())
}

fn add_to(book: &mut Book, address: Address, stock: Stock) -> Result<(), ReceiptError> {
    let current = book.get(&address).copied().unwrap_or(Stock::EMPTY);
    let sum = current.checked_add(stock).map_err(ReceiptError::Stock)?;
    book.insert(address, sum);
    Ok(())
}

fn validate_conversion(
    kind: Conversion,
    from: Address,
    to: Address,
    input: Stock,
    output: Stock,
) -> Result<(), ReceiptError> {
    if input.total() != output.total() {
        return Err(ReceiptError::ConversionMassMismatch);
    }
    let input_amounts = input.amounts();
    let output_amounts = output.amounts();
    match kind {
        Conversion::Synthesis
            if matches!(from, Address::Soil(_))
                && matches!(to, Address::Part(_, _))
                && input_amounts[0] > 0
                && input_amounts[1..].iter().all(|amount| *amount == 0)
                && output_amounts[0] == 0
                && output_amounts[1] > 0
                && output_amounts[2..].iter().all(|amount| *amount == 0) =>
        {
            Ok(())
        },
        Conversion::Digestion
            if matches!(from, Address::Part(_, _))
                && matches!(to, Address::Part(_, _) | Address::Reserve(_))
                && input_amounts[1..].iter().any(|amount| *amount > 0)
                && ((input_amounts[0] == 0
                    && output_amounts[0] == 0
                    && output_amounts[1..].iter().any(|amount| *amount > 0))
                    || (matches!(to, Address::Reserve(_))
                        && output_amounts[0] > 0
                        && output_amounts[1..].iter().all(|amount| *amount == 0))) =>
        {
            Ok(())
        },
        Conversion::Mineralization
            if matches!(to, Address::Soil(_))
                && matches!(from, Address::Part(_, _) | Address::Soil(_))
                && input_amounts[0] == 0
                && input_amounts[1..].iter().any(|amount| *amount > 0)
                && output_amounts[0] > 0
                && output_amounts[1..].iter().all(|amount| *amount == 0) =>
        {
            Ok(())
        },
        _ => Err(ReceiptError::InvalidConversion(kind)),
    }
}

#[cfg(test)]
mod tests;
