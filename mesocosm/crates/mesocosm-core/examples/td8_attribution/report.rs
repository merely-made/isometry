// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What one run of the probe read, and the two ways it is written down.
//!
//! Split out of `main.rs` in TD9 when the fourth and fifth targets took that
//! file past this repo's six-hundred-line ceiling — the same split-before-adding
//! move `ecology/tests` and `ecology/rates` made next door. What stayed there is
//! the run; what moved here is the reading and its two renderings.

use std::path::PathBuf;

const KINGDOMS: [&str; 3] = ["producers", "consumers", "decomposers"];

/// Everything one seeded run says about the three targets.
pub struct Reading {
    pub seed: u64,
    // Ruling 1: recruitment.
    pub born: [u64; 3],
    pub died: [u64; 3],
    /// Deaths split by which of the ecology's two exits took the body: it was
    /// at or under `STARVATION_MG` when it died, or it had simply run out of
    /// lifespan. A kingdom dying of old age is a different failure from a
    /// kingdom dying hungry, and the difference is not in the verdict.
    pub died_starved: [u64; 3],
    pub died_aged: [u64; 3],
    pub alive_end: [u64; 3],
    /// Mean of `biomass_mg * 100 / mass_ceiling_mg` over the living, per
    /// kingdom, sampled at the horizon. TD7 measured 0.32 / 0.23 / 0.00 as
    /// fractions; this is the same number in percent.
    pub adult_pct_end: [u64; 3],
    /// The same reading averaged over every sample, so a kingdom that is
    /// extinct at the horizon still reports how it lived.
    pub adult_pct_mean: [u64; 3],
    /// Living bodies that pass `can_reproduce` at the horizon.
    pub breeding_end: [u64; 3],
    /// Meals taken, and body-ticks lived, per kingdom. The ratio is the **prey
    /// hit rate** TD2c named as the number a grazer's viability turns on: at
    /// `GRAZES_BASE_MG` 3 a fed tick nets roughly double its rent, so a grazer
    /// needs to eat on something like a third to a half of its ticks. This is
    /// what says whether a starving consumer is short of food or short of
    /// reach.
    pub fed_events: [u64; 3],
    /// Milligrams those meals actually delivered. `fed_mg / fed_events` is the
    /// **mouthful**, which is the other half of the viability question: a body
    /// that reaches food on most of its ticks and starves anyway is being paid
    /// too little per bite, not failing to find one.
    pub fed_mg: [u64; 3],
    pub alive_ticks: [u64; 3],
    // Ruling 2: decomposer persistence.
    /// Mean standing carrion count across the samples. TD7 read 12-15.
    pub carrion_mean: u64,
    pub carrion_end: u64,
    pub scavenged_mg: u64,
    /// Ticks since its last meal, averaged over decomposers at death.
    pub decomposer_fast_at_death: u64,
    pub decomposer_deaths: u64,
    // Ruling 3: the free-lunch species.
    /// Consumers and decomposers founded with `actuator_span() == 0`.
    pub unlimbed_founders: u64,
    /// Of those, how many were still alive at the horizon.
    pub unlimbed_alive_end: u64,
    /// `Moved` events emitted by an unlimbed consumer or decomposer. Zero is
    /// what "no actuator, no travel" means in receipt terms.
    pub unlimbed_moves: u64,
    /// Every `Moved` event, by kingdom. A producer is unlimbed by construction,
    /// so this is where a ruling about actuators shows its whole reach.
    pub moves: [u64; 3],
    /// What those bodies ate, and what every other consumer/decomposer ate, so
    /// the free lunch can be seen being withdrawn rather than merely counted.
    pub unlimbed_fed_mg: u64,
    pub limbed_fed_mg: u64,
    /// Total living count at the horizon, so the probe's own window can be
    /// checked against the instrument's.
    pub alive_total_end: u64,
    // Ruling 4 (TD9): the income gap.
    /// Living biomass by kingdom at the horizon. The mouthful and the hit rate
    /// say what a body earns per bite; this says whether the kingdom is
    /// keeping any of it.
    pub biomass_end: [u64; 3],
    /// Alive count and biomass per kingdom at a handful of ticks — the
    /// trajectory, so a kingdom holding a level can be told from one sliding to
    /// zero at a gentler angle. **Sampled densely early on purpose**: TD9's
    /// leave-one-out read consumers gone by tick 600 in one arm and tick 2,400
    /// in another, which is a founding transient rather than a horizon, and a
    /// five-point curve cannot see one.
    pub curve: Vec<(u32, [u64; 3], [u64; 3])>,
    /// Mean age at death, per kingdom, and the mean gestation interval the
    /// dying bodies' own plans asked for. The pair is the recruitment question
    /// in its bluntest form: a kingdom whose bodies do not live as long as one
    /// brood interval cannot recruit at any income.
    pub age_at_death: [u64; 3],
    pub gestation_at_death: [u64; 3],
    /// Milligrams taken **out of** each kingdom. `Event::Fed` names both sides
    /// and every earlier metric here only ever read the eater's, which hides
    /// the thing the death split cannot see: a body eaten from 590 mg down to
    /// 20 mg dies reading `starved`, exactly like one that found nothing.
    pub eaten_mg: [u64; 3],
    /// Of those, what consumers took out of consumers. A limbed consumer is a
    /// `Predator`, and `choose_living_target` lets a predator take any plain
    /// living body — so the founding cohort is its own pasture.
    pub consumer_on_consumer_mg: u64,
    /// Of *those*, what a consumer took out of its own species. The line
    /// between a food web and a lineage eating itself: `choose_living_target`
    /// filters a predator's candidates by signal and reach and by nothing else,
    /// so a conspecific of the same size is an ordinary meal.
    pub cannibal_mg: u64,
    // Ruling 5 (TD9): spread without the free lunch.
    /// Distinct `CELL`-edged buckets holding at least one living body of the
    /// kingdom at the horizon. Occupancy rather than headcount: a stand that
    /// doubles in place has not spread.
    pub cells_end: [u64; 3],
    // Ruling 6 (TD10): the cohort eating itself.
    /// The first tick the consumer kingdom read zero living bodies, if it ever
    /// did. TD9's finding is a *crossing*: extinct by ~300 ticks against a
    /// ~580-tick first brood interval, so no income change could reach it.
    pub consumer_extinct_tick: Option<u32>,
    /// The lowest living consumer count of the run and the tick it happened,
    /// so a kingdom that dips and recovers can be told from one that does not.
    pub consumer_trough: (u32, u64),
    /// Consumers the world founded, so the trough reads as a share.
    pub consumer_founders: u64,
    /// The brood interval each kingdom's *founders* ask for, read at genesis.
    /// TD9 took this off the bodies that died, which a seed where nothing dies
    /// cannot report at all.
    pub founding_brood_ticks: [u64; 3],
    /// Per curve sample: the tick, plain non-consumer bodies inside the tick's
    /// own scan window per thousand living consumers, living consumers with at
    /// least one such body per thousand, and tenths of a voxel to the nearest
    /// one whether or not the window reaches it. A kinship discount can only
    /// prefer what is standing there, so this is what says whether an unmoved
    /// cannibalism number is the rule failing or the pasture being gone — and
    /// the last column says whether the pasture is gone or merely out of sight.
    /// TD11 adds two columns: tenths of a voxel of scan window the mean living
    /// consumer actually carries, and consumers per thousand whose window reads
    /// past the blind floor of eight. A sight rule no anatomy in the world
    /// triggers has to be legible as such.
    pub pool: Vec<Pool>,
}

