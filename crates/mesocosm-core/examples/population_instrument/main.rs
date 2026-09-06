// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! TD1's headless population instrument, carried through TD2, TD2b, TD2c,
//! TD2d, TD5, TD5b, TD6, TD7, S1, TD8 and TD9 as each round's measuring stick.
//!
//! Founds real worlds and drives them with nothing but `Intent::Idle` for
//! several starter lifespans, sampling per-kingdom alive counts, total
//! biomass, and cumulative births/deaths every `SAMPLE_INTERVAL` ticks. A
//! verdict is read off the curve by simple, stated arithmetic: no fitted
//! models, no significance tests. See `design_docs/2026-08-29_terrarium_dynamics_plan.md`, TD1.
//!
//! Four batches run: a **baseline** whose every tier still draws
//! (`Founding::Drawn`, DC1.5's founding), an **archetype arm** founding the
//! consumer tier alone from the browsing hexapod (`Founding::BrowsingConsumer`,
//! DC2), the **roster** founding all eight archetypes (`Founding::Roster`,
//! DC4 — and what ships), and a **control** founded with a single organism and
//! no producer to feed it (expected to collapse). The control is what proves
//! the verdict logic can say something other than the baseline's answer — an
//! instrument that only ever reads one way has not shown anything.
//!
//! **Split at the 600-line ceiling** into `measure` (a run and its verdict) and
//! `receipt` (the JSON) when DC4 added the third batch.
//!
//! A count inside its band is not enough to pass. [`Verdict::Thins`] catches
//! the world that holds its numbers by becoming a producer stand; TD2b's
//! receipt called six such runs "breathes" before this verdict existed.
//!
//! Boil and collapse both exit a run early once decided (see [`run`]) — a
//! fixed 10,000-tick horizon is not practical to *run* to the end on a
//! chain that is actively boiling, since its own per-tick cost grows with
//! it. Only a run that never tips either way pays for the full horizon.
//!
//! ```text
//! cargo run -p mesocosm-core --example population_instrument --release
//! ```
//!
//! Writes `Code/testing/mesocosm/dc4_roster.json` (curves + verdicts
//! per seed) and prints a terminal summary; each earlier round's receipt
//! keeps its own filename and none is overwritten. `Code/testing/<repo>/` is this
//! workspace's standing receipts convention; the path is found by walking up
//! from this crate to the `repos` ancestor rather than a fixed `../` count,
//! so it survives the crate moving depth within the repo.

use std::env;
use std::fs;
use std::thread;

use mesocosm_core::Founding;
use mesocosm_core::world::FOUNDERS;

mod measure;
mod receipt;

use measure::{RunResult, Verdict, run};
use receipt::{explicit_receipt_path, receipt_path, render_json};

/// `lifespan_for_mass(1000mg)` — the starter mass every `World::new` founds
/// its played critter with — is 3,000 ticks after the 2026-08-29 retune
/// (verified against `ecology.rs`'s quarter-power tables), so this many ticks
/// is three and a third starter lifespans. It was ten before the retune
/// stretched the tempo; the horizon stayed put so the two receipts compare.
const TICKS: u32 = 10_000;
/// Sample cadence. 100 samples over the run is enough to see the shape of
/// the curve without writing a receipt per tick.
const SAMPLE_INTERVAL: u32 = 100;
/// Ten arbitrary seeds. Nothing about them is tuned; the point is that the
/// verdict holds across an uncherrypicked handful, not one lucky draw. Widened
/// from five on 2026-08-29 for TD2, because a retune judged on five draws is
/// judged on one bad founder composition either way.
const DEFAULT_SEED_COUNT: u64 = 10;
/// The first five of those seeds, reused for the control so a reader can see
/// the same draws produce opposite verdicts under a different founding.

