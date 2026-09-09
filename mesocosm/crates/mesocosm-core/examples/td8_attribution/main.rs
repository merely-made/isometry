// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The attribution probe: one run, one target per ruling.
//!
//! Written for TD8's three rulings and extended in TD9 for its two, because a
//! second probe measuring the same world from the same events would only be a
//! second thing to keep in step. The file keeps its TD8 name; the receipt is
//! per round, so `td8_attribution.json` stays as TD8 recorded it and this
//! writes `td9_attribution.json`.
//!
//! The population instrument reads a *verdict*; this reads whether each ruling
//! moved the thing it was aimed at. A round that only moves the total has not
//! shown its rulings did anything, so each gets its own numbers:
//!
//! 1. **Reproduction gates on adult mass** was aimed at **recruitment** —
//!    consumers and decomposers reaching a breeding size. Reported as births
//!    and deaths per kingdom, and the mean of `biomass_mg / mass_ceiling_mg`
//!    over the living, which is TD7's finding table re-measured.
//! 2. **Corpses persist longer** was aimed at **decomposer persistence** —
//!    carrion as a standing resource rather than an event. Reported as the mean
//!    standing carrion count, total scavenged milligrams, decomposers alive at
//!    the horizon, and how long a dying decomposer had gone unfed.
//! 3. **No actuator, no travel** was aimed at the **free-lunch species** — a
//!    body with no `Limb` tagma that grazed at a plant's price. Reported as the
//!    founding count of unlimbed consumers and decomposers, the `Moved` events
//!    they emitted, what they ate, and whether they were still standing at the
//!    horizon.
//! 4. **The bite scales with build** (TD9) was aimed at the **specific gap TD8
//!    measured**: consumers clearing TD2c's ~75% prey hit-rate bar and starving
//!    anyway on 5-11 mg mouthfuls. Reported by the numbers that named it — hit
//!    rate, mouthful, the starvation-against-age death split — plus the
//!    consumer biomass trajectory, so a kingdom that is merely dying more
//!    slowly can be told from one that is holding.
//! 5. **Producers creep** (TD9) was aimed at **spread without the free lunch**.
//!    Reported as producer `Moved` events and the count of distinct occupancy
//!    cells each kingdom stands in, against the unlimbed consumers and
//!    decomposers of target 3, who must stay at zero movements.
//! 6. **Kinship tempers the appetite** (TD10) was aimed at the **cohort eating
//!    itself**. Reported as the same-species share of consumer-on-consumer
//!    predation (TD9 read 90-94%), and as the tick the consumer kingdom first
//!    reads zero against the brood interval its founders' own plans ask for —
//!    the crossing TD9 named, extinct by ~300 ticks against ~580. Seed 2 is the
//!    control the world supplied: its consumers are unlimbed `Grazer`s that
//!    never cannibalised, so nothing here may move for it. The **prey pool** is
//!    reported beside it, because a discount can only prefer what is there:
//!    per thousand consumers, how many plain non-consumer bodies stand inside
//!    the window the tick actually scans, and how many consumers have any.
//!
//! ```text
//! cargo run -p mesocosm-core --example td8_attribution --release
//! ```
//!
//! Writes `Code/testing/mesocosm/td11_attribution.json` and prints the same
//! numbers. Seeds and horizon are arguments so a before/after pair is one
//! command each; the defaults are TD7's own probe window (3,000 ticks) over the
//! seeds its finding table quoted plus seed 2, whose consumer species is the
//! free-lunch draw the third ruling names.

use std::collections::BTreeMap;
use std::fs;

use mesocosm_core::{Event, Intent, Kingdom, MealKind, OrganismId, Signal, Stage, World};

