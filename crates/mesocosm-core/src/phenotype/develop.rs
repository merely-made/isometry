// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One proposal, one validator, one atomic commit.
//!
//! # Two authors, not two biologies
//!
//! A player arranging a founder by hand and the game arranging one for an
//! unplayed lineage are **proposal sources over one developmental
//! authority** (plan §7). Both produce an [`AllocationProposal`]; both are
//! checked by [`BodyPhenotype::develop`](super::BodyPhenotype::develop);
//! neither can invoke a privileged mutation or skip a refusal the other would
//! receive. [`Arrangement`] rides along as diagnostic metadata and the
//! validator never reads it — which is why the parity receipt can assert that
//! the same candidate lowers to the same [`Instruction`] from both.
//!
//! # A complete desired state, not a series of drags
//!
//! A proposal names the parts it rewrites and the complete set of sites each
//! should end up with, so it is order-independent and a stale one is
//! refusable: it carries the digest of the phenotype it was authored against,
//! and a phenotype that moved underneath it refuses rather than applying a
//! plan for a body that no longer exists.
//!
//! # Atomic
//!
//! Validation runs to completion against a candidate before anything is
//! published. A refusal leaves body, allocation and revision byte-identical,
//! which is what keeps a multi-part development from half-landing.

use super::BodyPhenotype;
use super::mosaic::{CellId, MAX_SITES, Mosaic, SiteId};
use crate::body::PartId;
use crate::plan::classify;
use crate::process::{ProcessRef, Registry};

/// Who authored a proposal. **Diagnostic only.**
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Arrangement {
    /// A player arranged it in the editor.
    Direct,
    /// The game arranged it, for a founder preview or an unplayed lineage.
    Automatic,
}

/// One site a proposal wants to exist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposedSite {
    pub part: PartId,
    pub process: ProcessRef,
    /// Existing cell ids, sorted and deduplicated.
    pub cells: Vec<CellId>,
}

/// The complete desired allocation for the parts it names.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AllocationProposal {
    /// The digest of the phenotype this was authored against.
    pub expect: u64,
    pub source: Arrangement,
    /// The parts this proposal rewrites, sorted and deduplicated. A part named
    /// here with no site in `sites` is being cleared.
    pub parts: Vec<PartId>,
    pub sites: Vec<ProposedSite>,
}

/// What a validated proposal commits. Independent of who proposed it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instruction {
    /// The phenotype revision this development created.
    pub revision: u32,
    pub parts: Vec<PartId>,
    /// Every site that now exists on a rewritten part, in commit order.
    pub sites: Vec<(PartId, SiteId, ProcessRef)>,
    /// What it cost, in cells whose **expression** changed: tissue that now
    /// does something other than what it did, free tissue included.
    ///
    /// A countable unit rather than an invented milligram. PD2 prices it when
    /// a played process gives the price a consumer.
    pub cost_cells: u32,
    /// The same count, split by part, in `parts` order. (P3)
    ///
    /// **Because a cell is worth what its own part's tissue is worth.** One
    /// total is enough to price a single-part development and cannot price a
    /// multi-part one without inventing a rate; a graft is the first
    /// development that names several parts at once, and this is the validator
    /// answering that question rather than a caller running the comparison a
    /// second time.
    pub cost_by_part: Vec<(PartId, u32)>,
    /// The phenotype digest after the commit.
    pub digest: u64,
}

/// A committed development: the instruction, and who asked for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Development {
    pub instruction: Instruction,
    pub source: Arrangement,
}

/// Why a proposal was refused. Every variant names the boundary that failed.
///
/// Serialized since PD2, because a refusal now reaches a player through
/// [`Outcome::Rejected`](crate::world::Outcome::Rejected) and a recorded
/// outcome has to be able to say which boundary it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Refusal {
    /// The phenotype moved under the proposal.
    Stale {
        expected: u64,
        actual: u64,
    },
    /// It rewrites nothing.
    NothingProposed,
    /// `parts` is not sorted and deduplicated, so the proposal is not a
    /// canonical desired state.
    UnorderedParts,
    NoSuchPart(PartId),
    SeveredPart(PartId),
    /// A site addresses a part the proposal did not claim.
    UnclaimedPart(PartId),
    /// This world's ruleset does not hold that definition. Never substituted.
    UnknownProcess(ProcessRef),
    /// A part of this shape does not express that process.
    SiteMismatch {
        part: PartId,
        process: ProcessRef,
    },
    EmptySite(PartId),
    UnorderedCells(PartId),
    NoSuchCell {
        part: PartId,
        cell: CellId,
    },
    /// The cell was taken by irreversible loss.
    LostCell {
        part: PartId,
        cell: CellId,
    },
    /// Two sites claim the same cell.
    Overlap {
        part: PartId,
        cell: CellId,
    },
    /// A site's cells are not one connected region.
    Disconnected(PartId),
    TooManySites(PartId),
}

/// What an automatic arrangement is trying to achieve.
///
/// A declared aim, optimized inside the same rules the editor obeys. It buys
/// no privilege: an aim that cannot be satisfied validly is refused exactly as
/// a hand-drawn candidate would be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    /// Express every living part's geometry across all of its tissue: the
    /// seeding rule, restated as a proposal.
    Express,
    /// Keep everything expressed on the least tissue that can express it, and
    /// leave the rest of each part free.
    Spare,
}

