// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The lineage turn: what a line could commit, what growing it would be worth,
//! and what it decides. (P4b, PE3a)
//!
//! # A candidate is scored by growing it, never by a formula
//!
//! **Ruled by Mark, 2026-09-01.** A candidate is worth what it earns: the world
//! is copied, the revision is committed on the candidate's line in the copy, the
//! copy is grown for a bounded run with nobody at the keyboard, and the flow
//! record is read — income against rent, for that line's bodies. There is no
//! static formula over body readings, no fitness term, and no number invented
//! here. [`Score`] is the figures the record already separates plus the run
//! length, and the comparison between two of them is
//! [`Score::beats`] and nothing else.
//!
//! The copy is discarded. Scoring takes `&self`, so it cannot reach the world it
//! is reasoning about, and the receipt asserts the real world's hash is the same
//! afterwards.
//!
//! # Initiative is descending complexity, and commits land immediately
//!
//! [`crate::epoch`]'s ordering idea, kept and brought over to the real world
//! model: the most elaborate lines commit first and the simpler ones answer a
//! world those commits have already changed. What is not brought over is that
//! module's trait array and its `fitness`, which this replaces. The reading is
//! `Species::recipe.complexity()` — the same one [`World::intricacy`] reads and
//! the complexity frontier binds on — ties broken by species id so a round
//! replays identically.
//!
//! # Nothing here knows which lineage is the player's, except to skip it
//!
//! Every unplayed line takes its turn through [`World::revise`], the identical
//! transaction a player's `Intent::Revise` reaches. The played line is left
//! alone: its turn is the review, and the review is PE3b.
//!
//! # The reckoning is not here, and that is the split
//!
//! [`World::apply`] ends the epoch — the Timed rule is a versioned world rule,
//! so a headless enclosure obeys it too — and runs the round. What it does not
//! do is *reckon*: [`World::reckon`] reads the past, and history lives beside a
//! world rather than inside it, so whoever holds the past does that half.
//!
//! # One boundary, one door (DT4)
//!
//! There used to be a second one. `World::end_epoch(history)` reckoned, bumped
//! the epoch and restarted the budget — but it never ran the adaptation round
//! and it left `at_boundary` *false*, so an epoch closed through it gave the
//! unplayed lines no turn and stood at no lineage checkpoint. The boundary
//! block in [`World::apply`] does both. Two doors that disagreed about what a
//! boundary is are one authority too many, so the manual one is **deleted**:
//! the epoch ends when the world's own rule says so, or when a hand asks
//! through `Intent::EndEpoch`, and both are that block. A caller that wants the
//! epoch closed now applies the intent; a caller that wants to know what an
//! epoch came to calls [`World::reckon`], which is the read-the-past half and
//! was always separate.
//!
//! # What an unplayed line may consider
//!
//! **Inherited or already-discovered candidates only** (playable ecology plan
//! §6, ruling 5, still open at the acquisition end). Discovery is played-only
//! and stays that way: [`World::candidates`] reads the discoveries this world
//! holds and nothing proposes a new one. So an enclosure nobody has played has
//! nothing for any line to weigh, and its rounds are empty.

use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;
use crate::discovery::ConditionId;
use crate::flow::Process;
use crate::history::Event;
use crate::phenotype::Arrangement;
use crate::program::RevisionId;
use crate::score::Reading;

use super::{Intent, World};

/// What one candidate came to, as the flow record read it.
///
/// **Figures, not a verdict.** Every field says what moved and over how many
/// ticks, which is the shape the readings contract rules; the single number a
/// choice needs is derived by [`Self::net_mg`] at the point of comparison
/// rather than stored as a rank.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    /// Ticks the run covered. Stated, because a figure without its window is
    /// not a reading.
    pub ticks: u64,
    /// Matter that reached this line's bodies over the run: what its producers
    /// drew out of the ground, what its mouths took, and what its parents
    /// handed their children.
    pub income_mg: u64,
    /// Rent: what those bodies spent per tick simply existing. The flow
    /// record's own [`Process::Upkeep`], not a recomputation of it.
    pub rent_mg: u64,
    /// Everything else that left them — travel, developing an organ, spill,
    /// and what death returned to the ground. Kept beside rent rather than
    /// folded into it, because the record separates them and a score that
    /// merged them could not say whether a line is paying to build or paying
    /// to stand still.
    pub outflow_mg: u64,
    /// Bodies of this line born during the run — the ones that could have been
    /// born expressing the revision.
    pub born: u32,
}

