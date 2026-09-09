// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The lineage checkpoint's reading: what the epoch came to, and what the
//! played line could do about it. (PE3b)
//!
//! # Why it is here and not in the world
//!
//! Three of the four halves a review is made of live beside a world rather than
//! inside one. The **reckoning** is [`World::reckon`]'s, which takes the past;
//! the **trend** is [`crate::readings::FlowWindows`]', which is a bounded
//! reduction the snapshot deliberately excludes; and the **second proposal
//! source** reads a pack off a disk, which a deterministic integer-only core
//! must not do. The fourth half — the candidates, their scores, their prices
//! and their previews — *is* the world's, and it stays there as
//! [`World::offers`]. This assembles them and adds nothing of its own.
//!
//! It is built when the checkpoint opens and again after a commit, never per
//! frame: each row costs a bounded scoring run, which is the same price an
//! unplayed line's turn pays and not one a redraw should.
//!
//! # Two proposal sources over one authority
//!
//! A candidate has a proposal the game builds — `Candidate::propose`, the first
//! part of the right shape and tissue off the top of its lattice — and, where a
//! pack declares an expression script for that candidate's process, a second
//! one the author wrote. Both are *proposals*; the one validator still decides,
//! and neither is shown as a decision. The row says which is which by name,
//! and a script that refuses says so in its own words.
//!
//! Entropy for the authored call is the host's and comes from
//! [`World::draw`] — this world's own seeded stream, read without moving it —
//! so a review built twice makes the same call twice. A fresh
//! [`Runner`](mesocosm_phenotype::express::Runner) is loaded per call for the
//! same reason: script determinism is per runner, so a runner reused across
//! calls could answer differently the second time.

use std::path::Path;

use mesocosm_core::{
    Arrangement, ConditionId, Offer, Reading, RevisionId, SpeciesId, Trend, World,
};
use mesocosm_phenotype::express::{Entropy, Policy, Request, Runner};
use mesocosm_phenotype::{Admission, asset, discover};

/// Where a proposal for a candidate came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// The game's own construction: `Candidate::propose`, which is what
    /// `World::candidate_proposal` and a filial expression both build.
    Discovery,
    /// A pack's declared expression script. (PD4)
    Authored,
}

impl Source {
    pub fn name(self) -> &'static str {
        match self {
            Source::Discovery => "discovered",
            Source::Authored => "authored",
        }
    }
}

/// One proposal for one candidate, and where it came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proposed {
    pub source: Source,
    /// The part it would land on and the tissue it would claim. `None` when it
    /// proposes nothing, which is a real answer rather than a failure.
    pub site: Option<(u32, u32)>,
    /// Why it proposes nothing, by name.
    pub refused: Option<String>,
}

/// One row of the review: an offer, and the proposals that would express it.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub offer: Offer,
    /// One entry, unless a pack declares an expression that applies to this
    /// candidate. The status quo has none: there is nothing to propose.
    pub sources: Vec<Proposed>,
}

/// What the boundary came to, and what the played line may do about it.
#[derive(Clone, Debug, PartialEq)]
pub struct Review {
    pub tick: u64,
    /// The epoch that just closed.
    pub epoch: u64,
    /// The line under your hand — the one that has not taken a turn.
    pub lineage: SpeciesId,
    /// The reckoning: every reading, each carrying whether it took the record.
    /// **The review's evidence** (epoch boundary plan §3), read off the world's
    /// own record rather than an authored table.
    pub readings: Vec<Reading>,
    /// The bounded ecology windows as they stand at the boundary.
    pub trend: Trend,
    /// What a founder of this line will have banked to pay a development with.
    pub budget_mg: u64,
    /// The revision the line is currently born under. `None` is the founding
    /// revision, which is stored nowhere.
    pub current: Option<RevisionId>,
    pub rows: Vec<Row>,
}

impl Review {
    /// Reads the review off a world standing at its lineage checkpoint.
    ///
    /// `None` when nobody is embodied: a review is one line's turn, and with no
    /// hand on a body there is no line whose turn it is.
    pub fn of(
        world: &World,
        readings: &[Reading],
        trend: Trend,
        authored: Option<&Authored>,
    ) -> Option<Self> {
        let lineage = world.controlled()?.species;
        let rows = world
            .offers(lineage)
            .into_iter()
            .map(|offer| Row {
                sources: match offer.candidate {
                    None => Vec::new(),
                    Some(condition) => sources_for(world, condition, authored),
                },
                offer,
            })
            .collect();
        Some(Self {
            tick: world.tick,
            epoch: world.epoch,
            lineage,
            readings: readings.to_vec(),
            trend,
            budget_mg: world.lineage_budget(lineage),
            current: world
                .lineages()
                .get(lineage)
                .and_then(|line| line.program().current())
                .map(|revision| revision.id),
            rows,
        })
    }

