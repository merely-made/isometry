// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Synchronized 1080p G2 tracer measurement. `--backend gl` holds the same
//! workload under the downlevel-WebGL2 limit profile.

use std::time::Instant;

use mesocosm_core::places::{Ground, Places};
use mesocosm_lens::{
    BrickFrameInput, BrickMap, BrickRevision, BrickTracer, CritterPose, Flight, Grade,
    critter::Capsule,
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;
const WARM_FRAMES: usize = 24;
const SAMPLES: usize = 96;

#[derive(serde::Serialize)]
struct Receipt {
    gate: &'static str,
    profile: &'static str,
    backend: String,
    adapter: String,
    size: [u32; 2],
    samples: usize,
    frame_us_median: u64,
    frame_us_min: u64,
    frame_us_max: u64,
    fps_median: f64,
    tracer_cpu_prepare_us_median: u64,
    steady_brick_upload_bytes: u64,
    steady_uniform_upload_bytes: u64,
}

fn main() -> Result<(), String> {
    let backend = std::env::args().nth(1).unwrap_or_else(|| "vulkan".into());
    let (backends, profile) = match backend.as_str() {
        "vulkan" => (wgpu::Backends::VULKAN, "raster-baseline"),
        "gl" => (wgpu::Backends::GL, "downlevel-webgl2-defaults"),
        other => return Err(format!("unknown backend {other:?}; use vulkan or gl")),
    };
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = backends;
    let instance = wgpu::Instance::new(descriptor);
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .map_err(|error| format!("no {backend} adapter: {error}"))?;
    let limits = if backend == "gl" {
        wgpu::Limits::downlevel_webgl2_defaults()
    } else {
        wgpu::Limits::default()
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("Mesocosm G2 1080p measurement"),
        required_limits: limits,
        ..Default::default()
    }))
    .map_err(|error| format!("{backend} device request failed: {error}"))?;
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Mesocosm G2 1080p target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target_view = target.create_view(&Default::default());
    let ground = Ground::grow(&Places::grown(4_242, 4, 64), 64);
    let map = BrickMap::from_ground(&ground).map_err(|error| error.to_string())?;
    let (flight, pose) = view(&ground)?;
    let grade = Grade::retro(3);
    let input = BrickFrameInput::new(&map, BrickRevision(ground.revision()), &flight, &grade)
        .with_pose(&pose);
    let mut tracer = BrickTracer::with_device(device.clone(), queue.clone(), WIDTH, HEIGHT);
    for _ in 0..WARM_FRAMES {
        draw(&device, &queue, &mut tracer, &target_view, input)?;
    }
    let mut frames = Vec::with_capacity(SAMPLES);
    let mut prepares = Vec::with_capacity(SAMPLES);
    let mut steady_bricks = 0;
    let mut steady_uniforms = 0;
    for _ in 0..SAMPLES {
        let started = Instant::now();
        let diagnostics = draw(&device, &queue, &mut tracer, &target_view, input)?;
        frames.push(started.elapsed().as_micros() as u64);
        prepares.push(diagnostics.cpu_prepare_us);
        steady_bricks += diagnostics.brick_upload_bytes;
        steady_uniforms += diagnostics.uniform_upload_bytes;
    }
    frames.sort_unstable();
    prepares.sort_unstable();
    let median = frames[frames.len() / 2];
    let info = adapter.get_info();
    let receipt = Receipt {
        gate: "G2",
        profile,
        backend: format!("{:?}", info.backend),
        adapter: info.name,
        size: [WIDTH, HEIGHT],
        samples: SAMPLES,
        frame_us_median: median,
        frame_us_min: frames[0],
        frame_us_max: *frames.last().expect("non-empty samples"),
        fps_median: 1_000_000.0 / median.max(1) as f64,
        tracer_cpu_prepare_us_median: prepares[prepares.len() / 2],
        steady_brick_upload_bytes: steady_bricks,
        steady_uniform_upload_bytes: steady_uniforms,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn draw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    tracer: &mut BrickTracer,
    view: &wgpu::TextureView,
    input: BrickFrameInput<'_>,
) -> Result<mesocosm_lens::BrickDiagnostics, String> {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Mesocosm G2 measured frame"),
    });
    let diagnostics = tracer
        .encode(&mut encoder, view, input)
        .map_err(|error| error.to_string())?;
    queue.submit([encoder.finish()]);
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|error| error.to_string())?;
    Ok(diagnostics)
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
