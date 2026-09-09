// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Whether a branch off one line can be carried by another. (P3)
//!
//! # A table, not an enum
//!
//! The ProcessDef plan's "directed graft affinity" ruling is explicit that the
//! graph is **world or pack data over tissue domains**, and explicitly *not* the
//! existing `Kingdom` trophic role: soup organisms, colonies, mineral life and
//! magical worlds may admit different domains and different edges. So a
//! [`Domain`] is an opaque number a world assigns to a lineage, an [`Affinity`]
//! is the directed graph a world holds, and neither is a fixed vocabulary the
//! rules read by name.
//!
//! [`Affinity::cycle`] is the default the ruling describes: same domain is
//! ordinarily native, one favoured directed edge leads out of each domain, and
//! the edges that are not favoured are disfavoured. Three domains in a cycle is
//! what a default world uses; the shape generalizes and PE4's generated worlds
//! arrive by admitting a different table rather than by rerunning a generator
//! from a name.
//!
//! # The domains are numbers on purpose
//!
//! The ruling's illustration names them animal-like, fungal-like and
//! plant-like. Those are English for a default world, not game data, and naming
//! them is a naming round rather than an implementation decision — so a domain
//! is an integer here and a panel says what the *verdict* was.
//!
//! # Three verdicts, and what each of them costs
//!
//! [`Verdict`] is the ruling's three: connect directly, require an adapter, or
//! refuse. What they decide is which [`Crossing`] routes are feasible, which is
//! the wing contract's own rule — *an incompatible carry is refused or
//! redirected to regrowth rather than silently rewritten*.

use serde::{Deserialize, Serialize};

/// A tissue domain, as one world numbers them.
///
/// Opaque. Nothing in the rules reads a particular value; the world's
/// [`Affinity`] is the only thing that gives one meaning, and a lineage carries
/// the domain the world assigned it.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Domain(pub u16);

/// What a world says about carrying tissue from one domain into another.
///
/// Directed: the answer for donor-into-recipient is not the answer for
/// recipient-into-donor, which is the whole point of an affinity *graph*.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Verdict {
    /// The boundary connects directly. A carried branch keeps the arrangement
    /// it had.
    Native,
    /// A favoured cross-domain edge. The branch attaches and its cut boundary
    /// does not speak this body's language, so it arrives expressing nothing
    /// until an adapter is grown on it.
    Adapter,
    /// A disfavoured edge. This tissue cannot be carried into this body at all;
    /// regrowing it here is the feasible route.
    Refused,
}

impl Verdict {
    /// The plain word a receipt or a panel uses.
    pub fn name(self) -> &'static str {
        match self {
            Verdict::Native => "native",
            Verdict::Adapter => "needs an adapter",
            Verdict::Refused => "refused",
        }
    }
}

/// Which crossing a graft takes, in the wing contract's own two words.
///
/// The contract separates *carry this body* from *regrow here* for individuals
/// crossing between vessels; a branch crossing between bodies is the same
/// question one scale down, and it has the same two answers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Crossing {
    /// Preserve what the donor had: the branch's topology **and** its
    /// allocation, cell for cell. Feasible only where the affinity permits it.
    Carry,
    /// Preserve identity, topology and provenance, and realize the allocation
    /// under the recipient's own rules. Always feasible, and it does not
    /// promise the donor's arrangement.
    Regrow,
}

impl Crossing {
    pub fn name(self) -> &'static str {
        match self {
            Crossing::Carry => "carried",
            Crossing::Regrow => "regrown",
        }
    }
}

/// One world's directed affinity over its own tissue domains.
///
/// World data, serialized and hashed with everything else, so two worlds that
/// agree about a domain number and disagree about its edges cannot trade a
/// branch: [`Self::digest`] is over the rule-bearing bytes, the same discipline
/// `ConditionId` and `ProcessRef` carry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Affinity {
    /// How many domains this world admits. A domain outside the count is a
    /// domain this world does not hold.
    domains: u16,
    /// The favoured directed edges, sorted and deduplicated. Every other
    /// cross-domain pair is disfavoured; same-domain needs no edge.
    favoured: Vec<(u16, u16)>,
}

/// Domains a default world admits.
///
/// Three, because the ruling's default is a three-domain cycle. Not a balance
/// number and not a vocabulary: a world that admits five arrives by holding a
/// different table.
pub const DEFAULT_DOMAINS: u16 = 3;

impl Default for Affinity {
    fn default() -> Self {
        Self::native()
    }
}

