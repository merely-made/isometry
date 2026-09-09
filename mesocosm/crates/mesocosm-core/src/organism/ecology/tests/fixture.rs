// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The bodies, soils and drivers every ecology test borrows.
//!
//! Split out of `tests/mod.rs` on 2026-09-01 at the six-hundred-line ceiling.
//! Nothing here asserts anything; it is the world these tests are run against.

use super::*;
use crate::body::{SpeciesId, VolumeRef};
use crate::flow::{Ledger, RecordedEvent, Records};
use crate::history::Event;
use crate::organism::Kingdom;

/// The two record streams a world would own, for a fixture that has no world.
///
/// Places are `None`: these tests assert on what happened, not on where, and a
/// fixture that invented a region would be asserting on its own scaffolding.
#[derive(Default)]
pub struct Sink {
    pub events: Vec<RecordedEvent>,
    pub flows: Ledger,
}

impl Sink {
    /// One tick's writing end. Opens the ledger, as `World::apply` does, so a
    /// fixture that steps four thousand times holds one tick of flow rather
    /// than four thousand.
    pub fn stream(&mut self) -> Records<'_> {
        self.flows.open(0);
        Records::new(0, None, &mut self.events, &mut self.flows)
    }

    /// What happened, without the envelopes these fixtures do not read.
    pub fn events(&self) -> Vec<Event> {
        self.events.iter().map(|record| record.record).collect()
    }
}

/// A fixture body big enough to be the mass it is given.
///
/// The half-extent used to be `[1, 1, 1]` — twenty-seven voxels carrying three
/// hundred milligrams, which nothing noticed until TD6 gave a body plan an
/// adult mass derived from its own volume. `[5, 5, 5]` is 1,331 voxels, so
/// these fixtures sit well under their ceiling and can still be watched
/// growing. (2026-08-29 TD6)
pub fn organism(kingdom: Kingdom, mass: u64) -> Organism {
    Organism::founding(
        OrganismId(0),
        SpeciesId(2),
        kingdom,
        VolumeRef::from_tag(16),
        [5, 5, 5],
        [0, 0, 0],
        mass,
    )
}

/// A fixture soil with enough matter that these tests measure the rule they
/// name rather than the enclosure running out. Real worlds seed theirs from
/// `world::genesis`; the conservation invariant is proved there, over a
/// founded world, in `tests/matter.rs`.
pub fn soil() -> Soil {
    Soil::seeded(32, 100_000)
}

pub fn run(organisms: &mut Vec<Organism>, ticks: u32) -> Tally {
    let mut rng = Rng::from_seed(1);
    let mut next = 100;
    let mut total = Tally::default();
    let mut ground = soil();
    let lineages = registry(organisms);
    let mut sink = Sink::default();
    for _ in 0..ticks {
        let t = step(
            organisms,
            &mut next,
            &mut rng,
            &mut sink.stream(),
            &lineages,
            PartPalette::primitive(),
            &mut ground,
        );
        total.matured += t.matured;
        total.born += t.born;
        total.died += t.died;
        total.returned += t.returned;
    }
    total
}

/// Steps until `done`, up to a generous cap. Returns whether it happened.
///
/// Tick counts stopped being stable when upkeep became a function of body
/// mass and a banked budget, so tests state the outcome they are waiting
/// for rather than a number that has to be re-tuned.
pub fn until(world: &mut Vec<Organism>, done: impl Fn(&[Organism]) -> bool) -> bool {
    let mut next_id = 900;
    let mut rng = Rng::from_seed(7);
    let mut ground = soil();
    let lineages = registry(world);
    let mut sink = Sink::default();
    for _ in 0..4_000 {
        if done(world) {
            return true;
        }
        step(
            world,
            &mut next_id,
            &mut rng,
            &mut sink.stream(),
            &lineages,
            PartPalette::primitive(),
            &mut ground,
        );
    }
    done(world)
}

pub fn registry(organisms: &[Organism]) -> Lineages {
    let mut lineages = Lineages::new();
    for organism in organisms {
        lineages.found(organism.species);
    }
    lineages
}
