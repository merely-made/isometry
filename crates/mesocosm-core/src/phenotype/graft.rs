// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Lifting a branch out of one body and setting it into another. (P3)
//!
//! # The transfer in five steps, and where each of them lives
//!
//! The wing contract lists what a graft has to do. Four of the five are here:
//!
//! 1. the source operation names a living subtree — [`BodyPhenotype::harvest`];
//! 2. destination-local part ids are freshly allocated — every part goes
//!    through [`BodyPhenotype::attach`], which is the only allocator there is;
//! 3. internal parent relations are remapped to the new ids — the branch's own
//!    offsets and yaws are preserved and its parents are rewritten as they
//!    attach, so the branch arrives shaped the way it grew;
//! 4. the graft root attaches to one destination part — the caller resolves
//!    that site through the ordinary body plan;
//! 5. every destination part retains its source address —
//!    `Origin::Incorporated { from_species, from_part }` names **that part's**
//!    donor id, not the branch root's and not the donor's root.
//!
//! The fifth of the contract's steps, that severing the graft root later
//! removes the imported subtree, needs no code at all: the branch is attached
//! under one part, and `sever` already cascades.
//!
//! # Allocation moves through the one validator
//!
//! Attaching seeds each arriving part's mosaic from its own geometry, which is
//! what [`BodyPhenotype::attach`] has always done. What the *donor* had
//! allocated then arrives as one ordinary [`AllocationProposal`] over exactly
//! the grafted parts, lowered by [`BodyPhenotype::develop`]. So there is no
//! second attachment authority and no second developmental authority: a carried
//! arrangement that this body's rules would not accept is refused by the same
//! validator that would refuse it from an editor, and it is never substituted.
//!
//! [`AllocationProposal`]: super::AllocationProposal

use super::mosaic::{CellId, Mosaic};
use super::{AllocationProposal, Arrangement, BodyPhenotype, ProposedSite, Refusal};
use crate::body::{Attachment, Origin, PartId, Provenance, SpeciesId};
use crate::process::{IntakePort, ProcessRef};

/// One part, lifted out of the body it grew in.
///
/// It carries its **source address** — the donor line and the donor-local part
/// id — because that is what makes a grafted part's provenance a fact rather
/// than a decoration, and its place inside the branch, because a branch that
/// arrived flattened would not be a branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cutting {
    /// The donor-local id this came off.
    pub source: PartId,
    /// The donor-local parent. `None` for the branch root, whose new site the
    /// recipient's own plan decides.
    pub source_parent: Option<PartId>,
    pub volume: crate::body::VolumeRef,
    pub mass_mg: u64,
    pub half_extent: [i32; 3],
    /// The joint this part had inside the branch: offset and yaw, preserved
    /// exactly. `None` for the branch root.
    pub joint: Option<Attachment>,
    /// What the donor had allocated here, in site order.
    pub sites: Vec<(ProcessRef, Vec<CellId>)>,
    /// The donor's declared intake admission, carried even if its current
    /// allocation is later adapted or temporarily inactive.
    pub port: IntakePort,
}

/// A living subtree, harvested. Parents before children.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
    /// The line the branch grew in.
    pub species: SpeciesId,
    /// The donor-local id of the branch root.
    pub root: PartId,
    pub parts: Vec<Cutting>,
}

impl Branch {
    /// What the whole branch weighs. The exact number that has to leave one
    /// body and arrive in the other.
    pub fn mass_mg(&self) -> u64 {
        self.parts.iter().map(|part| part.mass_mg).sum()
    }

    pub fn len(&self) -> usize {
        self.parts.len()
    }

