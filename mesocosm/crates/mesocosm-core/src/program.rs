// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a lineage commits, and what a descendant is born expressing. (P4, PD5)
//!
//! # A program, not a body
//!
//! **Ruled by Mark, 2026-08-03** (phenotype plan §3): the adaptation editor may
//! arrange a candidate body, but a lineage commits a **developmental program**,
//! never the candidate's literal allocation mosaic. So a [`Revision`] states
//! *declared sites* — a part role, an admitted definition, and a bounded number
//! of cells — and never a cell address or a body snapshot. Two descendants of
//! one revision may realize differently under different materials or grounds,
//! and that variance is expression of one inherited program rather than an
//! implicit mutation.
//!
//! This is the second of the ProcessDef plan's [three compiled
//! programs](../../design_docs/2026-08-01_processdef_plan.md): a **development
//! program**, run at a named discrete trigger, proposing through the existing
//! atomic validator. PE2 built the first (the condition program); the third is
//! [`ProcessDef`](crate::process::ProcessDef).
//!
//! # Revisions append; nothing edits one
//!
//! Epoch-boundary plan §2: every committed adaptation creates an **immutable
//! child revision**, and neither adopting nor branching edits the parent. So a
//! [`Program`] is an append-only list, every entry carries a digest over its
//! own rule-bearing bytes, and there is no `&mut Revision` anywhere.
//!
//! # The founding revision is stored nowhere
//!
//! What a body does with no program at all — allocation seeded from geometry —
//! *is* the founding revision. It has no parent, cites no discovery, and
//! declares no site, so writing it down would put a record in every snapshot
//! for the absence of one. [`Program::current`] answers `None` for it, and
//! nothing a world serializes moves until a line actually commits.
//!
//! # The ground charges what a line grows
//!
//! One declared condition decides how much of a revision a founder expresses,
//! and it is the game's own dormancy rule rather than a new number:
//! [`Organism::charged_mg`](crate::Organism::charged_mg) already says an
//! acquired process works only where the column under the body could replace
//! what it holds. [`Conditions::affords`] asks that at development time — a
//! line does not grow more than the ground it founds on can charge — which is
//! the same rule PD4's authored script reads, so the packed fixtures' rich and
//! lean grounds mean here what they mean there.

use serde::{Deserialize, Serialize};

use crate::body::PartId;
use crate::development::{DevelopmentError, PartPalette};
use crate::discovery::{Candidate, ConditionId, Discovery};
use crate::phenotype::{Arrangement, BodyPhenotype, Refusal};
use crate::plan::{Role, classify};
use crate::process::{ProcessRef, Registry};
use crate::species::Species;

/// Where a revision sits in one lineage's program.
///
/// Ordinals within a line, never reused, so a record naming a species and a
/// revision names exactly one committed program.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RevisionId(pub u32);

/// The discovery a revision cites.
///
/// **Both halves, because either alone is ambiguous.** The condition says which
/// question the line answered; the discovery digest pins the exact realized
/// candidate, evidence and tick it answered it with, so two worlds that agree
/// about a condition's name and disagree about what it granted cannot trade
/// revisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Citation {
    pub condition: ConditionId,
    /// [`Discovery::digest`], which resolves the whole record through
    /// [`World::discoveries`](crate::World::discoveries).
    pub discovery: u64,
}

impl Citation {
    pub fn of(discovery: &Discovery) -> Self {
        Self {
            condition: discovery.condition,
            discovery: discovery.digest,
        }
    }
}

/// One site a descendant of this line is born expressing.
///
/// **A role, not a part.** A program cannot name part 3, because the descendant
/// that grows under it has not been developed yet and its part 3 is nobody's to
/// predict. What it names is the shape the site needs, which is exactly what
/// [`Candidate`] names and what [`ProcessDef::admits`] gates.
///
/// [`ProcessDef::admits`]: crate::process::ProcessDef::admits
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclaredSite {
    /// The shape a part must classify as to carry it.
    pub role: Role,
    /// The exact admitted definition, as a content address.
    pub process: ProcessRef,
    /// How much tissue the line declares for it. What a founder actually grows
    /// is this or less; see [`Conditions::affords`].
    pub cells: u32,
}

