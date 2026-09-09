// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Anatomy and allocation, held together.
//!
//! # Why a wrapper
//!
//! PD1a compared three owners for process allocation and ruled this one. The
//! two it rejected are worth keeping visible, because both are the shape this
//! file exists to prevent:
//!
//! - **Allocation fields inside [`Part`](crate::body::Part)** would make the
//!   structural anatomy document depend on Mesocosm's process semantics, and
//!   change the body-only bytes mesh, Lens and other vessels consume.
//! - **A freely mutable allocation state beside a freely mutable
//!   [`BodyDocument`]** permits exactly the split account PD0 spent a
//!   migration removing: a part attached, severed or restored without its
//!   allocation following.
//!
//! So both representations stay independently *readable* while every mutation
//! that affects both is one operation. The fields here are private and no
//! caller can obtain `&mut BodyDocument` from a live organism; body-only
//! authoring still uses `BodyDocument` directly, which keeps primitive
//! topology usable without loading the biology system.
//!
//! # The invariants
//!
//! - Every living part has exactly one mosaic, and every living allocation
//!   names a living part. The mosaics are index-aligned with
//!   `BodyDocument::parts`, so an anatomy tombstone **is** an allocation
//!   tombstone; they cannot disagree because there is nothing to synchronize.
//! - Every part's mosaic conserves capacity: occupied plus free equals the
//!   count of its living cells.
//! - Rearrangement happens only through [`BodyPhenotype::develop`], which
//!   validates a complete proposal, bumps the revision, and returns the record
//!   of what it did. There is no other way in.
//!
//! # Severing keeps history
//!
//! A severed part's mosaic stays addressable through [`BodyPhenotype::mosaic`]
//! so an injury is still explainable, and is excluded from
//! [`BodyPhenotype::allocations`] so it cannot contribute. Historical cells and
//! sites remain readable; they are not capacity and they express nothing.

use serde::{Deserialize, Deserializer, Serialize};

use crate::body::{
    AttachError, Attachment, BodyDocument, PartId, Provenance, SpeciesId, VolumeRef,
};
use crate::process::{IntakePort, NisKind, Process, ProcessRef, Registry};

pub mod develop;
pub mod graft;
pub mod mosaic;
pub mod substance;

pub use develop::{
    Aim, AllocationProposal, Arrangement, Development, Instruction, ProposedSite, Refusal, arrange,
};
pub use graft::{Branch, Cutting, Graftage, Lowering};
pub use mosaic::{CellId, Expressed, MAX_CELLS, MAX_SITES, Mosaic, Site, SiteId};
pub use substance::{AttachStockError, StockMassError, SubstanceError};

/// One critter's anatomy and its process allocation, as one transactional
/// value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct BodyPhenotype {
    body: BodyDocument,
    /// Index-aligned with `body.parts`. That alignment is the invariant: a
    /// part and its mosaic are addressed by the same number.
    mosaics: Vec<Mosaic>,
    /// Bumped by every committed development, so a proposal authored against
    /// an earlier arrangement is refusable and the order of rearrangements is
    /// on the record.
    revision: u32,
}

impl BodyPhenotype {
    /// Wraps a finished anatomy, seeding one mosaic per part from its shape.
    ///
    /// The only constructor. A `BodyDocument` becomes an organism's body by
    /// passing through here, so there is no path that produces a live body
    /// with no allocation.
    pub fn seed(body: BodyDocument) -> Self {
        let mosaics = body.parts.iter().map(Mosaic::seed).collect();
        let mut phenotype = Self {
            body,
            mosaics,
            revision: 0,
        };
        phenotype.seed_intake_ports();
        debug_assert!(phenotype.conserves(), "a seeded mosaic must conserve");
        phenotype
    }

    /// The anatomy, for everything that draws, weighs, places or projects it.
    pub fn body(&self) -> &BodyDocument {
        &self.body
    }

    /// The anatomy alone — the **carry this body** projection's structural
    /// half, and what a body-only consumer receives.
    pub fn into_body(self) -> BodyDocument {
        self.body
    }

    /// Every living part's allocation, in part order.
    pub fn allocations(&self) -> impl Iterator<Item = (PartId, &Mosaic)> {
        self.body
            .living()
            .map(|part| (part.id, &self.mosaics[part.id.0 as usize]))
    }

    /// One part's mosaic, severed parts included, because an injury still has
    /// to be explainable.
    pub fn mosaic(&self, part: PartId) -> Option<&Mosaic> {
        self.mosaics.get(part.0 as usize)
    }

