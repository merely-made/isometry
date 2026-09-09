// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-side wall-clock receipt for G3 grounded ecology.
//!
//! Run in release mode:
//!
//! ```text
//! cargo run -p mesocosm-core --example grounded_ecology_receipt --release --offline
//! ```
//!
//! Set `MESOCOSM_ECOLOGY_POPULATION` to compare another starting count. The
//! default is the G3 target of 300 founders.
//!
//! Timing is intentionally here rather than in `mesocosm-core`: clocks
//! measure a host, never deterministic world authority.

use std::time::Instant;

use mesocosm_core::cohort;
use mesocosm_core::places::{Tier, WALKER_HEIGHT};
use mesocosm_core::{Intent, World, state_hash};

const SEED: u64 = 4_242;
const POPULATION: u32 = 300;
const WARMUP_TICKS: u32 = 16;
const MEASURED_TICKS: u32 = 64;
const SAMPLES: u32 = 5;

fn main() {
    let population = std::env::var("MESOCOSM_ECOLOGY_POPULATION")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|count| *count > 0)
        .unwrap_or(POPULATION);
    let world = World::new(SEED, population - 1);
    assert_eq!(world.organisms.len(), population as usize);

    let origin = world.clone();
    let mut warmup = origin.clone();
    for _ in 0..WARMUP_TICKS {
        warmup.apply(Intent::Idle);
    }
    let (start_near, start_far) = living_tiers(&origin);
    let mut elapsed_windows = Vec::with_capacity(SAMPLES as usize);
    let mut result = None;
    let mut expected_hash = None;
    for _ in 0..SAMPLES {
        let mut sample = origin.clone();
        let started = Instant::now();
        for _ in 0..MEASURED_TICKS {
            sample.apply(Intent::Idle);
        }
        elapsed_windows.push(started.elapsed().as_micros());
        let hash = state_hash(&sample);
        if let Some(expected_hash) = expected_hash {
            assert_eq!(hash, expected_hash, "receipt samples diverged");
        } else {
            expected_hash = Some(hash);
        }
        result = Some(sample);
    }
    let world = result.expect("at least one timing sample");

    let mut near = 0u64;
    let mut far = 0u64;
    let mut far_biomass_mg = 0u64;
    let mut far_energy_mg = 0u64;
    for organism in world
        .organisms
        .iter()
        .filter(|organism| organism.is_alive())
    {
        match organism.tier {
            Tier::Near => {
                near += 1;
                assert!(
                    world.ground().stands(organism.position, WALKER_HEIGHT),
                    "near organism {:?} ended without footing at {:?}",
                    organism.id,
                    organism.position
                );
            },
            Tier::Far => {
                far += 1;
                far_biomass_mg += organism.biomass_mg();
                far_energy_mg += organism.energy_mg;
            },
        }
    }

    let cohorts = world.far_cohorts();
    let (cohort_members, cohort_biomass_mg, cohort_energy_mg) = cohort::conserved_totals(&cohorts);
    assert_eq!(
        (cohort_members, cohort_biomass_mg, cohort_energy_mg),
        (far, far_biomass_mg, far_energy_mg),
        "far-cohort projection changed a scalar"
    );
    if population == POPULATION {
        assert!(far > 0, "the receipt never exercised the far tier");
    }
    elapsed_windows.sort_unstable();
    let ticks = u128::from(MEASURED_TICKS);
    let elapsed_total: u128 = elapsed_windows.iter().sum();
    let median = elapsed_windows[elapsed_windows.len() / 2];
    println!(
        "grounded-ecology receipt: seed={SEED}, founders={population}, warmup_ticks={WARMUP_TICKS}, measured_ticks={MEASURED_TICKS}, samples={SAMPLES}"
    );
    println!(
        "median_us_per_tick={:.2}, min_us_per_tick={:.2}, max_us_per_tick={:.2}, mean_us_per_tick={:.2}, start_live_near={start_near}, start_live_far={start_far}, end_live_near={near}, end_live_far={far}, far_cohorts={}, far_members={cohort_members}, far_biomass_mg={cohort_biomass_mg}, far_energy_mg={cohort_energy_mg}",
        median as f64 / ticks as f64,
        elapsed_windows[0] as f64 / ticks as f64,
        elapsed_windows[elapsed_windows.len() - 1] as f64 / ticks as f64,
        elapsed_total as f64 / (ticks * u128::from(SAMPLES)) as f64,
        cohorts.len(),
    );
    println!("sample_elapsed_us={elapsed_windows:?}");
    println!("state_hash={:?}", state_hash(&world));
}

fn living_tiers(world: &World) -> (u64, u64) {
    world
        .organisms
        .iter()
        .filter(|organism| organism.is_alive())
        .fold((0, 0), |(near, far), organism| match organism.tier {
            Tier::Near => (near + 1, far),
            Tier::Far => (near, far + 1),
        })
}
