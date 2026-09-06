// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a world has seen, and what it remembers of who did it.
//!
//! Significance is **abnormality measured against the world's own record**, so
//! there has to be a record. This is the smallest one that answers the
//! question, and its shape was chosen for a property that turns out to matter
//! more than its size.
//!
//! # It merges without a protocol
//!
//! Worlds fork, graft, and combine, so two records will need joining. This one
//! joins by taking the higher mark per axis, and that operation is
//! **commutative, associative, and idempotent**. Those three together make it a
//! join-semilattice, which means peers can hand each other records in any
//! order, twice, interleaved, and still converge. No coordination, no merge
//! protocol, no last-writer-wins.
//!
//! That is why this is a few integers rather than a search index. A text index
//! can be merged but its relevance stops meaning the same thing; a vector index
//! merges only if both sides embedded with the same model, which is the
//! ruleset-binding problem wearing a hat. **Mergeability is the requirement
//! that picks the structure**, not size.
//!
//! It is also the one place the wing's guidance actually calls for a
//! conflict-free type: introduce one only where a domain proves it needs
//! mergeable concurrent values. This domain proves it, and it is the trivial
//! case.
//!
//! # Thresholds forget, holders are retold
//!
//! A pure maximum forgets *who*. Beating a record would erase the name of
//! whoever held it, which is exactly the fact loss that makes a history feel
//! fake.
//!
//! So a [`Mark`] keeps both: the threshold as a maximum, and the holders of
//! *that* threshold as a set. Ties union, a higher mark replaces, and the set
//! stays small on its own without an arbitrary cap. Remembering every past
//! holder forever is the unbounded version, and it is the tulpa selector's
//! problem rather than this type's: the journal holds everything, tulpa holds what
//! is retold. Same rule, world scale.
//!
//! # What this does not answer
//!
//! Abnormality is a lookup: *has anyone reached this*. Significance in the
//! fuller sense is a traversal: *what later depended on this*, which is
//! the journal's causal graph. Two questions, two structures, deliberately.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;

/// The kind of thing a lineage did.
///
/// Harmony and domination sit beside each other rather than opposing, so a
/// lineage can score highly on both. That is a real ecological posture: made
/// itself indispensable and dangerous at once.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Feat {
    /// Biomass gained.
    Growth,
    /// Taken from others.
    Predation,
    /// Given to, or depended on by, others.
    Symbiosis,
    /// Survived what others did not.
    Endurance,
    /// Reached where others had not.
    Spread,
    /// Changed the world rather than living in it.
    Construction,
}

impl Feat {
    pub const ALL: [Feat; 6] = [
        Feat::Growth,
        Feat::Predation,
        Feat::Symbiosis,
        Feat::Endurance,
        Feat::Spread,
        Feat::Construction,
    ];
}

/// How far a feat reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Scale {
    Local,
    Regional,
    Worldwide,
}

/// A high-water mark, and who stands at it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Mark {
    /// The highest anyone has reached. Integer, like everything the simulation
    /// decides, so two worlds compare exactly.
    pub high: i64,
    /// Who reached it. Ties share the mark; being beaten gives it up.
    pub holders: BTreeSet<SpeciesId>,
}

impl Mark {
    fn new(high: i64, by: SpeciesId) -> Self {
        Self {
            high,
            holders: BTreeSet::from([by]),
        }
    }

    /// Joins another mark into this one.
    ///
    /// The whole semilattice, in one match. A higher mark wins outright, an
    /// equal one shares, and the operation does not care which side it was
    /// called on or how many times.
    fn join(&mut self, other: &Mark) {
        match other.high.cmp(&self.high) {
            std::cmp::Ordering::Greater => *self = other.clone(),
            std::cmp::Ordering::Equal => self.holders.extend(other.holders.iter().copied()),
            std::cmp::Ordering::Less => {},
        }
    }
}

/// Everything a world has seen, keyed by what and how far.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldRecord {
    /// Ordered, so iteration and serialization are deterministic.
    marks: BTreeMap<(Feat, Scale), Mark>,
}

impl WorldRecord {
    pub fn new() -> Self {
        Self::default()
    }

    /// The standing mark for one axis, if anyone has set one.
    pub fn standing(&self, feat: Feat, scale: Scale) -> Option<&Mark> {
        self.marks.get(&(feat, scale))
    }

