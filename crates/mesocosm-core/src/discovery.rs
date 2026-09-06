// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a line comes to a new developmental option. (PE2)
//!
//! # Unlocks are evidence, not a diet tree
//!
//! Eating may supply material, a donor, and an observation. It does **not** map
//! a food category to a matching reward category (playable ecology plan §1). So
//! the thing that unlocks is a [`Condition`]: a small declared program over
//! accepted [`Evidence`], which may cite a donor part, survival through a
//! quantified stress, or — later — repeated use, exposure, a relationship or a
//! lineage achievement. `World::learn_from` taught every non-innate appendage in
//! the *donor's whole recipe* on every meal; it was named a migration input and
//! this is what replaced it.
//!
//! # The execution boundary, made structural
//!
//! The traits brief's 2026-09-01 boundary asks for four things and each is a
//! property of the types here rather than a discipline somebody has to keep:
//!
//! - **Event-driven.** [`evaluate`] runs once per accepted piece of evidence.
//!   Nothing polls organisms, and no condition runs on an ecology tick.
//! - **Declared inputs.** A [`Condition`] names the [`Input`] lanes it will
//!   accept, and evidence of any other kind cannot reach it — it is recorded
//!   as [`Miss::UndeclaredInput`] instead. A meal therefore cannot satisfy an
//!   endurance condition however hungry the eater was.
//! - **Bounded.** The condition table is fixed and small, a rule is a compare
//!   against one integer, and the only accumulator a rule reads is an
//!   authoritative one the world keeps (`World::hunger_run`) — never a
//!   view-only trend.
//! - **Recorded.** A [`Discovery`] carries the matched evidence, the route it
//!   arrived through, the **realized candidate reference** (an exact
//!   [`ProcessRef`], never a name), its parameters, its [`Source`], and a
//!   digest over all of it.
//!
//! # A candidate, not an applied change
//!
//! What a condition grants is a [`Candidate`]: a proposal the **one validator**
//! ([`BodyPhenotype::develop`](crate::phenotype::BodyPhenotype::develop)) can
//! lower, and nothing else. [`Candidate::propose`] builds an ordinary
//! [`AllocationProposal`] from it, which is why direct and automatic
//! arrangement cannot diverge here: there is one proposal shape and one
//! validator under it, and [`Arrangement`] is diagnostic metadata the validator
//! never reads.
//!
//! Discovering it is therefore **developmental availability**, and it is a
//! different fact from **expression** — the player still arranges it — and from
//! **inheritance**, which is [`Candidate::word`]: the lexicon entry a
//! descendant is born able to grow.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::axis::Appendage;
use crate::body::{PartId, SpeciesId};
use crate::phenotype::{AllocationProposal, Arrangement, BodyPhenotype, CellId, ProposedSite};
use crate::plan::{Role, classify};
use crate::process::ProcessRef;

mod conditions;

pub use conditions::{
    Condition, ConditionId, HUNGER_TICKS, MEAL_EVIDENCE_MG, conditions, name_of, resolve,
};

/// A lane evidence can arrive through.
///
/// **A condition declares the lanes it accepts**, and routing checks the
/// declaration before it checks the rule. That is the whole of "a meal routes
/// only to conditions that declared that input".
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Input {
    /// Something was eaten, and a donor part was observed doing it.
    Meal,
    /// A body came through a quantified stress.
    Endurance,
}

impl Input {
    /// The lane's name, for a panel that has to say which route a discovery
    /// came by.
    pub fn name(self) -> &'static str {
        match self {
            Input::Meal => "eaten",
            Input::Endurance => "endured",
        }
    }
}

/// A stress a body can be measured as having come through.
///
/// One today, because one condition consumes one. A stress is admitted here
/// only when an **authoritative bounded accumulator** already exists for it;
/// a view-only trend cannot become a stress by being named.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Stress {
    /// The budget under the starved line, with a hand on the body.
    Hunger,
}