impl DeclaredSite {
    /// The declared site a discovered candidate amounts to.
    ///
    /// **The same three rule-bearing fields**, which is why committing a
    /// revision needs no second vocabulary: a [`Candidate`] is already *which
    /// admitted process, on what shape, at what bounded capacity*, and the
    /// fourth field it carries ([`Candidate::word`]) is inheritance of a body
    /// *shape* and belongs to the recipe rather than to allocation.
    pub fn of(candidate: &Candidate) -> Self {
        Self {
            role: candidate.site,
            process: candidate.process,
            cells: candidate.cells,
        }
    }

    /// The candidate this declared site proposes as.
    ///
    /// **The same proposal construction `candidate_proposal` uses**, so a
    /// descendant expressing its line's program and a player expressing a
    /// discovery reach the one validator by the same road.
    fn candidate(&self, cells: u32) -> Candidate {
        Candidate {
            process: self.process,
            site: self.role,
            cells,
            word: None,
        }
    }
}

/// One immutable committed revision of a lineage's development program.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision {
    pub id: RevisionId,
    /// The revision this one descends from.
    ///
    /// `None` names the **founding revision**, which is stored nowhere: a body
    /// grown from geometry alone. So the first commit on a line answers `None`
    /// and every later one names its predecessor, and a child branch and its
    /// parent are distinguishable without a second table.
    pub parent: Option<RevisionId>,
    /// The discovery this revision was committed against.
    pub cites: Citation,
    /// What a descendant is born expressing.
    pub sites: Vec<DeclaredSite>,
    /// The tick it was committed on.
    pub founded: u64,
    /// Over the parent link, the citation and every declared site. Identity,
    /// so two worlds cannot hold the same revision and disagree about it.
    pub digest: u64,
}

impl Revision {
    fn digest_of(
        id: RevisionId,
        parent: Option<RevisionId>,
        cites: &Citation,
        sites: &[DeclaredSite],
    ) -> u64 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&id.0.to_le_bytes());
        match parent {
            Some(parent) => {
                bytes.push(1);
                bytes.extend_from_slice(&parent.0.to_le_bytes());
            },
            None => bytes.push(0),
        }
        bytes.extend_from_slice(&cites.condition.0.to_le_bytes());
        bytes.extend_from_slice(&cites.discovery.to_le_bytes());
        for site in sites {
            bytes.push(site.role as u8);
            bytes.extend_from_slice(&site.process.definition.0.to_le_bytes());
            bytes.extend_from_slice(&site.cells.to_le_bytes());
        }
        crate::snapshot::hash_bytes(&bytes)
    }
}

/// A lineage's development program: the revisions it has committed, in order.
///
/// Empty is the ordinary state and the **founding** one. A fork inherits its
/// parent's program whole, for the same reason it inherits the recipe: a
/// founder does not forget what its line had come to.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Program {
    committed: Vec<Revision>,
}

impl Program {
    /// The revision a descendant of this line is currently born under.
    ///
    /// `None` is the founding revision: geometry seeding, which is what every
    /// birth in this world does today.
    pub fn current(&self) -> Option<&Revision> {
        self.committed.last()
    }

    pub fn get(&self, id: RevisionId) -> Option<&Revision> {
        self.committed.iter().find(|revision| revision.id == id)
    }

    /// Every committed revision, oldest first.
    pub fn revisions(&self) -> &[Revision] {
        &self.committed
    }

    pub fn len(&self) -> usize {
        self.committed.len()
    }

    pub fn is_empty(&self) -> bool {
        self.committed.is_empty()
    }

    /// This program's identity: a digest over every revision it holds.
    ///
    /// Zero for the founding program, which is the absence of one. It moves
    /// only when a revision is committed, never when a descendant realizes one
    /// differently — which is the whole distinction a founder preview rests on.
    pub fn digest(&self) -> u64 {
        if self.committed.is_empty() {
            return 0;
        }
        let bytes: Vec<u8> = self
            .committed
            .iter()
            .flat_map(|revision| revision.digest.to_le_bytes())
            .collect();
        crate::snapshot::hash_bytes(&bytes)
    }

