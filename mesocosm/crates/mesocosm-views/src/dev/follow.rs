// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What the dev lane says about the critter the camera is following. (DT2)
//!
//! **Every line here is a core query put into words, and nothing else.** The
//! dev tools plan's third principle is the whole of this file's discipline: no
//! figure below is computed, averaged, inferred or defaulted. Where a fact was
//! not readable it was added in core with a test rather than worked out here —
//! [`Accounts`] and [`mesocosm_core::History::ending`] are the two DT2 added.
//!
//! The words are this crate's, exactly as the vitals panel's are: core answers
//! what is, and a panel decides how to say it.

use mesocosm_core::flow::Accounts;
use mesocosm_core::{Ending, Organism, OrganismId, PartId, Passing, Role, World, classify};

use super::super::vitals::condition_word;

/// How many part rows the tile shows before it starts counting instead.
///
/// The tile is one tile (the workbench rendering surface a second one would
/// need is a stack gap, not this slice's), so a long body is truncated with a
/// count the way the section's roster is rather than running off the bottom.
pub const MAX_PART_ROWS: usize = 3;

/// How many condition names the discoveries row prints before counting.
pub const MAX_DISCOVERY_NAMES: usize = 2;

/// The followed critter, read off core and put into the panel's lines.
///
/// Each field is one row. They are `String`s rather than numbers because the
/// panel's claim is about the *line a player reads*, and a test that compares
/// the line against the core query it came from can only do that on the line
/// itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Follow {
    /// The organism id, and whether this is the critter under the hand.
    pub id: String,
    /// Its lineage, and the lineage's name when it has one.
    pub species: String,
    /// Where it stands.
    pub at: String,
    /// What it has banked: the flow record's `Account::Reserve`, which is an
    /// organism's `energy_mg`.
    pub reserve: String,
    /// What its living parts weigh: the flow record's `Account::Substance`.
    pub substance: String,
    /// Income, rent and everything else that left it, as [`Accounts`]
    /// separates them.
    pub flows: String,
    /// The window those figures cover. Its own row, because a figure without
    /// its window is not a reading.
    pub window: String,
    /// The revision its line is currently born under, or `founding`.
    pub revision: String,
    /// What this world has come to, by condition name.
    pub discovered: String,
    /// How many living parts the body has.
    pub parts: String,
    /// One line per part shown: its role, its half-extent, and its sites.
    pub part_rows: Vec<(String, String)>,
    /// Living parts the tile had no room for.
    pub more_parts: usize,
}

/// Reads the followed critter off the world.
///
/// `accounts` comes from the driver, which is the declared home of a
/// replay-derived reduction of the flow stream — the same reason
/// [`vitals_of`](super::super::vitals::vitals_of) is handed a `Trend`: a world
/// can say what is, never what moved.
///
/// `None` when the world holds no organism with that id, which is honest: a
/// body eaten to nothing is gone from the roster and the panel says so through
/// [`Lost`] instead.
pub fn follow_of(world: &World, followed: OrganismId, accounts: Accounts) -> Option<Follow> {
    let organism = world.organisms.iter().find(|o| o.id == followed)?;
    Some(Follow {
        id: id_words(organism, world.controlled_id()),
        species: species_words(world, organism),
        at: at_words(organism.position),
        reserve: format!("{} mg", organism.energy_mg),
        substance: format!("{} mg", organism.biomass_mg()),
        flows: flow_words(accounts),
        window: window_words(accounts),
        revision: revision_words(world, organism),
        discovered: discovered_words(world),
        parts: organism.body().living().count().to_string(),
        part_rows: part_rows(organism),
        more_parts: organism
            .body()
            .living()
            .count()
            .saturating_sub(MAX_PART_ROWS),
    })
}

fn id_words(organism: &Organism, controlled: Option<OrganismId>) -> String {
    if controlled == Some(organism.id) {
        // The state the snap-back key returns to, said rather than left to be
        // inferred from two numbers being equal.
        format!("{} (controlled)", organism.id.0)
    } else {
        organism.id.0.to_string()
    }
}

fn species_words(world: &World, organism: &Organism) -> String {
    let named = world
        .lineages()
        .get(organism.species)
        .and_then(|line| line.name.clone());
    match named {
        Some(name) => format!("{} — {name}", organism.species.0),
        None => organism.species.0.to_string(),
    }
}

fn at_words(at: [i32; 3]) -> String {
    format!("{}, {}, {}", at[0], at[1], at[2])
}

