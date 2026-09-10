// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Exact substance held by an anatomical part.
//!
//! A [`Part`](crate::body::Part) retains its scalar mass because anatomy,
//! geometry and older callers read it. The mosaic's scruple is the authority
//! for composition. These methods update both in one transaction and insist
//! their totals agree.

use crate::body::{AttachError, Attachment, PartId, Provenance, VolumeRef};
use crate::matter::{Material, Stock, StockError};

use super::BodyPhenotype;

/// A stock whose total cannot be represented by a part's scalar mass, or does
/// not match that mass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StockMassError {
    UnknownPart(PartId),
    Mismatch {
        part: PartId,
        part_mass_mg: u64,
        stock_total_mg: u128,
    },
    BodyMismatch {
        body_mass_mg: u128,
        stock_total_mg: u128,
    },
    MosaicMismatch {
        parts: usize,
        mosaics: usize,
    },
}

/// Why a typed attachment could not be committed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachStockError {
    Attachment(AttachError),
    Mass(StockMassError),
}

/// Why substance could not be added to a body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SubstanceError {
    Stock(StockError),
    MassOverflow { total_mg: u128 },
    UnknownPart(PartId),
}

impl BodyPhenotype {
    /// The exact material currently held by one part, including a severed
    /// part's preserved history.
    pub fn part_stock(&self, part: PartId) -> Option<&Stock> {
        self.mosaic(part).map(|mosaic| &mosaic.scruple)
    }

    /// The aggregate of every attached part's stock. The checked result keeps
    /// a body with independently bounded parts from silently wrapping a
    /// material channel in a whole-body reading.
    pub fn total_stock(&self) -> Result<Stock, StockError> {
        self.body
            .living()
            .filter_map(|part| self.part_stock(part.id).copied())
            .try_fold(Stock::EMPTY, Stock::checked_add)
    }

    /// Adds a typed lot to the root without partially applying an overflow.
    pub fn gain_root_stock(&mut self, stock: Stock) -> Result<(), SubstanceError> {
        let root = self.body.root;
        let Some(current) = self.part_stock(root).copied() else {
            return Err(SubstanceError::UnknownPart(root));
        };
        let next = current.checked_add(stock).map_err(SubstanceError::Stock)?;
        let total = next.total();
        let mass_mg =
            u64::try_from(total).map_err(|_| SubstanceError::MassOverflow { total_mg: total })?;
        let index = root.0 as usize;
        let Some((part, mosaic)) = self
            .body
            .parts
            .get_mut(index)
            .zip(self.mosaics.get_mut(index))
        else {
            return Err(SubstanceError::UnknownPart(root));
        };
        if part.id != root {
            return Err(SubstanceError::UnknownPart(root));
        }
        part.mass_mg = mass_mg;
        mosaic.scruple = next;
        debug_assert!(self.conserves());
        Ok(())
    }

    /// Attaches a part whose scalar mass and supplied composition are already
    /// known to agree. This is the typed landing seam for meals and grafts.
    pub fn attach_stock(
        &mut self,
        volume: VolumeRef,
        mass_mg: u64,
        stock: Stock,
        half_extent: [i32; 3],
        attachment: Attachment,
        provenance: Provenance,
    ) -> Result<PartId, AttachStockError> {
        if stock.total() != u128::from(mass_mg) {
            return Err(AttachStockError::Mass(StockMassError::Mismatch {
                part: PartId(self.body.parts.len() as u32),
                part_mass_mg: mass_mg,
                stock_total_mg: stock.total(),
            }));
        }
        let id = self
            .attach(volume, mass_mg, half_extent, attachment, provenance)
            .map_err(AttachStockError::Attachment)?;
        self.replace_part_stock(id, stock)
            .map_err(AttachStockError::Mass)?;
        Ok(id)
    }

    /// Replaces one part's composition only when it still weighs exactly the
    /// supplied amount.
    pub fn replace_part_stock(&mut self, part: PartId, stock: Stock) -> Result<(), StockMassError> {
        let index = part.0 as usize;
        let Some((found, mosaic)) = self
            .body
            .parts
            .get_mut(index)
            .zip(self.mosaics.get_mut(index))
        else {
            return Err(StockMassError::UnknownPart(part));
        };
        if found.id != part {
            return Err(StockMassError::UnknownPart(part));
        }
        if stock.total() != u128::from(found.mass_mg) {
            return Err(StockMassError::Mismatch {
                part,
                part_mass_mg: found.mass_mg,
                stock_total_mg: stock.total(),
            });
        }
        mosaic.scruple = stock;
        debug_assert!(self.conserves());
        Ok(())
    }