    /// Whether this would be the first, or the best, anyone has managed.
    ///
    /// **The abnormality query.** One comparison, which is the whole reason
    /// the record is a handful of integers rather than an index.
    pub fn is_unprecedented(&self, feat: Feat, scale: Scale, value: i64) -> bool {
        self.standing(feat, scale)
            .is_none_or(|mark| value > mark.high)
    }

    /// Whether anyone has ever done this at all, at any magnitude.
    ///
    /// A goal generator wants this: "something no species on the planet has
    /// done" is a different question from "more than anyone has done."
    pub fn untouched(&self, feat: Feat, scale: Scale) -> bool {
        self.standing(feat, scale).is_none()
    }

    /// Records what a lineage did. Returns whether it took the record.
    pub fn note(&mut self, feat: Feat, scale: Scale, value: i64, by: SpeciesId) -> bool {
        let took = self.is_unprecedented(feat, scale, value);
        self.marks
            .entry((feat, scale))
            .and_modify(|mark| mark.join(&Mark::new(value, by)))
            .or_insert_with(|| Mark::new(value, by));
        took
    }

    /// Joins another world's record into this one.
    ///
    /// Order-independent and repeatable, so a moot can fold records from peers
    /// as they arrive without sequencing them.
    pub fn merge(&mut self, other: &WorldRecord) {
        for (axis, mark) in &other.marks {
            self.marks
                .entry(*axis)
                .and_modify(|mine| mine.join(mark))
                .or_insert_with(|| mark.clone());
        }
    }

