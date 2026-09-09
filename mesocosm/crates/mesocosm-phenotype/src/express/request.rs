// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The frozen developmental context. (PD4)
//!
//! Plan §4: *a frozen view containing only declared facts*. Every field here is
//! an owned copy of something the host decided to show — no borrowed world, no
//! handle, nothing a script could follow back to a live value. What is not in
//! this struct is not visible to an author, which is how "scripts cannot
//! inspect hidden world state" is enforced rather than asked for.

use mesocosm_core::{BodyPhenotype, ConditionId, Registry, RulesetDigest, World, classify};
use serde::{Deserialize, Serialize};

/// Why the host is asking. (Plan §4's bounded triggers.)
///
/// One today, because one is played. §4 lists founding and filial regrowth, a
/// chosen adaptation, assimilation or grafting, growth and repair, and
/// lifecycle change; each arrives with the gate that plays it, rather than as a
/// vocabulary written ahead of a consumer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trigger {
    /// A line came to a developmental option and is taking it up.
    Discovery,
}

impl Trigger {
    /// The word a script reads.
    pub fn word(self) -> &'static str {
        match self {
            Trigger::Discovery => "discovery",
        }
    }
}

/// One admitted definition, as an author sees it.
///
/// Identity, site requirement and seeding — the same three things the digest
/// folds. A script is shown what a definition *rules*, never a native binding
/// or a label, because neither is rule-bearing.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Definition {
    /// `namespace:name`.
    pub id: String,
    /// The shape words a part must classify as: `mass`, `limb`, `plate`,
    /// `sensor`.
    pub expressed_by: Vec<String>,
    /// `geometry` or `acquired`.
    pub seeding: String,
}

/// One site a part already expresses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteView {
    /// `namespace:name`, or `unknown` when this world's ruleset no longer holds
    /// the definition the site cites. Never the nearest local one.
    pub process: String,
    pub cells: u32,
}

/// One living part: a stable address, its shape, its tissue, and what it does.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartView {
    /// The stable address, and what a proposal names.
    pub part: u32,
    pub role: String,
    pub cells: u32,
    pub free: u32,
    /// What one cell of this part's tissue is worth, in milligrams. The price
    /// a development is charged at, shown so an author can weigh it — and
    /// **not** so an author can set it: the host prices the accepted proposal
    /// itself (plan §4, "the proposal does not choose its own cost").
    pub cell_mg: u64,
    pub sites: Vec<SiteView>,
}

/// One quantized world reading a script may branch on.
///
/// Named and integer, so a fixture can state the context it was recorded under
/// and a different one is a different declared context rather than a different
/// afternoon.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ambient {
    pub name: String,
    pub value: i64,
}

/// The whole frozen picture one expression call is given.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub trigger: Trigger,
    /// Which biology this is, as its content address. A fixture recorded under
    /// one ruleset is visibly a fixture for that ruleset.
    pub ruleset: RulesetDigest,
    /// The body revision this was frozen at.
    pub revision: u32,
    /// The phenotype digest a lowered proposal must expect. Staleness is
    /// refusable because this travels.
    pub expect: u64,
    /// Every definition this world admitted, sorted by id.
    pub definitions: Vec<Definition>,
    /// Every living part, in part order.
    pub parts: Vec<PartView>,
    /// What this request is about: the qualified ids the line has come to and
    /// may express here, sorted. A script that proposes anything else is
    /// making it up, and the validator will say so.
    ///
    /// Empty when this world's ruleset does not hold what the line came to,
    /// which is the missing-ruleset answer one door up (plan §6): the id is
    /// dropped rather than replaced with the nearest local definition.
    pub candidates: Vec<String>,
    /// The body's own reserve, in milligrams. An integer budget, per §4.
    pub material_mg: u64,
    /// Declared quantized world conditions, sorted by name.
    pub conditions: Vec<Ambient>,
}

