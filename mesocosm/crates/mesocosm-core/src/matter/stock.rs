// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Fixed, exact material amounts.
//!
//! This is TG2a's accounting primitive. It deliberately has four fixed
//! channels rather than a growable composition collection: untyped returned
//! stock and the three living provenance kinds. World accounts decide where a
//! stock belongs and when a living channel returns to untyped stock.

use serde::{Deserialize, Serialize};

use crate::process::NisKind;

/// One fixed material channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Material {
    /// Matter without retained nis, including fully returned soil stock.
    Untyped,
    Producer,
    Consumer,
    Decomposer,
}

impl Material {
    /// Every channel, in the deterministic tie order used by [`Stock::take`].
    pub const ALL: [Self; 4] = [
        Self::Untyped,
        Self::Producer,
        Self::Consumer,
        Self::Decomposer,
    ];

    /// The channel's stable position in a [`Stock`].
    pub const fn index(self) -> usize {
        match self {
            Self::Untyped => 0,
            Self::Producer => 1,
            Self::Consumer => 2,
            Self::Decomposer => 3,
        }
    }

    /// The living provenance channel for this kind.
    pub const fn nis(kind: NisKind) -> Self {
        match kind {
            NisKind::Producer => Self::Producer,
            NisKind::Consumer => Self::Consumer,
            NisKind::Decomposer => Self::Decomposer,
        }
    }
}

/// An arithmetic failure while combining exact material amounts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockError {
    /// Adding this channel would exceed a milligram counter.
    Overflow { material: Material },
    /// Removing a stock asked for more of this channel than was present.
    Underflow {
        material: Material,
        available_mg: u64,
        requested_mg: u64,
    },
}

/// Four exact material amounts, one for each [`Material`] channel.
///
/// Its array is private so callers cannot create a stock with a missing or
/// reordered channel. [`Self::amounts`] remains available for receipts and
/// serialization uses the same fixed layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stock([u64; 4]);

impl Stock {
    /// An empty stock in every channel.
    pub const EMPTY: Self = Self([0; 4]);

    /// A stock from values in [`Material::ALL`] order.
    pub const fn from_amounts(amounts: [u64; 4]) -> Self {
        Self(amounts)
    }

    /// A stock containing matter in exactly one channel.
    pub const fn single(material: Material, mg: u64) -> Self {
        let mut amounts = [0; 4];
        amounts[material.index()] = mg;
        Self(amounts)
    }

    /// The exact amount held by one channel.
    pub const fn amount(self, material: Material) -> u64 {
        self.0[material.index()]
    }

    /// The channel values in [`Material::ALL`] order.
    pub const fn amounts(self) -> [u64; 4] {
        self.0
    }

    /// The total material, widened so all four `u64` channels can coexist.
    pub const fn total(self) -> u128 {
        self.0[0] as u128 + self.0[1] as u128 + self.0[2] as u128 + self.0[3] as u128
    }

    /// Combines two stocks without silently wrapping any channel.
    pub fn checked_add(self, other: Self) -> Result<Self, StockError> {
        let mut amounts = [0; 4];
        for material in Material::ALL {
            let index = material.index();
            amounts[index] = self.0[index]
                .checked_add(other.0[index])
                .ok_or(StockError::Overflow { material })?;
        }
        Ok(Self(amounts))
    }

    /// Removes every channel in `other`, refusing a partial subtraction.
    pub fn checked_sub(self, other: Self) -> Result<Self, StockError> {
        let mut amounts = [0; 4];
        for material in Material::ALL {
            let index = material.index();
            amounts[index] =
                self.0[index]
                    .checked_sub(other.0[index])
                    .ok_or(StockError::Underflow {
                        material,
                        available_mg: self.0[index],
                        requested_mg: other.0[index],
                    })?;
        }
        Ok(Self(amounts))
    }

