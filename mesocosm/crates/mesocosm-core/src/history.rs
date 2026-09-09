// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What happened, and what it followed from.
//!
//! A [`Journal`] of [`Event`]s, with causality. Until now `World::apply`
//! returned an outcome and dropped it, so the world had a present and no past,
//! and every proposal that reads history had nothing to read.
//!
//! # It sits beside the world, not inside it
//!
//! History is **derivable**: a seed plus ordered intents reproduces it exactly.
//! So it must not live in the snapshot, or whole-state capture would grow
//! without bound and stop being the cheap memcpy the wing's rollback thinking
//! rests on. It is materialised rather than recomputed only because queries
//! want it now, which makes it a durable cache and exactly what a muniment slot
//! is for.
//!
//! The world therefore buffers at most one tick of events and a caller drains
//! them. That buffer is bounded by definition.
//!
//! # Causality without a bookkeeping burden
//!
//! Every event cites **the last event about each subject it touches**. One
//! `Seq` per organism is the whole apparatus, and it produces a genuine graph
//! rather than a chain: an event involving two creatures cites both of their
//! histories, so eating *joins* two causal lines that were independent until
//! they met.
//!
//! That is the structure significance is made of. `muniment`'s
//! [`effects`](Journal::effects) then answers what followed from an event,
//! which is the retroactive definition of significance: a thing mattered
//! because of what later depended on it.
//!
//! # What it deliberately does not do
//!
//! It does not decide what was *significant*. Abnormality is a lookup against
//! [`WorldRecord`](crate::record::WorldRecord); this is the traversal half.
//! Two questions, two structures.
//!
//! It also does not carry resource movement. Every milligram of upkeep and soil
//! draw in a permanent causal log would be the wrong record at the wrong
//! frequency; [`flow`](crate::flow) is the other half, and the two share one
//! commit point and one [`Envelope`](crate::flow::Envelope).

use std::collections::BTreeMap;

use muniment::{Journal, Seq};
use serde::{Deserialize, Serialize};

use crate::body::{PartId, SpeciesId};
use crate::flow::RecordedEvent;
use crate::organism::OrganismId;

/// Why a feeding event happened. Keeping this on the event makes predation
/// and scavenging distinguishable without reconstructing world state from
/// later deaths.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MealKind {
    Grazing,
    Predation,
    Scavenging,
}