/// Founders beyond the played critter: the world's own area-scaled cohort,
/// read from `world::FOUNDERS` rather than typed here, so the instrument
/// measures the terrarium that ships instead of a fixture that used to.
/// (2026-08-29 S1; it was a literal 60 through TD7.)
const BASELINE_ORGANISM_COUNT: u32 = FOUNDERS;
/// Zero extra: the played critter (a Consumer) founds alone, with no
/// producer in the world to ever feed it. It can find no prey, so its
/// income is permanently zero and upkeep starves it out. This is the
/// broken control, not a baseline variant.
const CONTROL_ORGANISM_COUNT: u32 = 0;

/// A run reads **boil** when the end population is at least this many times
/// the start population and is still rising on the last sample. The
/// playtest finding this instrument documents (61 -> 8,155, ~134x) clears
/// this bound by more than an order of magnitude, so 10x is a conservative
/// floor for "still exploding" rather than a hair-trigger.
const BOIL_MULTIPLE: u64 = 10;
/// A run reads **collapse** when the end population has both fallen below
/// the start population and is at or below this fraction of it (integer
/// division, no forced floor — see the comment on the check itself for why
/// a one-founder start needs true zero rather than a rounded-up floor).
/// 1/50th is "near enough to zero that stragglers don't matter," not a
/// demand for literal extinction.
const COLLAPSE_FRACTION: u64 = 50;

/// Edge of the occupancy bucket this receipt reports.
///
/// It used to mirror `ecology.rs`'s `CROWD_CELL`, the grid the producer income
/// rule divided by. **TD6 retired crowding** — the soil does that job now, per
/// voxel column and on mass rather than on head count — so this is a plain
/// density statistic kept at 8 so the `max_cell` column still compares against
/// every earlier round's receipt.
const CROWD_CELL: i32 = 8;

struct Config {
    seed_count: u64,
    jobs: usize,
    output: Option<String>,
    arms: Option<Vec<String>>,
}

impl Config {
    fn from_args() -> Self {
        let mut seed_count = DEFAULT_SEED_COUNT;
        let mut jobs = 1usize;
        let mut output = None;
        let mut arms: Option<Vec<String>> = None;
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--seeds" => {
                    seed_count = args
                        .next()
                        .expect("--seeds needs a positive count")
                        .parse()
                        .expect("--seeds must be a positive integer")
                },
                "--output" => output = Some(args.next().expect("--output needs a path")),
                "--jobs" => {
                    jobs = args
                        .next()
                        .expect("--jobs needs a positive count")
                        .parse()
                        .expect("--jobs must be a positive integer")
                },
                "--arms" => {
                    arms = Some(
                        args.next()
                            .expect("--arms needs comma-separated names")
                            .split(',')
                            .map(str::to_ascii_lowercase)
                            .collect(),
                    )
                },
                "--help" | "-h" => {
                    println!(
                        "population_instrument [--seeds N] [--output PATH] [--arms drawn,roster]"
                    );
                    std::process::exit(0);
                },
                other => panic!("unknown argument {other}"),
            }
        }
        assert!(seed_count > 0, "--seeds must be positive");
        assert!(jobs > 0, "--jobs must be positive");
        if let Some(selected) = &arms {
            assert!(
                !selected.is_empty()
                    && selected
                        .iter()
                        .all(|arm| matches!(arm.as_str(), "drawn" | "roster")),
                "--arms must contain one or both of: drawn,roster"
            );
            assert!(
                selected
                    .iter()
                    .enumerate()
                    .all(|(index, arm)| !selected[..index].contains(arm)),
                "--arms entries must be unique"
            );
        }
        Self {
            seed_count,
            jobs,
            output,
            arms,
        }
    }

    fn enabled(&self, name: &str) -> bool {
        self.arms
            .as_ref()
            .map_or(true, |arms| arms.iter().any(|arm| arm == name))
    }
}