    /// The rows a commit could actually take, in table order.
    pub fn takeable(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter().filter(|row| row.offer.takeable())
    }

    /// The intent that would commit the row at `index`, if it can be committed.
    ///
    /// The status quo commits nothing, and neither does an untakeable
    /// candidate; both answer `None`, so a host cannot send a revision the
    /// world would only refuse.
    pub fn commit(&self, index: usize) -> Option<mesocosm_core::Intent> {
        let row = self.rows.get(index)?;
        row.offer
            .takeable()
            .then_some(row.offer.candidate)
            .flatten()
            .map(|condition| mesocosm_core::Intent::Revise { condition })
    }
}

/// The proposals for one candidate: the game's, and the pack's where one
/// applies.
fn sources_for(
    world: &World,
    condition: ConditionId,
    authored: Option<&Authored>,
) -> Vec<Proposed> {
    // The game's own construction, on the body under the hand — the same body
    // PD4's request freezes, so the two sources are answers to one question.
    let mut sources = vec![
        match world.candidate_proposal(condition, Arrangement::Automatic) {
            Some(proposal) => Proposed {
                source: Source::Discovery,
                site: proposal
                    .sites
                    .last()
                    .map(|site| (site.part.0, site.cells.len() as u32)),
                refused: None,
            },
            None => Proposed {
                source: Source::Discovery,
                site: None,
                refused: Some("nowhere on this body to put it".to_owned()),
            },
        },
    ];
    if let Some(authored) = authored {
        sources.extend(authored.propose(world, condition));
    }
    sources
}

/// A pack's declared expression scripts, held as source. (PD4)
///
/// **Source rather than loaded runners**, because a script's determinism is per
/// runner: the same context and seed always give the same answer from a fresh
/// [`Runner::load`], and a runner kept across calls may carry globals. A review
/// that could be built twice and compared has to load fresh, so this holds the
/// text and pays a chunk load per call.
#[derive(Clone, Debug, Default)]
pub struct Authored {
    scripts: Vec<(String, String)>,
    policy: Policy,
}

impl Authored {
    /// Reads the expression scripts a pack declares.
    ///
    /// Through [`asset`], which is the pack door's own path check: a file the
    /// manifest did not declare is refused, and one that leaves the root is
    /// refused before it is opened. A host does not get to name a script.
    pub fn load(root: &Path) -> Result<Self, Admission> {
        let manifest = discover(root)?;
        let mut scripts = Vec::with_capacity(manifest.expression.len());
        for relative in &manifest.expression {
            let path = asset(root, &manifest, relative)?;
            let source = std::fs::read_to_string(&path).map_err(|error| Admission::Unreadable {
                path: path.display().to_string(),
                why: error.to_string(),
            })?;
            scripts.push((relative.clone(), source));
        }
        Ok(Self {
            scripts,
            policy: Policy::default(),
        })
    }

    /// How many expression scripts this pack declared.
    pub fn len(&self) -> usize {
        self.scripts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.scripts.is_empty()
    }

    /// What each declared script proposes for one candidate.
    ///
    /// A script that returns no site is one that does not apply here — a
    /// candidate whose process it does not author, or a body it will not put
    /// one on — so it contributes no row rather than an empty one. That is what
    /// "if no pack expression applies, the row simply has one source" means.
    fn propose(&self, world: &World, condition: ConditionId) -> Vec<Proposed> {
        let Some(request) = Request::of(world, condition) else {
            return Vec::new();
        };
        // Host-owned, off this world's own stream, and the same number every
        // time it is asked.
        let entropy = Entropy::from_seed(world.draw());
        let mut out = Vec::new();
        for (_, source) in &self.scripts {
            let proposed = match Runner::load(source, self.policy)
                .and_then(|mut runner| runner.propose(&request, &entropy))
            {
                Ok(proposal) => match proposal.sites.first() {
                    Some(site) => Proposed {
                        source: Source::Authored,
                        site: Some((site.part, site.cells)),
                        refused: None,
                    },
                    // Declined rather than refused: this script does not author
                    // this candidate.
                    None => continue,
                },
                Err(refused) => Proposed {
                    source: Source::Authored,
                    site: None,
                    refused: Some(refused.words()),
                },
            };
            out.push(proposed);
        }
        out
    }
}
