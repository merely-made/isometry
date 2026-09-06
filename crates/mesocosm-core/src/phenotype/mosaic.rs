// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One part's allocation mosaic: an authoritative graph of capacity cells.
//!
//! # There is no capacity scalar
//!
//! PD1a removed it. **Structural capacity is the number of living cells**, and
//! a site's allocation is the disjoint set of cells it occupies, so occupied
//! plus free equals capacity by construction rather than by two integers
//! agreeing. Conservation is countable, and [`Mosaic::conserves`] counts it.
//!
//! # The graph is authority; a layout is a projection
//!
//! Cells carry ids and adjacency, never coordinates and never renderer voxels.
//! The initial generator is a coarse orthogonal lattice whose axes follow the
//! part's `half_extent`, so a long limb is a chain and a plate is a sheet, and
//! adjacency is derived from the lattice's dimensions rather than stored per
//! cell — one graph, compactly written. A Diablo-like 2D inventory may draw the
//! same graph any way it likes; its screen coordinates are not authority.
//!
//! Admitting a different topology is itself a paid, ordered developmental
//! event, and [`Mosaic::neighbours`] is the seam it arrives through: every
//! reader already asks the mosaic for adjacency rather than doing lattice
//! arithmetic of its own.
//!
//! # Current availability is a separate question
//!
//! Mass, condition, starvation or a missing input may make allocated cells
//! ineffective without moving them. Nothing here evaluates that; ordinary
//! grazing does not shuffle organs. Irreversible loss is the other case, and
//! that tombstones cells through [`Mosaic::tombstone`].

use serde::{Deserialize, Serialize};

use crate::body::Part;
use crate::plan::classify;
use crate::process::{IntakePort, ProcessRef, Registry};

/// A cell's address inside one part's mosaic. Stable, never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CellId(pub u16);

/// A site's address inside one part's mosaic. Stable, never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SiteId(pub u16);

/// Why a site is where it is.
///
/// Provenance for the expression itself, distinct from the part's own
/// [`Provenance`](crate::body::Provenance), which says where the tissue came
/// from. The 2026-08-07 audit's ruling that provenance is several separable
/// kinds is why these are not one field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Expressed {
    /// The part's shape expressed it when the part was developed.
    Geometry,
    /// A validated allocation proposal placed it, at this phenotype revision.
    Arranged { revision: u32 },
}

/// One expressed process: what it is, and which tissue it occupies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Site {
    pub id: SiteId,
    /// The exact admitted definition. Resolved through a [`Registry`]; never
    /// substituted when the registry does not hold it.
    pub process: ProcessRef,
    /// The cells it occupies: sorted, disjoint from every other site on this
    /// part, and a connected subgraph.
    pub cells: Vec<CellId>,
    pub cause: Expressed,
}

/// Voxels of half-extent per cell of tissue.
///
/// Coarse on purpose. The mosaic is a competition space a player reads, not a
/// second voxel grid: the primitive palette's limb `[4, 1, 1]` becomes a chain
/// of three and its sensor `[1, 1, 1]` a single cell, which is what those
/// organs are.
pub const CELL_QUANTUM: i32 = 2;

/// The most cells one axis may carry. Core safety policy, not world rules:
/// with [`MAX_CELLS`] it is what bounds the graph a validator has to walk.
pub const MAX_AXIS_CELLS: i32 = 4;

/// The hard per-part cell ceiling, derived rather than typed: a lattice
/// bounded on every axis is bounded overall, and stating it twice is how the
/// two would eventually disagree.
pub const MAX_CELLS: u32 = (MAX_AXIS_CELLS * MAX_AXIS_CELLS * MAX_AXIS_CELLS) as u32;

/// The most sites one part may carry at once.
///
/// A bound on the graph rather than a balance number: the whole native
/// vocabulary is four processes, and a part that claimed eight sites would be
/// proposing something the explanation path cannot render.
pub const MAX_SITES: usize = 8;

/// One part's finite process capacity, and what occupies it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mosaic {
    /// Lattice extent in cells. A cell's index is `x + dx * (y + dy * z)`.
    dims: [u8; 3],
    /// Cells that irreversible loss took, sorted. They stay addressable so an
    /// injury is still explainable; they are not capacity.
    lost: Vec<CellId>,
    /// Occupied regions, ordered by site id.
    sites: Vec<Site>,
    /// The next site ordinal. Ids are never reused within a part.
    next_site: u16,
    /// The part's declared intake admission. Kept beside allocation so a
    /// graft, loss and snapshot carry it with the named anatomy.
    #[serde(default)]
    port: IntakePort,
}