/// One prey-pool sample: tick, alternatives per thousand consumers, share with
/// any per thousand, tenths of a voxel to the nearest, tenths of a voxel of
/// window, and sighted consumers per thousand.
pub type Pool = (u32, u64, u64, u64, u64, u64);

/// Edge of the occupancy bucket, mirroring the population instrument's own
/// (itself `ecology::rates::CROWD_CELL`, which is crate-private). A receipt
/// reads the rule; it does not get to change it.
pub const CELL: i32 = 8;

pub fn report(r: &Reading) {
    println!("  seed {}", r.seed);
    println!("    1. recruitment (the reproduction gate's target)");
    for (k, name) in KINGDOMS.iter().enumerate() {
        println!(
            "       {name:<12} born {:<6} died {:<6} (starved {:<5} aged {:<5}) alive_end {:<6} breeding_end {:<5} adult% end {:<4} mean {}",
            r.born[k],
            r.died[k],
            r.died_starved[k],
            r.died_aged[k],
            r.alive_end[k],
            r.breeding_end[k],
            r.adult_pct_end[k],
            r.adult_pct_mean[k],
        );
    }
    println!(
        "       prey hit rate (meals per body-tick lived) C {}% D {}%; mouthful C {} mg D {} mg",
        r.fed_events[1] * 100 / r.alive_ticks[1].max(1),
        r.fed_events[2] * 100 / r.alive_ticks[2].max(1),
        r.fed_mg[1] / r.fed_events[1].max(1),
        r.fed_mg[2] / r.fed_events[2].max(1),
    );
    println!("    2. decomposer persistence (the carrion ruling's target)");
    println!(
        "       standing carrion mean {} end {}; scavenged {} mg; decomposer deaths {} after a mean {} ticks unfed",
        r.carrion_mean,
        r.carrion_end,
        r.scavenged_mg,
        r.decomposer_deaths,
        r.decomposer_fast_at_death,
    );
    println!("    3. the free-lunch species (the dispersal floor's target)");
    println!(
        "       unlimbed consumers/decomposers founded {}, alive at horizon {}, Moved events {}, ate {} mg (limbed peers ate {} mg)",
        r.unlimbed_founders,
        r.unlimbed_alive_end,
        r.unlimbed_moves,
        r.unlimbed_fed_mg,
        r.limbed_fed_mg,
    );
    println!(
        "       every Moved event by kingdom P/C/D: {}/{}/{}",
        r.moves[0], r.moves[1], r.moves[2],
    );
    println!("    4. the income gap (the build-scaled bite's target)");
    println!(
        "       end biomass by kingdom P/C/D: {}/{}/{} mg",
        r.biomass_end[0], r.biomass_end[1], r.biomass_end[2],
    );
    for (k, name) in KINGDOMS.iter().enumerate() {
        let curve: Vec<String> = r
            .curve
            .iter()
            .map(|(tick, alive, biomass)| format!("{tick}:{}@{}", alive[k], biomass[k]))
            .collect();
        println!("       {name:<12} tick:alive@mg  {}", curve.join("  "));
    }
    println!(
        "       taken OUT of each kingdom P/C/D: {}/{}/{} mg, of which consumer-on-consumer {} mg",
        r.eaten_mg[0], r.eaten_mg[1], r.eaten_mg[2], r.consumer_on_consumer_mg,
    );
    println!(
        "       of the consumer-on-consumer, same-species (cannibalism) {} mg",
        r.cannibal_mg,
    );
    println!(
        "       mean age at death P/C/D {}/{}/{} ticks, against a brood interval its own plan asks for of {}/{}/{}",
        r.age_at_death[0],
        r.age_at_death[1],
        r.age_at_death[2],
        r.gestation_at_death[0],
        r.gestation_at_death[1],
        r.gestation_at_death[2],
    );
    println!("    5. spread without the free lunch (the creep's target)");
    println!(
        "       occupied cells at horizon P/C/D: {}/{}/{}; producer Moved {}; unlimbed consumer/decomposer Moved {}",
        r.cells_end[0], r.cells_end[1], r.cells_end[2], r.moves[0], r.unlimbed_moves,
    );
    println!("    6. the cohort eating itself (kinship's target)");
    println!(
        "       same-species share of consumer-on-consumer predation: {}% ({} of {} mg)",
        r.cannibal_mg * 100 / r.consumer_on_consumer_mg.max(1),
        r.cannibal_mg,
        r.consumer_on_consumer_mg,
    );
    println!(
        "       consumers founded {}, trough {} at tick {}, first zero {}, against a founding brood interval of {} ticks",
        r.consumer_founders,
        r.consumer_trough.1,
        r.consumer_trough.0,
        r.consumer_extinct_tick
            .map_or("never".to_string(), |tick| tick.to_string()),
        r.founding_brood_ticks[1],
    );
    let pool: Vec<String> = r
        .pool
        .iter()
        .map(|(tick, per_body, share, nearest, window, sighted)| {
            format!(
                "{tick}:{}.{:03}@{}%/{}.{}v w{}.{}/{}%",
                per_body / 1_000,
                per_body % 1_000,
                share / 10,
                nearest / 10,
                nearest % 10,
                window / 10,
                window % 10,
                sighted / 10,
            )
        })
        .collect();
    println!(
        "       prey pool  tick:alternatives@share/voxels-to-nearest w-window/share-sighted  {}",
        pool.join("  "),
    );
    println!("       total alive at horizon {}\n", r.alive_total_end);
}