    /// The declaration on one living part. A declared port is inactive when
    /// that part has no living allocation, without discarding its declaration.
    pub fn part_port(&self, part: PartId) -> Option<IntakePort> {
        if !self.body.is_living(part) {
            return None;
        }
        let mosaic = self.mosaic(part)?;
        let port = mosaic.port();
        port.support()
            .is_some_and(|support| {
                mosaic.sites().iter().any(|site| {
                    site.process == support && site.cells.iter().any(|cell| mosaic.is_living(*cell))
                })
            })
            .then_some(port)
    }

    /// The union of all active intake declarations, in deterministic part
    /// order. Views may inspect this directly; rules ask `admits` below.
    pub fn intake_ports(&self) -> IntakePort {
        self.allocations()
            .fold(IntakePort::none(), |all, (part, _)| {
                let Some(port) = self.part_port(part) else {
                    return all;
                };
                let mut merged = all;
                for kind in [NisKind::Producer, NisKind::Consumer, NisKind::Decomposer] {
                    if port.admits_live(kind) {
                        merged = merged.with_live(kind);
                    }
                }
                if port.admits_deadstock() {
                    merged = merged.with_deadstock();
                }
                merged
            })
    }

    /// Declares one part's intake admission. This is the explicit anatomy
    /// authoring seam used by roster construction and retained by snapshots.
    pub fn declare_port(&mut self, part: PartId, port: IntakePort) -> bool {
        if !self.body.is_living(part) {
            return false;
        }
        let Some(mosaic) = self.mosaics.get_mut(part.0 as usize) else {
            return false;
        };
        mosaic.declare_port(port);
        true
    }

    fn seed_intake_ports(&mut self) {
        let registry = Registry::native();
        // Existing mixotroph precedence remains: a canopy is a producer even
        // if its geometry also happens to carry a head-borne feeding shape.
        if self.canopy() {
            return;
        }
        let mouths: Vec<_> = self
            .body
            .living()
            .filter_map(|part| {
                part.attachment
                    .is_some_and(|at| at.parent == self.body.root && at.offset[1] < 0)
                    .then_some((part.id, crate::plan::classify(part.half_extent)))
            })
            .filter(|(_, role)| matches!(role, crate::plan::Role::Mass | crate::plan::Role::Limb))
            .collect();
        if mouths.is_empty() {
            self.declare_port(
                self.body.root,
                IntakePort::deadstock()
                    .supported_by(registry.of_native(Process::Intake).reference()),
            );
        } else {
            for (part, role) in mouths {
                let port = match role {
                    crate::plan::Role::Mass => IntakePort::live(NisKind::Producer)
                        .supported_by(registry.of_native(Process::Intake).reference()),
                    crate::plan::Role::Limb => IntakePort::live(NisKind::Consumer)
                        // Initial jaw ports admit consumer nis only. A jaw that
                        // also admits decomposer nis is an authored declaration,
                        // not a hidden consequence of being predatory.
                        .supported_by(registry.of_native(Process::Contract).reference()),
                    crate::plan::Role::Plate | crate::plan::Role::Sensor => continue,
                };
                self.declare_port(part, port);
            }
        }
    }

    /// How many developments this phenotype has committed.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// What a proposal must expect, and what a commit reports.
    ///
    /// Over anatomy, allocation and revision together, so a body that grew, an
    /// allocation that moved, and a rearrangement that happened are all
    /// staleness.
    pub fn digest(&self) -> u64 {
        let bytes = crate::snapshot::encode(self).expect("a phenotype is always encodable");
        crate::snapshot::hash_bytes(&bytes)
    }

    /// Whether any living allocation expresses this definition.
    ///
    /// The allocation-side reading of what a body does. It agrees with
    /// [`BodyDocument::performs`] by construction while geometry is the whole
    /// seeding rule, and the receipts assert that it does.
    pub fn expresses(&self, process: ProcessRef) -> bool {
        self.allocations()
            .any(|(_, mosaic)| mosaic.sites().iter().any(|site| site.process == process))
    }

    /// Whether one living part has tissue allocated to a definition.
    ///
    /// The part-scoped form of [`Self::expresses`], and the question PD2's
    /// anatomy readings ask: "is this plate actually fixing" is a different
    /// question from "does this body have a plate", and the two only diverge
    /// once a development can take the tissue away.
    pub fn expresses_on(&self, part: PartId, process: ProcessRef) -> bool {
        self.body.is_living(part)
            && self
                .mosaic(part)
                .is_some_and(|mosaic| mosaic.sites().iter().any(|site| site.process == process))
    }

