// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V1: a headed continuous-zoom residency receipt.
//!
//! Paredros pulls close, leading, and planning pages from one larger exact
//! Ground. The run records every page load and eviction, the bytes uploaded
//! to the shared DDA, frame spans, and recovery after two abrupt zooms.

use std::{path::Path, sync::Arc, time::Instant};

use mesocosm_core::places::BRICK;
use mesocosm_lens::{BrickDiagnostics, BrickRevision};
use paredros_room::{
    gpu::{self, Composer, DdaTenant, SIZE},
    residency::{
        RESIDENT_BUDGET_BYTES, ResidencyMetrics, ResidencyPolicy, ResidencyScene, V1_FRAMES,
        visible_range, zoom_distance,
    },
    scene,
};
use serde::Serialize;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

const CAPTURE: &str = r"C:\Users\mark_\Code\testing\paredros\v1_residency.png";
const RECEIPT: &str = r"C:\Users\mark_\Code\testing\paredros\v1_residency.json";
const TRAVEL_FRAME: u64 = 72;

fn main() {
    let scene = ResidencyScene::grow();
    println!(
        "V1 world: {} exact bricks, focus {:?}, revision {}",
        scene.ground.brick_count(),
        scene.focus,
        scene.ground.revision()
    );
    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = ResidencyApp {
        instance: wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        ),
        scene,
        policy: ResidencyPolicy::default(),
        live: None,
        samples: Vec::with_capacity(V1_FRAMES as usize),
    };
    event_loop.run_app(&mut app).expect("V1 headed run");
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tenant: DdaTenant,
    composer: Composer,
    adapter: String,
    current: ResidencyMetrics,
    initial_prepare_us: u64,
}

struct ResidencyApp {
    instance: wgpu::Instance,
    scene: ResidencyScene,
    policy: ResidencyPolicy,
    live: Option<Live>,
    samples: Vec<FrameSample>,
}

impl ApplicationHandler for ResidencyApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Paredros V1: continuous-zoom residency")
            .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1]));
        let window = Arc::new(event_loop.create_window(attributes).expect("V1 window"));
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("V1 surface");
        let handles = gpu::boot(&self.instance, Some(&surface));
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let capabilities = surface.get_capabilities(&handles.adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|format| !format.is_srgb())
            .unwrap_or(capabilities.formats[0]);
        let adapter = handles.adapter.get_info().name;

        let aspect = SIZE[0] as f32 / SIZE[1] as f32;
        let visible = visible_range(zoom_distance(0), aspect);
        let began = Instant::now();
        let initial = self
            .policy
            .prepare(&self.scene, visible)
            .expect("initial V1 page")
            .expect("V1 starts without a resident page");
        let initial_prepare_us = began.elapsed().as_micros() as u64;
        let current = initial.metrics;
        let tenant = DdaTenant::from_map(
            &handles,
            initial.map,
            BrickRevision(self.scene.ground.revision()),
            SIZE,
        );
        let composer = Composer::new(handles, SIZE);
        let mut live = Live {
            window,
            surface,
            format,
            device,
            queue,
            tenant,
            composer,
            adapter,
            current,
            initial_prepare_us,
        };
        configure(&mut live);
        live.window.request_redraw();
        self.live = Some(live);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(_) => {
                if let Some(live) = self.live.as_mut() {
                    configure(live);
                }
            },
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {},
        }
    }
}

impl ResidencyApp {
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let frame = self.samples.len() as u64;
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let began = Instant::now();
        let distance = zoom_distance(frame);
        let aspect = SIZE[0] as f32 / SIZE[1] as f32;
        let visible = visible_range(distance, aspect);
        let mut page_prepare_us = 0;
        let mut loaded_bricks = 0;
        let mut evicted_bricks = 0;
        let mut page_transition = false;
        let travel = frame == TRAVEL_FRAME;

        if travel {
            assert!(
                self.scene.move_focus_x(BRICK),
                "the V1 travel receipt needs a valid destination stance"
            );
        }

        if frame == 0 {
            page_prepare_us = live.initial_prepare_us;
            loaded_bricks = live.current.loaded_bricks;
            evicted_bricks = live.current.evicted_bricks;
            page_transition = true;
        } else {
            let page_began = Instant::now();
            let prepared = self
                .policy
                .prepare(&self.scene, visible)
                .expect("camera range has a V1 page");
            assert!(
                !travel || prepared.is_some(),
                "same-band travel must select a new projection"
            );
            if let Some(prepared) = prepared {
                page_prepare_us = page_began.elapsed().as_micros() as u64;
                loaded_bricks = prepared.metrics.loaded_bricks;
                evicted_bricks = prepared.metrics.evicted_bricks;
                live.current = prepared.metrics;
                live.tenant
                    .replace_map(prepared.map, BrickRevision(self.scene.ground.revision()))
                    .expect("V1 page changes advance projection revision");
                page_transition = true;
            }
        }

