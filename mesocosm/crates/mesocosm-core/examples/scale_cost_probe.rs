// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S1's cost probe: what a tick costs, and where the cost went.
//!
//! The scale plan (`design_docs/2026-08-29_scale_plan.md`) asks each rung for a
//! measured receipt, and S5 asks for the whole envelope. This is the S1 half of
//! it, built so S5 inherits an instrument rather than a paragraph.
//!
//! [`world::ENCLOSURE`](mesocosm_core::world::ENCLOSURE) is a compile-time
//! constant, so **one run measures one enclosure**. The before/after table is
//! two runs of this same binary with the constant moved between them, which is
//! exactly the before/after trace discipline the plan admits optimizations by.
//!
//! ```text
//! cargo run -p mesocosm-core --release --example scale_cost_probe
//! cargo run -p mesocosm-core --release --example scale_cost_probe -- --out PATH
//! ```
//!
//! # What is attributed, and how
//!
//! - **Percolation sweep** is measured directly: `Soil::percolate` on the
//!   world's own store, off to one side, so the number is that pass and
//!   nothing else.
//! - **The rest of a tick** is `World::apply(Idle)` minus that. It is reported
//!   as a *residue* rather than a guess, and split by founder count: the
//!   `founders = 0` row is the population-independent floor (tier pass, the
//!   density map, the living/carrion reads, breeding, retain), and every row
//!   above it is what a population costs on top.
//! - **Target selection** has no separate timer — it is inside the organism
//!   loop's borrow — so it is attributed by its own work product instead:
//!   `far_members` (from the tick's own [`Tally`], no instrumentation added)
//!   times the living roster is the pair count the unbounded far-tier scan
//!   walks per tick. A rung that grows that product faster than the organism
//!   loop grows is the plan's O(far x N) finding showing up.
//!
//! Every founder count is run at every enclosure, so the same population can be
//! compared across two areas: that is what separates a density effect from a
//! world-size one.

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use mesocosm_core::places::Soil;
use mesocosm_core::world::{ENCLOSURE, PLACE_SALT, PLACE_SIDE};
use mesocosm_core::{Intent, Places, World, snapshot, state_hash};

/// Ticks timed per configuration, after the warmup. Two hundred is enough for
/// a stable mean at every population here and short enough that the widest
/// configuration is still seconds rather than minutes.
const TICK_SAMPLE: u32 = 200;
/// Ticks run before timing starts, so a measurement is of a settled world
/// rather than of genesis' first transients.
const WARMUP: u32 = 20;
/// Repeats for the two cheap-but-noisy standalone measurements.
const REPEATS: u32 = 50;

/// The seed every configuration is founded from. One seed, because this probe
/// measures cost rather than ecology — the instrument is what reads verdicts
/// across seeds.
const SEED: u64 = 1;

/// Founders beyond the played critter, swept. `0` is the population-independent
/// floor; `60` and `916` are the shipping cohorts at +/-16 and +/-64, and `240`
/// sits between so the exponent has a middle point.
const FOUNDER_SWEEP: [u32; 4] = [0, 60, 240, 916];

/// The enclosure S1 replaced. The percolation sweep is a pure function of the
/// store's size, so both extents are measured in **one** run on flat stores —
/// the before/after ratio for the one O(columns) pass in the tick, read off the
/// same machine in the same second rather than compared across two builds.
const REFERENCE_ENCLOSURE: i32 = 16;
/// What genesis puts under every column, mirrored from `world::genesis` so the
/// flat reference stores hold what a founded world holds.
const SOIL_SEED_MG_PER_COLUMN: u64 = 100;

/// `places::TierLine`'s default demotion threshold, copied because the field is
/// set by `Default` rather than exported. A body whose region is this many hops
/// from the focus runs the coarse mind and the unbounded target scan.
const DEMOTE_HOPS: u32 = 2;