    /// What one cell of a part's tissue is worth, in milligrams.
    ///
    /// **Derived twice over, and no new constant.** The numerator is the
    /// adult mass this part's own voxel volume implies — TD6's
    /// [`part_ceiling_mg`](crate::organism::ecology::part_ceiling_mg), the
    /// same number rent, breeding and intake room are all measured against —
    /// and the denominator is the part's living cell count, which is the
    /// mosaic's own structural capacity. So a cell is *this part's adult mass
    /// divided by the tissue it is divided into*, which is what a cell has
    /// always meant; nothing here picks a price.
    ///
    /// Floored at one, for the reason `part_ceiling_mg` is: a legal part must
    /// not have worthless tissue.
    pub fn cell_mg(&self, part: PartId) -> u64 {
        let Some(found) = self.body.part(part) else {
            return 0;
        };
        let Some(mosaic) = self.mosaic(part) else {
            return 0;
        };
        (crate::organism::ecology::part_ceiling_mg(found.half_extent)
            / u64::from(mosaic.capacity()).max(1))
        .max(1)
    }

    /// The toxin this body carries: every living cell allocated to
    /// [`Process::Secrete`](crate::process::Process::Secrete), priced as the
    /// tissue it is.
    ///
    /// **The first quantitative consumer of the mosaic.** Everything before
    /// PD2 asked allocation a yes-or-no question; this asks *how much*, which
    /// is what makes taking one more cell off a frond a decision with a
    /// magnitude rather than a flag to flip.
    pub fn secretory_mg(&self) -> u64 {
        let gland = Self::gland_reference();
        self.allocations()
            .map(|(part, mosaic)| {
                let cells: u64 = mosaic
                    .sites()
                    .iter()
                    .filter(|site| site.process == gland)
                    .map(|site| site.cells.iter().filter(|c| mosaic.is_living(**c)).count() as u64)
                    .sum();
                cells * self.cell_mg(part)
            })
            .sum()
    }

    /// The living parts carrying secretory tissue, and how many cells each.
    ///
    /// Plain working vocabulary: the process is `secrete` and the organ is a
    /// gland. Neither word is a product name, and the in-product naming round
    /// is Mark's, as it is for `fix`.
    pub fn glands(&self) -> Vec<(PartId, u32)> {
        let gland = Self::gland_reference();
        self.allocations()
            .filter_map(|(part, mosaic)| {
                let cells: u32 = mosaic
                    .sites()
                    .iter()
                    .filter(|site| site.process == gland)
                    .map(|site| site.cells.iter().filter(|c| mosaic.is_living(**c)).count() as u32)
                    .sum();
                (cells > 0).then_some((part, cells))
            })
            .collect()
    }

    /// The parts that carried secretory tissue and were severed.
    ///
    /// **The fourth state, and why the mosaics of dead parts stay
    /// addressable.** Severing takes the consequence away — a lost gland
    /// expresses nothing and costs nothing — and a player is still owed the
    /// sentence "that branch is where your sting was".
    pub fn lost_glands(&self) -> Vec<PartId> {
        let gland = Self::gland_reference();
        self.body
            .parts
            .iter()
            .filter(|part| part.severed)
            .filter(|part| {
                self.mosaic(part.id)
                    .is_some_and(|m| m.sites().iter().any(|site| site.process == gland))
            })
            .map(|part| part.id)
            .collect()
    }

    fn gland_reference() -> ProcessRef {
        Registry::native()
            .of_native(crate::process::Process::Secrete)
            .reference()
    }