impl Mosaic {
    /// The mosaic a part's geometry seeds.
    ///
    /// **Whole-part expression.** Today's rule is exactly "this shape does
    /// this thing", so its honest lowering gives the seeded site every cell
    /// the part has and leaves nothing free. A part therefore arrives fully
    /// committed, and the first developmental event that wants a second
    /// process has to take tissue off the first. That is the intended
    /// tradeoff — inside the organ, not in a flat organism-wide score — and
    /// pre-donating free tissue would have invented a number to make it
    /// painless.
    pub fn seed(part: &Part) -> Self {
        let dims = lattice(part.half_extent);
        let count = cell_count(dims);
        let role = classify(part.half_extent);
        let registry = Registry::native();

        let mut sites = Vec::new();
        let mut next_site = 0u16;
        // The **seeding** rule, not the site requirement. Since PD2 a plate
        // admits two definitions and grows one, so asking the wrong question
        // here would hand every plate in the world a gland it never paid for.
        let expressed: Vec<ProcessRef> = registry.seeds(role).map(|def| def.reference()).collect();
        // Even shares in registry order, front to back. A prefix of the
        // row-major order is always connected in a lattice, and so is each
        // whole-slab run of one while the shares divide it evenly; the
        // validator is the authority that says so, and `develop` runs it over
        // anything a later vocabulary produces.
        if !expressed.is_empty() {
            let each = count / expressed.len() as u32;
            let mut cursor = 0u32;
            for (index, process) in expressed.iter().enumerate() {
                let last = index + 1 == expressed.len();
                let take = if last { count - cursor } else { each.max(1) };
                let cells: Vec<CellId> = (cursor..(cursor + take).min(count))
                    .map(|i| CellId(i as u16))
                    .collect();
                cursor = (cursor + take).min(count);
                if cells.is_empty() {
                    continue;
                }
                sites.push(Site {
                    id: SiteId(next_site),
                    process: *process,
                    cells,
                    cause: Expressed::Geometry,
                });
                next_site += 1;
            }
        }

        Self {
            dims,
            lost: Vec::new(),
            sites,
            next_site,
            port: IntakePort::none(),
        }
    }

    /// The lattice extent in cells, for an inspector that wants to draw it.
    pub fn dims(&self) -> [u8; 3] {
        self.dims
    }

    /// Every cell the lattice holds, living or lost.
    pub fn extent(&self) -> u32 {
        cell_count(self.dims)
    }

    /// Whether a cell exists in this mosaic at all.
    pub fn holds(&self, cell: CellId) -> bool {
        u32::from(cell.0) < self.extent()
    }

    /// Whether a cell exists and irreversible loss has not taken it.
    pub fn is_living(&self, cell: CellId) -> bool {
        self.holds(cell) && !self.lost.contains(&cell)
    }