impl Score {
    /// Income against rent. **The one ordering**, and the only place a score
    /// becomes a single number.
    pub fn net_mg(self) -> i128 {
        i128::from(self.income_mg) - i128::from(self.rent_mg)
    }

    /// Whether this candidate is worth taking over that one.
    ///
    /// Strictly greater, so a tie leaves whatever was already there. A line
    /// that revised itself on an equal reading would be paying the price of
    /// change for nothing.
    pub fn beats(self, other: Self) -> bool {
        self.net_mg() > other.net_mg()
    }
}

/// What one lineage weighed, and what it did about it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    pub lineage: SpeciesId,
    /// Everything it weighed, in the order it weighed them. The first entry is
    /// always `None` — the founding revision, no change — so *the status quo
    /// beat every candidate* is a reading off this list rather than an absence.
    pub considered: Vec<(Option<ConditionId>, Score)>,
    /// `None` means exactly that outcome.
    pub chosen: Option<ConditionId>,
    /// The revision it committed, when it did. `None` beside a `chosen` would
    /// mean the transaction refused, which is a fact worth being able to read.
    pub committed: Option<RevisionId>,
}

impl Turn {
    /// What the line already had, as it scored this round.
    pub fn standing(&self) -> Score {
        self.considered.first().map(|(_, s)| *s).unwrap_or_default()
    }
}

/// One adaptation round: who acted, in what order, and what happened.
///
/// The record is the feature. A trophic cascade nobody watched is
/// indistinguishable from procedural noise, so every turn is a transcript
/// entry rather than a population number to infer from.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Round {
    /// The epoch this round closed.
    pub epoch: u64,
    /// Turns in the order they were taken, which is initiative order. A line
    /// with nothing to weigh takes no turn and is not here.
    pub turns: Vec<Turn>,
}

impl Round {
    /// Turns that committed something.
    pub fn changes(&self) -> impl Iterator<Item = &Turn> {
        self.turns.iter().filter(|turn| turn.committed.is_some())
    }

    /// What one lineage decided this round.
    pub fn turn(&self, lineage: SpeciesId) -> Option<&Turn> {
        self.turns.iter().find(|turn| turn.lineage == lineage)
    }

    /// The order lineages acted in.
    pub fn order(&self) -> Vec<SpeciesId> {
        self.turns.iter().map(|turn| turn.lineage).collect()
    }
}

/// The candidate this round would take, if any.
///
/// **The ordering, in one place.** `considered[0]` is the status quo; a
/// candidate is taken only when it strictly beats it, and the best of several
/// that do is the one with the most net income, earliest in the list on a tie.
fn best_of(considered: &[(Option<ConditionId>, Score)]) -> Option<ConditionId> {
    let standing = considered.first()?.1;
    let mut best: Option<(ConditionId, Score)> = None;
    for (candidate, score) in considered.iter().skip(1) {
        let Some(condition) = *candidate else {
            continue;
        };
        if !score.beats(standing) {
            continue;
        }
        if best.is_none_or(|(_, held)| score.beats(held)) {
            best = Some((condition, *score));
        }
    }
    best.map(|(condition, _)| condition)
}