impl Stress {
    pub fn name(self) -> &'static str {
        match self {
            Stress::Hunger => "hunger",
        }
    }
}

/// One accepted fact, quantified.
///
/// Organism-neutral by construction: nothing here names the player. Which
/// bodies get to *offer* evidence is the world's question and is currently the
/// played one only (playable ecology plan §6, ruling 5).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Evidence {
    /// A part of another body was consumed: whose it was, which part of it,
    /// what shape that part is, and what it weighed.
    Meal {
        donor: SpeciesId,
        part: PartId,
        role: Role,
        mass_mg: u64,
    },
    /// A body came through a stress, and this is how long it lasted.
    Endured { stress: Stress, ticks: u64 },
}

impl Evidence {
    /// The lane this arrived through.
    pub fn input(&self) -> Input {
        match self {
            Evidence::Meal { .. } => Input::Meal,
            Evidence::Endured { .. } => Input::Endurance,
        }
    }

    /// Where the candidate would have come from, if this evidence granted one.
    pub fn source(&self) -> Source {
        match *self {
            Evidence::Meal { donor, part, .. } => Source::Donor {
                lineage: donor,
                part,
            },
            Evidence::Endured { .. } => Source::Endured,
        }
    }

    /// The evidence in plain words, quantities kept.
    pub fn words(&self) -> String {
        match *self {
            Evidence::Meal {
                donor,
                part,
                role,
                mass_mg,
            } => format!(
                "{} part {} of line {}, {mass_mg} mg",
                shape_word(role),
                part.0,
                donor.0
            ),
            Evidence::Endured { stress, ticks } => {
                format!("{} for {ticks} ticks", stress.name())
            },
        }
    }
}

/// A part shape, in the plain word a player would use for it.
fn shape_word(role: Role) -> &'static str {
    match role {
        Role::Mass => "bulk",
        Role::Limb => "limb",
        Role::Plate => "plate",
        Role::Sensor => "sensor",
    }
}

/// The bounded program a condition is.
///
/// One compare against one integer each. A rule can read only what the
/// evidence carries, so there is nowhere for an unbounded walk to hide.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rule {
    /// A consumed part of this shape, weighing at least this much.
    Consumed { role: Role, mass_mg: u64 },
    /// This many consecutive ticks under this stress.
    Endured { stress: Stress, ticks: u64 },
}

impl Rule {
    /// Whether this evidence satisfies the rule. Never asked before the input
    /// declaration has been checked.
    pub fn met_by(&self, evidence: &Evidence) -> bool {
        match (*self, *evidence) {
            (
                Rule::Consumed {
                    role: want,
                    mass_mg: least,
                },
                Evidence::Meal { role, mass_mg, .. },
            ) => role == want && mass_mg >= least,
            (
                Rule::Endured {
                    stress: want,
                    ticks: least,
                },
                Evidence::Endured { stress, ticks },
            ) => stress == want && ticks >= least,
            _ => false,
        }
    }
}

/// What a condition grants: a proposal the one validator can lower.
///
/// Never an applied change, and never a capability. The definition travels as
/// a content address for PD1b's reason — a world that does not hold that exact
/// definition must refuse rather than substitute the nearest thing it does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    /// The exact admitted definition this would express.
    pub process: ProcessRef,
    /// The shape a site must be. **A parameter, not an authority**: the one
    /// validator checks it through [`ProcessDef::admits`], and this is what a
    /// proposal builder aims at.
    ///
    /// [`ProcessDef::admits`]: crate::process::ProcessDef::admits
    pub site: Role,
    /// How much tissue the proposal would take.
    pub cells: u32,
    /// The word this adds to the line's lexicon, when it adds one.
    ///
    /// **Inheritance, and a different fact from expression.** A word lets a
    /// descendant be *born* with the shape; expressing the process on this body
    /// is still a development somebody has to pay for.
    pub word: Option<Appendage>,
}