    /// Every living cell, in id order.
    pub fn cells(&self) -> impl Iterator<Item = CellId> + '_ {
        (0..self.extent())
            .map(|i| CellId(i as u16))
            .filter(|cell| self.is_living(*cell))
    }

    /// **Structural capacity**: how many living cells this part has.
    pub fn capacity(&self) -> u32 {
        self.extent() - self.lost.len() as u32
    }

    /// How many living cells are occupied by a site.
    pub fn occupied(&self) -> u32 {
        self.sites
            .iter()
            .flat_map(|site| site.cells.iter())
            .filter(|cell| self.is_living(**cell))
            .count() as u32
    }

    /// How many living cells nothing occupies.
    pub fn free(&self) -> u32 {
        self.capacity() - self.occupied()
    }

    pub fn sites(&self) -> &[Site] {
        &self.sites
    }

    pub fn port(&self) -> IntakePort {
        self.port
    }

    pub(super) fn declare_port(&mut self, port: IntakePort) {
        self.port = port;
    }

    /// The site occupying a cell, if any.
    pub fn site_of(&self, cell: CellId) -> Option<&Site> {
        self.sites.iter().find(|site| site.cells.contains(&cell))
    }

    /// A cell's living neighbours in the graph, in id order.
    ///
    /// **The topology seam.** Every reader asks here rather than doing lattice
    /// arithmetic, so admitting a different admitted topology later changes
    /// this function and nothing else.
    pub fn neighbours(&self, cell: CellId) -> Vec<CellId> {
        if !self.is_living(cell) {
            return Vec::new();
        }
        let [dx, dy, _] = self.dims.map(u32::from);
        let index = u32::from(cell.0);
        let (x, y, z) = (index % dx, (index / dx) % dy, index / (dx * dy));
        let mut found = Vec::new();
        for (axis, span) in [(0usize, dx), (1, dy), (2, u32::from(self.dims[2]))] {
            let at = [x, y, z][axis];
            for step in [-1i64, 1] {
                let moved = at as i64 + step;
                if moved < 0 || moved as u32 >= span {
                    continue;
                }
                let mut to = [x, y, z];
                to[axis] = moved as u32;
                let id = CellId((to[0] + dx * (to[1] + dy * to[2])) as u16);
                if self.is_living(id) {
                    found.push(id);
                }
            }
        }
        found.sort_unstable();
        found
    }

    /// Whether a set of cells forms one connected region.
    ///
    /// The property a site must satisfy: an organ is a piece of tissue, not a
    /// scatter of cells that happen to share a name.
    pub fn connected(&self, cells: &[CellId]) -> bool {
        let Some(first) = cells.first() else {
            return false;
        };
        let mut reached = vec![*first];
        let mut next = 0;
        while next < reached.len() {
            let at = reached[next];
            next += 1;
            for neighbour in self.neighbours(at) {
                if cells.contains(&neighbour) && !reached.contains(&neighbour) {
                    reached.push(neighbour);
                }
            }
        }
        reached.len() == cells.len()
    }

    /// Whether this mosaic's own arithmetic holds.
    ///
    /// Occupied plus free is capacity, every occupied cell is living, and no
    /// cell is claimed twice. Asserted by the receipts and debug-asserted at
    /// every commit, because a mosaic that fails this is the split account
    /// PD0 spent a migration removing.
    pub fn conserves(&self) -> bool {
        let mut seen = Vec::new();
        for site in &self.sites {
            if site.cells.is_empty() {
                return false;
            }
            for cell in &site.cells {
                if !self.is_living(*cell) || seen.contains(cell) {
                    return false;
                }
                seen.push(*cell);
            }
        }
        self.occupied() + self.free() == self.capacity()
    }

    /// Takes cells irreversibly, and deactivates any site left without a
    /// valid connected subgraph.
    ///
    /// Not reached by ordinary play in this slice — shrinkage and sub-part
    /// injury are phenotype D3a's gate — but the mosaic owns the rule, so the
    /// rule lives with it rather than being invented by the first caller.
    pub fn tombstone(&mut self, cells: &[CellId]) {
        for cell in cells {
            if self.holds(*cell) && !self.lost.contains(cell) {
                self.lost.push(*cell);
            }
        }
        self.lost.sort_unstable();
        let mut sites = std::mem::take(&mut self.sites);
        for site in sites.iter_mut() {
            site.cells.retain(|cell| self.is_living(*cell));
        }
        // A site that no longer owns a connected region is deactivated
        // deterministically rather than left holding a scatter.
        sites.retain(|site| !site.cells.is_empty() && self.connected(&site.cells));
        self.sites = sites;
    }

    /// Replaces every site on this part. Only [`super::develop`] calls it,
    /// and only after the validator has accepted the whole proposal.
    pub(super) fn rewrite(&mut self, sites: super::develop::Rewrite, revision: u32) {
        self.sites = sites
            .into_iter()
            .map(|(process, cells)| {
                let id = SiteId(self.next_site);
                self.next_site += 1;
                Site {
                    id,
                    process,
                    cells,
                    cause: Expressed::Arranged { revision },
                }
            })
            .collect();
    }
}

/// The lattice a part's integer geometry seeds, in cells per axis.
fn lattice(half_extent: [i32; 3]) -> [u8; 3] {
    half_extent.map(|half| (half.abs().max(1) / CELL_QUANTUM + 1).clamp(1, MAX_AXIS_CELLS) as u8)
}

fn cell_count(dims: [u8; 3]) -> u32 {
    let count = u32::from(dims[0]) * u32::from(dims[1]) * u32::from(dims[2]);
    count.min(MAX_CELLS)
}
