// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a run is and how it is judged: one world driven to the horizon, the
//! samples taken off it, and the verdict arithmetic. Split out of
//! `population_instrument.rs` at the 600-line ceiling before DC4 added its
//! third batch.

use std::collections::BTreeMap;
use std::time::Instant;

use mesocosm_core::process::FeedingMode;
use mesocosm_core::world::ENCLOSURE;
use mesocosm_core::{Founding, Intent, Kingdom, World};

use super::{BOIL_MULTIPLE, COLLAPSE_FRACTION, CROWD_CELL, SAMPLE_INTERVAL, TICKS};

#[derive(Clone, Copy)]
pub struct Sample {
    pub tick: u32,
    pub alive: [u32; 3],
    pub total_biomass_mg: u64,
    pub cum_born: u64,
    pub cum_died: u64,
    /// Occupancy of the fullest crowding cell. This is the number the
    /// producer income rule divides by, so it says directly whether
    /// self-thinning is even being asked to do anything.
    pub max_cell: u32,
    /// Furthest occupied position from the enclosure's centre, on either
    /// horizontal axis. Anything past `world::ENCLOSURE` is a body standing
    /// where the ground the enclosure grew does not reach.
    pub span: i32,
    /// Count of living organisms strictly past `ENCLOSURE` on either
    /// horizontal axis: the escapee proof for TD2b's wall. Zero across a run
    /// is what "the wall holds" means in receipt terms. (2026-08-29 TD2b)
    pub outside: u32,
    /// Matter held in the ground. (2026-08-29 TD6)
    pub soil_mg: u64,
    /// Soil plus every body's substance and reserve — the conserved total.
    /// Flat across a whole run is the closed cycle's own receipt.
    pub total_matter_mg: u64,
    /// Living feeding-mode readings at this sample, in the stable order
    /// producer/grazer/predator/omnivore/scavenger. The omnivore slot is
    /// retained by name while the core API grows into TG1.
    pub feeding_modes: [u32; 5],
}

impl Sample {
    pub fn total_alive(&self) -> u64 {
        self.alive.iter().map(|&count| u64::from(count)).sum()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    Collapse,
    Boil,
    /// The count held its band, but a kingdom the world was founded with is
    /// gone at the horizon. A pure producer stand is not a terrarium
    /// breathing, and before this verdict existed it read as one — TD2b's
    /// receipt called six such runs "breathes". (2026-08-29 TD2c)
    Thins,
    Breathes,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Collapse => "collapse",
            Verdict::Boil => "boil",
            Verdict::Thins => "thins",
            Verdict::Breathes => "breathes",
        }
    }
}

pub const KINGDOM_NAMES: [&str; 3] = ["producers", "consumers", "decomposers"];

pub struct RunResult {
    pub seed: u64,
    pub founders: u32,
    pub samples: Vec<Sample>,
    pub verdict: Verdict,
    pub reason: String,
    /// The tick the verdict was read at. Equal to `TICKS` for a run that had
    /// to watch the full horizon (breathes); earlier for a run that decided
    /// itself before that, per the early-exit note on [`run`].
    pub decided_tick: u32,
    pub elapsed_ms: u128,
}

pub fn kingdom_index(kingdom: Kingdom) -> usize {
    match kingdom {
        Kingdom::Producer => 0,
        Kingdom::Consumer => 1,
        Kingdom::Decomposer => 2,
    }
}

pub fn sample_of(world: &World, tick: u32, cum_born: u64, cum_died: u64) -> Sample {
    let mut alive = [0u32; 3];
    let mut total_biomass_mg = 0u64;
    let mut density: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    let mut span = 0i32;
    let mut outside = 0u32;
    let mut feeding_modes = [0u32; 5];
    for organism in world.organisms.iter().filter(|o| o.is_alive()) {
        alive[kingdom_index(organism.kingdom())] += 1;
        let mode_index = match organism.feeding_mode() {
            FeedingMode::Producer => 0,
            FeedingMode::Grazer => 1,
            FeedingMode::Predator => 2,
            FeedingMode::Omnivore => 3,
            FeedingMode::Scavenger => 4,
        };
        feeding_modes[mode_index] += 1;
        total_biomass_mg += organism.biomass_mg();
        *density
            .entry((
                organism.position[0].div_euclid(CROWD_CELL),
                organism.position[2].div_euclid(CROWD_CELL),
            ))
            .or_default() += 1;
        span = span
            .max(organism.position[0].abs())
            .max(organism.position[2].abs());
        if organism.position[0].abs() > ENCLOSURE || organism.position[2].abs() > ENCLOSURE {
            outside += 1;
        }
    }
    Sample {
        tick,
        alive,
        total_biomass_mg,
        cum_born,
        cum_died,
        max_cell: density.into_values().max().unwrap_or(0),
        span,
        outside,
        soil_mg: world.soil().total_mg(),
        total_matter_mg: world.total_matter_mg(),
        feeding_modes,
    }
}