impl Candidate {
    /// The proposal this candidate would submit, or `None` when this body has
    /// nowhere to put it.
    ///
    /// **One proposal shape, one validator.** The result is an ordinary
    /// [`AllocationProposal`] and goes through the same
    /// [`BodyPhenotype::develop`](crate::phenotype::BodyPhenotype::develop) a
    /// hand-drawn one does; `source` rides along as diagnostic metadata the
    /// validator never reads, which is why a direct and an automatic caller
    /// lower the same candidate to the same instruction.
    ///
    /// `None` is a real state and not a failure: a candidate is *available*
    /// before it is expressible, and a consumer that has never grown a plate
    /// has nowhere to put a gland until it does.
    pub fn propose(
        &self,
        phenotype: &BodyPhenotype,
        source: Arrangement,
    ) -> Option<AllocationProposal> {
        let body = phenotype.body();
        // The first living part of the right shape, in part order, so the
        // answer is deterministic rather than whichever iteration reached it.
        let (part, mosaic) = phenotype.allocations().find(|(part, _)| {
            body.part(*part)
                .is_some_and(|found| classify(found.half_extent) == self.site)
        })?;

        // The high end of the lattice. A suffix of the row-major order is a
        // connected region and so is the prefix left behind, which is the same
        // property `Mosaic::seed` relies on when it shares a part out.
        let living: Vec<CellId> = mosaic.cells().collect();
        let take = (self.cells as usize).min(living.len());
        if take == 0 {
            return None;
        }
        let taken: Vec<CellId> = living[living.len() - take..].to_vec();

        // A complete desired state for the part: what the rest of it keeps
        // doing, plus the new site. Anything left with no cells is cleared.
        let mut sites: Vec<ProposedSite> = Vec::new();
        for site in mosaic.sites() {
            let kept: Vec<CellId> = site
                .cells
                .iter()
                .copied()
                .filter(|cell| !taken.contains(cell) && mosaic.is_living(*cell))
                .collect();
            if kept.is_empty() {
                continue;
            }
            sites.push(ProposedSite {
                part,
                process: site.process,
                cells: kept,
            });
        }
        match sites.iter_mut().find(|site| site.process == self.process) {
            // Already expressed here: widen it rather than proposing a second
            // site for the same definition.
            Some(existing) => {
                existing.cells.extend(taken);
                existing.cells.sort_unstable();
                existing.cells.dedup();
            },
            None => sites.push(ProposedSite {
                part,
                process: self.process,
                cells: taken,
            }),
        }

        Some(AllocationProposal {
            expect: phenotype.digest(),
            source,
            parts: vec![part],
            sites,
        })
    }
}

/// Where a granted candidate came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Source {
    /// The body's own experience. There is no donor.
    Endured,
    /// Off a donor: which lineage, and which of its parts.
    Donor { lineage: SpeciesId, part: PartId },
}

/// A landed discovery: everything the execution boundary asks be recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discovery {
    pub tick: u64,
    pub epoch: u64,
    pub condition: ConditionId,
    /// The declared lane the evidence came through.
    pub route: Input,
    /// The evidence that matched, quantities kept.
    pub evidence: Evidence,
    /// The realized candidate: exact definition reference and parameters.
    pub candidate: Candidate,
    pub source: Source,
    /// Over the condition, the candidate, the evidence and when it happened.
    pub digest: u64,
}

/// Why a condition did not take a piece of evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Miss {
    /// **The boundary, structurally.** The condition never declared this lane,
    /// so the evidence could not reach its rule at all. A meal cannot satisfy
    /// an endurance condition, whatever it weighed.
    UndeclaredInput,
    /// It declared the lane, and this evidence does not satisfy the rule.
    RuleUnmet,
    /// Already discovered. A second one is just a meal.
    AlreadyKnown,
    /// The rule *was* met and another condition took the evidence first.
    ///
    /// Unreachable with today's two conditions, which read different lanes, and
    /// present because the alternative is recording a met rule as unmet — a
    /// lie the first generated condition that overlaps another would tell.
    AnotherTook,
}

