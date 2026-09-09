// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Worlds, and the pressures they put on the things living in them.
//!
//! **Authored, not generated.** Wave 2.2 rules three authored worlds rather
//! than procedural world generation, and the reason is diagnostic: an authored
//! world is legible enough to debug an ecology against, while a generated one
//! moves the question before it is answered. When a lineage does something
//! surprising here, the world is not a suspect.
//!
//! # The grammar begins with a question
//!
//! Taken from Exocosm's method: start with a question — life under high
//! gravity, on a tidally locked world, through a violent year — and *derive*
//! the niches from the pressures that question implies. A decorative biome
//! picked from a list produces decoration; a question produces consequences.
//!
//! So a [`WorldProfile`] records the question it was built from, the parameters
//! that answer it, and the pressures those parameters derive. The derivation is
//! the interesting part and is kept visible rather than folded into a constant.
//!
//! Exocosm is a worldbuilding reference, not a runtime dependency and not a
//! demand for astrophysical realism.
//!
//! # Why this file outlived the module it was written in
//!
//! It was `epoch::worlds` until 2026-09-04, when the `epoch` module was
//! deleted whole — the scalar trait array, its squared-deficit scoring, its
//! standing table and its round, which the real adaptation round in
//! [`crate::world`] had already replaced without borrowing any of them
//! (phenotype plan §D4, ruled by Mark 2026-09-02). This half was kept because
//! nothing in it ever read that array: a [`Pressure`] is a question about a
//! world, a [`WorldProfile`] is an authored answer, and the playable ecology
//! plan's PE4 seeds its world criteria from both.
//!
//! It lives beside [`crate::rules`] rather than inside it. `WorldRules` is
//! settings a particular run is played under, folded into the record's
//! identity so two worlds on different rules are different worlds; these are
//! authored reference data deliberately kept **off** the wire, so a profile
//! can be corrected without invalidating a save. Same neighbourhood, opposite
//! relationship to the record, so they stay separate files.

use serde::{Deserialize, Serialize};

/// What a world does to the things living in it.
///
/// Deliberately few. Each is something a body can be measurably better or worse
/// at, so an adaptation can be scored against it without a simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Pressure {
    /// Weight to carry and falls that hurt.
    Gravity,
    /// Heat that leaves faster than it arrives.
    Cold,
    /// Solvent that is scarce, locked up, or hard to keep.
    Drought,
    /// Too little light to see or to fix energy by.
    Dark,
    /// A medium that attacks what sits in it.
    Corrosive,
    /// Too many neighbours for the room available.
    Crowding,
    /// Something is eating you.
    Predation,
}

impl Pressure {
    pub const ALL: [Pressure; 7] = [
        Pressure::Gravity,
        Pressure::Cold,
        Pressure::Drought,
        Pressure::Dark,
        Pressure::Corrosive,
        Pressure::Crowding,
        Pressure::Predation,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Pressure::Gravity => "gravity",
            Pressure::Cold => "cold",
            Pressure::Drought => "drought",
            Pressure::Dark => "dark",
            Pressure::Corrosive => "corrosive",
            Pressure::Crowding => "crowding",
            Pressure::Predation => "predation",
        }
    }
}

/// One pressure at one strength. Integer, like everything the simulation
/// touches, so a run replays exactly.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Force {
    pub pressure: Pressure,
    /// Strength, nominally 0..=10. Nothing clamps it; a world that wants an
    /// unsurvivable value is allowed to say so.
    pub strength: i32,
}

impl Force {
    pub const fn new(pressure: Pressure, strength: i32) -> Self {
        Self { pressure, strength }
    }
}

/// A world: the question it answers, how it answers it, and what that costs
/// the things living there.
///
/// Borrowed and static rather than owned, because worlds are **authored** —
/// they are written here as consts, not built at runtime. That also keeps them
/// off the wire deliberately: a saved run records which world by name, not a
/// copy of it, so a world can be corrected without invalidating saves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorldProfile {
    pub name: &'static str,
    /// The question this world was built from. Kept because the derivation
    /// below is only checkable against it.
    pub question: &'static str,
    /// The parameters that answer the question, by grammar family. Prose on
    /// purpose: these are authoring notes, and turning them into knobs before
    /// the pressures are proven would invent a configuration system nothing
    /// reads.
    pub parameters: &'static [(&'static str, &'static str)],
    /// What the parameters cost. The part the simulation actually reads.
    pub forces: &'static [Force],
}

impl WorldProfile {
    /// How hard this world pushes on one axis. Absent means no pressure.
    pub fn strength(&self, pressure: Pressure) -> i32 {
        self.forces
            .iter()
            .find(|force| force.pressure == pressure)
            .map(|force| force.strength)
            .unwrap_or(0)
    }

    /// Total pressure. A rough measure of how hard the world is, used to sanity
    /// check that the three authored worlds are hard in different ways rather
    /// than in the same way at different volumes.
    pub fn severity(&self) -> i32 {
        self.forces.iter().map(|force| force.strength).sum()
    }

    /// The pressure this world is *about* — its largest. What a lineage here
    /// must answer sooner or later.
    pub fn defining(&self) -> Option<Pressure> {
        self.forces
            .iter()
            .max_by_key(|force| force.strength)
            .map(|force| force.pressure)
    }
}