/// Runs one seeded world for up to `TICKS` idle ticks, sampling every
/// `SAMPLE_INTERVAL`, and reads a verdict off the resulting curve.
///
/// **Boil and collapse both exit early**, the moment [`verdict_for`] can
/// already read one off the samples taken so far. Neither verdict changes
/// its mind later: a boiling chain's per-tick cost only grows as its
/// population does (this is what made a fixed 10,000-tick horizon
/// impractical to *run* at today's constants — the instrument itself boiled
/// alongside the world), and a collapsed chain has no organisms left to
/// recover with. **Breathes is the one verdict that cannot be read early**:
/// no sample can prove a population stays bounded for ticks it has not
/// reached yet, so that path always watches the full horizon.
pub fn run(seed: u64, organism_count: u32, founding: Founding) -> RunResult {
    let started = Instant::now();
    let mut world =
        World::founded(seed, organism_count, founding).expect("the founding palette is admissible");
    let founders = world.organisms.len() as u32;

    let mut cum_born = 0u64;
    let mut cum_died = 0u64;
    let mut samples = vec![sample_of(&world, 0, 0, 0)];
    let mut decision: Option<(Verdict, String, u32)> = None;

    for tick in 1..=TICKS {
        world.apply(Intent::Idle);
        let tally = world.last_tally();
        cum_born += u64::from(tally.born);
        cum_died += u64::from(tally.died);
        if !tick.is_multiple_of(SAMPLE_INTERVAL) {
            continue;
        }
        let sample = sample_of(&world, tick, cum_born, cum_died);
        eprintln!(
            "    seed {seed}: tick {tick:>6} alive={:<7} elapsed_ms={}",
            sample.total_alive(),
            started.elapsed().as_millis()
        );
        samples.push(sample);

        let (verdict, reason) = verdict_for(&samples);
        if matches!(verdict, Verdict::Boil | Verdict::Collapse) {
            decision = Some((verdict, reason, tick));
            break;
        }
    }

    let (verdict, reason, decided_tick) = match decision {
        Some(decided) => decided,
        None => {
            // Ran the full horizon without an early boil or collapse.
            // Always land the final tick, even if TICKS isn't a multiple of
            // the sample interval (it is, today, but the receipt shouldn't
            // silently drop the end state if that ever changes).
            if !TICKS.is_multiple_of(SAMPLE_INTERVAL) {
                samples.push(sample_of(&world, TICKS, cum_born, cum_died));
            }
            let (verdict, reason) = verdict_for(&samples);
            (verdict, reason, TICKS)
        },
    };

    RunResult {
        seed,
        founders,
        samples,
        verdict,
        reason,
        decided_tick,
        elapsed_ms: started.elapsed().as_millis(),
    }
}

/// Reads collapse / boil / breathes off a sample curve by the arithmetic
/// documented on [`BOIL_MULTIPLE`] and [`COLLAPSE_FRACTION`]. Collapse is
/// checked first: a population that is both near-zero and still nominally
/// "10x start" (start of 1, end of 0) must not be misread as neither.
pub fn verdict_for(samples: &[Sample]) -> (Verdict, String) {
    let start = samples.first().expect("at least the tick-0 sample");
    let end = samples.last().expect("at least the tick-0 sample");
    let start_total = start.total_alive();
    let end_total = end.total_alive();

    // No forced minimum of 1 here: a one-founder start (the collapse
    // control) has a floor of 1/50 = 0, which is deliberate. A floor
    // clamped up to the start count itself would read "collapse" from the
    // very first sample regardless of what actually happened, since the
    // population never has to move to satisfy `end <= start`. Requiring
    // `end_total < start_total` on top of the floor is what turns this back
    // into "it fell," not "it started small."
    let collapse_floor = start_total / COLLAPSE_FRACTION;
    if end_total <= collapse_floor && end_total < start_total {
        return (
            Verdict::Collapse,
            format!(
                "end population {end_total} <= collapse floor {collapse_floor} (start {start_total} / {COLLAPSE_FRACTION}) and below start {start_total}"
            ),
        );
    }

    let previous_total = samples
        .get(samples.len().saturating_sub(2))
        .map(Sample::total_alive)
        .unwrap_or(end_total);
    let boil_bound = start_total.saturating_mul(BOIL_MULTIPLE);
    if end_total >= boil_bound && end_total > previous_total {
        return (
            Verdict::Boil,
            format!(
                "end population {end_total} >= {BOIL_MULTIPLE}x start {start_total} ({boil_bound}) and still rising (previous sample {previous_total})"
            ),
        );
    }

    // A count inside its band is necessary but not sufficient. A world that
    // founded three kingdoms and ends holding one is a producer stand, not a
    // food web, so it must not be able to spend the word "breathes". Only
    // kingdoms actually present at tick 0 are required to survive: the
    // collapse control founds a lone consumer and would otherwise be failed
    // for missing producers it never had.
    let lost: Vec<&str> = (0..3)
        .filter(|&k| start.alive[k] > 0 && end.alive[k] == 0)
        .map(|k| KINGDOM_NAMES[k])
        .collect();
    if !lost.is_empty() {
        return (
            Verdict::Thins,
            format!(
                "end population {end_total} held its band, but the world founded {}/{}/{} P/C/D and ends {}/{}/{}: {} died out",
                start.alive[0],
                start.alive[1],
                start.alive[2],
                end.alive[0],
                end.alive[1],
                end.alive[2],
                lost.join(" and "),
            ),
        );
    }

    (
        Verdict::Breathes,
        format!(
            "end population {end_total} stayed above the collapse floor ({collapse_floor}) and below or falling from the boil bound ({boil_bound}), all {} founded kingdoms still alive",
            start.alive.iter().filter(|&&n| n > 0).count(),
        ),
    )
}