/// Something that happened to somebody.
///
/// Small on purpose. Each variant names the organisms it touches, because that
/// is what the causal links are built from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Event {
    /// A creature entered the world.
    Born {
        organism: OrganismId,
        species: SpeciesId,
        parent: Option<OrganismId>,
    },
    /// A creature came of age.
    Matured { organism: OrganismId },
    /// A creature took substance from another. The join that makes this a
    /// graph rather than a set of chains.
    Fed {
        eater: OrganismId,
        from: OrganismId,
        mass_mg: u64,
        kind: MealKind,
    },
    /// A creature changed position under its own drive.
    Moved {
        organism: OrganismId,
        from: [i32; 3],
        to: [i32; 3],
    },
    /// A creature took a meal and kept it as body.
    Grew { organism: OrganismId, part: PartId },
    /// A creature took a meal and spent it.
    Burned {
        organism: OrganismId,
        energy_mg: u64,
    },
    /// A creature lost a part, and everything below it.
    Severed { organism: OrganismId, part: PartId },
    /// A creature died.
    Died {
        organism: OrganismId,
        species: SpeciesId,
    },
    /// A carcass returned to the world.
    Returned { organism: OrganismId },
    /// The player took a body.
    Inhabited { organism: OrganismId },
    /// A line split off another, and was named.
    Speciated {
        species: SpeciesId,
        from: SpeciesId,
        founder: OrganismId,
    },
    /// A creature removed ground: a burrow, a bore, a den. World-shaping
    /// enters the same history as eating, because a carved refuge is as
    /// biographical as a meal.
    Carved {
        organism: OrganismId,
        at: [i32; 3],
        removed: u32,
    },
    /// A line came to a new developmental option, and this names the condition
    /// it came through. (PE2)
    ///
    /// **Replaces `Learned`.** That variant recorded one appendage word per
    /// meal, because `World::learn_from` taught every non-innate appendage in
    /// the donor's whole recipe; the plan named it a migration input. What is
    /// recorded now is the *condition*, whose digest resolves to the matched
    /// evidence, the route, the realized candidate and its parameters through
    /// [`World::discoveries`](crate::World::discoveries). A meal that teaches
    /// is still a different kind of event from a meal that feeds — it is just
    /// no longer only a meal that can teach.
    Discovered {
        organism: OrganismId,
        species: SpeciesId,
        condition: crate::discovery::ConditionId,
    },
    /// A creature rebuilt one of its organs, and paid for it. (PD2)
    ///
    /// The first event about a body changing what it *does* rather than what
    /// it is made of. Growing and severing are already here; this is the third
    /// way a body becomes different, and PD1a ruled it must never happen
    /// except through a discrete event with a cost and a cause — so it is as
    /// biographical as a meal, and recorded like one.
    Expressed {
        organism: OrganismId,
        part: PartId,
        cost_mg: u64,
    },
    /// A branch left one body and arrived on another. (P3)
    ///
    /// **One event with two subjects**, which is what the wing contract asks
    /// for: the source's loss and the destination's acquisition are one
    /// causally linked fact, not two unrelated ones that happen to agree about
    /// a tick. `Fed` is the only other event shaped this way, for the same
    /// reason.
    Grafted {
        organism: OrganismId,
        from: OrganismId,
        /// The branch root's new id on the recipient.
        part: PartId,
        /// The donor-local id it came off.
        from_part: PartId,
        /// How many parts came with it.
        parts: u32,
    },
    /// A line committed a revision of its development program. (P4)
    ///
    /// **What a lineage did, not what a body did.** Growing, losing and
    /// rebuilding an organ are facts about one creature; this is a fact about
    /// everything descended from here, which is why it names a species and a
    /// revision rather than a part.
    Revised {
        species: SpeciesId,
        revision: crate::program::RevisionId,
        /// The discovery the revision cites, as its condition.
        condition: crate::discovery::ConditionId,
        /// The creature whose hand was on the line. `None` for an unplayed
        /// lineage, which takes the same world transaction with nobody in it.
        by: Option<OrganismId>,
    },
    /// A descendant was born expressing its line's current revision. (PD5)
    ///
    /// **The third record, and it is not the other two.** Somatic
    /// incorporation ([`Grew`](Event::Grew), [`Expressed`](Event::Expressed))
    /// changes the body you are in; dormant acquisition
    /// ([`Discovered`](Event::Discovered)) widens what a line may later grow;
    /// this is a body that arrived already growing it. The revision resolves
    /// to the discovery it cites, so the provenance survives without a second
    /// reference here.
    Inherited {
        organism: OrganismId,
        species: SpeciesId,
        revision: crate::program::RevisionId,
        part: PartId,
        cost_mg: u64,
    },
    /// A descendant was born under a revision its body could not express, and
    /// this names which and why. (PD5)
    ///
    /// **A refusal is a fact.** The birth still happened, under geometry
    /// seeding; growing the old body and saying nothing would make an
    /// inherited program unfalsifiable, and "a candidate that cannot be taken
    /// is the ordinary case" (PE2) is exactly as true one generation down.
    Unexpressed {
        organism: OrganismId,
        species: SpeciesId,
        revision: crate::program::RevisionId,
        why: crate::program::Unexpressed,
    },
}

