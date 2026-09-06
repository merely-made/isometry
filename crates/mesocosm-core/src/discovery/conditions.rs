// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The conditions this world admits, and the numbers they are made of.
//!
//! Split out of `discovery.rs` at the 600-line ceiling. What stayed next door
//! is the evidence, the rules and the routing; what moved here is the **table**
//! — the fixed, small set of conditions a piece of evidence is offered to, and
//! the thresholds they read.
//!
//! # A digest, not a name
//!
//! [`ConditionId`] is a hash over a condition's rule-bearing bytes, exactly as
//! [`ProcessRef`](crate::process::ProcessRef) is over a definition's. The
//! friendly name is presentation. Two worlds that agree about a name and
//! disagree about the rule under it hold different digests, so a discovery made
//! in one cannot be resolved against the other — which is the same protection
//! PD1b gave allocation, applied to acquisition.
//!
//! # Where the numbers come from
//!
//! None of the four is picked. The stress horizon is the starved line itself,
//! the evidence floor is the ecology's own answer to "how much substance is a
//! body at all", and the two cell counts are the ones PD2's fixtures already
//! use for the same organ.

use serde::{Deserialize, Serialize};

use crate::axis::Appendage;
use crate::plan::Role;
use crate::process::{Process, Registry};

use super::{Candidate, Input, Rule, Stress};

/// Consecutive ticks under the starved line that count as coming through it.
///
/// **Derived, not picked**: it is [`STARVED_UPKEEP_TICKS`] itself — you have to
/// endure the horizon you are inside. The line is "your budget holds fewer than
/// a hundred ticks of doing nothing"; this is a hundred ticks of standing on the
/// wrong side of it while somebody is actually holding the body.
///
/// [`STARVED_UPKEEP_TICKS`]: crate::world::STARVED_UPKEEP_TICKS
pub const HUNGER_TICKS: u64 = crate::world::STARVED_UPKEEP_TICKS;

/// The smallest consumed part that is evidence about a body rather than a
/// crumb.
///
/// `ecology::STARVATION_MG` is the mass below which an organism cannot sustain
/// itself — the ecology's own answer to "how much substance is a body at all" —
/// so a scrap under it teaches nothing.
pub const MEAL_EVIDENCE_MG: u64 = crate::organism::ecology::STARVATION_MG;

/// Cells of tissue a granted gland candidate proposes taking.
///
/// The same five `tests/embodied/gland.rs` and PD2's receipt use, for the same
/// reason: on the twelve-cell frond those fixtures grow it is enough gland to
/// out-hold a fresh soil column, so the dormancy state is reachable rather than
/// theoretical.
const GLAND_CELLS: u32 = 5;

/// Cells a granted fixing candidate proposes taking.
///
/// Four, the other count PD2's receipts already develop with on the same
/// twelve-cell plate. Fewer than the gland's five on purpose: fixing pays a
/// body its living, so a candidate that proposed to take *more* tissue for it
/// than for a defence would be quietly ranking the two.
const FROND_CELLS: u32 = 4;

/// A condition's content address: a digest over its rule-bearing bytes.
///
/// A digest rather than a name, exactly as [`ProcessRef`] is: the friendly
/// name is presentation, and two worlds that agree about a name and disagree
/// about the rule under it must not be able to trade discoveries.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct ConditionId(pub u64);

/// One declared discovery condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Condition {
    /// Qualified, and presentation only.
    pub name: &'static str,
    /// The lanes this will accept evidence through. Evidence arriving by any
    /// other lane never reaches [`Self::rule`].
    pub inputs: &'static [Input],
    pub rule: Rule,
    pub grants: Candidate,
}

impl Condition {
    /// The digest over identity, declared inputs, rule and granted candidate.
    pub fn id(&self) -> ConditionId {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(0);
        for input in self.inputs {
            bytes.push(*input as u8);
        }
        bytes.push(0);
        bytes.extend_from_slice(&rule_bytes(self.rule));
        bytes.extend_from_slice(&self.grants.process.definition.0.to_le_bytes());
        bytes.push(self.grants.site as u8);
        bytes.extend_from_slice(&self.grants.cells.to_le_bytes());
        bytes.push(self.grants.word.map(|word| word as u8 + 1).unwrap_or(0));
        ConditionId(crate::snapshot::hash_bytes(&bytes))
    }

    /// Whether this condition accepts evidence from that lane.
    pub fn declares(&self, input: Input) -> bool {
        self.inputs.contains(&input)
    }
}

fn rule_bytes(rule: Rule) -> Vec<u8> {
    let mut bytes = Vec::new();
    match rule {
        Rule::Consumed { role, mass_mg } => {
            bytes.push(0);
            bytes.push(role as u8);
            bytes.extend_from_slice(&mass_mg.to_le_bytes());
        },
        Rule::Endured { stress, ticks } => {
            bytes.push(1);
            bytes.push(stress as u8);
            bytes.extend_from_slice(&ticks.to_le_bytes());
        },
    }
    bytes
}

/// Every condition this world's ruleset admits.
///
/// Fixed and small, and that is the bound: routing one piece of evidence is one
/// pass over this table. PE4's generated conditions arrive through the same
/// shape, admitted rather than coined.
pub fn conditions() -> [Condition; 2] {
    let registry = Registry::native();
    [
        // **The non-food route.** Coming through hunger teaches a body to make
        // a toxin out of the ground it is standing on. Nothing about the
        // evidence is a meal, and nothing about the reward is a food category:
        // this is the ruling against a diet tree, in one entry.
        Condition {
            name: "mesocosm:endured-hunger",
            inputs: &[Input::Endurance],
            rule: Rule::Endured {
                stress: Stress::Hunger,
                ticks: HUNGER_TICKS,
            },
            grants: Candidate {
                process: registry.of_native(Process::Secrete).reference(),
                site: Role::Plate,
                cells: GLAND_CELLS,
                // Nothing new to say: a plate is already innate to nobody, and
                // the line learns the *word* for one through the meal route
                // below. What this grants is the tissue's use.
                word: None,
            },
        },
        // **The meal route**, narrowed to the part actually consumed. The old
        // lesson read the donor's whole recipe; this reads the organ in your
        // mouth, and only a plate teaches a plate.
        Condition {
            name: "mesocosm:plate-eaten",
            inputs: &[Input::Meal],
            rule: Rule::Consumed {
                role: Role::Plate,
                mass_mg: MEAL_EVIDENCE_MG,
            },
            grants: Candidate {
                process: registry.of_native(Process::Fix).reference(),
                site: Role::Plate,
                cells: FROND_CELLS,
                word: Some(Appendage::Plate),
            },
        },
    ]
}

/// The condition a digest names, when this world's ruleset holds it.
pub fn resolve(id: ConditionId) -> Option<Condition> {
    conditions().into_iter().find(|found| found.id() == id)
}

/// The friendly name a digest resolves to. `None` is the missing-ruleset
/// diagnostic, never a licence to print a similar local name.
pub fn name_of(id: ConditionId) -> Option<&'static str> {
    resolve(id).map(|found| found.name)
}