    /// The box the whole branch occupies, in the branch root's own frame:
    /// where its centre sits relative to the root's pivot, and its half-extent.
    ///
    /// **What the recipient's plan has to be asked about.** A branch keeps its
    /// internal joints, so asking for room for its *root* asks the wrong
    /// question: a two-part branch whose second part reaches back along the
    /// axis it grew on will be told there is room and then land inside the body
    /// it was joining. The plan decides where a thing this big goes, and the
    /// thing is the branch.
    ///
    /// Same arithmetic `BodyDocument::world_pivot` and `world_yaw` do, over
    /// cuttings instead of parts, with the root at the origin and unturned —
    /// which is how [`crate::growth`] places anything.
    pub fn bounds(&self) -> ([i32; 3], [i32; 3]) {
        let mut placed: Vec<(PartId, [i32; 3], crate::body::Yaw)> = Vec::new();
        let (mut min, mut max) = ([i32::MAX; 3], [i32::MIN; 3]);
        for cutting in &self.parts {
            let (at, yaw) = match cutting.joint {
                None => ([0i32; 3], crate::body::Yaw::Zero),
                Some(joint) => {
                    let Some((_, parent_at, parent_yaw)) = placed
                        .iter()
                        .find(|(id, _, _)| Some(*id) == cutting.source_parent)
                        .copied()
                    else {
                        continue;
                    };
                    let swung = parent_yaw.rotate(joint.offset);
                    (
                        [
                            parent_at[0] + swung[0],
                            parent_at[1] + swung[1],
                            parent_at[2] + swung[2],
                        ],
                        parent_yaw.compose(joint.yaw),
                    )
                },
            };
            placed.push((cutting.source, at, yaw));
            for axis in 0..3 {
                let half = cutting.half_extent[axis].abs();
                min[axis] = min[axis].min(at[axis] - half);
                max[axis] = max[axis].max(at[axis] + half);
            }
        }
        if placed.is_empty() {
            return ([0; 3], [0; 3]);
        }
        let centre = [0, 1, 2].map(|axis| (min[axis] + max[axis]) / 2);
        // Rounded up, so the box the plan is asked for never under-reports the
        // branch it has to hold.
        let half = [0, 1, 2].map(|axis| ((max[axis] - min[axis] + 1) / 2).max(1));
        (centre, half)
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }
}

/// How an arriving branch's allocation is lowered.
///
/// One of three, chosen by the world from the crossing the player took and the
/// verdict its affinity table returned. Each is an ordinary proposal; none of
/// them is a privileged mutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lowering {
    /// The donor's arrangement, cell for cell. A cell the recipient's mosaic
    /// does not hold is refused by the validator rather than dropped.
    Carried,
    /// Nothing. The cut boundary does not speak this body's language, so the
    /// tissue arrives free and an adapter has to be grown on it before the
    /// branch does anything.
    Adapted,
    /// The recipient's own seeding rule for each arriving shape — what this
    /// body would have grown had it grown that part itself.
    Regrown,
}

/// What a landed graft did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graftage {
    /// The branch root's new id on the recipient.
    pub root: PartId,
    /// Every arriving part's new id, in branch order.
    pub parts: Vec<PartId>,
    /// The development the transfer committed, through the one validator.
    pub development: super::Development,
    /// What the development cost, priced part by part in each part's own
    /// tissue. Zero for a regrowth, which asks this body for nothing it would
    /// not have grown anyway.
    pub cost_mg: u64,
}

impl BodyPhenotype {
    /// Lifts a living subtree out, without changing anything.
    ///
    /// Reading and taking are two operations on purpose: the world resolves
    /// where a branch would land, and whether the recipient can hold it, before
    /// the donor loses anything.
    ///
    /// `None` for a part this body does not have, one that loss already took,
    /// or the root — a body without a root is not an injured creature, and the
    /// whole of a body is a meal rather than a branch.
    pub fn harvest(&self, root: PartId) -> Option<Branch> {
        let body = self.body();
        if root == body.root || !body.is_living(root) {
            return None;
        }
        let parts = body
            .descendants(root)
            .into_iter()
            .filter_map(|id| {
                let part = body.part(id)?;
                let mosaic = self.mosaic(id)?;
                // The branch root's own joint stays behind: it named a part of
                // the donor, and where the branch lands here is the recipient's
                // plan to decide.
                let inside = id != root;
                Some(Cutting {
                    source: id,
                    source_parent: inside
                        .then(|| part.attachment.map(|at| at.parent))
                        .flatten(),
                    volume: part.volume,
                    mass_mg: part.mass_mg,
                    half_extent: part.half_extent,
                    joint: inside.then_some(part.attachment).flatten(),
                    sites: mosaic
                        .sites()
                        .iter()
                        .map(|site| (site.process, site.cells.clone()))
                        .collect(),
                    port: mosaic.port(),
                })
            })
            .collect::<Vec<_>>();
        (!parts.is_empty()).then_some(Branch {
            species: body.species,
            root,
            parts,
        })
    }