impl Event {
    /// The organisms this event is about, in a fixed order.
    ///
    /// What the causal links are built from, so it must be exhaustive: an
    /// organism left out here has its history quietly forked.
    ///
    /// [`Revised`](Event::Revised) is the one event that may name nobody, and
    /// that is the fact rather than an omission: a lineage revision committed
    /// on an unplayed line happened to the line and to no creature in it.
    pub fn subjects(&self) -> Vec<OrganismId> {
        match *self {
            Event::Born {
                organism, parent, ..
            } => parent
                .map(|p| vec![p, organism])
                .unwrap_or_else(|| vec![organism]),
            Event::Fed { eater, from, .. } => vec![eater, from],
            // Both ends, for `Fed`'s reason: a transfer that named only one of
            // them would fork the other one's history.
            Event::Grafted { organism, from, .. } => vec![organism, from],
            Event::Matured { organism }
            | Event::Grew { organism, .. }
            | Event::Burned { organism, .. }
            | Event::Severed { organism, .. }
            | Event::Moved { organism, .. }
            | Event::Died { organism, .. }
            | Event::Returned { organism }
            | Event::Inhabited { organism }
            | Event::Expressed { organism, .. }
            | Event::Inherited { organism, .. }
            | Event::Unexpressed { organism, .. }
            | Event::Carved { organism, .. } => vec![organism],
            // A revision on an unplayed line names nobody, which is the fact:
            // it happened to the line. A played one names the hand.
            Event::Revised { by, .. } => by.into_iter().collect(),
            // The founder's line continues through the split, which is what
            // makes a speciation visible in its own history rather than only
            // in the world's.
            Event::Speciated { founder, .. } => vec![founder],
            Event::Discovered { organism, .. } => vec![organism],
        }
    }
}

/// Which way a creature stopped living. (DT2)
///
/// The record's own two endings, not a third classification over them: the
/// ecology writes [`Event::Died`] when a body starved or aged out and left a
/// corpse, and [`Event::Returned`] when there was nothing left of it to leave
/// one. Anything finer — starved against aged — is not in the record, so it is
/// not offered here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passing {
    /// It starved or aged out, and a corpse was left where it stood.
    Died,
    /// Nothing was left of it, and what there was went back to the world.
    Returned,
}

/// When a creature stopped living, and which way. (DT2)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ending {
    pub organism: OrganismId,
    /// The tick the record was stamped with, off the same [`Envelope`] every
    /// other reading takes a tick from.
    ///
    /// [`Envelope`]: crate::flow::Envelope
    pub tick: u64,
    pub how: Passing,
}