fn flow_words(accounts: Accounts) -> String {
    format!(
        "in {}, rent {}, out {}",
        accounts.income_mg, accounts.rent_mg, accounts.outflow_mg
    )
}

fn window_words(accounts: Accounts) -> String {
    format!("{} ticks", accounts.ticks)
}

/// The revision the line is currently born under.
///
/// `founding` is the word for a line that has committed nothing, which is a
/// real state and not an absent one — it is what every line starts at.
fn revision_words(world: &World, organism: &Organism) -> String {
    world
        .lineages()
        .get(organism.species)
        .and_then(|line| line.program().current())
        .map(|revision| revision.id.0.to_string())
        .unwrap_or_else(|| "founding".to_string())
}

/// What this world has come to, by condition name.
///
/// **The world's list, which core keeps for the played line**:
/// `World::observe` records against whoever is under the hand, so a world holds
/// one discovery list and it is the played line's. A per-line ledger is
/// reported as a core gap rather than invented here, so a followed critter of
/// another line reads this row as what it is.
fn discovered_words(world: &World) -> String {
    let discoveries = world.discoveries();
    if discoveries.is_empty() {
        return "none".to_string();
    }
    let shown: Vec<String> = discoveries
        .iter()
        .take(MAX_DISCOVERY_NAMES)
        .map(|discovery| condition_word(discovery.condition))
        .collect();
    let hidden = discoveries.len().saturating_sub(MAX_DISCOVERY_NAMES);
    if hidden == 0 {
        shown.join(", ")
    } else {
        format!("{}, +{hidden} more", shown.join(", "))
    }
}

/// One row per living part, in part order, truncated at [`MAX_PART_ROWS`].
fn part_rows(organism: &Organism) -> Vec<(String, String)> {
    organism
        .body()
        .living()
        .take(MAX_PART_ROWS)
        .map(|part| {
            (
                format!("part {}", part.id.0),
                format!(
                    "{} {} — {}",
                    role_word(classify(part.half_extent)),
                    extent_words(part.half_extent),
                    site_words(organism, part.id),
                ),
            )
        })
        .collect()
}

/// A part's role, in the plain word for the shape.
///
/// Distinct from the vitals panel's own site wording ("bulk", "a limb"), which
/// says where a candidate could *go*; this names what a part **is**.
pub fn role_word(role: Role) -> &'static str {
    match role {
        Role::Mass => "mass",
        Role::Limb => "limb",
        Role::Plate => "plate",
        Role::Sensor => "sensor",
    }
}

fn extent_words(half_extent: [i32; 3]) -> String {
    format!("{}x{}x{}", half_extent[0], half_extent[1], half_extent[2])
}

/// What one part's mosaic expresses, and on how much tissue.
///
/// Straight off [`BodyPhenotype::explain`](mesocosm_core::BodyPhenotype::explain),
/// whose `named` is `None` exactly when this world's ruleset does not hold the
/// definition — the missing-ruleset diagnostic, said rather than papered over
/// with a similar local name.
fn site_words(organism: &Organism, part: PartId) -> String {
    let Some(explanation) = organism.phenotype.explain(part) else {
        return "no mosaic".to_string();
    };
    if explanation.sites.is_empty() {
        return "no sites".to_string();
    }
    explanation
        .sites
        .iter()
        .map(|site| {
            let name = site
                .named
                .as_ref()
                .map(|id| id.name.clone())
                .unwrap_or_else(|| "an unknown process".to_string());
            format!("{name} on {} cells", site.cells)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A followed critter that stopped being one. (DT2)
///
/// Kept and shown after follow snaps back to the controlled critter, so a death
/// under the camera is reported rather than silently dropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Lost {
    pub id: u32,
    pub tick: u64,
    pub how: &'static str,
}

/// What became of a followed critter, off the record's own ending.
///
/// `at` is the world's current tick, used only when this past holds no ending
/// at all — which is the honest reading for a body that left the roster without
/// one, and is still said rather than dropped.
pub fn lost_of(organism: OrganismId, ending: Option<Ending>, at: u64) -> Lost {
    match ending {
        Some(ending) => Lost {
            id: organism.0,
            tick: ending.tick,
            how: match ending.how {
                Passing::Died => "died",
                Passing::Returned => "went back to the ground",
            },
        },
        None => Lost {
            id: organism.0,
            tick: at,
            how: "left the roster",
        },
    }
}

/// The one sentence the tile prints for a lost follow target.
pub fn lost_words(lost: Lost) -> String {
    format!("critter {} {} at tick {}", lost.id, lost.how, lost.tick)
}