    /// Assigns one exact lot across attached parts in stable part order.
    ///
    /// The body's scalar masses are retained. Each active part takes its mass
    /// from the remaining lot, so the last part closes the exact total. The
    /// total is checked before any scruple changes, keeping a failed birth
    /// allocation from partially relabelling a child.
    pub fn distribute_stock(&mut self, stock: Stock) -> Result<(), StockMassError> {
        if self.body.parts.len() != self.mosaics.len() {
            return Err(StockMassError::MosaicMismatch {
                parts: self.body.parts.len(),
                mosaics: self.mosaics.len(),
            });
        }
        let body_mass_mg = self
            .body
            .living()
            .map(|part| u128::from(part.mass_mg))
            .sum();
        if stock.total() != body_mass_mg {
            return Err(StockMassError::BodyMismatch {
                body_mass_mg,
                stock_total_mg: stock.total(),
            });
        }

        let mut remaining = stock;
        // Severed parts are outside the child's active body mass and retain
        // their historical scruples. Start from every existing lot so the
        // final commit changes only active parts.
        let mut allocations: Vec<Stock> =
            self.mosaics.iter().map(|mosaic| mosaic.scruple).collect();
        for (index, part) in self.body.parts.iter().enumerate() {
            if part.severed {
                continue;
            }
            let (allocation, rest) = remaining.take(part.mass_mg);
            allocations[index] = allocation;
            remaining = rest;
        }
        debug_assert_eq!(remaining, Stock::EMPTY);
        for (mosaic, allocation) in self.mosaics.iter_mut().zip(allocations) {
            mosaic.scruple = allocation;
        }
        debug_assert!(self.conserves());
        Ok(())
    }

    /// Takes one named part's whole mixed stock. Its scalar mass becomes zero
    /// with the scruple, while its anatomy and children remain in place.
    pub fn take_part_stock(&mut self, part: PartId) -> Stock {
        let index = part.0 as usize;
        let Some((found, mosaic)) = self
            .body
            .parts
            .get_mut(index)
            .zip(self.mosaics.get_mut(index))
        else {
            return Stock::EMPTY;
        };
        if found.id != part {
            return Stock::EMPTY;
        }
        found.mass_mg = 0;
        let taken = std::mem::replace(&mut mosaic.scruple, Stock::EMPTY);
        debug_assert!(self.conserves());
        taken
    }

    /// Removes up to `want_mg` from living parts in stable part order.
    ///
    /// Each part gives a deterministic proportional split of its real mixed
    /// stock. The returned lot is what was paid, not an untyped scalar proxy.
    pub fn spend_stock(&mut self, want_mg: u64) -> Stock {
        let mut unpaid = want_mg;
        let mut spent = Stock::EMPTY;
        for index in 0..self.body.parts.len().min(self.mosaics.len()) {
            if unpaid == 0 || self.body.parts[index].severed {
                continue;
            }
            let (taken, remainder) = self.mosaics[index].scruple.take(unpaid);
            let paid = u64::try_from(taken.total()).expect("a requested payment fits u64");
            self.mosaics[index].scruple = remainder;
            self.body.parts[index].mass_mg =
                u64::try_from(remainder.total()).expect("a part stock always fits its scalar mass");
            spent = spent
                .checked_add(taken)
                .expect("a payment cannot exceed the requested milligrams");
            unpaid -= paid;
        }
        debug_assert!(self.conserves());
        spent
    }

    /// Compatibility wrapper for scalar growth. New scalar matter is untyped.
    pub fn gain_root_mass(&mut self, mg: u64) -> bool {
        self.gain_root_stock(Stock::single(Material::Untyped, mg))
            .is_ok()
    }

    /// Compatibility wrapper for callers that only understand scalar meals.
    pub fn take_part_mass(&mut self, part: PartId) -> u64 {
        u64::try_from(self.take_part_stock(part).total()).expect("a part mass fits u64")
    }

    /// Compatibility wrapper returning the unpaid scalar remainder.
    pub fn spend_mass(&mut self, mg: u64) -> u64 {
        mg - u64::try_from(self.spend_stock(mg).total()).expect("a payment fits u64")
    }
}
