// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What each lineage did, measured at the end of an epoch.
//!
//! [`WorldRecord::note`](crate::record::WorldRecord::note) was built first
//! because its *shape* was the question, and it has had no callers since. This
//! is the caller. It is last rather than first because a reckoning needs all
//! three of the other pieces at once: the log to read what happened, the species
//! tree to have lineages worth telling apart, and places for [`Scale`] to mean
//! anything.
//!
//! # Read, never accumulated
//!
//! Nothing here is a counter the simulation maintains. Every figure is computed
//! from the world and its past when the epoch ends, which is the same discipline
//! capability, temperament, and the possibility space already run on: a stored
//! verdict drifts from the facts it was drawn from, and a derived one cannot.
//!
//! # Predation is the log paying for itself
//!
//! Taking from the living and taking from the dead are the same [`Event::Fed`],
//! and the difference between a predator and a scavenger is **only** answerable
//! because the log preserves order: a meal counts as predation when no `Died`
//! about that creature came before it. A tally kept as the world ran could have
//! recorded this too. Nothing else could have recovered it afterwards, and
//! everything else about the past would have needed its own counter.
//!
//! # Two axes stay empty, deliberately
//!
//! Nothing in the world yet gives to another creature or changes the enclosure
//! itself, so `Symbiosis` and `Construction` are never noted. That is worth
//! more than a zero: `untouched` answers *has anyone ever* rather than *how
//! much*, and a lineage that first does either takes an axis no world has
//! touched. Writing zeroes would fill the record with feats nobody performed and
//! silence the question permanently.

use std::collections::{BTreeMap, BTreeSet};

use crate::body::SpeciesId;
use crate::history::{Event, History};
use crate::organism::OrganismId;
use crate::record::{Feat, Scale};
use crate::world::World;

/// One measurement of one lineage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reading {
    pub species: SpeciesId,
    pub feat: Feat,
    /// How far the lineage that did it reaches.
    pub scale: Scale,
    pub value: i64,
    /// Whether it took the record. False until the reading is noted.
    pub took: bool,
}