/// TD7's probe window, kept so the two rounds' tables compare directly.
const TICKS: u32 = 3_000;
/// Mirrors `ecology::rates::STARVATION_MG`, which is crate-private. The same
/// mirroring the population instrument does for `CROWD_CELL`, and for the same
/// reason: a receipt reads the rule, it does not get to change it.
const STARVATION_MG: u64 = 20;
/// Seeds 1 and 5 are the pair TD7's recruitment table quoted; 2 is the seed
/// whose consumer species draws an unlimbed recipe.
const SEEDS: [u64; 3] = [1, 2, 5];
/// Extra curve samples over the founding transient, on top of the quarters of
/// whatever horizon the run is given.
const CURVE_TICKS: [u32; 6] = [50, 100, 200, 300, 400, 600];
/// Mirrors `ecology::rates::gestation_for_mass`, crate-private like
/// `STARVATION_MG` above: `GESTATION_BASE` 480 quarter-scaled against the
/// reference mass's own fourth root of 3.
fn gestation_for_mass(mass_mg: u64) -> u64 {
    fn root(value: u64) -> u64 {
        (1..).find(|n| n * n > value).unwrap_or(1) - 1
    }
    (480 * root(root(mass_mg.max(1))).max(1) / 3).max(1)
}

fn kingdom_index(kingdom: Kingdom) -> usize {
    match kingdom {
        Kingdom::Producer => 0,
        Kingdom::Consumer => 1,
        Kingdom::Decomposer => 2,
    }
}

mod report;

use report::{CELL, Pool, Reading, receipt_path, render_json, report};

fn main() {
    let mut args = std::env::args().skip(1);
    let ticks: u32 = args.next().and_then(|a| a.parse().ok()).unwrap_or(TICKS);
    let seeds: Vec<u64> = {
        let rest: Vec<u64> = args.filter_map(|a| a.parse().ok()).collect();
        if rest.is_empty() {
            SEEDS.to_vec()
        } else {
            rest
        }
    };

    println!("TD8 attribution probe: {ticks} idle ticks, seeds {seeds:?}\n");
    let census = founding_census();
    println!("  founding census, seeds 1-10 (the free-lunch draw, before anything moves)");
    for (seed, unlimbed, consumers_decomposers, blind, mean_sight) in &census {
        println!(
            "    seed {seed:>2}: of {consumers_decomposers} consumers+decomposers, {unlimbed:>3} have no actuator and {blind:>3} no sense organ; mean near horizon {}.{} voxels",
            mean_sight / 10,
            mean_sight % 10
        );
    }
    println!();

    let readings: Vec<Reading> = seeds.iter().map(|&seed| run(seed, ticks)).collect();
    for reading in &readings {
        report(reading);
    }

    let path = receipt_path();
    fs::create_dir_all(path.parent().expect("receipt path has a parent"))
        .expect("Code/testing/mesocosm already exists; create_dir_all is insurance");
    fs::write(&path, render_json(ticks, &readings, &census)).expect("writing the receipt");
    println!("wrote {}", path.display());
}

/// The free-lunch draw at genesis across the instrument's own ten seeds:
/// consumers and decomposers whose recipe grew no contractile part. Cheap — a
/// `World::new` per seed and no ticks — so it can widen past the seeds the run
/// half of the probe can afford. (TD7 measured 22 of 160 and 20 of 50 at the
/// ±16 founding; S1's pyramid names a species by its tier, so the draw is now
/// all-or-nothing per species.)
///
/// **TD11 adds the sensory draw beside it**, and it is the round's own residue:
/// `sight_range` now reads sense organs a founding cohort almost never has. A
/// sense organ is a *geometry*, not an appendage — `plan::classify` calls any
/// part whose half-extents are all within 1 a `Role::Sensor` — and `axis::seed`
/// draws a body plan per species, so a whole tier is sighted or blind together.
/// Five of these ten seeds contain no sensing part at all. The mean near
/// horizon is reported with it, in tenths of a voxel.
fn founding_census() -> Vec<(u64, u64, u64, u64, u64)> {
    (1..=10)
        .map(|seed| {
            let world = World::new(seed, mesocosm_core::world::FOUNDERS);
            let fauna: Vec<_> = world
                .living()
                .filter(|o| o.kingdom() != Kingdom::Producer)
                .collect();
            let unlimbed = fauna.iter().filter(|o| o.actuator_span() == 0).count() as u64;
            let blind = fauna.iter().filter(|o| o.sensor_span() == 0).count() as u64;
            let sight: u64 = fauna
                .iter()
                .map(|o| {
                    let ceiling = o.mass_ceiling_mg().max(1);
                    80 * (ceiling + u64::from(o.sensor_span()) * 100) / ceiling
                })
                .sum();
            (
                seed,
                unlimbed,
                fauna.len() as u64,
                blind,
                sight / (fauna.len() as u64).max(1),
            )
        })
        .collect()
}

