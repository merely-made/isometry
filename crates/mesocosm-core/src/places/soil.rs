// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The enclosure's matter, held per voxel column.
//!
//! TD6's closed cycle (ruled 2026-08-29). Producers draw matter from the
//! column they stand on, bodies return it where they fall, and the player's
//! deposit enriches it, so **total matter is conserved**: mass cannot run away
//! because it has to be somewhere. Light stays the one open input — a producer
//! spends free energy to do the drawing, and that energy never enters this
//! ledger.
//!
//! # Not the same thing as [`Ground`](super::Ground)
//!
//! Ground holds bricks: what is solid, what can be walked on, what a
//! projection draws. This holds milligrams: what can be *eaten out of* the
//! floor. A carve changes the first and not the second, which is why the
//! matter account and the terrain account are separate stores over the same
//! coordinates.
//!
//! # Why per voxel column
//!
//! Ruled on measured evidence (`Code/testing/mesocosm/soil_granularity_probe.md`,
//! 96 configs). At the then-shipping enclosure a direct index into a 4 KB array
//! was the *fastest* grain for point uptake — 1.75us against 5.66us for a
//! nearest-site scan — and it is the only grain that can express a forage
//! radius at all: a coarse grain's r=3 neighbourhood already covers the whole
//! world, so roots hunting minerals through soil cannot be represented in it
//! at any price.
//!
//! S1 widened the store to 129x129 (130 KiB), which leaves point uptake
//! unchanged — an index is an index — and makes [`Soil::percolate`], the one
//! pass that sweeps every column every tick, the store's whole cost. It is
//! measured per rung in `Code/testing/mesocosm/s1_wide.json`.
//!
//! # The forage radius, built
//!
//! Addressing ([`Column`], [`Soil::column_at`], [`Soil::columns_within`]) is
//! separate from transfer ([`Soil::matter_mg`], [`Soil::draw`],
//! [`Soil::deposit`]), so TD7's root forage is exactly what that separation
//! promised: a read over `columns_within` followed by the same `draw` uptake
//! always took. See [`Soil::draw_richest_within`] and [`FORAGE_RADIUS`].

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::matter::{Stock, StockError, transport};

/// Fraction of a column that percolates outward each tick, as a divisor.
///
/// **Measured, and the round's own structural finding put it here.** With
/// sealed columns, a producer drains the one it stands on within tens of ticks
/// and thereafter earns exactly the rent it just paid — net zero, forever —
/// while the other thousand columns keep their matter and no root can reach
/// it. The probe read 17 producers standing on 0 mg with 340,000 mg lying in
/// the enclosure around them; the whole chain starved above them. Percolation
/// is the medium's own property (dissolved minerals move through soil, which
/// is *why* a root that searches a radius finds more), so it is not the
/// foraging behaviour this round deliberately left unbuilt, and it is what
/// makes "the enclosure gets a finite matter budget" mean one budget.
const PERCOLATION_DIVISOR: u64 = 8;

/// How far a root searches for its next milligram, in voxel columns.
///
/// Three, because that is the reach the per-voxel grain was ruled for: 49
/// columns of the enclosure's 16,641, where every coarser grain's r=3 already
/// covered the whole world and a forage radius could not be expressed at all.
/// (TD7. The count was 49 of 1,089 when the reach was ruled; S1 widened the
/// world and left the reach alone — a root's search is anatomy, not geography.)
pub const FORAGE_RADIUS: i32 = 3;

/// One voxel column of the enclosure: a direct index into [`Soil`].
///
/// Opaque, and only [`Soil`] mints one, so an index can never address a
/// column a differently-sized store does not have.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Column(u32);

/// A rejected typed soil transfer.
///
/// Soil keeps the scalar world's `u64` bound per column even though a
/// [`Stock`] has four independent channels. This makes a failed typed deposit
/// explicit instead of silently dropping a channel or widening one old
/// account behind its callers' backs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoilError {
    /// The serialized extent cannot form a finite square store.
    InvalidExtent(i32),
    /// The serialized field does not cover exactly one square enclosure.
    InvalidShape { extent: i32, columns: usize },
    /// A deserialized or otherwise stale column does not belong to this soil.
    InvalidColumn(Column),
    /// One provenance channel overflowed while combining stocks.
    Stock(StockError),
    /// The combined channels would exceed a scalar column's finite capacity.
    TotalOverflow { column: Column },
    /// The whole finite world's scalar conservation ledger would overflow.
    GlobalTotalOverflow,
    /// Percolation could not produce a valid next field.
    Transport(transport::TransportError),
}

impl fmt::Display for SoilError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for SoilError {}

/// Matter in the ground, one entry per voxel column of the enclosure.
///
/// World state: serialized, hashed, and deterministic. Row-major in z then x
/// over `-extent..=extent` on both axes, so the whole store is one contiguous
/// array and a lookup is arithmetic rather than a search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Soil {
    extent: i32,
    /// Matter per column, in canonical index order.
    matter_mg: Vec<Stock>,
    /// Derived whole-field total. Kept outside the snapshot wire so Stock is
    /// the only serialized authority and deposits need not rescan the field.
    total_mg: u64,
}