/// Beside every other round's receipt, found the same way the population
/// instrument finds its own.
pub fn receipt_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repos_ancestor = manifest_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|name| name == "repos"))
        .expect("mesocosm-core is checked out under a `repos/` directory, per Code/CLAUDE.md");
    let workspace_root = repos_ancestor
        .parent()
        .expect("`repos/` has a parent — the `Code` workspace root");
    workspace_root.join("testing/mesocosm/td11_attribution.json")
}

pub fn render_json(
    ticks: u32,
    readings: &[Reading],
    census: &[(u64, u64, u64, u64, u64)],
) -> String {
    let rows: Vec<String> = census
        .iter()
        .map(|(seed, unlimbed, fauna, blind, sight)| {
            format!(
                "    {{\"seed\": {seed}, \"unlimbed\": {unlimbed}, \"consumers_decomposers\": {fauna}, \"blind\": {blind}, \"mean_near_sight_tenths\": {sight}}}"
            )
        })
        .collect();
    let runs: Vec<String> = readings.iter().map(render_run).collect();
    format!(
        "{{\n  \"ticks\": {ticks},\n  \"founding_census\": [\n{}\n  ],\n  \"runs\": [\n{}\n  ]\n}}\n",
        rows.join(",\n"),
        runs.join(",\n")
    )
}

