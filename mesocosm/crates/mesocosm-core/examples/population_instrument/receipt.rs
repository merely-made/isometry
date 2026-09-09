// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where the run's numbers go: the receipt path, the JSON, and the escaping.
//! Split out of `population_instrument.rs` at the 600-line ceiling before DC4
//! added its third batch.

use std::path::PathBuf;

use super::measure::RunResult;
use super::{BOIL_MULTIPLE, COLLAPSE_FRACTION, SAMPLE_INTERVAL, TICKS};
use mesocosm_core::TROPHIC_GRAMMAR_REVISION;

/// Finds `Code/testing/mesocosm/dc4_roster.json` by walking up from
/// this crate to the `repos` ancestor documented in `Code/CLAUDE.md`'s layout
/// section, rather than counting `../` — the crate's depth under `repos/`
/// is not this example's business to hardcode. Each round's receipt keeps its
/// own filename and none is overwritten: `td1_population.json`,
/// `td2_retune.json`, `td2b_walls.json`, `td2c_persistence.json`,
/// `td2d_scavengers.json`, `td5_economy.json`, `td5b_midlife.json`,
/// `td6_matter.json`, `td7_priced.json`, `s1_wide_instrument.json`,
/// `td11_chain.json`, `dc1_palette.json`, `dc15_kingdom.json`,
/// `dc2_browser.json`, and now this. (2026-08-29 TD8; renamed per round)
pub fn receipt_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repos_ancestor = manifest_dir
        .ancestors()
        .find(|p| p.file_name().is_some_and(|name| name == "repos"))
        .expect("mesocosm-core is checked out under a `repos/` directory, per Code/CLAUDE.md");
    let workspace_root = repos_ancestor
        .parent()
        .expect("`repos/` has a parent — the `Code` workspace root");
    workspace_root.join("testing/mesocosm/dc4_roster.json")
}

pub fn explicit_receipt_path(path: &str) -> PathBuf {
    PathBuf::from(path)
}

/// The receipt's shape: one array per batch, in the order they ran.
pub fn render_json(batches: &[(&str, &[RunResult])]) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str(&format!("  \"ticks\": {TICKS},\n"));
    out.push_str(&format!("  \"sample_interval\": {SAMPLE_INTERVAL},\n"));
    out.push_str(&format!("  \"boil_multiple\": {BOIL_MULTIPLE},\n"));
    out.push_str(&format!("  \"collapse_fraction\": {COLLAPSE_FRACTION},\n"));
    out.push_str(&format!(
        "  \"trophic_grammar\": {TROPHIC_GRAMMAR_REVISION},\n"
    ));
    out.push_str("  \"feeding_mode_order\": [\"producer\", \"grazer\", \"predator\", \"omnivore\", \"scavenger\"],\n");
    for (index, (name, runs)) in batches.iter().enumerate() {
        out.push_str(&format!("  \"{name}\": "));
        render_batch(&mut out, runs, 2);
        out.push_str(if index + 1 < batches.len() {
            ",\n"
        } else {
            "\n"
        });
    }
    out.push_str("}\n");
    out
}

fn render_batch(out: &mut String, runs: &[RunResult], indent: usize) {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    out.push_str("[\n");
    for (index, run) in runs.iter().enumerate() {
        out.push_str(&inner);
        render_run(out, run, indent + 2);
        if index + 1 < runs.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&pad);
    out.push(']');
}

fn render_run(out: &mut String, run: &RunResult, indent: usize) {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    out.push_str("{\n");
    out.push_str(&format!("{inner}\"seed\": {},\n", run.seed));
    out.push_str(&format!("{inner}\"founders\": {},\n", run.founders));
    out.push_str(&format!(
        "{inner}\"verdict\": \"{}\",\n",
        run.verdict.label()
    ));
    out.push_str(&format!(
        "{inner}\"reason\": \"{}\",\n",
        json_escape(&run.reason)
    ));
    out.push_str(&format!("{inner}\"decided_tick\": {},\n", run.decided_tick));
    out.push_str(&format!("{inner}\"elapsed_ms\": {},\n", run.elapsed_ms));
    out.push_str(&format!("{inner}\"samples\": [\n"));
    let sample_indent = " ".repeat(indent + 4);
    for (index, sample) in run.samples.iter().enumerate() {
        out.push_str(&sample_indent);
        out.push_str(&format!(
            "{{\"tick\": {}, \"producer\": {}, \"consumer\": {}, \"decomposer\": {}, \"total_biomass_mg\": {}, \"cum_born\": {}, \"cum_died\": {}, \"max_cell\": {}, \"span\": {}, \"outside\": {}, \"soil_mg\": {}, \"total_matter_mg\": {}, \"feeding_modes\": {{\"producer\": {}, \"grazer\": {}, \"predator\": {}, \"omnivore\": {}, \"scavenger\": {}}}}}",
            sample.tick,
            sample.alive[0],
            sample.alive[1],
            sample.alive[2],
            sample.total_biomass_mg,
            sample.cum_born,
            sample.cum_died,
            sample.max_cell,
            sample.span,
            sample.outside,
            sample.soil_mg,
            sample.total_matter_mg,
            sample.feeding_modes[0],
            sample.feeding_modes[1],
            sample.feeding_modes[2],
            sample.feeding_modes[3],
            sample.feeding_modes[4],
        ));
        if index + 1 < run.samples.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&inner);
    out.push_str("]\n");
    out.push_str(&pad);
    out.push('}');
}

/// Minimal JSON string escaping. Every string this instrument writes is
/// built from `format!` over its own labels and numbers, so the only
/// character that plausibly needs escaping is the `"` inside a `reason`
/// message; this covers that plus the standard control cases rather than
/// assuming it never happens.
fn json_escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