fn run_batch(seeds: &[u64], jobs: usize, founders: u32, founding: Founding) -> Vec<RunResult> {
    let mut results = Vec::with_capacity(seeds.len());
    for group in seeds.chunks(jobs) {
        let group_results = thread::scope(|scope| {
            group
                .iter()
                .map(|&seed| scope.spawn(move || run(seed, founders, founding)))
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .expect("population instrument worker panicked")
                })
                .collect::<Vec<_>>()
        });
        for result in &group_results {
            eprintln!(
                "completed seed {}: {} at tick {} ({} ms)",
                result.seed,
                result.verdict.label(),
                result.decided_tick,
                result.elapsed_ms
            );
        }
        results.extend(group_results);
    }
    results
}

fn main() {
    let config = Config::from_args();
    let seeds: Vec<u64> = (1..=config.seed_count).collect();
    println!(
        "TD1 population instrument: {TICKS} ticks, sampling every {SAMPLE_INTERVAL}, jobs {}",
        config.jobs
    );
    println!(
        "verdict arithmetic: collapse when end < start and end <= start/{COLLAPSE_FRACTION}; boil >= {BOIL_MULTIPLE}x start and still rising; thins when the band held but a founded kingdom died out; else breathes\n"
    );

    println!("== baseline (current constants, {BASELINE_ORGANISM_COUNT} extra founders) ==");
    let baseline = if config.enabled("drawn") {
        run_batch(
            &seeds,
            config.jobs,
            BASELINE_ORGANISM_COUNT,
            Founding::Drawn,
        )
    } else {
        Vec::new()
    };
    baseline.iter().for_each(report);

    println!("\n== archetype arm (the consumer tier founds the browsing hexapod) ==");
    let archetype = if config.arms.is_none() {
        run_batch(
            &seeds,
            config.jobs,
            BASELINE_ORGANISM_COUNT,
            Founding::BrowsingConsumer,
        )
    } else {
        Vec::new()
    };
    archetype.iter().for_each(report);

    println!("\n== roster (all eight archetypes, one lineage each) ==");
    let roster = if config.enabled("roster") {
        run_batch(
            &seeds,
            config.jobs,
            BASELINE_ORGANISM_COUNT,
            Founding::Roster,
        )
    } else {
        Vec::new()
    };
    roster.iter().for_each(report);

    println!("\n== roster stand only (producers authored, fauna drawn) ==");
    let stand = if config.arms.is_none() {
        run_batch(
            &seeds,
            config.jobs,
            BASELINE_ORGANISM_COUNT,
            Founding::RosterStand,
        )
    } else {
        Vec::new()
    };
    stand.iter().for_each(report);

    println!("\n== roster fauna only (consumers and decomposers authored, stand drawn) ==");
    let fauna = if config.arms.is_none() {
        run_batch(
            &seeds,
            config.jobs,
            BASELINE_ORGANISM_COUNT,
            Founding::RosterFauna,
        )
    } else {
        Vec::new()
    };
    fauna.iter().for_each(report);

    println!("\n== control (single founder, no producer to feed it) ==");
    let control = if config.arms.is_none() {
        run_batch(
            &seeds[..seeds.len().min(5)],
            config.jobs,
            CONTROL_ORGANISM_COUNT,
            Founding::Drawn,
        )
    } else {
        Vec::new()
    };
    control.iter().for_each(report);

    let tally =
        |runs: &[RunResult], verdict: Verdict| runs.iter().filter(|r| r.verdict == verdict).count();
    let control_all_collapse = control.iter().all(|r| r.verdict == Verdict::Collapse);
    for (label, runs) in [
        ("baseline ", &baseline),
        ("archetype", &archetype),
        ("roster   ", &roster),
        ("stand    ", &stand),
        ("fauna    ", &fauna),
    ] {
        println!(
            "\n{label}: {} breathes, {} thins, {} boil, {} collapse (of {})",
            tally(runs, Verdict::Breathes),
            tally(runs, Verdict::Thins),
            tally(runs, Verdict::Boil),
            tally(runs, Verdict::Collapse),
            runs.len(),
        );
    }
    println!("control all collapse: {control_all_collapse}");
    for (label, runs) in [
        ("baseline ", &baseline),
        ("archetype", &archetype),
        ("roster   ", &roster),
        ("stand    ", &stand),
        ("fauna    ", &fauna),
    ] {
        kingdom_line(label, runs);
    }
    // TD2b's wall proof: any nonzero count here is a body standing where
    // `Ground::grow` never laid terrain.
    let max_outside = baseline
        .iter()
        .flat_map(|r| r.samples.iter())
        .map(|s| s.outside)
        .max()
        .unwrap_or(0);
    println!("max escapees observed across baseline (any sample, any seed): {max_outside}");

    let path = config
        .output
        .as_deref()
        .map(explicit_receipt_path)
        .unwrap_or_else(receipt_path);
    let mut batches = vec![
        ("baseline", baseline.as_slice()),
        ("archetype", archetype.as_slice()),
        ("roster", roster.as_slice()),
        ("roster_stand_only", stand.as_slice()),
        ("roster_fauna_only", fauna.as_slice()),
        ("control", control.as_slice()),
    ];
    batches.retain(|(_, runs)| !runs.is_empty());
    let json = render_json(&batches);
    fs::create_dir_all(path.parent().expect("receipt path has a parent")).expect(
        "Code/testing/mesocosm already exists in this workspace; create_dir_all is just insurance",
    );
    fs::write(&path, json).expect("writing the receipt");
    println!("\nwrote {}", path.display());
}