    /// Sets a harvested branch into this body at `at`, lowering its allocation.
    ///
    /// Attaches every part — remapping the branch's internal parents onto the
    /// freshly allocated ids and preserving each joint — then submits **one**
    /// proposal for exactly those parts and commits it through
    /// [`BodyPhenotype::develop`].
    ///
    /// **A refusal leaves this phenotype already changed**, because the parts
    /// are attached before the proposal that arranges them can be validated
    /// against the body they are now part of. So callers work on a candidate
    /// clone and publish only on success — the same one-transaction discipline
    /// every other landing verb here uses, and the reason `receive` does not
    /// try to undo itself.
    pub fn receive(
        &mut self,
        registry: &crate::process::Registry,
        branch: &Branch,
        at: Attachment,
        epoch: u64,
        lowering: Lowering,
    ) -> Result<Graftage, Refusal> {
        let mut mapped: Vec<(PartId, PartId)> = Vec::new();
        let mut parts = Vec::new();
        for cutting in &branch.parts {
            let attachment = match cutting.joint {
                None => at,
                Some(joint) => {
                    let parent = cutting
                        .source_parent
                        .and_then(|source| {
                            mapped
                                .iter()
                                .find(|(from, _)| *from == source)
                                .map(|(_, to)| *to)
                        })
                        // A branch is closed under its own parents, so this is
                        // unreachable through `harvest`; refusing rather than
                        // panicking keeps a hand-built branch honest.
                        .ok_or(Refusal::NoSuchPart(cutting.source))?;
                    Attachment { parent, ..joint }
                },
            };
            let id = self
                .attach(
                    cutting.volume,
                    cutting.mass_mg,
                    cutting.half_extent,
                    attachment,
                    // **The source address, per part.** Not the branch root's,
                    // not the donor's root: the id this exact tissue had in the
                    // body it grew in.
                    Provenance {
                        origin: Origin::Incorporated {
                            from_species: branch.species,
                            from_part: cutting.source,
                        },
                        epoch,
                    },
                )
                .map_err(|_| Refusal::NoSuchPart(attachment.parent))?;
            mapped.push((cutting.source, id));
            parts.push(id);
            self.declare_port(id, cutting.port);
        }

        let sites = self.arriving_sites(branch, &parts, lowering);
        let proposal = AllocationProposal {
            expect: self.digest(),
            // A graft is the game arranging tissue, not a hand drawing it. The
            // validator never reads this; the parity receipt is what makes that
            // a property rather than a promise.
            source: Arrangement::Automatic,
            parts: parts.clone(),
            sites,
        };
        let development = self.develop(registry, &proposal)?;
        // **PD2's price, one part at a time.** A cell is worth what its own
        // part's tissue is worth, so a multi-part development cannot be priced
        // at one part's rate; the validator counted the cells whose expression
        // changed on each part, and this is only the multiplication. No second
        // comparison, and no new number.
        let cost_mg = development
            .instruction
            .cost_by_part
            .iter()
            .map(|(part, cells)| u64::from(*cells) * self.cell_mg(*part))
            .sum();
        Ok(Graftage {
            root: *parts.first().expect("a branch has a root"),
            parts,
            development,
            cost_mg,
        })
    }

    /// The sites an arriving branch proposes, by how it is being lowered.
    fn arriving_sites(
        &self,
        branch: &Branch,
        parts: &[PartId],
        lowering: Lowering,
    ) -> Vec<ProposedSite> {
        let mut sites = Vec::new();
        for (cutting, part) in branch.parts.iter().zip(parts) {
            match lowering {
                // Nothing proposed for a part that is still claimed is how the
                // validator is told to clear it.
                Lowering::Adapted => {},
                Lowering::Carried => {
                    for (process, cells) in &cutting.sites {
                        sites.push(ProposedSite {
                            part: *part,
                            process: *process,
                            cells: cells.clone(),
                        });
                    }
                },
                Lowering::Regrown => {
                    // The seeding rule, asked rather than reimplemented — the
                    // same call automatic arrangement makes, so a regrown
                    // branch cannot drift from what this body would have grown.
                    let Some(found) = self.body().part(*part) else {
                        continue;
                    };
                    for site in Mosaic::seed(found).sites() {
                        sites.push(ProposedSite {
                            part: *part,
                            process: site.process,
                            cells: site.cells.clone(),
                        });
                    }
                },
            }
        }
        sites
    }
}
