// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_lens::BrickDiagnostics;
use netrender::profiling::FrameTimings;

use crate::burrow_scenario::SEED;
use crate::scenario::Scenario;

#[derive(Debug, serde::Serialize)]
pub struct Receipt {
    gate: &'static str,
    target: &'static str,
    profile: &'static str,
    frame: u32,
    size: [u32; 2],
    surface_format: String,
    adapter: AdapterReceipt,
    scenario: ScenarioReceipt,
    tracer: TracerReceipt,
    netrender: NetrenderReceipt,
}

#[derive(Debug, serde::Serialize)]
struct AdapterReceipt {
    name: String,
    backend: String,
    max_texture_dimension_2d: u32,
    max_bind_groups: u32,
}

#[derive(Debug, serde::Serialize)]
struct ScenarioReceipt {
    seed: u64,
    generated_route: Vec<[i32; 3]>,
    threshold_step: usize,
    place_boundary_step: usize,
    from_place: u16,
    to_place: u16,
    player_start: [i32; 3],
    player_after: [i32; 3],
    hunter_start: [i32; 3],
    hunter_after: [i32; 3],
    ordered_intents: usize,
    completed_steps: usize,
    all_outcomes_moved: bool,
    hunter_pursued: bool,
    hunter_crossed_threshold: bool,
    player_crossed_place_boundary: bool,
    hunter_crossed_place_boundary: bool,
    player_stutterless: bool,
    hunter_stutterless: bool,
    source_revision: u64,
    committed_revision: u64,
    dirty_slots: usize,
    replay_hash: u64,
    history_events: usize,
    body_subject: &'static str,
    body_revision: u64,
    body_capsules: usize,
    scene_digest: String,
    scene_bytes: usize,
}

#[derive(Debug, serde::Serialize)]
struct TracerReceipt {
    cpu_prepare_us: u64,
    brick_upload_bytes: u64,
    uniform_upload_bytes: u64,
    resource_creations: u32,
    bind_group_rebuilds: u32,
    map_recreated: bool,
    trace_passes: u32,
    readback_bytes: u64,
}

#[derive(Debug, serde::Serialize)]
struct NetrenderReceipt {
    total_us: u64,
    dirty_tiles: usize,
    spans: Vec<TimingReceipt>,
}

#[derive(Debug, serde::Serialize)]
struct TimingReceipt {
    name: &'static str,
    duration_us: u64,
}

impl Receipt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        frame: u32,
        size: [u32; 2],
        format: wgpu::TextureFormat,
        scenario: &Scenario,
        adapter: &wgpu::Adapter,
        tracer: BrickDiagnostics,
        netrender: FrameTimings,
        dirty_tiles: usize,
    ) -> Self {
        let info = adapter.get_info();
        let limits = adapter.limits();
        let scene = scenario.scene_bytes();
        Self {
            gate: "G4-P8-generated-place-crossing",
            target: if cfg!(target_arch = "wasm32") {
                "browser"
            } else {
                "native"
            },
            profile: "raster-baseline",
            frame,
            size,
            surface_format: format!("{format:?}"),
            adapter: AdapterReceipt {
                name: info.name,
                backend: format!("{:?}", info.backend),
                max_texture_dimension_2d: limits.max_texture_dimension_2d,
                max_bind_groups: limits.max_bind_groups,
            },
            scenario: ScenarioReceipt {
                seed: SEED,
                generated_route: scenario.route().to_vec(),
                threshold_step: 0,
                place_boundary_step: scenario.boundary_step(),
                from_place: scenario.origin_place().0,
                to_place: scenario.destination_place().0,
                player_start: scenario.player_start(),
                player_after: scenario.player_after(),
                hunter_start: scenario.hunter_start(),
                hunter_after: scenario.hunter_after(),
                ordered_intents: scenario.ordered_intents(),
                completed_steps: scenario.advanced_steps(),
                all_outcomes_moved: scenario.advanced_steps() == scenario.ordered_intents(),
                hunter_pursued: scenario.advanced_steps() == scenario.ordered_intents(),
                hunter_crossed_threshold: scenario.hunter_crossed_threshold(),
                player_crossed_place_boundary: scenario.player_crossed_boundary(),
                hunter_crossed_place_boundary: scenario.hunter_crossed_boundary(),
                player_stutterless: scenario.player_stutterless(),
                hunter_stutterless: scenario.hunter_stutterless(),
                source_revision: scenario.source_revision(),
                committed_revision: scenario.committed_revision(),
                dirty_slots: 0,
                replay_hash: scenario.replay_hash(),
                history_events: scenario.history_events(),
                body_subject: "hunter",
                body_revision: scenario.body_revision(),
                body_capsules: scenario.body_capsules(),
                scene_digest: format!("fnv1a64:{:016x}", digest(scene)),
                scene_bytes: scene.len(),
            },
            tracer: TracerReceipt {
                cpu_prepare_us: tracer.cpu_prepare_us,
                brick_upload_bytes: tracer.brick_upload_bytes,
                uniform_upload_bytes: tracer.uniform_upload_bytes,
                resource_creations: tracer.resource_creations,
                bind_group_rebuilds: tracer.bind_group_rebuilds,
                map_recreated: tracer.map_recreated,
                trace_passes: tracer.trace_passes,
                readback_bytes: tracer.readback_bytes,
            },
            netrender: NetrenderReceipt {
                total_us: netrender.total.as_micros() as u64,
                dirty_tiles,
                spans: netrender
                    .spans
                    .into_iter()
                    .map(|span| TimingReceipt {
                        name: span.name,
                        duration_us: span.duration.as_micros() as u64,
                    })
                    .collect(),
            },
        }
    }
}

fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
