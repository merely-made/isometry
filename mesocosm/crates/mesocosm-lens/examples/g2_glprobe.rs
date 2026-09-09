// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! WebGL2-class G2 tracer receipt. This deliberately probes the brick pass,
//! not netrender's Vello tenant: D0 owns rasterizer replacement separately.

use std::collections::BTreeSet;

use mesocosm_core::places::{Ground, Places};
use mesocosm_lens::{
    BrickFrameInput, BrickMap, BrickRevision, BrickTracer, CritterPose, Flight, Grade,
    critter::Capsule,
};

const WIDTH: u32 = 960;
const HEIGHT: u32 = 540;

#[derive(serde::Serialize)]
struct Receipt {
    gate: &'static str,
    profile: &'static str,
    backend: String,
    adapter: String,
    size: [u32; 2],
    max_texture_dimension_2d: u32,
    clay_colours: usize,
    retro_colours: usize,
    clay_upload_bytes: u64,
    retro_upload_bytes: u64,
    pixels_changed: bool,
}

fn main() -> Result<(), String> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::GL;
    let instance = wgpu::Instance::new(descriptor);
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .map_err(|error| format!("no wgpu-GL adapter: {error}"))?;
    let limits = wgpu::Limits::downlevel_webgl2_defaults();
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Mesocosm G2 WebGL2-class probe"),
        required_limits: limits.clone(),
        ..Default::default()
    }))
    .map_err(|error| format!("GL adapter declined WebGL2-class limits: {error}"))?;
    let ground = Ground::grow(&Places::grown(4_242, 4, 64), 64);
    let map = BrickMap::from_ground(&ground).map_err(|error| error.to_string())?;
    let (flight, pose) = view(&ground)?;
    let mut tracer = BrickTracer::with_device(device, queue, WIDTH, HEIGHT);
    let clay = tracer
        .capture(
            BrickFrameInput::new(
                &map,
                BrickRevision(ground.revision()),
                &flight,
                &Grade::clay(),
            )
            .with_pose(&pose),
        )
        .map_err(|error| error.to_string())?;
    let retro = tracer
        .capture(
            BrickFrameInput::new(
                &map,
                BrickRevision(ground.revision()),
                &flight,
                &Grade::retro(3),
            )
            .with_pose(&pose),
        )
        .map_err(|error| error.to_string())?;
    let info = adapter.get_info();
    let receipt = Receipt {
        gate: "G2",
        profile: "downlevel-webgl2-defaults",
        backend: format!("{:?}", info.backend),
        adapter: info.name,
        size: [WIDTH, HEIGHT],
        max_texture_dimension_2d: limits.max_texture_dimension_2d,
        clay_colours: colours(&clay.pixels),
        retro_colours: colours(&retro.pixels),
        clay_upload_bytes: clay.diagnostics.brick_upload_bytes,
        retro_upload_bytes: retro.diagnostics.brick_upload_bytes,
        pixels_changed: clay.pixels != retro.pixels,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn view(ground: &Ground) -> Result<(Flight, CritterPose), String> {
    let eye_top = ground
        .surface(4, 4)
        .ok_or("fixture eye is outside Ground")? as f32;
    let body_top = ground
        .surface(4, 18)
        .ok_or("fixture body is outside Ground")? as f32;
    let body = [4.5, body_top + 1.15, 18.5];
    let eye = [4.5, eye_top + 17.0, 4.5];
    let distance = ((body[0] - eye[0]).powi(2) + (body[2] - eye[2]).powi(2)).sqrt();
    let pose = CritterPose::from_capsules(
        vec![Capsule {
            a: [body[0] - 0.7, body[1], body[2]],
            ra: 0.65,
            b: [body[0] + 0.7, body[1], body[2]],
            rb: 0.52,
        }],
        [
            [body[0] - 0.45, body[1] + 0.15, body[2] - 0.35, 0.10],
            [body[0] - 0.45, body[1] - 0.15, body[2] - 0.35, 0.10],
        ],
        [0.15, 0.86, 0.32],
    );
    Ok((
        Flight {
            eye,
            yaw: 0.0,
            pitch: f32::atan2(body[1] - eye[1], distance),
            fov: 0.9,
            far: 48.0,
        },
        pose,
    ))
}

fn colours(pixels: &[u8]) -> usize {
    pixels
        .chunks_exact(4)
        .map(|pixel| [pixel[0], pixel[1], pixel[2], pixel[3]])
        .collect::<BTreeSet<_>>()
        .len()
}