impl Miss {
    pub fn words(self) -> &'static str {
        match self {
            Miss::UndeclaredInput => "not a question this asks",
            Miss::RuleUnmet => "not enough evidence",
            Miss::AlreadyKnown => "already known",
            Miss::AnotherTook => "something else took it first",
        }
    }
}

/// One routing of one piece of evidence, and what every condition made of it.
///
/// Bounded by the condition table, and kept because *evidence that unlocked
/// nothing is still evidence*: "a meal supplies evidence without unlocking an
/// incompatible candidate" is a claim about a record, and this is the record.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observation {
    pub tick: u64,
    pub route: Input,
    pub evidence: Evidence,
    /// The condition that took it, if one did.
    pub matched: Option<ConditionId>,
    /// The conditions that did not, and why. In table order.
    pub missed: Vec<(ConditionId, Miss)>,
}

/// What routing one piece of evidence came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub observation: Observation,
    /// `Some` only when a condition took the evidence and had not already.
    pub discovery: Option<Discovery>,
}

/// Routes one accepted piece of evidence against the condition table.
///
/// **Event-driven and bounded.** One call per accepted fact, one pass over a
/// fixed table, one integer compare per entry. Nothing here polls an organism,
/// reruns worldgen from a name, or reads a presentation trend.
pub fn evaluate(
    evidence: Evidence,
    tick: u64,
    epoch: u64,
    known: &BTreeSet<ConditionId>,
) -> Verdict {
    let route = evidence.input();
    let mut matched = None;
    let mut discovery = None;
    let mut missed = Vec::new();

    for condition in conditions() {
        let id = condition.id();
        // The declaration is checked before the rule, which is what makes
        // "routes only to conditions that declared that input" a property of
        // the routing rather than of each rule's own carefulness.
        if !condition.declares(route) {
            missed.push((id, Miss::UndeclaredInput));
            continue;
        }
        if !condition.rule.met_by(&evidence) {
            missed.push((id, Miss::RuleUnmet));
            continue;
        }
        if known.contains(&id) {
            missed.push((id, Miss::AlreadyKnown));
            continue;
        }
        // First match wins, and the rest of the table is still recorded, so a
        // receipt can say what else the evidence was offered to.
        if matched.is_none() {
            matched = Some(id);
            discovery = Some(Discovery {
                tick,
                epoch,
                condition: id,
                route,
                evidence,
                candidate: condition.grants,
                source: evidence.source(),
                digest: digest_of(id, &condition.grants, &evidence, tick),
            });
        } else {
            missed.push((id, Miss::AnotherTook));
        }
    }

    Verdict {
        observation: Observation {
            tick,
            route,
            evidence,
            matched,
            missed,
        },
        discovery,
    }
}

fn digest_of(id: ConditionId, candidate: &Candidate, evidence: &Evidence, tick: u64) -> u64 {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&id.0.to_le_bytes());
    bytes.extend_from_slice(&candidate.process.definition.0.to_le_bytes());
    bytes.push(candidate.site as u8);
    bytes.extend_from_slice(&candidate.cells.to_le_bytes());
    match *evidence {
        Evidence::Meal {
            donor,
            part,
            role,
            mass_mg,
        } => {
            bytes.push(0);
            bytes.extend_from_slice(&donor.0.to_le_bytes());
            bytes.extend_from_slice(&part.0.to_le_bytes());
            bytes.push(role as u8);
            bytes.extend_from_slice(&mass_mg.to_le_bytes());
        },
        Evidence::Endured { stress, ticks } => {
            bytes.push(1);
            bytes.push(stress as u8);
            bytes.extend_from_slice(&ticks.to_le_bytes());
        },
    }
    bytes.extend_from_slice(&tick.to_le_bytes());
    crate::snapshot::hash_bytes(&bytes)
}

#[cfg(test)]
mod tests;