#[derive(Serialize)]
struct SoilWireRef<'a> {
    extent: i32,
    matter_mg: &'a [Stock],
}

#[derive(Deserialize)]
struct SoilWire {
    extent: i32,
    matter_mg: Vec<Stock>,
}

impl Serialize for Soil {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SoilWireRef {
            extent: self.extent,
            matter_mg: &self.matter_mg,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Soil {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = SoilWire::deserialize(deserializer)?;
        Self::from_columns(wire.extent, wire.matter_mg).map_err(serde::de::Error::custom)
    }
}

impl Default for Soil {
    /// A single empty column. Not `derive`d: every operation here indexes, so
    /// the degenerate store still has to have somewhere to put a milligram.
    fn default() -> Self {
        Self::seeded(0, 0)
    }
}

impl Soil {
    /// A store over an enclosure of the given extent, every column holding
    /// `per_column_mg`.
    pub fn seeded(extent: i32, per_column_mg: u64) -> Self {
        let extent = extent.max(0);
        let side = Self::side_for(extent).expect("a seeded soil has a valid extent");
        let columns = side.checked_mul(side).expect("a seeded soil is finite");
        let count = u64::try_from(columns).expect("a seeded soil column count fits u64");
        let total_mg = per_column_mg
            .checked_mul(count)
            .expect("a seeded soil fits the world's scalar conservation ledger");
        Self {
            extent,
            matter_mg: vec![
                Stock::single(crate::matter::Material::Untyped, per_column_mg);
                columns
            ],
            total_mg,
        }
    }

    fn from_columns(extent: i32, matter_mg: Vec<Stock>) -> Result<Self, SoilError> {
        let side = Self::side_for(extent)?;
        let expected = side
            .checked_mul(side)
            .ok_or(SoilError::InvalidExtent(extent))?;
        if matter_mg.len() != expected {
            return Err(SoilError::InvalidShape {
                extent,
                columns: matter_mg.len(),
            });
        }
        let total_mg = Self::sum_total_mg(&matter_mg)?;
        Ok(Self {
            extent,
            matter_mg,
            total_mg,
        })
    }

    fn side_for(extent: i32) -> Result<usize, SoilError> {
        let side = extent
            .checked_mul(2)
            .and_then(|twice| twice.checked_add(1))
            .ok_or(SoilError::InvalidExtent(extent))?;
        usize::try_from(side).map_err(|_| SoilError::InvalidExtent(extent))
    }

    /// How far the store reaches from the middle, in voxels. Sized from the
    /// enclosure it was raised over, never assumed.
    pub fn extent(&self) -> i32 {
        self.extent
    }

    /// Columns to a side.
    pub fn side(&self) -> i32 {
        2 * self.extent + 1
    }

    pub fn columns(&self) -> usize {
        self.matter_mg.len()
    }

    /// Which column a position stands over.
    ///
    /// **Clamped, not refused.** Everything deposited has to land somewhere or
    /// the cycle would leak at the wall; since TD2b nothing lives outside the
    /// enclosure anyway, so the clamp is insurance rather than a behaviour.
    pub fn column_at(&self, position: [i32; 3]) -> Column {
        let x = position[0].clamp(-self.extent, self.extent) + self.extent;
        let z = position[2].clamp(-self.extent, self.extent) + self.extent;
        Column((z * self.side() + x) as u32)
    }

    /// Every material channel in one column.
    pub fn stock(&self, column: Column) -> Stock {
        self.matter_mg
            .get(column.0 as usize)
            .copied()
            .unwrap_or(Stock::EMPTY)
    }

    /// The untyped matter one scalar caller may still spend.
    ///
    /// This compatibility read deliberately excludes nis. A scalar caller
    /// cannot name its provenance, so letting it see mixed material would let
    /// its later [`Self::draw`] erase living history.
    pub fn matter_mg(&self, column: Column) -> u64 {
        self.stock(column).amount(crate::matter::Material::Untyped)
    }

    /// Takes up to `want_mg` out of one column, returning what was actually
    /// there to take. A column that is spent gives nothing, which is the whole
    /// point: a producer's income is limited by the ground under it.
    pub fn draw(&mut self, column: Column, want_mg: u64) -> u64 {
        let index = column.0 as usize;
        let Some(held) = self.matter_mg.get(index).copied() else {
            return 0;
        };
        let drawn = want_mg.min(held.amount(crate::matter::Material::Untyped));
        self.matter_mg[index] = held
            .checked_sub(Stock::single(crate::matter::Material::Untyped, drawn))
            .expect("a bounded untyped draw cannot underflow");
        self.total_mg -= drawn;
        drawn
    }