/// The unwindowed atlas ceiling, from `modulus::MAX_BRICKS`. Copied rather than
/// imported: `modulus` is the lens's dependency, not the core's, and adding it
/// here to read one number would put a renderer in the simulation crate's tree.
/// S2 is the rung that adopts the windowed atlas; until then this is the wall.
const ATLAS_MAX_BRICKS: usize = 2047;
/// Bytes one 8-cubed brick occupies in that atlas, at one byte per voxel.
const BRICK_BYTES: usize = 512;

struct Measured {
    founders: u32,
    /// Living organisms when timing started and when it ended.
    alive_start: usize,
    alive_end: usize,
    /// Mean far-tier members per timed tick, from the tick's own tally.
    far_members_mean: f64,
    /// Mean `far_members * alive` per timed tick: the pair count the
    /// unbounded far-tier target scan walks.
    far_pairs_mean: f64,
    /// Mean wall cost of one `World::apply(Intent::Idle)`.
    tick_us: f64,
    /// Mean wall cost of one `Soil::percolate` over this world's store.
    percolate_us: f64,
    /// Mean wall cost of one `state_hash`.
    state_hash_us: f64,
    snapshot_bytes: usize,
    soil_bytes: usize,
    bricks: usize,
}

impl Measured {
    /// The tick minus the percolation sweep: everything else the tick did.
    fn rest_us(&self) -> f64 {
        (self.tick_us - self.percolate_us).max(0.0)
    }

    fn percolate_share(&self) -> f64 {
        if self.tick_us <= 0.0 {
            0.0
        } else {
            self.percolate_us / self.tick_us
        }
    }
}

/// What the region tier looks like at this enclosure.
///
/// `PLACE_SIDE` is deliberately unchanged by S1, so a wider world does not get
/// more regions — it gets **bigger** ones, and the tier line is stated in hops
/// rather than voxels. This is what turns that into numbers.
struct PlaceGraph {
    regions: usize,
    /// Voxels across one region.
    region_side: i32,
    /// The longest shortest path in the grown graph, over the probe's seed.
    diameter: u32,
}