/// Reckoning what an epoch came to. **Closing one is not here** — see the
/// module docs and [`World::apply`]'s boundary block, which is the only door.
impl World {
    /// Reckons what the epoch came to, and writes it into the world's record.
    ///
    /// It takes the past because history lives beside a world rather than
    /// inside it: a world can say what is, never what happened. That is also
    /// why it is a separate call from the boundary itself — the world ends its
    /// own epochs (PE3), and whoever is holding the past reckons them.
    ///
    /// Returns every reading, each carrying whether it took the record, which is
    /// what an epoch-boundary screen is made of.
    pub fn reckon(&mut self, history: &crate::history::History) -> Vec<Reading> {
        let mut readings = crate::score::readings(self, history);
        for reading in &mut readings {
            reading.took =
                self.record
                    .note(reading.feat, reading.scale, reading.value, reading.species);
        }
        readings
    }

    /// The rules this world realized, under a different epoch rule or scoring
    /// window. **The PE4 seam**, and today what a test founds a mismatch with.
    ///
    /// The process digest is not negotiable here: it is the identity of the set
    /// this world is actually holding, so it is kept and only the rest is taken.
    pub fn with_rules(mut self, rules: crate::rules::WorldRules) -> Self {
        self.rules = crate::rules::WorldRules {
            processes: self.rules.processes,
            ..rules
        };
        self
    }
}

/// The round: what a line may weigh, what weighing it costs, and who acts.
impl World {
    /// Whether this world is standing at its lineage checkpoint. (PE3)
    pub fn at_boundary(&self) -> bool {
        self.at_boundary
    }

    /// The tick the current epoch began on.
    pub fn epoch_began(&self) -> u64 {
        self.epoch_began
    }

    /// What ends an epoch here.
    pub fn epoch_rule(&self) -> crate::rules::EpochRule {
        self.rules.epoch
    }

    /// What the most recent adaptation round came to.
    pub fn last_round(&self) -> &Round {
        &self.last_round
    }

    /// What a lineage could commit at this boundary.
    ///
    /// **The founding revision is always first**, as `None`: no change is a
    /// real candidate, so *the status quo beat every candidate* is an outcome
    /// a round can report rather than a silence. After it come the conditions
    /// this world has come to that the line does not already hold, whose
    /// definition this world admits, and that some living body of the line
    /// could actually carry — asked by building the very proposal
    /// [`World::candidate_proposal`] builds, so a candidate offered here and a
    /// candidate a player expresses are the same construction.
    pub fn candidates(&self, species: SpeciesId) -> Vec<Option<ConditionId>> {
        let mut offered = vec![None];
        let Some(line) = self.lineages.get(species) else {
            return offered;
        };
        let held = line
            .program()
            .current()
            .map(|revision| revision.cites.condition);
        // Any living body of the line: what a line can express is a fact about
        // the shape its bodies grow into, and its bodies are all grown from one
        // recipe. In id order, so the answer does not depend on the roster.
        let Some(body) = self.living().find(|o| o.species == species) else {
            return offered;
        };
        for discovery in &self.discoveries {
            if Some(discovery.condition) == held {
                continue;
            }
            // The same two refusals `World::revise` would give, asked before a
            // candidate is put on a list a round will spend ticks scoring.
            if discovery.candidate.cells == 0
                || self
                    .ruleset()
                    .resolve(discovery.candidate.process)
                    .is_none()
            {
                continue;
            }
            if discovery
                .candidate
                .propose(&body.phenotype, Arrangement::Automatic)
                .is_none()
            {
                continue;
            }
            offered.push(Some(discovery.condition));
        }
        offered
    }