fn run(seed: u64, ticks: u32) -> Reading {
    let mut world = World::new(seed, mesocosm_core::world::FOUNDERS);

    // The brood interval the founding cohort's own plans ask for, per kingdom,
    // read before anything happens. TD9 took this off the bodies that *died*,
    // which cannot be read at all in a seed where nothing does. (TD10)
    let mut brood_total = [0u64; 3];
    let mut brood_count = [0u64; 3];
    for organism in world.living() {
        let k = kingdom_index(organism.kingdom());
        brood_total[k] += gestation_for_mass(organism.life_history_mass_mg);
        brood_count[k] += 1;
    }
    let founding_brood_ticks: [u64; 3] =
        std::array::from_fn(|k| brood_total[k].checked_div(brood_count[k]).unwrap_or(0));
    let consumer_founders = brood_count[1];

    // The founding free-lunch draw, read before anything moves: a consumer or
    // decomposer whose recipe grew no contractile part at all.
    let unlimbed: Vec<OrganismId> = world
        .living()
        .filter(|o| o.kingdom() != Kingdom::Producer && o.actuator_span() == 0)
        .map(|o| o.id)
        .collect();
    let unlimbed_founders = unlimbed.len() as u64;
    // Genesis pushes a `Born` for every founder. Those are the world arriving,
    // not the world recruiting, so they are dropped before the count starts.
    world.drain_events();

    let mut born = [0u64; 3];
    let mut died = [0u64; 3];
    let mut died_starved = [0u64; 3];
    let mut died_aged = [0u64; 3];
    let mut scavenged_mg = 0u64;
    let mut carrion_total = 0u64;
    let mut carrion_samples = 0u64;
    let mut adult_pct_total = [0u64; 3];
    let mut adult_pct_samples = [0u64; 3];
    let mut unlimbed_moves = 0u64;
    let mut moves = [0u64; 3];
    let mut fed_events = [0u64; 3];
    let mut fed_mg = [0u64; 3];
    let mut alive_ticks = [0u64; 3];
    let mut unlimbed_fed_mg = 0u64;
    let mut limbed_fed_mg = 0u64;
    let mut decomposer_fast_total = 0u64;
    let mut decomposer_deaths = 0u64;
    let mut curve: Vec<(u32, [u64; 3], [u64; 3])> = Vec::new();
    let mut eaten_mg = [0u64; 3];
    let mut consumer_on_consumer_mg = 0u64;
    let mut cannibal_mg = 0u64;
    let mut age_total = [0u64; 3];
    let mut gestation_total = [0u64; 3];
    let mut age_samples = [0u64; 3];
    // TD10: the crossing TD9 named. Read every tick rather than off the curve,
    // because a kingdom that empties between two curve samples has still gone
    // extinct at a tick and the whole question is which tick.
    let mut consumer_extinct_tick: Option<u32> = None;
    let mut consumer_trough = (0u32, u64::MAX);
    // Sampled at genesis too: the whole question is whether the discount had
    // anything to prefer, and at tick 0 it does. (TD10)
    let mut pool: Vec<Pool> = vec![prey_pool(&world, 0)];

    // Kingdom by id, so an event about a body that has since been retained out
    // of the world can still be attributed. Filled at genesis and at birth.
    let mut kingdom_of: BTreeMap<u32, usize> = world
        .living()
        .map(|o| (o.id.0, kingdom_index(o.kingdom())))
        .collect();
    let unlimbed_ids: std::collections::BTreeSet<u32> = unlimbed.iter().map(|id| id.0).collect();
    let mut species_of: BTreeMap<u32, u32> =
        world.living().map(|o| (o.id.0, o.species.0)).collect();
    let mut last_meal: BTreeMap<u32, u32> = BTreeMap::new();

    for tick in 1..=ticks {
        world.apply(Intent::Idle);
        // Drained rather than read: `pending` accumulates until someone takes
        // it, so reading it every tick would count every event again.
        let events = world.drain_events();
        for event in &events {
            match &event.record {
                Event::Born { organism, .. } => {
                    // A newborn's kingdom is its parent's species' — read off
                    // the body itself, which is in the world by now.
                    if let Some(o) = world.organisms.iter().find(|o| o.id == *organism) {
                        let k = kingdom_index(o.kingdom());
                        kingdom_of.insert(organism.0, k);
                        species_of.insert(organism.0, o.species.0);
                        born[k] += 1;
                    }
                },
                Event::Died { organism, .. } => {
                    let k = kingdom_of.get(&organism.0).copied().unwrap_or(0);
                    died[k] += 1;
                    // The corpse is still in the roster this tick, carrying the
                    // mass it died holding — the same reading the ecology's own
                    // starvation clause makes.
                    let hungry = world
                        .organisms
                        .iter()
                        .find(|o| o.id == *organism)
                        .is_some_and(|o| o.biomass_mg() <= STARVATION_MG);
                    if hungry {
                        died_starved[k] += 1;
                    } else {
                        died_aged[k] += 1;
                    }
                    // How long the body lived, against how long its own plan
                    // makes it wait between broods.
                    if let Some(o) = world.organisms.iter().find(|o| o.id == *organism) {
                        age_total[k] += u64::from(o.age);
                        gestation_total[k] += gestation_for_mass(o.life_history_mass_mg);
                        age_samples[k] += 1;
                    }
                    if k == 2 {
                        decomposer_deaths += 1;
                        let last = last_meal.get(&organism.0).copied().unwrap_or(0);
                        decomposer_fast_total += u64::from(tick - last);
                    }
                },
                Event::Fed {
                    eater,
                    from,
                    mass_mg,
                    kind,
                } => {
                    last_meal.insert(eater.0, tick);
                    let eater_kingdom = kingdom_of.get(&eater.0).copied().unwrap_or(0);
                    let prey_kingdom = kingdom_of.get(&from.0).copied().unwrap_or(0);
                    fed_events[eater_kingdom] += 1;
                    fed_mg[eater_kingdom] += mass_mg;
                    eaten_mg[prey_kingdom] += mass_mg;
                    if eater_kingdom == 1 && prey_kingdom == 1 {
                        consumer_on_consumer_mg += mass_mg;
                        // Same species, not merely the same kingdom: the
                        // difference between a food web and a body eating its
                        // own lineage.
                        if species_of.get(&eater.0) == species_of.get(&from.0) {
                            cannibal_mg += mass_mg;
                        }
                    }
                    if *kind == MealKind::Scavenging {
                        scavenged_mg += mass_mg;
                    }
                    if kingdom_of.get(&eater.0).copied().unwrap_or(0) != 0 {
                        if unlimbed_ids.contains(&eater.0) {
                            unlimbed_fed_mg += mass_mg;
                        } else {
                            limbed_fed_mg += mass_mg;
                        }
                    }
                },
                Event::Moved { organism, .. } => {
                    if unlimbed_ids.contains(&organism.0) {
                        unlimbed_moves += 1;
                    }
                    moves[kingdom_of.get(&organism.0).copied().unwrap_or(0)] += 1;
                },
                _ => {},
            }
        }

        let mut consumers_now = 0u64;
        for organism in world.living() {
            let k = kingdom_index(organism.kingdom());
            alive_ticks[k] += 1;
            consumers_now += u64::from(k == 1);
        }
        if consumers_now < consumer_trough.1 {
            consumer_trough = (tick, consumers_now);
        }
        if consumers_now == 0 && consumer_extinct_tick.is_none() {
            consumer_extinct_tick = Some(tick);
        }

        // Sampled every hundred ticks, the instrument's own cadence.
        if tick.is_multiple_of(100) {
            carrion_total += world
                .organisms
                .iter()
                .filter(|o| o.stage == Stage::Carrion)
                .count() as u64;
            carrion_samples += 1;
            for (k, (total, samples)) in adult_pct_total
                .iter_mut()
                .zip(adult_pct_samples.iter_mut())
                .enumerate()
            {
                let (sum, count) = adult_share(&world, k);
                if let Some(mean) = sum.checked_div(count) {
                    *total += mean;
                    *samples += 1;
                }
            }
        }
        // Dense over the founding transient, sparse afterwards: the crash the
        // curve has to resolve happens inside the first few hundred ticks.
        if CURVE_TICKS.contains(&tick) || tick.is_multiple_of(ticks.div_ceil(4).max(1)) {
            let (mut alive, mut biomass) = ([0u64; 3], [0u64; 3]);
            for organism in world.living() {
                let k = kingdom_index(organism.kingdom());
                alive[k] += 1;
                biomass[k] += organism.biomass_mg();
            }
            curve.push((tick, alive, biomass));
            pool.push(prey_pool(&world, tick));
        }
    }

    let mut alive_end = [0u64; 3];
    let mut breeding_end = [0u64; 3];
    let mut adult_pct_end = [0u64; 3];
    let mut biomass_end = [0u64; 3];
    let mut cells: [std::collections::BTreeSet<(i32, i32)>; 3] = Default::default();
    for organism in world.living() {
        let k = kingdom_index(organism.kingdom());
        alive_end[k] += 1;
        biomass_end[k] += organism.biomass_mg();
        cells[k].insert((
            organism.position[0].div_euclid(CELL),
            organism.position[2].div_euclid(CELL),
        ));
        if organism.can_reproduce() {
            breeding_end[k] += 1;
        }
    }
    for (k, slot) in adult_pct_end.iter_mut().enumerate() {
        let (sum, count) = adult_share(&world, k);
        *slot = sum.checked_div(count).unwrap_or(0);
    }

    Reading {
        seed,
        born,
        died,
        died_starved,
        died_aged,
        alive_end,
        adult_pct_end,
        adult_pct_mean: std::array::from_fn(|k| {
            adult_pct_total[k]
                .checked_div(adult_pct_samples[k])
                .unwrap_or(0)
        }),
        breeding_end,
        fed_events,
        fed_mg,
        alive_ticks,
        carrion_mean: carrion_total.checked_div(carrion_samples).unwrap_or(0),
        carrion_end: world
            .organisms
            .iter()
            .filter(|o| o.stage == Stage::Carrion)
            .count() as u64,
        scavenged_mg,
        decomposer_fast_at_death: decomposer_fast_total
            .checked_div(decomposer_deaths)
            .unwrap_or(0),
        decomposer_deaths,
        unlimbed_founders,
        unlimbed_alive_end: world
            .living()
            .filter(|o| unlimbed_ids.contains(&o.id.0))
            .count() as u64,
        unlimbed_moves,
        moves,
        unlimbed_fed_mg,
        limbed_fed_mg,
        alive_total_end: alive_end.iter().sum(),
        biomass_end,
        curve,
        age_at_death: std::array::from_fn(|k| {
            age_total[k].checked_div(age_samples[k]).unwrap_or(0)
        }),
        gestation_at_death: std::array::from_fn(|k| {
            gestation_total[k].checked_div(age_samples[k]).unwrap_or(0)
        }),
        eaten_mg,
        consumer_on_consumer_mg,
        cannibal_mg,
        cells_end: std::array::from_fn(|k| cells[k].len() as u64),
        consumer_extinct_tick,
        consumer_trough,
        consumer_founders,
        founding_brood_ticks,
        pool,
    }
}