        let camera = self.scene.camera(distance, aspect);
        let pose = scene::body_pose(self.scene.focus);
        let diagnostics = live.tenant.draw(camera, &pose).expect("V1 DDA frame");
        if travel {
            assert!(
                diagnostics.projection_replaced,
                "V1 travel must retain and republish equal-sized textures"
            );
            assert_eq!(diagnostics.resource_creations, 0);
            assert_eq!(diagnostics.bind_group_rebuilds, 0);
            assert!(diagnostics.brick_upload_bytes > 0);
        }
        let chrome = scene::chrome(SIZE, frame as usize + 1, V1_FRAMES as usize);
        let master = live.composer.compose(&chrome, &live.tenant.view);
        let size = live.window.inner_size();

        use wgpu::CurrentSurfaceTexture as Acquired;
        match live.surface.get_current_texture() {
            Acquired::Success(surface_frame) | Acquired::Suboptimal(surface_frame) => {
                let target = surface_frame.texture.create_view(&Default::default());
                live.composer.present(
                    &master,
                    &target,
                    live.format,
                    [size.width.max(1), size.height.max(1)],
                );
                live.window.pre_present_notify();
                live.queue.present(surface_frame);
            },
            Acquired::Outdated | Acquired::Lost => {
                configure(live);
                live.window.request_redraw();
                return;
            },
            Acquired::Timeout | Acquired::Occluded => {
                live.window.request_redraw();
                return;
            },
            Acquired::Validation => panic!("V1 surface acquisition failed validation"),
        }

        let sample = FrameSample::new(
            frame,
            distance,
            visible,
            live.current,
            page_transition,
            page_prepare_us,
            loaded_bricks,
            evicted_bricks,
            diagnostics,
            began.elapsed().as_micros() as u64,
        );
        if sample.page_transition {
            println!(
                "page @{:02}: projection {}, visible {} -> range {}, {} resident, +{} -{}, {} bytes, prepare {} us, upload {} bytes, frame {} us",
                sample.frame,
                sample.projection_revision,
                sample.visible_range,
                sample.resident_range,
                sample.resident_bricks,
                sample.loaded_bricks,
                sample.evicted_bricks,
                sample.resident_bytes,
                sample.page_prepare_us,
                sample.brick_upload_bytes,
                sample.frame_us,
            );
        }
        self.samples.push(sample);

        if self.samples.len() == V1_FRAMES as usize {
            report(
                &live.composer,
                &master,
                &live.adapter,
                &self.scene,
                &self.samples,
            );
            event_loop.exit();
        } else {
            live.window.request_redraw();
        }
    }
}

fn configure(live: &mut Live) {
    let size = live.window.inner_size();
    live.surface.configure(
        &live.device,
        &wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: live.format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            color_space: wgpu::SurfaceColorSpace::Auto,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
}

#[derive(Clone, Copy, Serialize)]
struct FrameSample {
    frame: u64,
    projection_revision: u64,
    camera_distance: f32,
    visible_range: i32,
    resident_range: i32,
    page_transition: bool,
    page_prepare_us: u64,
    loaded_bricks: usize,
    evicted_bricks: usize,
    resident_bricks: usize,
    pointer_bytes: u64,
    atlas_bytes: u64,
    resident_bytes: u64,
    brick_upload_bytes: u64,
    tracer_cpu_prepare_us: u64,
    resource_creations: u32,
    bind_group_rebuilds: u32,
    projection_replaced: bool,
    frame_us: u64,
}

impl FrameSample {
    #[allow(clippy::too_many_arguments)]
    fn new(
        frame: u64,
        camera_distance: f32,
        visible_range: i32,
        resident: ResidencyMetrics,
        page_transition: bool,
        page_prepare_us: u64,
        loaded_bricks: usize,
        evicted_bricks: usize,
        diagnostics: BrickDiagnostics,
        frame_us: u64,
    ) -> Self {
        Self {
            frame,
            projection_revision: resident.projection_revision.0,
            camera_distance,
            visible_range,
            resident_range: resident.resident_range,
            page_transition,
            page_prepare_us,
            loaded_bricks,
            evicted_bricks,
            resident_bricks: resident.resident_bricks,
            pointer_bytes: resident.pointer_bytes,
            atlas_bytes: resident.atlas_bytes,
            resident_bytes: resident.resident_bytes,
            brick_upload_bytes: diagnostics.brick_upload_bytes,
            tracer_cpu_prepare_us: diagnostics.cpu_prepare_us,
            resource_creations: diagnostics.resource_creations,
            bind_group_rebuilds: diagnostics.bind_group_rebuilds,
            projection_replaced: diagnostics.projection_replaced,
            frame_us,
        }
    }
}

#[derive(Serialize)]
struct Recovery {
    event_frame: u64,
    resident_range: i32,
    steady_median_us: u64,
    threshold_us: u64,
    event_frame_us: u64,
    recovered_at_frame: Option<u64>,
    recovery_frames: Option<u64>,
}

#[derive(Serialize)]
struct Receipt<'a> {
    gate: &'static str,
    vessel: &'static str,
    camera_profile: &'static str,
    traversal_implementation: &'static str,
    resident_measure: &'static str,
    publication_mode: &'static str,
    adapter: &'a str,
    size: [u32; 2],
    frames: usize,
    world_extent: i32,
    world_bricks: usize,
    ground_revision: u64,
    resident_budget_bytes: u64,
    maximum_resident_bytes: u64,
    page_transitions: usize,
    equal_extent_travel_frame: u64,
    equal_extent_travel_proved: bool,
    total_brick_upload_bytes: u64,
    frame_us_min: u64,
    frame_us_median: u64,
    frame_us_max: u64,
    steady_frame_us_median: u64,
    rapid_close_recovery: Recovery,
    rapid_far_recovery: Recovery,
    capture: &'a str,
    capture_distinct_colours: usize,
    samples: &'a [FrameSample],
}