/// One run as a JSON object. Written as a field list rather than a run of
/// `push_str` calls so TD9's five new readings fit under the repo's
/// six-hundred line ceiling without a second probe file.
fn render_run(r: &Reading) -> String {
    let curve: Vec<String> = r
        .curve
        .iter()
        .map(|(tick, alive, biomass)| format!("[{tick}, {alive:?}, {biomass:?}]"))
        .collect();
    let fields: [(&str, String); 34] = [
        ("seed", r.seed.to_string()),
        ("born", format!("{:?}", r.born)),
        ("died", format!("{:?}", r.died)),
        ("died_starved", format!("{:?}", r.died_starved)),
        ("died_aged", format!("{:?}", r.died_aged)),
        ("alive_end", format!("{:?}", r.alive_end)),
        ("breeding_end", format!("{:?}", r.breeding_end)),
        ("fed_events", format!("{:?}", r.fed_events)),
        ("fed_mg", format!("{:?}", r.fed_mg)),
        ("alive_ticks", format!("{:?}", r.alive_ticks)),
        ("adult_pct_end", format!("{:?}", r.adult_pct_end)),
        ("adult_pct_mean", format!("{:?}", r.adult_pct_mean)),
        ("carrion_mean", r.carrion_mean.to_string()),
        ("carrion_end", r.carrion_end.to_string()),
        ("scavenged_mg", r.scavenged_mg.to_string()),
        ("decomposer_deaths", r.decomposer_deaths.to_string()),
        (
            "decomposer_fast_at_death",
            r.decomposer_fast_at_death.to_string(),
        ),
        ("unlimbed_founders", r.unlimbed_founders.to_string()),
        ("unlimbed_alive_end", r.unlimbed_alive_end.to_string()),
        ("unlimbed_moves", r.unlimbed_moves.to_string()),
        ("moves", format!("{:?}", r.moves)),
        ("biomass_end", format!("{:?}", r.biomass_end)),
        ("cells_end", format!("{:?}", r.cells_end)),
        ("age_at_death", format!("{:?}", r.age_at_death)),
        ("gestation_at_death", format!("{:?}", r.gestation_at_death)),
        ("eaten_mg", format!("{:?}", r.eaten_mg)),
        (
            "consumer_on_consumer_mg",
            r.consumer_on_consumer_mg.to_string(),
        ),
        ("cannibal_mg", r.cannibal_mg.to_string()),
        ("consumer_founders", r.consumer_founders.to_string()),
        (
            "consumer_extinct_tick",
            r.consumer_extinct_tick
                .map_or("null".to_string(), |tick| tick.to_string()),
        ),
        (
            "consumer_trough",
            format!("[{}, {}]", r.consumer_trough.0, r.consumer_trough.1),
        ),
        (
            "founding_brood_ticks",
            format!("{:?}", r.founding_brood_ticks),
        ),
        (
            "prey_pool",
            format!(
                "[{}]",
                r.pool
                    .iter()
                    .map(|(tick, per_body, share, nearest, window, sighted)| format!(
                        "[{tick}, {per_body}, {share}, {nearest}, {window}, {sighted}]"
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ),
        ("curve", format!("[{}]", curve.join(", "))),
    ];
    let body: Vec<String> = fields
        .iter()
        .map(|(key, value)| format!("      \"{key}\": {value}"))
        .collect();
    format!("    {{\n{}\n    }}", body.join(",\n"))
}