impl Request {
    /// Freezes the played body's situation under one discovered condition.
    ///
    /// `None` when nobody is embodied or the line has not come to that
    /// condition — the same two facts
    /// [`World::candidate_intent`](mesocosm_core::World::candidate_intent)
    /// answers, asked one door over.
    ///
    /// **The one declared world condition today is `ground_mg`**: what the soil
    /// column under the body holds. It is what PD2's played process already
    /// reads, so it is a reading this game has rather than a knob invented to
    /// give a script something to branch on.
    pub fn of(world: &World, condition: ConditionId) -> Option<Self> {
        let me = world.controlled()?;
        let discovery = world
            .discoveries()
            .iter()
            .find(|discovery| discovery.condition == condition)?;
        let registry = world.ruleset();
        let ground_mg = world.soil().matter_mg(world.soil().column_at(me.position));
        let candidate = registry
            .resolve(discovery.candidate.process)
            .map(|def| def.id.qualified())
            .into_iter()
            .collect();

        Some(Self {
            trigger: Trigger::Discovery,
            ruleset: registry.digest(),
            revision: me.phenotype.revision(),
            expect: me.phenotype.digest(),
            definitions: definitions_of(registry),
            parts: parts_of(registry, &me.phenotype),
            candidates: candidate,
            material_mg: me.energy_mg,
            conditions: vec![Ambient {
                name: "ground_mg".to_owned(),
                value: i64::try_from(ground_mg).unwrap_or(i64::MAX),
            }],
        })
    }

    /// The same picture, built straight from a phenotype and a declared
    /// context.
    ///
    /// What a fixture replays: no world, no soil, no roster — just the body
    /// plan, the ruleset, and the conditions somebody wrote down. That is what
    /// makes "contrasting contexts, one body plan" a claim a test can state
    /// without founding two worlds.
    pub fn frozen(
        registry: &Registry,
        phenotype: &BodyPhenotype,
        mut candidates: Vec<String>,
        material_mg: u64,
        mut conditions: Vec<Ambient>,
    ) -> Self {
        // Sorted here rather than trusted from the caller, so two fixtures that
        // declare the same context are the same context.
        candidates.sort();
        conditions.sort_by(|a, b| a.name.cmp(&b.name));
        Self {
            trigger: Trigger::Discovery,
            ruleset: registry.digest(),
            revision: phenotype.revision(),
            expect: phenotype.digest(),
            definitions: definitions_of(registry),
            parts: parts_of(registry, phenotype),
            candidates,
            material_mg,
            conditions,
        }
    }
}

fn definitions_of(registry: &Registry) -> Vec<Definition> {
    registry
        .all()
        .map(|def| Definition {
            id: def.id.qualified(),
            expressed_by: def
                .expressed_by
                .iter()
                .map(|role| role_word(*role))
                .collect(),
            seeding: if def.seeded() { "geometry" } else { "acquired" }.to_owned(),
        })
        .collect()
}

fn parts_of(registry: &Registry, phenotype: &BodyPhenotype) -> Vec<PartView> {
    phenotype
        .allocations()
        .map(|(part, mosaic)| PartView {
            part: part.0,
            role: role_word(classify(
                phenotype
                    .body()
                    .part(part)
                    .expect("a living part")
                    .half_extent,
            )),
            cells: mosaic.cells().count() as u32,
            free: mosaic.free(),
            cell_mg: phenotype.cell_mg(part),
            sites: mosaic
                .sites()
                .iter()
                .map(|site| SiteView {
                    // `None` is the missing-ruleset diagnostic, not a licence
                    // to name a similar local definition instead.
                    process: registry
                        .resolve(site.process)
                        .map(|def| def.id.qualified())
                        .unwrap_or_else(|| "unknown".to_owned()),
                    cells: site.cells.iter().filter(|c| mosaic.is_living(**c)).count() as u32,
                })
                .collect(),
        })
        .collect()
}

/// The plain shape word a pack and a script both speak. The same closed set
/// [`role_of`](crate::pack::role_of) reads back.
pub(crate) fn role_word(role: mesocosm_core::Role) -> String {
    match role {
        mesocosm_core::Role::Mass => "mass",
        mesocosm_core::Role::Limb => "limb",
        mesocosm_core::Role::Plate => "plate",
        mesocosm_core::Role::Sensor => "sensor",
    }
    .to_owned()
}
