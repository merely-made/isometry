// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The individual checkpoint: the game stops and asks one question.
//!
//! Reproduction is the checkpoint at the scale of one critter (playable ecology
//! plan §1). Descent becomes concrete, a parent pays for an offspring, and the
//! player may continue through a descendant. **It is not the epoch**, and it
//! must not silently invoke one: there is no program revision here, no lineage
//! budget, no founder preview, and nothing a player can edit. One question, two
//! answers, play resumes.
//!
//! # It lives in the driver, and that is the whole design
//!
//! PE0's seam finding is that the missing checkpoint is *host, control and
//! presentation composition rather than a second breeding system*. So nothing
//! below is a world rule. The breeding transaction is untouched: the adult-mass
//! gate, filial realization, the matter debit, the parent link and
//! `Event::Born` are exactly what they were, and this reads their records.
//!
//! The consequence is worth stating plainly. A pause is a question about **when
//! to step**, which is the driver's own job — it already owns the clock, the
//! step cap and the queue. So `World` gained one intent ([`Intent::Resume`]) and
//! nothing else: no checkpoint field, no held flag, no new snapshot byte. A
//! replay therefore lands on the byte-identical state hash it always did, and
//! the enclosure driven straight through `World::apply` — the population
//! instrument, every headless lab — cannot even observe that this module exists.
//!
//! # What opens one
//!
//! Three occasions, and all of them require a **hand on the critter**.
//! [`World::held`] is the ruled distinction between *control* (whose body a key
//! would move) and *holding* (whether anybody has moved it lately, TD4). An ant
//! farm nobody is touching is the feature; stopping it to ask an empty chair a
//! question would be the opposite. So an idle terrarium runs on, and every
//! existing headless fixture keeps its exact timing.
//!
//! # The third occasion is the lineage's, not an individual's (PE3a)
//!
//! [`Occasion::Epoch`] is the other checkpoint the epoch-boundary plan §0
//! separates from this one: the epoch's budget is spent, every unplayed line
//! has taken its turn, and the played line has not. **The world ends its own
//! epoch** — the Timed rule is a versioned world rule, so a headless enclosure
//! obeys it too — and what lives here is only the *hold*, exactly as it is for
//! a birth: stepping is the driver's job, so not stepping is too.
//!
//! It is still not the trait board. There is no review here, no founder
//! preview and no budget; PE3b owns those. What this adds is the one place a
//! revision is admitted at all.
//!
//! # What answers one
//!
//! [`Intent::TakeControl`] or [`Intent::Resume`] — take, or carry on. Both are
//! ordinary recorded intents, so the choice is in the trace and replays with
//! everything else, and eligibility stays where it was rather than growing a
//! second gate here. At the lineage checkpoint [`Intent::Revise`] is an answer
//! too, and the one answer that does **not** close the question: a revision is
//! taken *at* the checkpoint, and the world stays there until it is resumed.
//!
//! [`World::held`]: mesocosm_core::World::held

use mesocosm_core::flow::{Account, Process, RecordedEvent, RecordedFlow};
use mesocosm_core::history::Event;
use mesocosm_core::{History, Intent, OrganismId, SpeciesId, World};

/// A birth the played critter is the parent of.
///
/// Everything the checkpoint has to be able to point at: who bore it, what was
/// born, and what it cost — read off PE0's flow record rather than recomputed,
/// so the number on the panel is the number the ledger reconciled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Birth {
    pub parent: OrganismId,
    pub offspring: OrganismId,
    pub lineage: SpeciesId,
    /// Body the parent gave up, in milligrams.
    pub substance_mg: u64,
    /// Budget the parent handed over with it.
    pub reserve_mg: u64,
}

impl Birth {
    /// What the offspring cost its parent, both accounts together.
    pub fn cost_mg(self) -> u64 {
        self.substance_mg + self.reserve_mg
    }
}

/// The body a run was in stopped being one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Loss {
    pub organism: OrganismId,
    pub lineage: SpeciesId,
}

/// The epoch's budget ran out, and every unplayed line has taken its turn.
///
/// Read off the world's own round rather than recomputed, so the counts on the
/// panel are the counts the transcript holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Boundary {
    /// The epoch that just closed.
    pub epoch: u64,
    /// The line under your hand. It has *not* taken a turn: yours is the
    /// review, and the review is PE3b.
    pub lineage: SpeciesId,
    /// Lineages that weighed a candidate this round.
    pub turned: usize,
    /// Of those, the ones that committed a revision.
    pub committed: usize,
}

/// Why the game stopped to ask.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occasion {
    /// A birth involving the critter under your hand.
    Birth(Birth),
    /// Control ended with the life it was attached to.
    Loss(Loss),
    /// The epoch ended. **The lineage checkpoint** (PE3a).
    Epoch(Boundary),
}

/// One bounded question, and the answers available to it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    /// The tick the world holds at. It does not advance while this stands.
    pub tick: u64,
    pub occasion: Occasion,
    /// Living descendants this world would let the player inhabit, eldest
    /// first. A **reading of the ecology**, taken when the question opened:
    /// these organisms are not reserved, removed or held anywhere, and the run
    /// that declines leaves every one of them exactly where it was.
    pub heirs: Vec<OrganismId>,
}