    /// Every living part that expresses a definition, in part order.
    pub fn expressing(&self, process: ProcessRef) -> impl Iterator<Item = PartId> + '_ {
        self.allocations().filter_map(move |(part, mosaic)| {
            mosaic
                .sites()
                .iter()
                .any(|site| site.process == process)
                .then_some(part)
        })
    }

    /// Every expressed process and how it came to be there, sorted.
    ///
    /// What survives a re-realization: the same program under different
    /// declared conditions may grow a different body, and this is the identity
    /// and provenance that must not have moved with it.
    pub fn expressed(&self) -> Vec<(ProcessRef, Expressed)> {
        let mut found: Vec<(ProcessRef, Expressed)> = self
            .allocations()
            .flat_map(|(_, mosaic)| mosaic.sites())
            .map(|site| (site.process, site.cause))
            .collect();
        found.sort_unstable();
        found.dedup();
        found
    }

    /// A capability trace for one part: what it expresses, on how much tissue,
    /// how much is free, and where the expression came from.
    ///
    /// `None` for a part this body does not have. A severed part answers with
    /// its history, because "this used to fix, and the branch is gone" is the
    /// receipt a player is owed.
    pub fn explain(&self, part: PartId) -> Option<Explanation> {
        let mosaic = self.mosaic(part)?;
        let registry = Registry::native();
        Some(Explanation {
            part,
            living: self.body.is_living(part),
            capacity: mosaic.capacity(),
            free: mosaic.free(),
            sites: mosaic
                .sites()
                .iter()
                .map(|site| SiteReading {
                    id: site.id,
                    process: site.process,
                    // `None` is the missing-ruleset diagnostic, not a licence
                    // to name a similar local process instead.
                    named: registry.resolve(site.process).map(|def| def.id.clone()),
                    cells: site.cells.len() as u32,
                    cause: site.cause,
                })
                .collect(),
        })
    }

    /// Whether every invariant this wrapper exists to hold, holds.
    pub fn conserves(&self) -> bool {
        if self.body.parts.len() != self.mosaics.len() {
            return false;
        }
        self.body
            .parts
            .iter()
            .zip(&self.mosaics)
            .enumerate()
            .all(|(index, (part, mosaic))| {
                part.id.0 as usize == index
                    && mosaic.conserves()
                    && mosaic.capacity() <= MAX_CELLS
                    && mosaic.scruple().total() == u128::from(part.mass_mg)
            })
    }

    /// Attaches a part **and** seeds its mosaic, in one operation.
    ///
    /// Either the body gained a part with an allocation or it gained nothing.
    /// This is the whole reason `Organism` does not expose `&mut BodyDocument`.
    pub fn attach(
        &mut self,
        volume: VolumeRef,
        mass_mg: u64,
        half_extent: [i32; 3],
        attachment: Attachment,
        provenance: Provenance,
    ) -> Result<PartId, AttachError> {
        let id = self
            .body
            .attach(volume, mass_mg, half_extent, attachment, provenance)?;
        let part = self.body.part(id).expect("just attached");
        self.mosaics.push(Mosaic::seed(part));
        debug_assert!(self.conserves(), "an attach must leave the mosaics whole");
        Ok(id)
    }

    /// Severs a part and everything under it, taking their allocations out of
    /// activity in the same commit.
    ///
    /// Returns what was lost. The mosaics stay addressable as history.
    pub fn sever(&mut self, part: PartId) -> Vec<PartId> {
        self.body.sever(part)
    }

    /// Which lineage this body belongs to. Forking is the only caller.
    pub fn set_species(&mut self, species: SpeciesId) {
        self.body.species = species;
    }

    /// **The only way allocation moves.**
    ///
    /// Validates a complete proposal against this phenotype and either commits
    /// all of it or none of it. A refusal leaves body, allocation and revision
    /// byte-identical; an acceptance bumps the revision and returns the record
    /// of what changed hands.
    ///
    /// `registry` is **the ruleset this body's world admitted** (PD4), and the
    /// only place a definition may be resolved from. A caller inside a world
    /// passes [`World::ruleset`](crate::World::ruleset); nothing here reaches
    /// for a global.
    pub fn develop(
        &mut self,
        registry: &Registry,
        proposal: &AllocationProposal,
    ) -> Result<Development, Refusal> {
        let validated = develop::validate(registry, self, proposal)?;
        let revision = self.revision + 1;
        let mut sites = Vec::new();
        for (part, desired) in validated.rewrites {
            let mosaic = &mut self.mosaics[part.0 as usize];
            mosaic.rewrite(desired, revision);
            for site in mosaic.sites() {
                sites.push((part, site.id, site.process));
            }
        }
        self.revision = revision;
        debug_assert!(
            self.conserves(),
            "a committed development must leave the mosaics whole"
        );
        Ok(Development {
            instruction: Instruction {
                revision,
                parts: proposal.parts.clone(),
                sites,
                cost_cells: validated.cost_cells,
                cost_by_part: validated.cost_by_part,
                digest: self.digest(),
            },
            source: proposal.source,
        })
    }
}

#[derive(Deserialize)]
struct StoredPhenotype {
    body: BodyDocument,
    mosaics: Vec<Mosaic>,
    revision: u32,
}

impl<'de> Deserialize<'de> for BodyPhenotype {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let stored = StoredPhenotype::deserialize(deserializer)?;
        let phenotype = Self {
            body: stored.body,
            mosaics: stored.mosaics,
            revision: stored.revision,
        };
        phenotype
            .conserves()
            .then_some(phenotype)
            .ok_or_else(|| serde::de::Error::custom("invalid body phenotype mass or mosaic"))
    }
}

/// One part's allocation, in terms a journal or a failure receipt can print.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Explanation {
    pub part: PartId,
    /// `false` for a severed part, whose sites are history.
    pub living: bool,
    pub capacity: u32,
    pub free: u32,
    pub sites: Vec<SiteReading>,
}

/// One expressed process, explained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SiteReading {
    pub id: SiteId,
    pub process: ProcessRef,
    /// The qualified id, when this world's ruleset holds the definition.
    /// `None` is the missing-ruleset diagnostic.
    pub named: Option<crate::process::ProcessId>,
    pub cells: u32,
    pub cause: Expressed,
}

#[cfg(test)]
mod tests;