impl Affinity {
    /// The table a world holds when nothing has replaced it.
    pub fn native() -> Self {
        Self::cycle(DEFAULT_DOMAINS)
    }

    /// `n` domains, each favouring the next: `0 -> 1 -> ... -> n-1 -> 0`.
    ///
    /// The ruling's default shape, stated once. A one-domain world has no
    /// cross-domain edge at all and every graft in it is native, which is the
    /// honest reading rather than a special case.
    pub fn cycle(domains: u16) -> Self {
        let favoured = if domains < 2 {
            Vec::new()
        } else {
            (0..domains).map(|d| (d, (d + 1) % domains)).collect()
        };
        Self { domains, favoured }
    }

    /// How many domains this world admits.
    pub fn domains(&self) -> u16 {
        self.domains
    }

    /// Whether this world holds that domain at all.
    pub fn holds(&self, domain: Domain) -> bool {
        domain.0 < self.domains
    }

    /// What this world says about carrying `from` into `into`.
    ///
    /// A domain this world does not hold is refused rather than approximated,
    /// for the reason an unresolvable `ProcessRef` is: substituting the nearest
    /// thing the world does hold would be the compatibility table wearing a lab
    /// coat that the ruling refuses.
    pub fn verdict(&self, from: Domain, into: Domain) -> Verdict {
        if !self.holds(from) || !self.holds(into) {
            return Verdict::Refused;
        }
        if from == into {
            return Verdict::Native;
        }
        if self.favoured.contains(&(from.0, into.0)) {
            return Verdict::Adapter;
        }
        Verdict::Refused
    }

    /// Digest over the rule-bearing bytes of the whole table.
    pub fn digest(&self) -> u64 {
        let mut bytes = self.domains.to_le_bytes().to_vec();
        for (from, into) in &self.favoured {
            bytes.extend_from_slice(&from.to_le_bytes());
            bytes.extend_from_slice(&into.to_le_bytes());
        }
        crate::snapshot::hash_bytes(&bytes)
    }

    /// The domain a world assigns a lineage, drawn from that lineage's own
    /// stream.
    ///
    /// Worldgen's call. Its own salted stream, so assigning domains never
    /// advances the ecology's draws.
    pub fn draw(&self, stream: &mut crate::rng::Rng) -> Domain {
        if self.domains == 0 {
            return Domain(0);
        }
        Domain(stream.below(u64::from(self.domains)) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_domain_is_native_and_the_favoured_edge_needs_an_adapter() {
        let table = Affinity::native();
        assert_eq!(table.verdict(Domain(1), Domain(1)), Verdict::Native);
        assert_eq!(table.verdict(Domain(1), Domain(2)), Verdict::Adapter);
    }

    #[test]
    fn the_reverse_of_a_favoured_edge_is_refused() {
        // Directed, which is the whole point of a graph rather than a distance.
        let table = Affinity::native();
        assert_eq!(table.verdict(Domain(2), Domain(1)), Verdict::Refused);
        assert_eq!(table.verdict(Domain(0), Domain(2)), Verdict::Refused);
    }

    #[test]
    fn a_domain_this_world_does_not_hold_is_refused_not_approximated() {
        let table = Affinity::native();
        assert_eq!(table.verdict(Domain(9), Domain(0)), Verdict::Refused);
        assert_eq!(table.verdict(Domain(0), Domain(9)), Verdict::Refused);
    }

    #[test]
    fn a_one_domain_world_has_no_cross_domain_edge() {
        let table = Affinity::cycle(1);
        assert_eq!(table.verdict(Domain(0), Domain(0)), Verdict::Native);
        assert!(table.favoured.is_empty());
    }

    #[test]
    fn changing_one_edge_changes_the_digest() {
        // The rule this table carries has to be comparable, for the reason a
        // condition's does: two worlds agreeing about a domain number and
        // disagreeing about its edges must not trade a branch.
        let three = Affinity::cycle(3);
        let four = Affinity::cycle(4);
        assert_ne!(three.digest(), four.digest());
        assert_eq!(three.digest(), Affinity::native().digest());
    }

    #[test]
    fn a_drawn_domain_is_one_this_world_holds() {
        let table = Affinity::native();
        let mut stream = crate::rng::Rng::from_seed(7);
        for _ in 0..64 {
            assert!(table.holds(table.draw(&mut stream)));
        }
    }

    #[test]
    fn a_table_round_trips() {
        let table = Affinity::cycle(5);
        let bytes = crate::snapshot::encode(&table).unwrap();
        assert_eq!(crate::snapshot::decode::<Affinity>(&bytes).unwrap(), table);
    }
}