/// What a living consumer can actually choose among, at one tick: per thousand
/// consumers, how many plain non-consumer bodies stand inside the window the
/// ecology scans, how many consumers have at least one, and how far the nearest
/// one is whether or not the window reaches it.
///
/// **The window is the tick's own, and it is not the bite reach.** A near body
/// scans `sight_range`, which since TD11 is `8 * (ceiling + sensor_span * 100)
/// / ceiling` and nothing else — the body's own sensory build, no longer
/// clamped by the reach it bites at. Mirrored here the way `STARVATION_MG` and
/// `CELL` are: a receipt reads the rule.
fn prey_pool(world: &World, tick: u32) -> Pool {
    let bodies: Vec<_> = world
        .living()
        .map(|o| {
            (
                o.id,
                o.kingdom(),
                o.position,
                o.signal,
                // TD11's sight rule: the flat 8 is now the multiple's own base.
                (8 * (o.mass_ceiling_mg().max(1) + u64::from(o.sensor_span()) * 100)
                    / o.mass_ceiling_mg().max(1)) as i32,
            )
        })
        .collect();
    let (mut consumers, mut alternatives, mut with_one, mut nearest) = (0u64, 0u64, 0u64, 0u64);
    let (mut sight_total, mut sensored) = (0u64, 0u64);
    for (id, kingdom, at, _, window) in &bodies {
        if *kingdom != Kingdom::Consumer {
            continue;
        }
        let window = *window;
        sight_total += window.max(0).unsigned_abs() as u64;
        sensored += u64::from(window > 8);
        consumers += 1;
        let mut seen = 0u64;
        let mut closest = i32::MAX;
        for (other, other_kingdom, other_at, signal, _) in &bodies {
            if other == id || *signal != Signal::Plain || *other_kingdom == Kingdom::Consumer {
                continue;
            }
            let gap = (0..3)
                .map(|a| (at[a] - other_at[a]).abs())
                .max()
                .unwrap_or(0);
            closest = closest.min(gap);
            seen += u64::from(gap <= window);
        }
        alternatives += seen;
        with_one += u64::from(seen > 0);
        // Tenths of a voxel, so the mean survives integer arithmetic. Clamped
        // at the enclosure's own width for a tick with nothing left to find.
        nearest += u64::from(closest.clamp(0, 128).unsigned_abs()) * 10;
    }
    (
        tick,
        alternatives * 1_000 / consumers.max(1),
        with_one * 1_000 / consumers.max(1),
        nearest / consumers.max(1),
        // TD11: the window itself, in tenths of a voxel, and how many consumers
        // read past the blind floor of eight. A sight rule that no anatomy in
        // the world triggers has to be visible as such.
        sight_total * 10 / consumers.max(1),
        sensored * 1_000 / consumers.max(1),
    )
}

/// Sum of `biomass_mg * 100 / mass_ceiling_mg` over one kingdom's living, and
/// how many bodies that was. The ratio TD6 made meaningful and TD7 found sitting
/// at a fifth.
fn adult_share(world: &World, kingdom: usize) -> (u64, u64) {
    let mut sum = 0u64;
    let mut count = 0u64;
    for organism in world.living() {
        if kingdom_index(organism.kingdom()) != kingdom {
            continue;
        }
        sum += organism.biomass_mg() * 100 / organism.mass_ceiling_mg().max(1);
        count += 1;
    }
    (sum, count)
}