/// Everything worth noting about an epoch, in a deterministic order.
///
/// Zero is never a reading. A lineage that ate nothing has not set a predation
/// mark of nothing; it has not preyed, and the record must keep being able to
/// say so.
pub fn readings(world: &World, history: &History) -> Vec<Reading> {
    let mut out = Vec::new();

    // The lineage a creature belonged to, including creatures the roster has
    // long since lost. Speciation moves a founder, so the log has to be read
    // forward rather than sampled.
    let mut lineage_of: BTreeMap<OrganismId, SpeciesId> = BTreeMap::new();
    let mut dead: BTreeSet<OrganismId> = BTreeSet::new();
    let mut predation: BTreeMap<SpeciesId, i64> = BTreeMap::new();

    for event in history.log().entries() {
        match event.record {
            Event::Born {
                organism, species, ..
            } => {
                lineage_of.insert(organism, species);
            },
            Event::Speciated {
                species, founder, ..
            } => {
                lineage_of.insert(founder, species);
            },
            Event::Died { organism, .. } => {
                dead.insert(organism);
            },
            Event::Fed {
                eater,
                from,
                mass_mg,
                kind,
            } => {
                // Taken from something still alive. Eating carrion is how a
                // decomposer lives and is not the same feat.
                if kind == crate::history::MealKind::Predation
                    && !dead.contains(&from)
                    && let Some(species) = lineage_of.get(&eater)
                {
                    *predation.entry(*species).or_default() += mass_mg as i64;
                }
            },
            _ => {},
        }
    }

    // What the living amount to now. Read per lineage in id order, so two runs
    // of the same seed produce the same readings in the same sequence.
    let mut biomass: BTreeMap<SpeciesId, i64> = BTreeMap::new();
    let mut oldest: BTreeMap<SpeciesId, i64> = BTreeMap::new();
    for organism in world.living() {
        *biomass.entry(organism.species).or_default() += organism.biomass_mg() as i64;
        let age = oldest.entry(organism.species).or_default();
        *age = (*age).max(organism.age as i64);
    }

    let mut species: BTreeSet<SpeciesId> = biomass.keys().copied().collect();
    species.extend(predation.keys().copied());
    species.extend(world.ranges().keys().copied());

    for id in species {
        let empty = BTreeSet::new();
        let range = world.ranges().get(&id).unwrap_or(&empty);
        // A feat is scaled by how far the lineage that performed it reaches.
        // Where each individual act happened is finer than the events can say:
        // they carry who and how much, and no place. That is the next thing
        // places want, and it is recorded as such.
        let scale = world.places().scale(range);

        let mut note = |feat: Feat, value: i64| {
            if value > 0 {
                out.push(Reading {
                    species: id,
                    feat,
                    scale,
                    value,
                    took: false,
                });
            }
        };

        note(Feat::Growth, biomass.get(&id).copied().unwrap_or(0));
        note(Feat::Predation, predation.get(&id).copied().unwrap_or(0));
        note(Feat::Spread, range.len() as i64);
        // Age among the living. A creature that lived long and died between two
        // reckonings is not counted, because events carry no tick and its age
        // went with it.
        note(Feat::Endurance, oldest.get(&id).copied().unwrap_or(0));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Intent;

    /// Runs an enclosure for a while, recording everything.
    fn lived(ticks: usize) -> (World, History) {
        let mut world = World::new(4_242, 40);
        let mut history = History::new();
        for _ in 0..ticks {
            world.apply(Intent::Idle);
            history.record_all(world.drain_events());
        }
        (world, history)
    }

    fn of(readings: &[Reading], feat: Feat) -> Vec<Reading> {
        readings
            .iter()
            .copied()
            .filter(|r| r.feat == feat)
            .collect()
    }

    #[test]
    fn an_epoch_of_living_is_worth_measuring() {
        let (world, history) = lived(300);
        let readings = readings(&world, &history);

        assert!(!readings.is_empty(), "something happened worth noting");
        assert!(
            !of(&readings, Feat::Growth).is_empty(),
            "lineages have biomass"
        );
        assert!(
            !of(&readings, Feat::Spread).is_empty(),
            "and they are somewhere"
        );
    }

    #[test]
    fn a_scavenger_is_not_a_predator() {
        // The distinction only the log can make, and the reason predation is
        // read forward rather than counted as it happens.
        let (world, history) = lived(400);
        let readings = readings(&world, &history);

        let meals: i64 = history
            .log()
            .entries()
            .iter()
            .filter_map(|e| match e.record {
                Event::Fed { mass_mg, .. } => Some(mass_mg as i64),
                _ => None,
            })
            .sum();
        let preyed: i64 = of(&readings, Feat::Predation).iter().map(|r| r.value).sum();

        assert!(meals > 0, "the enclosure fed itself");
        assert!(preyed < meals, "some of those meals were already dead");
    }

    #[test]
    fn nobody_has_built_anything_or_helped_anyone() {
        // Two axes with nothing to fill them. Noting zeroes would answer "has
        // anyone ever" with yes, forever, on the first epoch of every world.
        let (world, history) = lived(200);
        let readings = readings(&world, &history);

        assert!(of(&readings, Feat::Symbiosis).is_empty());
        assert!(of(&readings, Feat::Construction).is_empty());
    }

    #[test]
    fn a_reading_of_nothing_is_not_a_reading() {
        let (world, history) = lived(200);
        assert!(readings(&world, &history).iter().all(|r| r.value > 0));
    }

    #[test]
    fn the_same_epoch_measures_the_same() {
        let (a_world, a) = lived(150);
        let (b_world, b) = lived(150);
        assert_eq!(readings(&a_world, &a), readings(&b_world, &b));
    }
}