    /// Appends a revision. **The only mutation this type has**: nothing edits
    /// one in place, and nothing removes one, so descent through a program is
    /// as durable as descent through the lineage tree.
    pub fn commit(&mut self, cites: Citation, sites: Vec<DeclaredSite>, at: u64) -> RevisionId {
        let id = RevisionId(self.committed.len() as u32);
        let parent = self.committed.last().map(|revision| revision.id);
        let digest = Revision::digest_of(id, parent, &cites, &sites);
        self.committed.push(Revision {
            id,
            parent,
            cites,
            sites,
            founded: at,
            digest,
        });
        id
    }
}

/// The declared world conditions a development happens under.
///
/// Two, and each has a consumer here: the ground decides how much of the
/// program a body expresses, and the material pays for it. They are the same
/// two facts PD4's authoring request carries under the same names —
/// `ground_mg` and `material_mg` — so a founder preview and an authored
/// proposal are declared against one context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Conditions {
    /// What the soil column under the body holds.
    pub ground_mg: u64,
    /// What the body has banked to pay a development with.
    pub material_mg: u64,
}

impl Conditions {
    /// How much of a declared site this ground can charge.
    ///
    /// **The dormancy rule, asked one step earlier.**
    /// [`Organism::charged_mg`](crate::Organism::charged_mg) says an acquired
    /// process works only where the column could replace what it holds; a line
    /// founding on ground that could never charge the site it declares grows a
    /// token one instead of a dead one. Nothing here is tuned: the threshold
    /// *is* the site's own price, `cells * cell_mg`.
    pub fn affords(&self, declared: u32, cell_mg: u64) -> u32 {
        if self.ground_mg >= u64::from(declared) * cell_mg {
            declared
        } else {
            1
        }
    }
}

/// Everything a founder preview is realized from.
///
/// **Declared inputs, all of them.** A preview is a prediction and an
/// explanation receipt (phenotype plan §3), so it must be reproducible from
/// what it says it was grown from and nothing else. Same program, same
/// [`Founder`], same seed — same body, twice.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Founder {
    /// What the descendant is provisioned with, in milligrams of body.
    pub mass_mg: u64,
    /// The local part vocabulary a recipe grows in.
    pub palette: PartPalette,
    pub conditions: Conditions,
}

/// What a revision came to on one body.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Filial {
    pub revision: RevisionId,
    /// The part the program landed on.
    pub part: PartId,
    /// Tissue whose expression changed, as the validator counted it.
    pub cost_cells: u32,
    /// What that tissue cost, at the part's own cell price.
    pub cost_mg: u64,
}

/// Why a body could not express its line's revision.
///
/// **Every arm is a named fact, and a birth still happens.** A descendant that
/// cannot express its program is born under geometry seeding and the record
/// says which revision it could not take and why; silently growing the old body
/// and saying nothing would make an inherited program unfalsifiable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Unexpressed {
    /// This body has no living part of the declared shape. The ordinary case:
    /// a bulk consumer has nowhere to put a gland until it grows a plate.
    NoSite { role: Role },
    /// The revision declares no site at all.
    ///
    /// Unreachable through [`World::revise`](crate::World), which refuses to
    /// commit one — and present because a decoded program is not this code's
    /// to trust, the same guard `Lineages::ancestry` keeps against a cycle.
    Nothing,
    /// The one validator refused it — a stale ruleset, a shape that does not
    /// admit the definition, tissue that is not one connected region. Carried
    /// whole, because PD1b made the refusal order part of the contract.
    Refused(Refusal),
    /// The body cannot pay for what its line declares.
    Unaffordable { needed_mg: u64, held_mg: u64 },
}

impl Unexpressed {
    /// The refusal in the plain sentence a receipt prints.
    pub fn words(&self) -> String {
        match *self {
            Unexpressed::NoSite { role } => {
                format!("nowhere on this body is a {}", role_word(role))
            },
            Unexpressed::Nothing => "the revision declares nothing".to_owned(),
            Unexpressed::Refused(refusal) => format!("the validator refused it: {refusal:?}"),
            Unexpressed::Unaffordable { needed_mg, held_mg } => {
                format!("it costs {needed_mg} mg and this body has {held_mg}")
            },
        }
    }
}

fn role_word(role: Role) -> &'static str {
    match role {
        Role::Mass => "bulk",
        Role::Limb => "limb",
        Role::Plate => "plate",
        Role::Sensor => "sensor",
    }
}

