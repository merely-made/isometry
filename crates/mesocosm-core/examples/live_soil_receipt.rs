// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Serial live-world tick cost after typed soil integration. No rendering.
use mesocosm_core::{Intent, World, matter::Material, snapshot, world::FOUNDERS};
use serde_json::json;
use std::{hint::black_box, time::Instant};

fn main() {
    let mut runs = Vec::new();
    for seed in [1, 4, 7] {
        let mut world = World::new(seed, FOUNDERS);
        let initial = world.total_matter_mg();
        let mut times = Vec::new();
        for tick in 0..220 {
            let start = Instant::now();
            black_box(&mut world).apply(Intent::Idle);
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            if tick >= 20 {
                times.push(ms);
            }
            assert_eq!(world.total_matter_mg(), initial);
            let soil = world.soil().total_stock();
            assert_eq!(soil.total(), u128::from(soil.amount(Material::Untyped)));
        }
        let bytes = snapshot::snapshot(&world).unwrap();
        let restored = snapshot::restore_under(&bytes, world.admitted()).unwrap();
        assert_eq!(
            snapshot::state_hash(&world),
            snapshot::state_hash(&restored)
        );
        times.sort_by(f64::total_cmp);
        runs.push(json!({"seed":seed, "founders_including_played":FOUNDERS+1,
            "living_at_end":world.living().count(), "median_tick_ms":times[100],
            "p95_tick_ms":times[189], "max_tick_ms":times[199],
            "snapshot_bytes":bytes.len(), "state_hash":format!("{:016x}", snapshot::state_hash(&world)),
            "total_mg":initial, "snapshot_roundtrip_exact":true}));
    }
    let output = json!({"scope":"Live typed soil, incumbent untyped body flows; full typed tissue TG2 remains open",
        "optimized":!cfg!(debug_assertions), "grammar_revision":mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        "warmup_ticks":20, "measured_ticks_per_seed":200,
        "timing_scope":"World::apply only; excludes validation, snapshot and rendering", "runs":runs});
    let rendered = serde_json::to_string_pretty(&output).unwrap();
    if let Some(path) = std::env::args().nth(1) {
        std::fs::write(path, &rendered).unwrap();
    }
    println!("{rendered}");
}