/// Builds the proposal an automatic arrangement would submit.
///
/// The result is an ordinary [`AllocationProposal`]. It goes through the same
/// `develop` as a hand-drawn one, so nothing here can be a second biology.
pub fn arrange(phenotype: &BodyPhenotype, aim: Aim) -> AllocationProposal {
    let mut parts = Vec::new();
    let mut sites = Vec::new();
    for (part, mosaic) in phenotype.allocations() {
        parts.push(part);
        // The seeding rule, asked rather than reimplemented, so an automatic
        // arrangement cannot drift from what development would have grown.
        let fresh;
        let source = match aim {
            Aim::Express => {
                fresh = Mosaic::seed(phenotype.body().part(part).expect("a living part"));
                fresh.sites()
            },
            Aim::Spare => mosaic.sites(),
        };
        for site in source {
            let cells: Vec<CellId> = match aim {
                Aim::Express => site.cells.clone(),
                // The least tissue that still expresses it: one cell, and the
                // lowest id so the answer is deterministic rather than
                // whichever the iteration happened to reach.
                Aim::Spare => site.cells.iter().copied().take(1).collect(),
            };
            sites.push(ProposedSite {
                part,
                process: site.process,
                cells,
            });
        }
    }
    AllocationProposal {
        expect: phenotype.digest(),
        source: Arrangement::Automatic,
        parts,
        sites,
    }
}

/// One part's complete desired allocation: what each site expresses, and on
/// which cells.
pub(super) type Rewrite = Vec<(ProcessRef, Vec<CellId>)>;

/// The validated form of a proposal, ready to publish.
pub(super) struct Validated {
    pub(super) rewrites: Vec<(PartId, Rewrite)>,
    pub(super) cost_cells: u32,
    pub(super) cost_by_part: Vec<(PartId, u32)>,
}

/// **The one validator.** Direct, automatic and authored arrangement all land
/// here.
///
/// **It resolves against the ruleset it is handed, never a global one** (PD4).
/// PD3 could read `Registry::native()` here honestly, because the shipped pack
/// lowered to a registry `==` to it; that stopped being honest the moment a
/// door existed that could carry a different admitted set. A world hands its
/// own [`Registry`], so a proposal citing a definition this world did not admit
/// is [`Refusal::UnknownProcess`] — the stale-ruleset boundary plan §4 names —
/// rather than a definition somebody else's build happened to hold.
///
/// Checks are ordered so the first boundary that fails is the one reported,
/// and the order is part of the contract: two callers submitting the same
/// invalid candidate must receive the same refusal.
pub(super) fn validate(
    registry: &Registry,
    phenotype: &BodyPhenotype,
    proposal: &AllocationProposal,
) -> Result<Validated, Refusal> {
    let actual = phenotype.digest();
    if proposal.expect != actual {
        return Err(Refusal::Stale {
            expected: proposal.expect,
            actual,
        });
    }
    if proposal.parts.is_empty() {
        return Err(Refusal::NothingProposed);
    }
    if proposal.parts.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(Refusal::UnorderedParts);
    }

    let body = phenotype.body();
    for part in &proposal.parts {
        let Some(found) = body.part(*part) else {
            return Err(Refusal::NoSuchPart(*part));
        };
        if found.severed {
            return Err(Refusal::SeveredPart(*part));
        }
    }
    for site in &proposal.sites {
        if !proposal.parts.contains(&site.part) {
            return Err(Refusal::UnclaimedPart(site.part));
        }
    }

    let mut rewrites: Vec<(PartId, Rewrite)> = Vec::new();
    let mut cost_cells = 0u32;
    let mut cost_by_part: Vec<(PartId, u32)> = Vec::new();
    for part in &proposal.parts {
        let mosaic = phenotype
            .mosaic(*part)
            .expect("a living part carries a mosaic");
        let role = classify(body.part(*part).expect("checked above").half_extent);
        let mut claimed: Vec<CellId> = Vec::new();
        let mut desired = Vec::new();
        for site in proposal.sites.iter().filter(|site| site.part == *part) {
            let Some(def) = registry.resolve(site.process) else {
                return Err(Refusal::UnknownProcess(site.process));
            };
            // Shape gates expression. This is where "a part cannot acquire a
            // capability by editing a number" is actually enforced: to make a
            // plate contract you would have to make it a limb.
            if !def.admits(role) {
                return Err(Refusal::SiteMismatch {
                    part: *part,
                    process: site.process,
                });
            }
            if site.cells.is_empty() {
                return Err(Refusal::EmptySite(*part));
            }
            if site.cells.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(Refusal::UnorderedCells(*part));
            }
            for cell in &site.cells {
                if !mosaic.holds(*cell) {
                    return Err(Refusal::NoSuchCell {
                        part: *part,
                        cell: *cell,
                    });
                }
                if !mosaic.is_living(*cell) {
                    return Err(Refusal::LostCell {
                        part: *part,
                        cell: *cell,
                    });
                }
                if claimed.contains(cell) {
                    return Err(Refusal::Overlap {
                        part: *part,
                        cell: *cell,
                    });
                }
                claimed.push(*cell);
            }
            if !mosaic.connected(&site.cells) {
                return Err(Refusal::Disconnected(*part));
            }
            desired.push((site.process, site.cells.clone()));
        }
        if desired.len() > MAX_SITES {
            return Err(Refusal::TooManySites(*part));
        }
        // What changed hands: every cell that ends up expressing something
        // other than what it expressed in the mosaic this proposal was
        // authored against — freed and newly occupied tissue included.
        let mut changed = 0u32;
        for cell in mosaic.cells() {
            let was = mosaic.site_of(cell).map(|site| site.process);
            let now = desired
                .iter()
                .find(|(_, cells)| cells.contains(&cell))
                .map(|(process, _)| *process);
            if was != now {
                changed += 1;
            }
        }
        cost_cells += changed;
        cost_by_part.push((*part, changed));
        rewrites.push((*part, desired));
    }

    Ok(Validated {
        rewrites,
        cost_cells,
        cost_by_part,
    })
}