/// One founder, realized and developed under one program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preview {
    /// The body, as far as the program and the conditions carried it.
    pub phenotype: BodyPhenotype,
    /// The program this was grown under, as its digest. It moves when a line
    /// commits and never when a descendant realizes the same program
    /// differently, which is what the rich-and-lean receipt asserts.
    pub program: u64,
    /// What the line's current revision came to. `None` under the founding
    /// revision, which declares nothing.
    pub filial: Option<Result<Filial, Unexpressed>>,
}

/// Realizes one founder from a lineage's program and declared conditions.
///
/// **Deterministic, and it touches no world.** It realizes the recipe exactly
/// as a birth does — `Species::realize` then
/// [`BodyPhenotype::seed`](crate::BodyPhenotype::seed), which is the founding
/// revision — and then runs the line's current revision through
/// [`express`], which is the same function a birth runs. That shared call is
/// what makes "a descendant grown under the same declared inputs reproduces its
/// founder preview" a property rather than two implementations agreeing.
pub fn preview(
    species: &Species,
    registry: &Registry,
    founder: Founder,
    seed: u64,
) -> Result<Preview, DevelopmentError> {
    let body = species.realize(seed, founder.mass_mg, founder.palette)?;
    let mut grown = BodyPhenotype::seed(body);
    let stock = species
        .initial_tissue
        .stock_for(grown.body().total_mass_mg())
        .expect("a lineage preview needs declared founding tissue");
    grown
        .distribute_stock(stock)
        .expect("the recipe was scaled to this preview body");
    species.apply_intake_ports(&mut grown);
    let program = species.program().digest();
    let Some(revision) = species.program().current() else {
        return Ok(Preview {
            phenotype: grown,
            program,
            filial: None,
        });
    };
    match express(revision, registry, &grown, founder.conditions) {
        Ok((phenotype, filial)) => Ok(Preview {
            phenotype,
            program,
            filial: Some(Ok(filial)),
        }),
        Err(why) => Ok(Preview {
            phenotype: grown,
            program,
            filial: Some(Err(why)),
        }),
    }
}

/// Runs one revision over one body, under declared conditions.
///
/// **Through the one validator, and nothing published on a refusal.** The
/// development happens on a candidate copy and the price is read off the
/// validated instruction, so a body that cannot afford its line's program keeps
/// both its milligrams and the arrangement geometry gave it — the same ordering
/// [`World::express`](crate::World) uses for the played door, for the same
/// reason.
pub fn express(
    revision: &Revision,
    registry: &Registry,
    phenotype: &BodyPhenotype,
    conditions: Conditions,
) -> Result<(BodyPhenotype, Filial), Unexpressed> {
    let mut candidate = phenotype.clone();
    let mut landed: Option<PartId> = None;
    let mut cost_cells = 0u32;
    let mut cost_mg = 0u64;

    for site in &revision.sites {
        let part = site_on(&candidate, site.role).ok_or(Unexpressed::NoSite { role: site.role })?;
        let cell_mg = candidate.cell_mg(part);
        let cells = conditions.affords(site.cells, cell_mg);
        let proposal = site
            .candidate(cells)
            .propose(&candidate, Arrangement::Automatic)
            .ok_or(Unexpressed::NoSite { role: site.role })?;
        let development = candidate
            .develop(registry, &proposal)
            .map_err(Unexpressed::Refused)?;
        cost_cells += development.instruction.cost_cells;
        cost_mg += u64::from(development.instruction.cost_cells) * cell_mg;
        landed = Some(part);
    }

    let part = landed.ok_or(Unexpressed::Nothing)?;
    if cost_mg > conditions.material_mg {
        return Err(Unexpressed::Unaffordable {
            needed_mg: cost_mg,
            held_mg: conditions.material_mg,
        });
    }
    Ok((
        candidate,
        Filial {
            revision: revision.id,
            part,
            cost_cells,
            cost_mg,
        },
    ))
}

/// The first living part of a shape, in part order.
///
/// The same rule [`Candidate::propose`] picks with, asked here so the price can
/// be read before the proposal is built. Two answers to *which part* would be
/// two biologies, so this is the only one.
fn site_on(phenotype: &BodyPhenotype, role: Role) -> Option<PartId> {
    let body = phenotype.body();
    phenotype.allocations().map(|(part, _)| part).find(|part| {
        body.part(*part)
            .is_some_and(|found| classify(found.half_extent) == role)
    })
}

#[cfg(test)]
mod tests;