    /// Takes up to `want_mg`, retaining each channel's share as closely as
    /// whole milligrams allow.
    ///
    /// Floors each exact share first, then assigns the remaining milligrams to
    /// the largest fractional remainders. Equal remainders use
    /// [`Material::ALL`] order, keeping snapshots and replays deterministic.
    pub fn take(self, want_mg: u64) -> (Self, Self) {
        let total = self.total();
        let wanted = (want_mg as u128).min(total);
        if wanted == 0 || total == 0 {
            return (Self::EMPTY, self);
        }
        if wanted == total {
            return (self, Self::EMPTY);
        }

        let mut taken = [0; 4];
        let mut remainders = [0; 4];
        let mut assigned = 0_u128;
        for material in Material::ALL {
            let index = material.index();
            let product = self.0[index] as u128 * wanted;
            taken[index] = (product / total) as u64;
            remainders[index] = product % total;
            assigned += taken[index] as u128;
        }

        let mut order = [0_usize, 1, 2, 3];
        order.sort_by(|left, right| remainders[*right].cmp(&remainders[*left]));
        for index in order.into_iter().take((wanted - assigned) as usize) {
            taken[index] += 1;
        }

        let taken = Self(taken);
        let remainder = self
            .checked_sub(taken)
            .expect("proportional take never exceeds a channel");
        (taken, remainder)
    }
}

#[cfg(test)]
mod tests {
    use super::{Material, Stock, StockError};

    fn stock(amounts: [u64; 4]) -> Stock {
        Stock(amounts)
    }

    #[test]
    fn checked_add_reports_the_overflowing_channel() {
        let error = Stock::single(Material::Consumer, u64::MAX)
            .checked_add(Stock::single(Material::Consumer, 1))
            .unwrap_err();
        assert_eq!(
            error,
            StockError::Overflow {
                material: Material::Consumer
            }
        );
    }

    #[test]
    fn checked_sub_refuses_partial_removal() {
        let original = stock([4, 3, 2, 1]);
        let error = original.checked_sub(stock([4, 4, 0, 0])).unwrap_err();
        assert_eq!(
            error,
            StockError::Underflow {
                material: Material::Producer,
                available_mg: 3,
                requested_mg: 4,
            }
        );
        assert_eq!(original.amounts(), [4, 3, 2, 1]);
    }

    #[test]
    fn odd_proportional_take_uses_material_order_for_equal_remainders() {
        let original = stock([1, 1, 1, 0]);
        let (taken, remainder) = original.take(2);
        assert_eq!(taken.amounts(), [1, 1, 0, 0]);
        assert_eq!(remainder.amounts(), [0, 0, 1, 0]);
        assert_eq!(taken.total() + remainder.total(), original.total());
    }

    #[test]
    fn take_handles_empty_and_the_full_u64_range() {
        assert_eq!(Stock::EMPTY.take(7), (Stock::EMPTY, Stock::EMPTY));

        let one_full_channel = Stock::single(Material::Untyped, u64::MAX);
        assert_eq!(
            one_full_channel.take(u64::MAX),
            (one_full_channel, Stock::EMPTY)
        );

        let original = stock([u64::MAX, u64::MAX, u64::MAX, u64::MAX]);
        let (taken, remainder) = original.take(u64::MAX);
        assert_eq!(taken.total(), u64::MAX as u128);
        assert_eq!(taken.total() + remainder.total(), original.total());
        assert_eq!(original.take(u64::MAX).0, taken);
    }

    #[test]
    fn taking_more_than_present_returns_every_channel() {
        let original = stock([2, 3, 5, 7]);
        let (taken, remainder) = original.take(u64::MAX);
        assert_eq!(taken, original);
        assert_eq!(remainder, Stock::EMPTY);
    }

    #[test]
    fn serde_round_trip_keeps_all_four_channels() {
        let original = stock([0, 3, 5, 8]);
        let encoded = serde_json::to_string(&original).unwrap();
        let decoded: Stock = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, original);
    }
}
