// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_lens::BrickDiagnostics;
use netrender::profiling::FrameTimings;

use crate::scenario::{Answer, Scenario, SweepLog};

#[derive(Debug, serde::Serialize)]
pub struct Receipt {
    pub gate: &'static str,
    pub vessel: &'static str,
    pub consumer: &'static str,
    pub lattice: &'static str,
    pub oracle: &'static str,
    pub frame: u32,
    pub size: [u32; 2],
    pub surface_format: String,
    pub adapter: AdapterReceipt,
    pub sweep: SweepReceipt,
    pub tracer: TracerReceipt,
    pub netrender_total_us: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct AdapterReceipt {
    pub name: String,
    pub backend: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SweepReceipt {
    /// Two full runs from freshly grown worlds agreed bit for bit; this is
    /// their FNV-1a hash over postcard bytes.
    pub replay_hash: String,
    pub stops: usize,
    pub ground_answers: usize,
    pub critter_answers: usize,
    pub nothing_answers: usize,
    pub carved_cell: [i32; 3],
    pub ground_revision_before: u64,
    pub ground_revision_after: u64,
    pub synced_cells: usize,
    pub log: SweepLog,
}

#[derive(Debug, serde::Serialize)]
pub struct TracerReceipt {
    pub cpu_prepare_us: u64,
    pub brick_upload_bytes: u64,
    pub uniform_upload_bytes: u64,
    pub resource_creations: u32,
    pub trace_passes: u32,
}

impl Receipt {
    pub fn new(
        frame: u32,
        size: [u32; 2],
        format: wgpu::TextureFormat,
        scenario: &Scenario,
        adapter: &wgpu::Adapter,
        tracer: BrickDiagnostics,
        netrender: FrameTimings,
    ) -> Self {
        let info = adapter.get_info();
        let count = |answers: &dyn Fn(&Answer) -> bool| {
            scenario
                .log
                .before
                .iter()
                .chain(scenario.log.after.iter())
                .filter(|stop| answers(&stop.answer))
                .count()
        };
        Self {
            gate: "T1",
            vessel: "mesocosm",
            consumer: "terrarium pointer picking",
            lattice: "Ground -> GroundVoxelProfile -> conatus::BodyWorld (Rapier private) \
                      via mesocosm_runtime::TactileWorld",
            oracle: "axis-aligned -z slab rays judged against an integer Ground::solid scan, \
                     independent of Rapier",
            frame,
            size,
            surface_format: format!("{format:?}"),
            adapter: AdapterReceipt {
                name: info.name,
                backend: format!("{:?}", info.backend),
            },
            sweep: SweepReceipt {
                replay_hash: format!("fnv1a64:{:016x}", scenario.log_hash),
                stops: scenario.stops.len(),
                ground_answers: count(&|answer| matches!(answer, Answer::Ground { .. })),
                critter_answers: count(&|answer| matches!(answer, Answer::Critter { .. })),
                nothing_answers: count(&|answer| matches!(answer, Answer::Nothing)),
                carved_cell: scenario.log.carved_cell,
                ground_revision_before: scenario.log.ground_revision_before,
                ground_revision_after: scenario.log.ground_revision_after,
                synced_cells: scenario.log.synced_cells,
                log: scenario.log.clone(),
            },
            tracer: TracerReceipt {
                cpu_prepare_us: tracer.cpu_prepare_us,
                brick_upload_bytes: tracer.brick_upload_bytes,
                uniform_upload_bytes: tracer.uniform_upload_bytes,
                resource_creations: tracer.resource_creations,
                trace_passes: tracer.trace_passes,
            },
            netrender_total_us: netrender.total.as_micros() as u64,
        }
    }
}