fn place_graph() -> PlaceGraph {
    let grown = Places::grown(SEED ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
    let places = grown.places;
    let ids: Vec<_> = places.all().map(|place| place.id).collect();
    let mut diameter = 0;
    for &a in &ids {
        for &b in &ids {
            diameter = diameter.max(places.hops(a, b).unwrap_or(0));
        }
    }
    PlaceGraph {
        regions: ids.len(),
        region_side: (2 * ENCLOSURE + 1) / i32::from(PLACE_SIDE),
        diameter,
    }
}

/// One percolation sweep over a flat store of the given extent.
fn flat_percolate_us(extent: i32) -> f64 {
    let mut store = Soil::seeded(extent, SOIL_SEED_MG_PER_COLUMN);
    store.percolate().expect("finite soil transport");
    let started = Instant::now();
    for _ in 0..REPEATS {
        store.percolate().expect("finite soil transport");
    }
    let elapsed = started.elapsed().as_secs_f64() * 1e6 / f64::from(REPEATS);
    std::hint::black_box(&store);
    elapsed
}

fn measure(founders: u32) -> Measured {
    let mut world = World::new(SEED, founders);
    for _ in 0..WARMUP {
        world.apply(Intent::Idle);
    }

    let alive_start = world.living().count();
    let mut far_members = 0u64;
    let mut far_pairs = 0u128;
    let started = Instant::now();
    for _ in 0..TICK_SAMPLE {
        world.apply(Intent::Idle);
        let tally = world.last_tally();
        let alive = world.living().count() as u64;
        far_members += u64::from(tally.far_members);
        far_pairs += u128::from(tally.far_members) * u128::from(alive);
    }
    let tick_us = started.elapsed().as_secs_f64() * 1e6 / f64::from(TICK_SAMPLE);
    let alive_end = world.living().count();

    // The percolation sweep, timed on a copy of the world's own store — the
    // run's content, not a flat fixture's. Successive calls on one copy rather
    // than a fresh clone each time: cloning 130 KiB is the same order as the
    // sweep itself, and subtracting one from the other put the sweep above the
    // whole tick it lives in.
    let mut store = world.soil().clone();
    let started = Instant::now();
    for _ in 0..REPEATS {
        store.percolate().expect("finite soil transport");
    }
    let percolate_us = started.elapsed().as_secs_f64() * 1e6 / f64::from(REPEATS);
    std::hint::black_box(&store);

    let started = Instant::now();
    for _ in 0..REPEATS {
        std::hint::black_box(state_hash(&world));
    }
    let state_hash_us = started.elapsed().as_secs_f64() * 1e6 / f64::from(REPEATS);

    Measured {
        founders,
        alive_start,
        alive_end,
        far_members_mean: far_members as f64 / f64::from(TICK_SAMPLE),
        far_pairs_mean: far_pairs as f64 / f64::from(TICK_SAMPLE),
        tick_us,
        percolate_us,
        state_hash_us,
        snapshot_bytes: snapshot(&world).map(|bytes| bytes.len()).unwrap_or(0),
        soil_bytes: world.soil().columns() * std::mem::size_of::<u64>(),
        bricks: world.ground().brick_count(),
    }
}

fn main() {
    let mut out: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" => out = args.next().map(PathBuf::from),
            other => eprintln!("ignoring unknown argument: {other}"),
        }
    }
    let path = out.unwrap_or_else(default_path);

    let span = 2 * ENCLOSURE + 1;
    let columns = (span as i64) * (span as i64);
    println!(
        "S1 cost probe: ENCLOSURE = {ENCLOSURE} ({span}-voxel span, {columns} columns), \
         {TICK_SAMPLE} timed ticks after {WARMUP} warmup, seed {SEED}"
    );

    let measured: Vec<Measured> = FOUNDER_SWEEP.iter().map(|&n| measure(n)).collect();

    println!(
        "\n{:>8} {:>7} {:>7} {:>10} {:>10} {:>10} {:>7} {:>11} {:>10} {:>10}",
        "founders",
        "alive0",
        "aliveN",
        "tick_us",
        "perc_us",
        "rest_us",
        "perc%",
        "far_pairs",
        "hash_us",
        "snap_KiB"
    );
    for m in &measured {
        println!(
            "{:>8} {:>7} {:>7} {:>10.1} {:>10.1} {:>10.1} {:>6.1}% {:>11.0} {:>10.1} {:>10.1}",
            m.founders,
            m.alive_start,
            m.alive_end,
            m.tick_us,
            m.percolate_us,
            m.rest_us(),
            m.percolate_share() * 100.0,
            m.far_pairs_mean,
            m.state_hash_us,
            m.snapshot_bytes as f64 / 1024.0,
        );
    }

    let graph = place_graph();
    println!(
        "\nplace graph: {} regions, {} voxels to a region, diameter {} hops; the tier line demotes \
         past {DEMOTE_HOPS}, so a far body can stand {} voxels away",
        graph.regions, graph.region_side, graph.diameter, graph.region_side,
    );

    let flat_here = flat_percolate_us(ENCLOSURE);
    let flat_reference = flat_percolate_us(REFERENCE_ENCLOSURE);
    println!(
        "\npercolation on a flat store: {:.1} us at +/-{ENCLOSURE} against {:.1} us at \
         +/-{REFERENCE_ENCLOSURE} ({:.1}x, over {:.1}x the columns)",
        flat_here,
        flat_reference,
        flat_here / flat_reference.max(f64::MIN_POSITIVE),
        columns as f64 / ((2 * REFERENCE_ENCLOSURE + 1) as f64).powi(2),
    );

    let bricks = measured.last().map(|m| m.bricks).unwrap_or(0);
    println!(
        "\nbricks {bricks} of {ATLAS_MAX_BRICKS} ({:.1}% of the unwindowed atlas, {} KiB of 1 MiB); \
         headroom {} bricks",
        bricks as f64 * 100.0 / ATLAS_MAX_BRICKS as f64,
        bricks * BRICK_BYTES / 1024,
        ATLAS_MAX_BRICKS.saturating_sub(bricks),
    );
    println!(
        "soil {} columns, {} bytes; ticks per wall second budget at 10 t/s: 100_000 us",
        measured.last().map(|m| m.soil_bytes / 8).unwrap_or(0),
        measured.last().map(|m| m.soil_bytes).unwrap_or(0),
    );

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the receipts directory");
    }
    fs::write(
        &path,
        render_json(&measured, bricks, flat_here, flat_reference),
    )
    .expect("writing the cost receipt");
    println!("\nwrote {}", path.display());
}