    /// Takes a deterministic proportional mixture from one column.
    pub fn draw_stock(&mut self, column: Column, want_mg: u64) -> Stock {
        let index = column.0 as usize;
        let Some(held) = self.matter_mg.get(index).copied() else {
            return Stock::EMPTY;
        };
        let (drawn, remainder) = held.take(want_mg);
        self.matter_mg[index] = remainder;
        self.total_mg -= u64::try_from(drawn.total()).expect("a soil draw stays scalar-bounded");
        drawn
    }

    /// Returns matter to one column. Decay, rent, and the player's deposit all
    /// land here.
    pub fn deposit(&mut self, column: Column, mg: u64) {
        self.deposit_stock(column, Stock::single(crate::matter::Material::Untyped, mg))
            .expect("scalar soil deposits must fit their finite column");
    }

    /// Returns typed matter to one column without losing an overflowing
    /// channel or exceeding the scalar column capacity.
    pub fn deposit_stock(&mut self, column: Column, stock: Stock) -> Result<(), SoilError> {
        let index = column.0 as usize;
        let Some(held) = self.matter_mg.get(index).copied() else {
            return Err(SoilError::InvalidColumn(column));
        };
        let deposited = held.checked_add(stock).map_err(SoilError::Stock)?;
        if deposited.total() > u64::MAX as u128 {
            return Err(SoilError::TotalOverflow { column });
        }
        let added = u64::try_from(stock.total()).expect("a scalar-bounded deposit fits u64");
        let total_mg = self
            .total_mg
            .checked_add(added)
            .ok_or(SoilError::GlobalTotalOverflow)?;
        self.matter_mg[index] = deposited;
        self.total_mg = total_mg;
        Ok(())
    }

    /// Every milligram the ground is holding. One half of the conservation
    /// ledger; living bodies, carrion, and budgets are the other.
    pub fn total_mg(&self) -> u64 {
        self.total_mg
    }

    /// Every material channel in the enclosure, for typed conservation
    /// receipts.
    pub fn total_stock(&self) -> Stock {
        self.matter_mg
            .iter()
            .copied()
            .try_fold(Stock::EMPTY, |total, held| total.checked_add(held))
            .expect("the finite world's typed totals fit its scalar ledger")
    }

    fn sum_total_mg(columns: &[Stock]) -> Result<u64, SoilError> {
        columns
            .iter()
            .enumerate()
            .try_fold(0_u64, |total, (index, stock)| {
                let held = u64::try_from(stock.total()).map_err(|_| SoilError::TotalOverflow {
                    column: Column(index as u32),
                })?;
                total
                    .checked_add(held)
                    .ok_or(SoilError::GlobalTotalOverflow)
            })
    }

    /// One tick of percolation: every column sheds a share of what it holds
    /// into the columns beside it.
    ///
    /// **Transport, not regrowth.** Nothing is created — a column's loss is
    /// exactly its neighbours' gain, in integers — so conservation is
    /// unaffected. What it buys is that the enclosure's matter budget is one
    /// budget rather than 16,641 sealed jars.
    ///
    /// **The one pass that is O(columns) rather than O(bodies)**, so it is the
    /// pass a wider enclosure charges for directly. S1 measured it.
    ///
    /// See [`PERCOLATION_DIVISOR`] for why the round needed it.
    pub fn percolate(&mut self) -> Result<(), SoilError> {
        let side = self.side() as usize;
        transport::percolate(&mut self.matter_mg, side, PERCOLATION_DIVISOR)
            .map_err(SoilError::Transport)
    }

    /// Takes up to `want_mg` out of the richest column within `radius`.
    ///
    /// **The reach is wide; the draw is not.** A root reads its whole
    /// neighbourhood and then takes the ordinary income out of the best column
    /// it found — at the speed of growth, on low-rent metabolism, never the
    /// radius' worth of columns at once. Ties go to the lowest column index, so
    /// a stand on flat ground forages the same way every replay.
    pub fn draw_richest_within(&mut self, column: Column, radius: i32, want_mg: u64) -> u64 {
        let richest = self
            .columns_within(column, radius)
            .max_by_key(|found| (self.matter_mg(*found), std::cmp::Reverse(found.0)))
            .unwrap_or(column);
        self.draw(richest, want_mg)
    }

    /// The columns within `radius` of one, in canonical order.
    ///
    /// The reach a root searches: [`Soil::draw_richest_within`] is this read
    /// followed by the same [`Soil::draw`] a point uptake always took, which
    /// is why the addressing was kept separate from the transfer.
    pub fn columns_within(&self, column: Column, radius: i32) -> impl Iterator<Item = Column> + '_ {
        let side = self.side();
        let (cx, cz) = (column.0 as i32 % side, column.0 as i32 / side);
        let radius = radius.max(0);
        ((cz - radius).max(0)..=(cz + radius).min(side - 1)).flat_map(move |z| {
            ((cx - radius).max(0)..=(cx + radius).min(side - 1))
                .map(move |x| Column((z * side + x) as u32))
        })
    }
}

#[cfg(test)]
mod tests;