fn report(
    composer: &Composer,
    master: &wgpu::Texture,
    adapter: &str,
    scene: &ResidencyScene,
    samples: &[FrameSample],
) {
    let capture = composer.capture(master);
    capture.write_png(Path::new(CAPTURE)).expect("V1 capture");
    assert!(
        !capture.is_trivial(),
        "V1 capture has only {} distinct colours",
        capture.distinct
    );
    let frame_spans: Vec<_> = samples.iter().map(|sample| sample.frame_us).collect();
    let steady_spans: Vec<_> = samples
        .iter()
        .filter(|sample| !sample.page_transition)
        .map(|sample| sample.frame_us)
        .collect();
    let rapid_close_recovery = recovery(samples, 48);
    let rapid_far_recovery = recovery(samples, 60);
    assert!(
        rapid_close_recovery.recovered_at_frame.is_some()
            && rapid_far_recovery.recovered_at_frame.is_some(),
        "both rapid zooms must recover within the V1 trace"
    );
    let travel = samples
        .iter()
        .find(|sample| sample.frame == TRAVEL_FRAME)
        .expect("V1 travel frame");
    let equal_extent_travel_proved = travel.page_transition
        && travel.projection_replaced
        && travel.resource_creations == 0
        && travel.bind_group_rebuilds == 0
        && travel.brick_upload_bytes > 0;
    assert!(
        equal_extent_travel_proved,
        "V1 must prove an equal-extent travel republish"
    );
    let receipt = Receipt {
        gate: "V1",
        vessel: "paredros",
        camera_profile: "third-person continuous zoom: near acts, mid leads, far plans",
        traversal_implementation: "modulus::BRICK_DDA_WGSL via mesocosm_lens::BrickTracer",
        resident_measure: "logical pointer plus atlas payload; excludes driver rounding and transition overlap",
        publication_mode: "retained equal-sized textures; full CPU republish when projection revision changes",
        adapter,
        size: SIZE,
        frames: samples.len(),
        world_extent: paredros_room::residency::WORLD_EXTENT,
        world_bricks: scene.ground.brick_count(),
        ground_revision: scene.ground.revision(),
        resident_budget_bytes: RESIDENT_BUDGET_BYTES,
        maximum_resident_bytes: samples
            .iter()
            .map(|sample| sample.resident_bytes)
            .max()
            .expect("V1 samples"),
        page_transitions: samples
            .iter()
            .filter(|sample| sample.page_transition)
            .count(),
        equal_extent_travel_frame: TRAVEL_FRAME,
        equal_extent_travel_proved,
        total_brick_upload_bytes: samples.iter().map(|sample| sample.brick_upload_bytes).sum(),
        frame_us_min: *frame_spans.iter().min().expect("V1 frames"),
        frame_us_median: median(&frame_spans),
        frame_us_max: *frame_spans.iter().max().expect("V1 frames"),
        steady_frame_us_median: median(&steady_spans),
        rapid_close_recovery,
        rapid_far_recovery,
        capture: CAPTURE,
        capture_distinct_colours: capture.distinct,
        samples,
    };
    let json = serde_json::to_string_pretty(&receipt).expect("V1 receipt JSON");
    std::fs::write(RECEIPT, &json).expect("write V1 receipt");
    println!("{json}");
}

fn recovery(samples: &[FrameSample], event_frame: u64) -> Recovery {
    let event = samples
        .iter()
        .find(|sample| sample.frame == event_frame)
        .expect("rapid zoom frame");
    let steady: Vec<_> = samples
        .iter()
        .filter(|sample| sample.resident_range == event.resident_range && !sample.page_transition)
        .map(|sample| sample.frame_us)
        .collect();
    let steady_median_us = median(&steady);
    let threshold_us = steady_median_us + steady_median_us / 4;
    let recovered_at_frame = samples
        .iter()
        .find(|sample| {
            sample.frame > event_frame
                && sample.resident_range == event.resident_range
                && sample.frame_us <= threshold_us
        })
        .map(|sample| sample.frame);
    Recovery {
        event_frame,
        resident_range: event.resident_range,
        steady_median_us,
        threshold_us,
        event_frame_us: event.frame_us,
        recovered_at_frame,
        recovery_frames: recovered_at_frame.map(|frame| frame - event_frame),
    }
}

fn median(values: &[u64]) -> u64 {
    assert!(!values.is_empty(), "a median needs samples");
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}