    /// Grows one candidate in a copy of this world and reads what it earned.
    ///
    /// Bounded by [`WorldRules::score_ticks`](crate::rules::WorldRules), driven
    /// with nothing but [`Intent::Idle`] so no host is in it, and deterministic:
    /// the copy carries this world's seeded stream, so the same world, the same
    /// candidate and the same length give the same score every time.
    ///
    /// `None` scores the status quo — the world exactly as it stands — which is
    /// what the ordering compares against.
    pub fn score(&self, species: SpeciesId, candidate: Option<ConditionId>) -> Score {
        let ticks = self.rules.score_ticks;
        let mut copy = self.clone();
        // A round inside a round would be a different game, and an unbounded
        // one: a scoring copy never ends an epoch of its own.
        copy.scoring = true;
        if let Some(condition) = candidate {
            // A refusal is not swallowed into a silently-different run:
            // `candidates` already asked the two questions `revise` would
            // refuse on, so this is the status quo scored twice if it ever
            // fires, and the round's own record shows the equal figures.
            let _ = copy.revise(species, condition);
        }

        let mut score = Score {
            ticks,
            ..Score::default()
        };
        for _ in 0..ticks {
            copy.apply(Intent::Idle);
            for flow in copy.flows() {
                let record = &flow.record;
                if record.to.is_some_and(|to| to.lineage == species) {
                    score.income_mg = score.income_mg.saturating_add(record.amount_mg);
                }
                if record.from.is_some_and(|from| from.lineage == species) {
                    match record.process {
                        Process::Upkeep => {
                            score.rent_mg = score.rent_mg.saturating_add(record.amount_mg);
                        },
                        _ => score.outflow_mg = score.outflow_mg.saturating_add(record.amount_mg),
                    }
                }
            }
            // Drained rather than read, so a bounded run does not accumulate a
            // tick buffer it never empties.
            for recorded in copy.drain_events() {
                if let Event::Born { species: born, .. } = recorded.record
                    && born == species
                {
                    score.born += 1;
                }
            }
        }
        score
    }

    /// Living lineages in the order they act: descending recipe complexity,
    /// ties broken by species id.
    ///
    /// The complexity reading is `Species::recipe.complexity()`, which is what
    /// [`World::intricacy`] already reads and what the complexity frontier
    /// binds on — vocabulary is expensive and repetition is cheap, so this
    /// orders by how elaborate a line's recipe is and not by how long its
    /// bodies are. Ties break by id purely so a round replays identically.
    pub fn initiative(&self) -> Vec<SpeciesId> {
        let mut living: Vec<SpeciesId> = Vec::new();
        for organism in self.living() {
            if !living.contains(&organism.species) {
                living.push(organism.species);
            }
        }
        living.sort_by_key(|id| {
            let complexity = self
                .lineages
                .get(*id)
                .map(|line| line.recipe.complexity() as i64)
                .unwrap_or(0);
            (-complexity, id.0)
        });
        living
    }

    /// One adaptation round: every living unplayed lineage takes a turn, in
    /// initiative order, and commits immediately.
    ///
    /// **Immediately is the whole point.** A later line scores its candidates
    /// against a world the earlier ones have already changed; freeze the world
    /// for the duration of a round and the order becomes decoration.
    ///
    /// The played line is skipped. Its turn is the review — choosing among
    /// several candidates, seeing what each would cost, previewing a founder —
    /// and the review is PE3b.
    pub(super) fn adapt_round(&mut self) -> Round {
        let played = self.controlled().map(|organism| organism.species);
        let mut round = Round {
            epoch: self.epoch,
            turns: Vec::new(),
        };
        for species in self.initiative() {
            if Some(species) == played {
                continue;
            }
            let candidates = self.candidates(species);
            // A line whose only option is the revision it already has does not
            // take a turn: there is nothing to weigh, and scoring the status
            // quo against itself would spend a bounded run to learn nothing.
            if candidates.len() < 2 {
                continue;
            }
            let considered: Vec<(Option<ConditionId>, Score)> = candidates
                .into_iter()
                .map(|candidate| (candidate, self.score(species, candidate)))
                .collect();
            let chosen = best_of(&considered);
            // Through `World::revise`, exactly as the player would: one
            // transaction, one commit path, and the record it writes names the
            // line rather than a hand.
            let committed = chosen.and_then(|condition| self.revise(species, condition).ok());
            round.turns.push(Turn {
                lineage: species,
                considered,
                chosen,
                committed,
            });
        }
        round
    }
}

#[cfg(test)]
mod tests;
