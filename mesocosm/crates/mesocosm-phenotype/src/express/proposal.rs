// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a script says, and how it becomes something the validator can read.
//! (PD4)
//!
//! # The smallest proposal plan §4 describes
//!
//! *Which admitted process should express on which existing part, at what
//! bounded capacity.* Three fields, and no fourth: not a cost (the host prices
//! the accepted result), not a cell address (the host lays tissue out), not a
//! revision (the host froze one into the request). Everything a script could
//! get wrong that the game would then have to live with is simply not
//! expressible.
//!
//! # Lowering is deterministic, and the script does not choose it
//!
//! A part's requested sites take tissue from the high end of its lattice
//! downward, in the order the script listed them, each run contiguous — the
//! same suffix rule
//! [`Candidate::propose`](mesocosm_core::Candidate) already relies on, and for
//! the same reason: a suffix of the row-major order is a connected region and
//! so is the prefix left behind. What the script did not claim keeps doing what
//! it did. The result is a **complete desired state** for the parts named,
//! which is the only shape the validator accepts.

use mesocosm_core::{
    AllocationProposal, Arrangement, BodyPhenotype, CellId, PartId, ProcessId, ProcessRef,
    ProposedSite, Registry,
};
use serde::{Deserialize, Serialize};

use super::Refused;

/// One thing a script asks a part to express.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expression {
    /// The stable part address, from the request.
    pub part: u32,
    /// `namespace:name`. Resolved against the world's admitted ruleset, and
    /// refused when it does not hold it.
    pub process: String,
    /// How much tissue, in cells. Bounded capacity, per plan §4.
    pub cells: u32,
}

/// What one expression call returned.
///
/// **A proposal, never a change.** Nothing here has happened; it is what an
/// author would like to have happen, on its way to the one validator.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proposal {
    pub sites: Vec<Expression>,
}

/// Lowers an authored proposal into the ordinary
/// [`AllocationProposal`](mesocosm_core::AllocationProposal).
///
/// The bridge's whole Lua-to-Rust half, and the last thing between a script and
/// the validator. It resolves ids against **this world's** ruleset, lays out
/// tissue deterministically, and hands over a complete desired state. It
/// decides nothing else: whether a plate may carry a gland, whether the
/// phenotype moved, whether a site is connected and whether the body can afford
/// it are all the validator's and the door's, and asking them twice is how two
/// biologies start.
pub fn lower(
    registry: &Registry,
    phenotype: &BodyPhenotype,
    proposal: &Proposal,
) -> Result<AllocationProposal, Refused> {
    // The parts the script named, sorted and deduplicated: the validator
    // requires a canonical claim, and sorting here rather than refusing means
    // an author writes what they mean instead of learning an ordering rule.
    // The order *within* a part is the script's and is kept, because that is
    // what decides which tissue each site gets.
    let mut parts: Vec<PartId> = proposal
        .sites
        .iter()
        .map(|site| PartId(site.part))
        .collect();
    parts.sort_unstable();
    parts.dedup();

    let mut sites: Vec<ProposedSite> = Vec::new();
    for part in &parts {
        // A part this body does not have at all. A part it *has and severed*
        // is not this refusal: the validator owns `SeveredPart`, and one
        // boundary is named in one place.
        let Some(mosaic) = phenotype.mosaic(*part) else {
            return Err(Refused::UnknownPart { part: *part });
        };
        let living_cells: Vec<CellId> = mosaic.cells().collect();
        let asked: u32 = proposal
            .sites
            .iter()
            .filter(|site| site.part == part.0)
            .map(|site| site.cells)
            .fold(0u32, u32::saturating_add);
        if asked as usize > living_cells.len() {
            return Err(Refused::TooMuchTissue {
                part: *part,
                asked,
                living: living_cells.len() as u32,
            });
        }

        // Hand out from the top down, in the order the script listed them.
        let mut taken: Vec<CellId> = Vec::new();
        let mut requested: Vec<(ProcessRef, Vec<CellId>)> = Vec::new();
        for site in proposal.sites.iter().filter(|site| site.part == part.0) {
            let process = resolve(registry, &site.process)?;
            let remaining = &living_cells[..living_cells.len() - taken.len()];
            let run: Vec<CellId> = remaining[remaining.len() - site.cells as usize..].to_vec();
            taken.extend(run.iter().copied());
            // A definition named twice on one part is one widened site, not two
            // sites for one process — the same reading `Candidate::propose`
            // takes, and the only one the mosaic can hold.
            match requested.iter_mut().find(|(held, _)| *held == process) {
                Some((_, cells)) => cells.extend(run),
                None => requested.push((process, run)),
            }
        }

        // **What the part already does keeps its place in the list**, and the
        // script's additions go after it. Not cosmetic: the validator hands out
        // site ids in proposal order, so a different order is a different
        // committed mosaic — and this is the order `Candidate::propose` builds,
        // which is what makes the authored and the native proposal lower to one
        // instruction rather than to two that merely look alike. Anything left
        // with no cells is dropped, which is how the validator is told to clear
        // it.
        let mut claimed: Vec<(ProcessRef, Vec<CellId>)> = mosaic
            .sites()
            .iter()
            .filter_map(|existing| {
                let kept: Vec<CellId> = existing
                    .cells
                    .iter()
                    .copied()
                    .filter(|cell| !taken.contains(cell) && mosaic.is_living(*cell))
                    .collect();
                (!kept.is_empty()).then_some((existing.process, kept))
            })
            .collect();
        for (process, run) in requested {
            match claimed.iter_mut().find(|(held, _)| *held == process) {
                Some((_, cells)) => cells.extend(run),
                None => claimed.push((process, run)),
            }
        }

        for (process, mut cells) in claimed {
            cells.sort_unstable();
            cells.dedup();
            sites.push(ProposedSite {
                part: *part,
                process,
                cells,
            });
        }
    }

    Ok(AllocationProposal {
        expect: phenotype.digest(),
        // **Authored is automatic.** `Arrangement` is diagnostic and the
        // validator never reads it; a script is the game arranging tissue
        // rather than a hand drawing it, which is what plan §7's two proposal
        // sources over one authority already say.
        source: Arrangement::Automatic,
        parts,
        sites,
    })
}

/// A qualified id, resolved against the world's own ruleset.
///
/// `None` is a real answer (plan §6, missing packs): an id this world did not
/// admit is refused by name and never replaced with the nearest local
/// definition.
fn resolve(registry: &Registry, qualified: &str) -> Result<ProcessRef, Refused> {
    let unknown = || Refused::UnknownProcess {
        id: qualified.to_owned(),
    };
    let (namespace, name) = qualified.split_once(':').ok_or_else(unknown)?;
    registry
        .get(&ProcessId::new(namespace, name))
        .map(|def| def.reference())
        .ok_or_else(unknown)
}