/// Per-kingdom end states, which is what DC2's finding is read against.
fn kingdom_line(label: &str, runs: &[RunResult]) {
    let mut ends = Vec::new();
    for run in runs {
        let end = run.samples.last().expect("final sample");
        ends.push(format!(
            "{}:{}/{}/{}",
            run.seed, end.alive[0], end.alive[1], end.alive[2]
        ));
    }
    println!("{label} end P/C/D by seed: {}", ends.join("  "));
}

fn report(run: &RunResult) {
    let start = run.samples.first().expect("tick-0 sample");
    let end = run.samples.last().expect("final sample");
    let peak = run
        .samples
        .iter()
        .max_by_key(|s| s.total_alive())
        .expect("at least one sample");
    println!(
        "  seed {:>3}: {:<9} start={:<5} peak={:<5} (tick {:<5}) end={:<5} (tick {:<5}) born={:<5} died={:<5} [{} ms]",
        run.seed,
        run.verdict.label(),
        start.total_alive(),
        peak.total_alive(),
        peak.tick,
        end.total_alive(),
        run.decided_tick,
        end.cum_born,
        end.cum_died,
        run.elapsed_ms,
    );
    // Kingdoms at both ends, because a total that "breathes" while one
    // kingdom is flat at zero is not the trophic balance TD2 is after.
    println!(
        "            kingdoms P/C/D start {}/{}/{} -> end {}/{}/{}; fullest cell {} -> {}; span {} -> {}; outside {} -> {}; end biomass {} mg",
        start.alive[0],
        start.alive[1],
        start.alive[2],
        end.alive[0],
        end.alive[1],
        end.alive[2],
        start.max_cell,
        end.max_cell,
        start.span,
        end.span,
        start.outside,
        end.outside,
        end.total_biomass_mg,
    );
    // The closed cycle's own receipt: soil against bodies, and a total that
    // must not move. (2026-08-29 TD6)
    println!(
        "            matter: soil {} -> {} mg; total {} -> {} mg{}",
        start.soil_mg,
        end.soil_mg,
        start.total_matter_mg,
        end.total_matter_mg,
        if start.total_matter_mg == end.total_matter_mg {
            " (conserved)"
        } else {
            " (NOT CONSERVED)"
        },
    );
    println!(
        "            feeding modes P/Grazer/Predator/Omnivore/Scavenger: {:?} -> {:?}",
        start.feeding_modes, end.feeding_modes
    );
    println!("            {}", run.reason);
}