    /// Axes anyone has reached, in a deterministic order.
    pub fn axes(&self) -> impl Iterator<Item = (Feat, Scale)> + '_ {
        self.marks.keys().copied()
    }

    /// How much of the possible record this world has filled.
    ///
    /// A world with nothing left untouched has no significance left to offer,
    /// which is the storyteller's actual brief: keep this below one by opening
    /// possibility rather than by manufacturing calamity.
    pub fn filled(&self) -> usize {
        self.marks.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: SpeciesId = SpeciesId(1);
    const B: SpeciesId = SpeciesId(2);
    const C: SpeciesId = SpeciesId(3);

    fn record(entries: &[(Feat, Scale, i64, SpeciesId)]) -> WorldRecord {
        let mut record = WorldRecord::new();
        for (feat, scale, value, by) in entries {
            record.note(*feat, *scale, *value, *by);
        }
        record
    }

    #[test]
    fn the_first_of_anything_is_unprecedented() {
        let mut world = WorldRecord::new();
        assert!(world.untouched(Feat::Spread, Scale::Worldwide));
        assert!(world.is_unprecedented(Feat::Spread, Scale::Worldwide, 1));

        assert!(
            world.note(Feat::Spread, Scale::Worldwide, 1, A),
            "and it takes the record"
        );
        assert!(!world.untouched(Feat::Spread, Scale::Worldwide));
    }

    #[test]
    fn a_world_gets_harder_to_impress() {
        // The difficulty curve nobody authored: the same act stops being
        // remarkable once the record has moved past it.
        let mut world = WorldRecord::new();
        world.note(Feat::Growth, Scale::Regional, 40, A);

        assert!(
            !world.is_unprecedented(Feat::Growth, Scale::Regional, 40),
            "matching is not beating"
        );
        assert!(!world.is_unprecedented(Feat::Growth, Scale::Regional, 10));
        assert!(world.is_unprecedented(Feat::Growth, Scale::Regional, 41));
    }

    #[test]
    fn beating_a_record_takes_it_and_matching_it_shares() {
        let mut world = WorldRecord::new();
        world.note(Feat::Predation, Scale::Local, 10, A);

        world.note(Feat::Predation, Scale::Local, 10, B);
        let mark = world.standing(Feat::Predation, Scale::Local).unwrap();
        assert_eq!(mark.holders, BTreeSet::from([A, B]), "a tie is shared");

        world.note(Feat::Predation, Scale::Local, 20, C);
        let mark = world.standing(Feat::Predation, Scale::Local).unwrap();
        assert_eq!(mark.high, 20);
        assert_eq!(
            mark.holders,
            BTreeSet::from([C]),
            "being beaten gives it up"
        );
    }

    #[test]
    fn scales_and_feats_are_separate_records() {
        // Doing something locally says nothing about having done it worldwide,
        // and a predator's record is not a symbiote's.
        let mut world = WorldRecord::new();
        world.note(Feat::Growth, Scale::Local, 90, A);

        assert!(world.untouched(Feat::Growth, Scale::Worldwide));
        assert!(world.untouched(Feat::Symbiosis, Scale::Local));
        assert!(world.is_unprecedented(Feat::Growth, Scale::Worldwide, 1));
    }

    #[test]
    fn harmony_and_domination_are_not_opposites() {
        // A lineage can hold both, which is a real posture rather than a
        // contradiction: indispensable and dangerous at once.
        let mut world = WorldRecord::new();
        world.note(Feat::Symbiosis, Scale::Regional, 50, A);
        world.note(Feat::Predation, Scale::Regional, 50, A);

        for feat in [Feat::Symbiosis, Feat::Predation] {
            let mark = world.standing(feat, Scale::Regional).unwrap();
            assert!(mark.holders.contains(&A));
        }
    }

    // --- the three laws that make merging safe without a protocol ---

    #[test]
    fn merging_is_commutative() {
        let left = record(&[
            (Feat::Growth, Scale::Local, 10, A),
            (Feat::Spread, Scale::Local, 5, A),
        ]);
        let right = record(&[(Feat::Growth, Scale::Local, 30, B)]);

        let mut a = left.clone();
        a.merge(&right);
        let mut b = right.clone();
        b.merge(&left);

        assert_eq!(a, b, "which side merged first cannot matter");
    }

    #[test]
    fn merging_is_associative() {
        let x = record(&[(Feat::Growth, Scale::Local, 10, A)]);
        let y = record(&[(Feat::Growth, Scale::Local, 30, B)]);
        let z = record(&[
            (Feat::Growth, Scale::Local, 20, C),
            (Feat::Endurance, Scale::Worldwide, 7, C),
        ]);

        let mut left = x.clone();
        left.merge(&y);
        left.merge(&z);

        let mut yz = y.clone();
        yz.merge(&z);
        let mut right = x.clone();
        right.merge(&yz);

        assert_eq!(left, right, "grouping cannot matter either");
    }

    #[test]
    fn merging_is_idempotent() {
        // The property that lets a peer resend a record without harm, which is
        // what removes the need for a protocol.
        let mine = record(&[(Feat::Growth, Scale::Local, 10, A)]);
        let theirs = record(&[(Feat::Growth, Scale::Local, 30, B)]);

        let mut once = mine.clone();
        once.merge(&theirs);

        let mut twice = once.clone();
        twice.merge(&theirs);
        twice.merge(&theirs);

        assert_eq!(once, twice, "merging again changes nothing");
    }

    #[test]
    fn merging_worlds_keeps_the_better_mark_and_both_ties() {
        let mut old = record(&[
            (Feat::Growth, Scale::Worldwide, 40, A),
            (Feat::Spread, Scale::Local, 12, A),
        ]);
        let new = record(&[
            (Feat::Growth, Scale::Worldwide, 60, B),
            (Feat::Spread, Scale::Local, 12, C),
            (Feat::Construction, Scale::Regional, 3, B),
        ]);

        old.merge(&new);

        let growth = old.standing(Feat::Growth, Scale::Worldwide).unwrap();
        assert_eq!(
            (growth.high, growth.holders.len()),
            (60, 1),
            "the better mark wins outright"
        );

        let spread = old.standing(Feat::Spread, Scale::Local).unwrap();
        assert_eq!(
            spread.holders,
            BTreeSet::from([A, C]),
            "an equal mark keeps both names"
        );

        assert!(
            !old.untouched(Feat::Construction, Scale::Regional),
            "and axes only they had arrive"
        );
    }

    #[test]
    fn a_record_round_trips() {
        let world = record(&[
            (Feat::Growth, Scale::Local, 10, A),
            (Feat::Spread, Scale::Worldwide, 2, B),
        ]);
        let bytes = crate::snapshot::encode(&world).unwrap();
        assert_eq!(
            crate::snapshot::decode::<WorldRecord>(&bytes).unwrap(),
            world
        );
    }

    #[test]
    fn an_empty_world_has_everything_left_to_offer() {
        let world = WorldRecord::new();
        assert_eq!(world.filled(), 0);
        for feat in Feat::ALL {
            for scale in [Scale::Local, Scale::Regional, Scale::Worldwide] {
                assert!(world.untouched(feat, scale));
            }
        }
    }
}
