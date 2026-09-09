// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{development::PartPalette, organism::Kingdom};

/// Where a founding tier's bodies come from.
///
/// **A per-tier set, since DC4.** It began as DC2's isolable arm — one tier
/// authored so the instrument could read that tier's cost alone — and the
/// roster made the natural shape a *list of bodies per tier* rather than one
/// body per tier, because how many lineages a tier founds is now part of the
/// answer. Which palette a world admits follows from the choice, because an
/// archetype's shapes are world state and its arrangement is the lineage's.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Founding {
    /// Every tier draws one recipe from [`axis::seed`](crate::axis::seed), the
    /// worldgen lottery DC1.5 left in place. Three non-played lineages, which
    /// is what the world founded through DC1.5.
    Drawn,
    /// The consumer tier founds from
    /// [`archetype::consumer_browser`](crate::axis::archetype::consumer_browser);
    /// producers and decomposers still draw. **DC2's arm**, kept so its
    /// measurement stays reproducible against the roster's.
    BrowsingConsumer,
    /// Only the producer tier founds authored bodies.
    ///
    /// **A DC4 diagnostic.** The roster moves the stand and the mouths at
    /// once, so a verdict on it cannot say which half did the moving. These
    /// two variants split it, and they exist for the instrument rather than
    /// for a world to ship.
    RosterStand,
    /// Only the consumer and decomposer tiers do. The other half of
    /// [`Founding::RosterStand`].
    RosterFauna,
    /// The full roster: one lineage per archetype, three producers, three
    /// consumers, two decomposers, and nothing drawn. **This is how the
    /// enclosure ships** (DC4) — `axis::seed` stays in the tree as the
    /// generator a soup world would still use.
    #[default]
    Roster,
    /// Explicit branching recipes; historical `Roster` remains reproducible.
    BranchingRoster,
    /// Paid appendage chains and separated leaves; previous sets stay fixed.
    JointedRoster,
    /// Spaced appendage chains, preserving the last jointed recording set.
    SpacedRoster,
}

impl Founding {
    /// The vocabulary a world founded this way has to admit. The archetype
    /// palette only fills spare slots, so the two differ in what they *can*
    /// express and not in what a drawn recipe develops.
    pub fn palette(self) -> PartPalette {
        match self {
            Self::Drawn => PartPalette::primitive(),
            Self::JointedRoster => crate::axis::archetype::jointed::palette(),
            Self::SpacedRoster => crate::axis::archetype::spaced::palette(),
            _ => crate::axis::archetype::palette(),
        }
    }

    /// The authored bodies this founding installs for a tier, one lineage
    /// each, in founding order. Empty means the tier still draws.
    pub(super) fn tier(self, kingdom: Kingdom) -> &'static [fn() -> crate::axis::Recipe] {
        use crate::axis::archetype;
        match (self, kingdom) {
            (Self::JointedRoster, Kingdom::Producer) => &archetype::jointed::PRODUCERS,
            (Self::JointedRoster, Kingdom::Consumer) => &archetype::jointed::CONSUMERS,
            (Self::JointedRoster, Kingdom::Decomposer) => &archetype::DECOMPOSERS,
            (Self::SpacedRoster, Kingdom::Producer) => &archetype::spaced::PRODUCERS,
            (Self::SpacedRoster, Kingdom::Consumer) => &archetype::spaced::CONSUMERS,
            (Self::SpacedRoster, Kingdom::Decomposer) => &archetype::DECOMPOSERS,
            (Self::BranchingRoster, Kingdom::Producer) => &archetype::branching::PRODUCERS,
            (Self::BranchingRoster, Kingdom::Consumer) => &archetype::branching::CONSUMERS,
            (Self::BranchingRoster, Kingdom::Decomposer) => &archetype::DECOMPOSERS,
            (Self::BrowsingConsumer, Kingdom::Consumer) => &archetype::CONSUMERS[..1],
            (Self::RosterStand, Kingdom::Producer) => &archetype::PRODUCERS,
            (Self::RosterFauna, Kingdom::Consumer) => &archetype::CONSUMERS,
            (Self::RosterFauna, Kingdom::Decomposer) => &archetype::DECOMPOSERS,
            (Self::Roster, Kingdom::Producer) => &archetype::PRODUCERS,
            (Self::Roster, Kingdom::Consumer) => &archetype::CONSUMERS,
            (Self::Roster, Kingdom::Decomposer) => &archetype::DECOMPOSERS,
            _ => &[],
        }
    }

    /// How many non-played lineages this tier founds. A drawn tier is one
    /// interbreeding species, which is the structural fact TD10 found and the
    /// roster exists to change.
    pub(super) fn lineages(self, kingdom: Kingdom) -> usize {
        self.tier(kingdom).len().max(1)
    }
}