/// **Question: what lives where the light never moves?**
///
/// A tidally locked world. One face burns, one face freezes, and everything
/// worth having is crowded into the ring between them. The interesting pressure
/// is not the heat or the cold but the *competition*, because the habitable
/// band is thin and everyone knows it.
pub const TIDAL_SHELF: WorldProfile = WorldProfile {
    name: "the tidal shelf",
    question: "what lives where the light never moves?",
    parameters: &[
        (
            "energy schedule",
            "tidally locked; no day, no year, a fixed terminator",
        ),
        ("medium", "thin air, standing meltwater along the ring"),
        (
            "chemistry",
            "solvent liquid only within the band; ice on one side, vapour on the other",
        ),
        (
            "topology",
            "one continuous habitable ring, dark side and bright side both lethal",
        ),
        (
            "cycles",
            "none — the defining absence, so nothing is seasonal and nothing gets a reprieve",
        ),
        (
            "initial ecology",
            "producers anchored to the light edge, consumers working the shade",
        ),
    ],
    forces: &[
        Force::new(Pressure::Crowding, 8),
        Force::new(Pressure::Dark, 5),
        Force::new(Pressure::Cold, 4),
        Force::new(Pressure::Predation, 3),
    ],
};

/// **Question: what shape is life when the air is heavy enough to swim in?**
///
/// High gravity under a dense, chemically active atmosphere. Structure is
/// expensive, falling is fatal, and the medium eats what sits in it. Buoyancy
/// pays for what legs cannot.
pub const HEAVY_DEEP: WorldProfile = WorldProfile {
    name: "the heavy deep",
    question: "what shape is life when the air is heavy enough to swim in?",
    parameters: &[
        (
            "energy schedule",
            "a dim red sun, most light scattered before it lands",
        ),
        (
            "medium",
            "roughly three gravities; atmosphere dense enough to be buoyant in",
        ),
        (
            "chemistry",
            "reducing atmosphere with acidic aerosols; abundant solvent",
        ),
        (
            "topology",
            "vertical stratification -- everything is a layer, and layers are the niches",
        ),
        (
            "cycles",
            "slow, deep convection storms that move whole layers",
        ),
        (
            "initial ecology",
            "floaters and anchored filterers; nothing walks far",
        ),
    ],
    forces: &[
        Force::new(Pressure::Gravity, 9),
        Force::new(Pressure::Corrosive, 6),
        Force::new(Pressure::Dark, 5),
    ],
};

/// **Question: what survives a year that tries to kill it twice?**
///
/// A violently eccentric orbit. The world is generous for a stretch and lethal
/// for a longer one, so the whole biosphere is shaped around not being alive in
/// the usual sense for most of the cycle.
pub const LONG_YEAR: WorldProfile = WorldProfile {
    name: "the long year",
    question: "what survives a year that tries to kill it twice?",
    parameters: &[
        (
            "energy schedule",
            "eccentric orbit; a short fierce summer and a long deep winter",
        ),
        (
            "medium",
            "thin air, thickening as volatiles boil off each summer",
        ),
        (
            "chemistry",
            "solvent locked as ice for most of the cycle, then abundant, then gone",
        ),
        (
            "topology",
            "basins that hold meltwater and highlands that never thaw",
        ),
        (
            "cycles",
            "the defining feature -- freeze, flood, bloom, desiccation, freeze",
        ),
        (
            "initial ecology",
            "everything reproduces explosively in the bloom and waits out the rest",
        ),
    ],
    forces: &[
        Force::new(Pressure::Cold, 8),
        Force::new(Pressure::Drought, 7),
        Force::new(Pressure::Crowding, 4),
    ],
};

/// The three authored worlds, in no particular order.
pub const AUTHORED: [&WorldProfile; 3] = [&TIDAL_SHELF, &HEAVY_DEEP, &LONG_YEAR];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn every_authored_world_answers_a_question() {
        for world in AUTHORED {
            assert!(
                world.question.ends_with('?'),
                "{} states a question",
                world.name
            );
            assert!(
                world.parameters.len() >= 5,
                "{} answers it across the grammar's families",
                world.name
            );
            assert!(!world.forces.is_empty(), "{} derives pressures", world.name);
        }
    }

    #[test]
    fn the_three_worlds_are_hard_in_different_ways() {
        // The point of authoring three rather than one. If they all pushed on
        // the same axis, the lab would test one world at three volumes and the
        // adaptation phase would look more general than it is.
        let defining: BTreeSet<_> = AUTHORED.iter().filter_map(|w| w.defining()).collect();
        assert_eq!(
            defining.len(),
            3,
            "each world is about a different pressure"
        );
    }

    #[test]
    fn no_world_pushes_on_everything() {
        // A world with every pressure at once has no niches, only a difficulty
        // slider. Each of these leaves room somewhere.
        for world in AUTHORED {
            assert!(
                world.forces.len() < Pressure::ALL.len(),
                "{} leaves some axis alone",
                world.name
            );
        }
    }

    #[test]
    fn absent_pressures_read_as_zero() {
        assert_eq!(HEAVY_DEEP.strength(Pressure::Gravity), 9);
        assert_eq!(HEAVY_DEEP.strength(Pressure::Predation), 0);
    }

    #[test]
    fn severity_is_comparable_without_being_identical() {
        // They should be hard, and not equally hard -- a lab where every world
        // scores the same total is measuring nothing.
        let severities: Vec<i32> = AUTHORED.iter().map(|w| w.severity()).collect();
        assert!(
            severities.iter().all(|s| *s > 10),
            "all three are demanding"
        );
        assert!(
            severities.iter().collect::<BTreeSet<_>>().len() > 1,
            "and not all demanding to the same degree"
        );
    }
}