/// `Code/testing/<repo>/`, found by walking up to the `repos` ancestor rather
/// than counting `../`, the way the population instrument does. The filename
/// carries the enclosure it measured, because one run measures one constant.
fn default_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repos = manifest_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|name| name == "repos"))
        .expect("mesocosm-core is checked out under a `repos/` directory, per Code/CLAUDE.md");
    repos
        .parent()
        .expect("`repos/` has a parent — the `Code` workspace root")
        .join("testing/mesocosm")
        .join(format!("s1_cost_e{ENCLOSURE}.json"))
}

fn render_json(
    measured: &[Measured],
    bricks: usize,
    flat_here: f64,
    flat_reference: f64,
) -> String {
    let span = 2 * ENCLOSURE + 1;
    let columns = (span as i64) * (span as i64);
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"enclosure\": {ENCLOSURE},\n"));
    out.push_str(&format!("  \"span_voxels\": {span},\n"));
    out.push_str(&format!("  \"soil_columns\": {columns},\n"));
    out.push_str(&format!("  \"seed\": {SEED},\n"));
    out.push_str(&format!("  \"timed_ticks\": {TICK_SAMPLE},\n"));
    out.push_str(&format!("  \"warmup_ticks\": {WARMUP},\n"));
    out.push_str(&format!("  \"bricks\": {bricks},\n"));
    out.push_str(&format!("  \"atlas_max_bricks\": {ATLAS_MAX_BRICKS},\n"));
    out.push_str(&format!(
        "  \"atlas_headroom_bricks\": {},\n",
        ATLAS_MAX_BRICKS.saturating_sub(bricks)
    ));
    out.push_str(&format!("  \"brick_bytes\": {},\n", bricks * BRICK_BYTES));
    out.push_str(&format!(
        "  \"flat_percolate_us\": {flat_here:.2},\n  \"flat_percolate_us_at_reference\": \
         {flat_reference:.2},\n  \"reference_enclosure\": {REFERENCE_ENCLOSURE},\n"
    ));
    let graph = place_graph();
    out.push_str(&format!(
        "  \"place_regions\": {},\n  \"place_region_side_voxels\": {},\n  \
         \"place_graph_diameter_hops\": {},\n  \"demote_hops\": {DEMOTE_HOPS},\n",
        graph.regions, graph.region_side, graph.diameter,
    ));
    out.push_str("  \"rows\": [\n");
    for (index, m) in measured.iter().enumerate() {
        out.push_str(&format!(
            "    {{\"founders\": {}, \"alive_start\": {}, \"alive_end\": {}, \
             \"tick_us\": {:.2}, \"percolate_us\": {:.2}, \"rest_us\": {:.2}, \
             \"percolate_share\": {:.4}, \"far_members_mean\": {:.2}, \
             \"far_pairs_mean\": {:.1}, \"state_hash_us\": {:.2}, \
             \"snapshot_bytes\": {}, \"soil_bytes\": {}}}",
            m.founders,
            m.alive_start,
            m.alive_end,
            m.tick_us,
            m.percolate_us,
            m.rest_us(),
            m.percolate_share(),
            m.far_members_mean,
            m.far_pairs_mean,
            m.state_hash_us,
            m.snapshot_bytes,
            m.soil_bytes,
        ));
        if index + 1 < measured.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}
