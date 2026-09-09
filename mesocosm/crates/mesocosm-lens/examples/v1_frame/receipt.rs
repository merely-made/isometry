// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_lens::FrameDiagnostics;
use netrender::profiling::FrameTimings;

#[derive(Debug, serde::Serialize)]
pub struct Receipt {
    pub gate: &'static str,
    pub target: &'static str,
    pub profile: &'static str,
    pub frame: u32,
    pub size: [u32; 2],
    pub surface_format: String,
    pub master_format: &'static str,
    pub scene_digest: String,
    pub scene_bytes: usize,
    pub adapter: AdapterReceipt,
    pub lens: LensReceipt,
    pub netrender: NetrenderReceipt,
}

#[derive(Debug, serde::Serialize)]
pub struct AdapterReceipt {
    pub name: String,
    pub backend: String,
    pub device_type: String,
    pub vendor: u32,
    pub device: u32,
    pub driver: String,
    pub driver_info: String,
    pub max_texture_dimension_2d: u32,
    pub max_bind_groups: u32,
}

#[derive(Debug, serde::Serialize)]
pub struct LensReceipt {
    pub cpu_prepare_us: u64,
    pub map_upload_bytes: u64,
    pub uniform_upload_bytes: u64,
    pub resource_creations: u32,
    pub bind_group_rebuilds: u32,
    pub map_recreated: bool,
    pub target_recreated: bool,
    pub march_passes: u32,
    pub grade_passes: u32,
    pub readback_bytes: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct NetrenderReceipt {
    pub total_us: u64,
    pub dirty_tiles: usize,
    pub spans: Vec<TimingReceipt>,
}

#[derive(Debug, serde::Serialize)]
pub struct TimingReceipt {
    pub name: &'static str,
    pub duration_us: u64,
}

impl Receipt {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        frame: u32,
        size: [u32; 2],
        surface_format: wgpu::TextureFormat,
        scene_bytes: &[u8],
        adapter: &wgpu::Adapter,
        lens: FrameDiagnostics,
        netrender: FrameTimings,
        dirty_tiles: usize,
    ) -> Self {
        let info = adapter.get_info();
        let limits = adapter.limits();
        Self {
            gate: "V1",
            target: if cfg!(target_arch = "wasm32") {
                "browser"
            } else {
                "native"
            },
            profile: "raster-baseline",
            frame,
            size,
            surface_format: format!("{surface_format:?}"),
            master_format: "Rgba8Unorm",
            scene_digest: format!("fnv1a64:{:016x}", digest(scene_bytes)),
            scene_bytes: scene_bytes.len(),
            adapter: AdapterReceipt {
                name: info.name,
                backend: format!("{:?}", info.backend),
                device_type: format!("{:?}", info.device_type),
                vendor: info.vendor,
                device: info.device,
                driver: info.driver,
                driver_info: info.driver_info,
                max_texture_dimension_2d: limits.max_texture_dimension_2d,
                max_bind_groups: limits.max_bind_groups,
            },
            lens: lens.into(),
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

impl From<FrameDiagnostics> for LensReceipt {
    fn from(value: FrameDiagnostics) -> Self {
        Self {
            cpu_prepare_us: value.cpu_prepare_us,
            map_upload_bytes: value.map_upload_bytes,
            uniform_upload_bytes: value.uniform_upload_bytes,
            resource_creations: value.resource_creations,
            bind_group_rebuilds: value.bind_group_rebuilds,
            map_recreated: value.map_recreated,
            target_recreated: value.target_recreated,
            march_passes: value.march_passes,
            grade_passes: value.grade_passes,
            readback_bytes: value.readback_bytes,
        }
    }
}

fn digest(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}
