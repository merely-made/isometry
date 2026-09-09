// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! TG2's accounting gate: bounded stocks, declared transformations and exact
//! reconciliation. These values are not yet the live World's tissue accounts.
//! Integration must replace scalar storage at its owners, not keep a shadow
//! ledger beside them. TG3's part scruple will use the same stock arithmetic.

pub mod receipt;
pub mod stock;
pub mod transport;

pub use stock::{Material, Stock, StockError};
