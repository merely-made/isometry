// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! G4's per-tick sight-line cost at the target critter population.
//!
//! Run in release mode:
//!
//! ```text
//! cargo run -p mesocosm-core --example sight_cost_receipt --release --offline
//! ```
//!
//! `grounded_ecology_receipt` already reports a per-tick figure, but it
//! measures 64 ticks of attrition: the founders begin 300 Near and end
//! near half that, so its number is an average over a shrinking
//! population rather than the cost *at* the target one. For a capacity
//! question that difference is the whole question, so this receipt
//! measures differently:
//!
//! - a short window taken while the population is still at target, so
//!   the headline figure is the cost of 300 bodies seeing and routing;
//! - the same window across a population sweep, so the *shape* of the
//!   cost is visible. Sight is the suspect term: every Near body may
//!   test lines against every candidate, and a quadratic shape at 300
//!   would mean the tier line, not the tick budget, is what holds the
//!   world up.
//!
//! Cost is reported per body as well as per tick, because the per-tick
//! number alone cannot distinguish "more bodies" from "each body got
//! more expensive".
//!
//! What this does **not** isolate: the measured span is a whole
//! `Intent::Idle` tick with sight and routing active, not sight alone.
//! There is no sight-off control to subtract, so the honest claim is
//! the tick's cost at population and the *shape* of its growth. The
//! shape is what implicates sight: a per-body cost that rises with
//! population is a pairwise term, and sight is the only pairwise term
//! in the tick.
//!
//! Timing is here rather than in `mesocosm-core` for the usual reason:
//! clocks measure a host, never deterministic world authority.

use std::time::Instant;

use mesocosm_core::places::Tier;
use mesocosm_core::{Intent, World, state_hash};

const SEED: u64 = 4_242;
/// The G3/G4 target population.
const TARGET: u32 = 300;
/// Short enough that attrition has not meaningfully bitten, long enough
/// to average out scheduler noise.
const MEASURED_TICKS: u32 = 8;
const SAMPLES: u32 = 5;
/// The sweep: quarter, half, and full target, plus a stress point past
/// it. A linear cost per body across these is the answer G4 wants; a
/// rising one names the population where sight stops being free.
const SWEEP: [u32; 4] = [75, 150, 300, 600];

struct Measurement {
    population: u32,
    live_near_at_measure: u64,
    median_us_per_tick: f64,
    us_per_near_body_per_tick: f64,
    hash: u64,
}

fn main() {
    println!(
        "sight-cost receipt: seed={SEED}, measured_ticks={MEASURED_TICKS}, samples={SAMPLES}, target={TARGET}"
    );

    let mut measurements = Vec::new();
    for population in SWEEP {
        measurements.push(measure(population));
    }

    for m in &measurements {
        println!(
            "population={:<4} live_near={:<4} median_us_per_tick={:>9.2} us_per_near_body={:>6.3} state_hash={}",
            m.population,
            m.live_near_at_measure,
            m.median_us_per_tick,
            m.us_per_near_body_per_tick,
            m.hash
        );
    }

    let target = measurements
        .iter()
        .find(|m| m.population == TARGET)
        .expect("the sweep includes the target population");
    println!(
        "at target: a tick with {} Near bodies seeing and routing costs {:.2} us ({:.3} us per body), which is {:.1}% of a 16.7ms frame",
        target.live_near_at_measure,
        target.median_us_per_tick,
        target.us_per_near_body_per_tick,
        target.median_us_per_tick / 16_667.0 * 100.0
    );

    // The shape. Doubling the population doubles the work if the per
    // body cost holds; it quadruples if every body is testing against
    // every other. Report the ratio rather than asserting a verdict,
    // then assert only the thing that would actually be a defect.
    let quarter = &measurements[0];
    let full = &measurements[2];
    let double = &measurements[3];
    let per_body_growth = full.us_per_near_body_per_tick / quarter.us_per_near_body_per_tick;
    let doubling_growth = double.us_per_near_body_per_tick / full.us_per_near_body_per_tick;
    println!(
        "shape: per-body cost grew {per_body_growth:.2}x from {} to {} bodies, and {doubling_growth:.2}x again from {} to {}",
        quarter.population, full.population, full.population, double.population
    );
    println!(
        "reading: a flat per-body cost is linear scaling; a per-body cost that grows with population is the pairwise term, and its slope says where the tier line has to sit"
    );
    println!(
        "boundary: this is a whole Idle tick with sight and routing active, not sight isolated; there is no sight-off control to subtract"
    );

    // The defect this receipt exists to catch: sight going quadratic
    // badly enough that the target population is not affordable. Four
    // times the per-body cost across a four-fold population would be
    // fully pairwise; the bar is deliberately loose because the point
    // is to catch a blow-up, not to freeze a constant.
    assert!(
        per_body_growth < 4.0,
        "per-body sight cost grew {per_body_growth:.2}x over a 4x population: sight is fully pairwise and the tier line cannot hold the target"
    );
    assert!(
        target.live_near_at_measure >= u64::from(TARGET) * 9 / 10,
        "the target measurement ran at {} Near bodies, not the target population: attrition confounded it",
        target.live_near_at_measure
    );
}

fn measure(population: u32) -> Measurement {
    let world = World::new(SEED, population - 1);
    assert_eq!(world.organisms.len(), population as usize);

    // No warmup ticks: warming is what lets attrition start, and the
    // measurement wants the population intact. The first sample pays
    // any cold cost, and taking the median across samples discards it.
    let live_near_at_measure = live_near(&world);

    let mut windows = Vec::with_capacity(SAMPLES as usize);
    let mut hash = None;
    for _ in 0..SAMPLES {
        let mut sample = world.clone();
        let started = Instant::now();
        for _ in 0..MEASURED_TICKS {
            sample.apply(Intent::Idle);
        }
        windows.push(started.elapsed().as_micros());
        let sampled = state_hash(&sample);
        match hash {
            None => hash = Some(sampled),
            Some(expected) => assert_eq!(
                sampled, expected,
                "samples diverged at population {population}: the measurement is not of one world"
            ),
        }
    }

    windows.sort_unstable();
    let median_us_per_tick = windows[windows.len() / 2] as f64 / f64::from(MEASURED_TICKS);
    Measurement {
        population,
        live_near_at_measure,
        median_us_per_tick,
        us_per_near_body_per_tick: median_us_per_tick / live_near_at_measure.max(1) as f64,
        hash: hash.expect("at least one sample"),
    }
}

fn live_near(world: &World) -> u64 {
    world
        .organisms
        .iter()
        .filter(|organism| organism.is_alive() && organism.tier == Tier::Near)
        .count() as u64
}
