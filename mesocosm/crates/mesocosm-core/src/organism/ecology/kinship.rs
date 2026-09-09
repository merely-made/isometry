// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **Kinship tempers the appetite** (TD10), and the first production caller of
//! the lineage tree.
//!
//! TD9 measured the fifth structural thing in the way: 92-97% of the mass taken
//! out of the consumer kingdom is taken by consumers, 90-94% of it by the
//! eater's *own species*, and the founding cohort is gone inside 300 ticks
//! against a ~580-tick first brood interval. `choose_living_target` filtered a
//! predator's candidates by reach and by signal and by nothing else, while the
//! score *preferred* mass — so the richest plain body within reach of a
//! founding consumer was another founding consumer.
//!
//! # The rule
//!
//! Relatedness is [`Lineages::distance`]: forks since two lines diverged,
//! integer-exact and already tested. It is spent here as a **remove** — how
//! much further away a body reads for being kin, in the same voxels the score
//! already measures distance in:
//!
//! ```text
//! remove = (span + 1) >> (forks + hungry)       relation known
//! remove = 0                                    no common ancestor
//! ```
//!
//! `span` is the far edge of whatever the eater is choosing among — bite reach
//! when it bites, sight when it is deciding where to walk. A body of the
//! eater's own line therefore reads as though it stood one voxel past that
//! edge, and ranks behind everything the eater could actually get to. **Each
//! fork of divergence halves that**, so a cousin is taken more readily than a
//! sibling and a distant cousin is barely noticed. Nothing is ever forbidden: a
//! discounted body is still a candidate, so a predator with nothing else in
//! reach still takes kin. Rare, not impossible.
//!
//! # An undefined distance costs nothing
//!
//! Genesis founds unparented roots, so two founding lineages share no ancestor
//! at all and `distance` answers `None` — a real answer rather than a missing
//! one, as `species.rs` says. **For predation the natural reading of "not
//! related" is full appetite**, so `None` is zero remove and a producer or a
//! stranger's line is eaten exactly as before this round. This answers the
//! traits brief's Q1 **for predation only**; what an undefined distance costs
//! an *incorporation* is a different question and stays open.
//!
//! # Hunger reads as one more fork
//!
//! A body inside the tick's own hunger horizon ([`is_hungry`], `energy_mg`
//! under eight ticks of rent) shifts one place further, halving its remove: a
//! starving predator takes a sibling as readily as a fed one takes a cousin.
//!
//! # No size gate, no species wall
//!
//! Ruled 2026-08-29: kinship alone. A predator may still take a body larger
//! than itself, and conspecifics are discounted rather than forbidden.
//!
//! [`is_hungry`]: super::is_hungry

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::body::SpeciesId;
use crate::species::Lineages;

/// The tick's reading of who is related to whom.
///
/// `Lineages::distance` walks two ancestries and allocates; the pairs are few
/// and a tick asks for them thousands of times, so the walk is memoized. Pure
/// memo over an immutable registry, so it changes no answer and no ordering.
pub(super) struct Kin<'a> {
    lineages: &'a Lineages,
    forks: RefCell<BTreeMap<(SpeciesId, SpeciesId), Option<u32>>>,
}

impl<'a> Kin<'a> {
    pub(super) fn new(lineages: &'a Lineages) -> Self {
        Self {
            lineages,
            forks: RefCell::new(BTreeMap::new()),
        }
    }

    /// How much further away `prey` reads to `eater` for being kin, in voxels.
    /// See this module's header for the rule and for what `None` costs.
    pub(super) fn remove(&self, eater: SpeciesId, prey: SpeciesId, span: i32, hungry: bool) -> i32 {
        let Some(forks) = self.forks(eater, prey) else {
            return 0;
        };
        let shift = forks.saturating_add(u32::from(hungry)).min(31);
        span.saturating_add(1) >> shift
    }

    fn forks(&self, eater: SpeciesId, prey: SpeciesId) -> Option<u32> {
        if let Some(known) = self.forks.borrow().get(&(eater, prey)) {
            return *known;
        }
        let measured = self.lineages.distance(eater, prey);
        self.forks.borrow_mut().insert((eater, prey), measured);
        measured
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// root -- a -- a2, and root -- b, beside an unrelated founder.
    fn tree() -> (Lineages, [SpeciesId; 5]) {
        let mut lineages = Lineages::new();
        let root = SpeciesId(1);
        lineages.found(root);
        let stranger = SpeciesId(2);
        lineages.found(stranger);
        let a = lineages.fork(root, "a".into(), 10).unwrap();
        let a2 = lineages.fork(a, "a2".into(), 20).unwrap();
        let b = lineages.fork(root, "b".into(), 30).unwrap();
        (lineages, [root, a, a2, b, stranger])
    }

    #[test]
    fn an_unrelated_line_costs_nothing_and_own_kind_costs_the_whole_reach() {
        // The undefined-distance decision, in the only place it is spent: two
        // founding lineages share no ancestor, and for predation that reads as
        // full appetite. Own kind reads one voxel past the far edge of reach.
        let (lineages, [root, .., stranger]) = tree();
        let kin = Kin::new(&lineages);

        assert_eq!(kin.remove(root, stranger, 7, false), 0);
        assert_eq!(kin.remove(root, root, 7, false), 8);
    }

    #[test]
    fn each_fork_of_divergence_halves_the_remove() {
        // Distant kin more readily than siblings, and it decays to nothing.
        let (lineages, [root, a, a2, b, _]) = tree();
        let kin = Kin::new(&lineages);

        assert_eq!(kin.remove(a, a, 15, false), 16, "self, the whole reach");
        assert_eq!(kin.remove(a, b, 15, false), 8, "siblings, one fork");
        assert_eq!(kin.remove(a2, b, 15, false), 4, "cousins, two forks");
        assert_eq!(kin.remove(a2, root, 15, false), 4);
        assert_eq!(
            kin.remove(a, a, 0, false),
            1,
            "a body with no reach at all still discounts its own line by a voxel"
        );
    }

    #[test]
    fn hunger_reads_as_one_more_fork() {
        // A starving predator takes a sibling as readily as a fed one takes a
        // cousin, which is that sentence written as an assertion.
        let (lineages, [_, a, _, b, _]) = tree();
        let kin = Kin::new(&lineages);

        assert_eq!(kin.remove(a, a, 15, true), kin.remove(a, b, 15, false));
        assert_eq!(kin.remove(a, b, 15, true), kin.remove(a, a, 15, false) / 4);
        assert_eq!(
            kin.remove(a, b, 15, true),
            4,
            "and never to zero for a sibling inside a real reach"
        );
    }

    #[test]
    fn the_memo_changes_no_answer() {
        let (lineages, [root, a, a2, b, stranger]) = tree();
        let kin = Kin::new(&lineages);
        for pair in [(root, a), (a, b), (a2, root), (root, stranger), (a, a)] {
            let first = kin.remove(pair.0, pair.1, 9, false);
            assert_eq!(kin.remove(pair.0, pair.1, 9, false), first);
            assert_eq!(
                first,
                Kin::new(&lineages).remove(pair.0, pair.1, 9, false),
                "a cold cache reads the same"
            );
        }
    }
}