/// The world's past.
///
/// Entries are [`RecordedEvent`]s: the event plus when and where it happened.
/// The envelope arrived with PE0's flow record and is shared with it, because a
/// past with no tick on it cannot answer *how many died in the last two hundred
/// ticks*, which is what the first ecology reading is made of.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct History {
    log: Journal<RecordedEvent>,
    /// The most recent event about each organism, which is how a new one finds
    /// its causes. Ordered, so recording is deterministic.
    latest: BTreeMap<OrganismId, Seq>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends an event, citing the last event about each subject it touches.
    ///
    /// A creature's first event has no cause, which is honest: nothing in this
    /// log led to it. Every later one continues that creature's line, and an
    /// event touching two creatures joins theirs.
    pub fn record(&mut self, event: RecordedEvent) -> Seq {
        let subjects = event.record.subjects();
        let causes: Vec<Seq> = subjects
            .iter()
            .filter_map(|who| self.latest.get(who).copied())
            .collect();

        let seq = self
            .log
            .append_caused_by(causes, event)
            .expect("causes are drawn from this log, so they always exist");

        for who in subjects {
            self.latest.insert(who, seq);
        }
        seq
    }

    /// Records a tick's worth of events, in order.
    pub fn record_all(&mut self, events: impl IntoIterator<Item = RecordedEvent>) {
        for event in events {
            self.record(event);
        }
    }

    /// The log itself, for causal queries.
    pub fn log(&self) -> &Journal<RecordedEvent> {
        &self.log
    }

    pub fn len(&self) -> usize {
        self.log.len()
    }

    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    /// The record, if any: the event with its tick and place.
    pub fn get(&self, seq: Seq) -> Option<&RecordedEvent> {
        self.log.get(seq)
    }

    /// Just the event, for a caller that only wants what happened.
    pub fn event(&self, seq: Seq) -> Option<&Event> {
        self.get(seq).map(|recorded| &recorded.record)
    }

    /// The most recent event about an organism.
    pub fn latest(&self, organism: OrganismId) -> Option<Seq> {
        self.latest.get(&organism).copied()
    }

    /// Everything that followed from an event.
    ///
    /// **The significance query.** An event mattered because of what later
    /// depended on it, and a flat log cannot answer that at all.
    pub fn consequences(&self, seq: Seq) -> Vec<Seq> {
        self.log.effects(seq)
    }

    /// Everything that led to an event, nearest first.
    pub fn antecedents(&self, seq: Seq) -> Vec<Seq> {
        self.log.causes(seq)
    }

    /// Whether neither event led to the other.
    pub fn concurrent(&self, a: Seq, b: Seq) -> bool {
        self.log.concurrent(a, b)
    }

    /// How a creature's life ended, if this record holds the moment. (DT2)
    ///
    /// **The event the record already carries**, with the tick off its own
    /// envelope — not a state read of the roster, which cannot say when, and
    /// which a body eaten to nothing has already left. `None` means this past
    /// has no ending for it: it is still alive, or it ended before this record
    /// began.
    ///
    /// The most recent one, walking back, because a `Died` is followed by a
    /// `Returned` when the corpse finishes decaying and the later of the two is
    /// the one that is true now.
    pub fn ending(&self, organism: OrganismId) -> Option<Ending> {
        self.log.entries().iter().rev().find_map(|recorded| {
            let how = match recorded.record {
                Event::Died { organism: who, .. } if who == organism => Passing::Died,
                Event::Returned { organism: who } if who == organism => Passing::Returned,
                _ => return None,
            };
            Some(Ending {
                organism,
                tick: recorded.tick,
                how,
            })
        })
    }

    /// An organism's own line, oldest first.
    pub fn line_of(&self, organism: OrganismId) -> Vec<Seq> {
        let Some(latest) = self.latest(organism) else {
            return Vec::new();
        };
        let mut line: Vec<Seq> = self
            .antecedents(latest)
            .into_iter()
            .filter(|seq| self.touches(*seq, organism))
            .collect();
        line.push(latest);
        line.sort();
        line
    }

    /// Everyone descended from an organism, nearest generation first.
    ///
    /// **Descent is already written down.** `Event::Born` has carried its
    /// parent since the breeding transaction existed, so PE1 needs no second
    /// link on the body and no lineal table beside the world: the question is a
    /// walk over the record that answers it. Transitive, because a line
    /// continues through a grandchild when the child is gone, and the visited
    /// set is what keeps a walk over a log finite.
    ///
    /// Ids within a generation come out ascending, so "the eldest" is a
    /// deterministic thing to ask for rather than whatever iteration found
    /// first.
    pub fn descendants(&self, of: OrganismId) -> Vec<OrganismId> {
        let mut children: BTreeMap<OrganismId, Vec<OrganismId>> = BTreeMap::new();
        for recorded in self.log.entries() {
            if let Event::Born {
                organism,
                parent: Some(parent),
                ..
            } = recorded.record
            {
                children.entry(parent).or_default().push(organism);
            }
        }

        let mut found = Vec::new();
        let mut seen = std::collections::BTreeSet::from([of]);
        let mut generation = children.get(&of).cloned().unwrap_or_default();
        while !generation.is_empty() {
            generation.sort_unstable();
            let mut next = Vec::new();
            for child in generation {
                if !seen.insert(child) {
                    continue;
                }
                found.push(child);
                next.extend(children.get(&child).into_iter().flatten().copied());
            }
            generation = next;
        }
        found
    }

    fn touches(&self, seq: Seq, organism: OrganismId) -> bool {
        self.event(seq)
            .is_some_and(|event| event.subjects().contains(&organism))
    }
}

#[cfg(test)]
mod tests;