impl Checkpoint {
    /// The one heir a single keystroke takes: the eldest living descendant.
    ///
    /// **Deliberately one, not a list.** Offering the brood as a numbered
    /// roster is how siblings become menu inventory, which PE1 forbids by
    /// name; a player who wants a particular one still has the ordinary
    /// `TakeControl` and has to reach it in the world.
    pub fn heir(&self) -> Option<OrganismId> {
        self.heirs.first().copied()
    }

    /// Whether this intent settles the question. Anything else is not an
    /// answer, and the world stays where it is.
    pub fn answers(&self, intent: &Intent) -> bool {
        match intent {
            Intent::Resume => true,
            Intent::TakeControl { organism } => self.heirs.contains(organism),
            // Only at the lineage checkpoint, and it is the one answer that
            // leaves the question standing — see [`Self::closed_by`].
            Intent::Revise { .. } => matches!(self.occasion, Occasion::Epoch(_)),
            _ => false,
        }
    }

    /// Whether answering with this intent also ends the hold.
    ///
    /// Every answer but one does. A revision is taken *at* the boundary rather
    /// than on the way out of it, so committing does not throw the player back
    /// into the terrarium — resuming does.
    pub fn closed_by(&self, intent: &Intent) -> bool {
        self.answers(intent) && !matches!(intent, Intent::Revise { .. })
    }

    /// The answer that changes nothing — **the world default**, and the one a
    /// host binds to its "carry on" key.
    ///
    /// Continuing is the default because it is the only answer that can be
    /// taken back. The offspring stays alive in the enclosure and
    /// `TakeControl` still reaches it; a default that moved control would
    /// silently discard a body the player spent the whole run growing, and
    /// nothing undoes that. Which of the three the game *should* default to is
    /// an open interaction ruling (playable ecology plan §6, ruling 1) and this
    /// is the implementation's placeholder for it, not an answer to it.
    pub fn default_answer(&self) -> Intent {
        Intent::Resume
    }
}

/// Opens a checkpoint, if this tick warrants one.
///
/// `hand` is [`World::held`] read **before** the tick — after it, a critter
/// that just died is no longer held by anyone, and the question would never be
/// asked at the one moment it matters most.
pub(crate) fn opened(
    world: &World,
    history: &History,
    hand: Option<(OrganismId, SpeciesId)>,
    events: &[RecordedEvent],
    flows: &[RecordedFlow],
) -> Option<Checkpoint> {
    let (hand, hand_lineage) = hand?;

    // Loss first. A tick that both bore a child and killed the parent is one
    // question, and it is the one about who you are now.
    //
    // The lineage comes off the hand rather than off the roster because a
    // critter can lose its body by being *eaten*, and an eaten one is no longer
    // in the roster to be asked what it was.
    if let Some(lost) = world.control_lost().filter(|lost| *lost == hand) {
        return Some(Checkpoint {
            tick: world.tick,
            occasion: Occasion::Loss(Loss {
                organism: lost,
                lineage: hand_lineage,
            }),
            heirs: world.heirs(history, lost),
        });
    }

    let (offspring, lineage) = match events.iter().find_map(|recorded| match recorded.record {
        Event::Born {
            organism,
            species,
            parent: Some(parent),
        } if parent == hand => Some((organism, species)),
        _ => None,
    }) {
        Some(born) => born,
        // **The lineage checkpoint, after the individual one.** A tick that
        // both bore a child and ended the epoch is asked about the child,
        // because the heir list is read at the moment it exists and is gone
        // afterwards while the epoch's own round is already committed and
        // recorded. The cost is real and worth naming: that epoch's review is
        // not offered, and the next boundary is a whole budget away.
        //
        // `World::at_boundary` is a one-tick fact; what makes it last is that
        // a held world does not step, which is exactly as long as the hold
        // below lasts.
        None if world.at_boundary() => {
            let round = world.last_round();
            return Some(Checkpoint {
                tick: world.tick,
                occasion: Occasion::Epoch(Boundary {
                    epoch: world.epoch,
                    lineage: hand_lineage,
                    turned: round.turns.len(),
                    committed: round.changes().count(),
                }),
                // Nobody is offered a body here. This is the lineage's
                // checkpoint, not an individual's.
                heirs: Vec::new(),
            });
        },
        None => return None,
    };

    let mut substance_mg = 0;
    let mut reserve_mg = 0;
    for flow in flows {
        let record = &flow.record;
        if record.process != Process::Birth || record.to.map(|to| to.organism) != Some(offspring) {
            continue;
        }
        match record.destination {
            Account::Substance => substance_mg += record.amount_mg,
            Account::Reserve => reserve_mg += record.amount_mg,
            // Neither is a body's account, so neither is a provisioning: a
            // birth never lands in the ground, and the dev source is only ever
            // a *source*.
            Account::Soil | Account::Dev => {},
        }
    }

    Some(Checkpoint {
        tick: world.tick,
        occasion: Occasion::Birth(Birth {
            parent: hand,
            offspring,
            lineage,
            substance_mg,
            reserve_mg,
        }),
        // The one just born, and only if this world would let anyone hold it.
        // Not the brood.
        heirs: world
            .is_eligible(offspring)
            .then_some(vec![offspring])
            .unwrap_or_default(),
    })
}
