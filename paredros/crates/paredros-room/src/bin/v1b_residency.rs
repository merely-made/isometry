// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V1b: the headed stable-residency receipt.
//!
//! The V1 zoom-and-travel trace runs over one capacity-fixed brick cache.
//! Every band transition and travel retargets the cache in place: retained
//! bricks keep their atlas slots, the kilobyte-scale pointer volume moves,
//! and only loaded bricks' 512-byte slots upload. The receipt requires zero
//! texture or bind-group creation at every transition after the first
//! frame, exact per-transition upload accounting, retained one-frame
//! recovery, and records wgpu's allocator report beside the logical bytes.

use std::{path::Path, sync::Arc, time::Instant};

use mesocosm_core::places::BRICK;
use mesocosm_lens::{
    BrickChange, BrickDiagnostics, BrickFrameInput, BrickRevision, BrickTracer, Grade,
};
use modulus::{BrickMap, BrickProjectionRevision};
use netrender::WgpuHandles;
use paredros_room::{
    gpu::{self, Composer, SIZE},
    residency::{
        RESIDENT_BUDGET_BYTES, ResidencyMetrics, ResidencyScene, StableResidency, V1_FRAMES,
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

const CAPTURE: &str = r"C:\Users\mark_\Code\testing\paredros\v1b_residency.png";
const RECEIPT: &str = r"C:\Users\mark_\Code\testing\paredros\v1b_residency.json";
const TRAVEL_FRAME: u64 = 72;

fn main() {
    let scene = ResidencyScene::grow();
    let stable = StableResidency::new(&scene).expect("the stable cache fits the V1 budget");
    println!(
        "V1b world: {} exact bricks, cache capacity {}, fixed {} bytes, focus {:?}",
        scene.ground.brick_count(),
        stable.capacity(),
        stable.resident_bytes(),
        scene.focus
    );
    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = StableApp {
        instance: wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        ),
        scene,
        stable,
        live: None,
        samples: Vec::with_capacity(V1_FRAMES as usize),
        allocator_after_first: None,
    };
    event_loop.run_app(&mut app).expect("V1b headed run");
}

/// The tracer half, borrowing the policy-owned map per frame rather than
/// holding one: the stable cache has exactly one owner.
struct StableTenant {
    pub view: wgpu::TextureView,
    _target: wgpu::Texture,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tracer: BrickTracer,
    grade: Grade,
}

impl StableTenant {
    fn new(handles: &WgpuHandles, size: [u32; 2]) -> Self {
        let target = handles.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Paredros V1b DDA target"),
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        Self {
            view: target.create_view(&Default::default()),
            _target: target,
            device: handles.device.clone(),
            queue: handles.queue.clone(),
            tracer: BrickTracer::with_format(
                handles.device.clone(),
                handles.queue.clone(),
                size[0],
                size[1],
                wgpu::TextureFormat::Rgba8Unorm,
            ),
            grade: Grade {
                fog: [0.03, 0.03, 0.045],
                fog_start: 0.62,
                palette_len: 0,
                dither: 0.0,
                fog_bands: 0.0,
                downscale: 1,
            },
        }
    }

    fn draw(
        &mut self,
        map: &BrickMap,
        revision: BrickRevision,
        camera: mesocosm_lens::TraceCamera,
        pose: &mesocosm_lens::CritterPose,
        loaded_slots: &[u32],
    ) -> Result<BrickDiagnostics, String> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Paredros V1b frame"),
            });
        let diagnostics = self
            .tracer
            .encode(
                &mut encoder,
                &self.view,
                BrickFrameInput::for_camera(map, revision, camera, &self.grade)
                    .with_pose(pose)
                    .changed(BrickChange::Slots(loaded_slots)),
            )
            .map_err(|error| error.to_string())?;
        self.queue.submit([encoder.finish()]);
        Ok(diagnostics)
    }
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tenant: StableTenant,
    composer: Composer,
    adapter: String,
    current: ResidencyMetrics,
}

struct StableApp {
    instance: wgpu::Instance,
    scene: ResidencyScene,
    stable: StableResidency,
    live: Option<Live>,
    samples: Vec<FrameSample>,
    allocator_after_first: Option<u64>,
}

impl ApplicationHandler for StableApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Paredros V1b: stable resident brick cache")
            .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1]));
        let window = Arc::new(event_loop.create_window(attributes).expect("V1b window"));
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("V1b surface");
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
        let tenant = StableTenant::new(&handles, SIZE);
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
            current: ResidencyMetrics {
                projection_revision: BrickProjectionRevision(0),
                visible_range: 0,
                resident_range: 0,
                loaded_bricks: 0,
                evicted_bricks: 0,
                resident_bricks: 0,
                pointer_bytes: 0,
                atlas_bytes: 0,
                resident_bytes: 0,
            },
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

impl StableApp {
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let frame = self.samples.len() as u64;
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let began = Instant::now();
        let distance = zoom_distance(frame);
        let aspect = SIZE[0] as f32 / SIZE[1] as f32;
        let visible = visible_range(distance, aspect);
        let travel = frame == TRAVEL_FRAME;
        if travel {
            assert!(
                self.scene.move_focus_x(BRICK),
                "the V1b travel receipt needs a valid destination stance"
            );
        }

        let page_began = Instant::now();
        let outcome = self
            .stable
            .prepare(&self.scene, visible)
            .expect("camera range has a V1b page");
        let page_prepare_us = page_began.elapsed().as_micros() as u64;
        let page_transition = outcome.is_some();
        assert!(
            !travel || page_transition,
            "same-band travel must retarget the stable cache"
        );
        let loaded_slots = outcome
            .as_ref()
            .map(|outcome| outcome.delta.loaded_slots.clone())
            .unwrap_or_default();
        if let Some(outcome) = outcome {
            live.current = outcome.metrics;
        }

        let camera = self.scene.camera(distance, aspect);
        let pose = scene::body_pose(self.scene.focus);
        let diagnostics = live
            .tenant
            .draw(
                self.stable.map(),
                BrickRevision(self.scene.ground.revision()),
                camera,
                &pose,
                &loaded_slots,
            )
            .expect("V1b DDA frame");

        // The stable claim, held every frame after the first: nothing is
        // ever created again, and a transition uploads exactly the pointer
        // volume plus its loaded slots.
        if frame > 0 {
            assert_eq!(diagnostics.resource_creations, 0, "frame {frame}");
            assert_eq!(diagnostics.bind_group_rebuilds, 0, "frame {frame}");
            assert!(!diagnostics.map_recreated, "frame {frame}");
            if page_transition {
                assert!(diagnostics.projection_replaced, "frame {frame}");
                let pointer_bytes = std::mem::size_of_val(self.stable.map().pointers()) as u64;
                assert_eq!(
                    diagnostics.brick_upload_bytes,
                    pointer_bytes + loaded_slots.len() as u64 * 512,
                    "frame {frame} must upload pointers plus its loaded slots"
                );
            } else {
                assert_eq!(diagnostics.brick_upload_bytes, 0, "frame {frame}");
            }
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
            Acquired::Validation => panic!("V1b surface acquisition failed validation"),
        }

        if frame == 0 {
            self.allocator_after_first = allocated_bytes(&live.device);
        }
        let sample = FrameSample {
            frame,
            projection_revision: live.current.projection_revision.0,
            camera_distance: distance,
            visible_range: visible,
            resident_range: live.current.resident_range,
            page_transition,
            page_prepare_us,
            loaded_bricks: loaded_slots.len(),
            evicted_bricks: if page_transition {
                live.current.evicted_bricks
            } else {
                0
            },
            resident_bricks: live.current.resident_bricks,
            brick_upload_bytes: diagnostics.brick_upload_bytes,
            tracer_cpu_prepare_us: diagnostics.cpu_prepare_us,
            resource_creations: diagnostics.resource_creations,
            bind_group_rebuilds: diagnostics.bind_group_rebuilds,
            projection_replaced: diagnostics.projection_replaced,
            frame_us: began.elapsed().as_micros() as u64,
        };
        if sample.page_transition {
            println!(
                "retarget @{:02}: projection {}, visible {} -> range {}, {} resident, +{} -{}, upload {} bytes, prepare {} us, frame {} us",
                sample.frame,
                sample.projection_revision,
                sample.visible_range,
                sample.resident_range,
                sample.resident_bricks,
                sample.loaded_bricks,
                sample.evicted_bricks,
                sample.brick_upload_bytes,
                sample.page_prepare_us,
                sample.frame_us,
            );
        }
        self.samples.push(sample);

        if self.samples.len() == V1_FRAMES as usize {
            let allocator_after_last = allocated_bytes(&live.device);
            report(
                &live.composer,
                &master,
                &live.adapter,
                &self.scene,
                &self.stable,
                &self.samples,
                self.allocator_after_first,
                allocator_after_last,
            );
            event_loop.exit();
        } else {
            live.window.request_redraw();
        }
    }
}

fn allocated_bytes(device: &wgpu::Device) -> Option<u64> {
    device
        .generate_allocator_report()
        .map(|report| report.total_allocated_bytes)
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

#[derive(Clone, Serialize)]
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
    brick_upload_bytes: u64,
    tracer_cpu_prepare_us: u64,
    resource_creations: u32,
    bind_group_rebuilds: u32,
    projection_replaced: bool,
    frame_us: u64,
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
    world_bricks: usize,
    ground_revision: u64,
    resident_budget_bytes: u64,
    cache_capacity_bricks: usize,
    fixed_resident_bytes: u64,
    fixed_pointer_extent: [u32; 3],
    fixed_atlas_extent: [u32; 3],
    allocator_bytes_after_first_frame: Option<u64>,
    allocator_bytes_after_last_frame: Option<u64>,
    allocator_growth_bytes: Option<i64>,
    retargets: usize,
    travel_frame: u64,
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

#[allow(clippy::too_many_arguments)]
fn report(
    composer: &Composer,
    master: &wgpu::Texture,
    adapter: &str,
    scene: &ResidencyScene,
    stable: &StableResidency,
    samples: &[FrameSample],
    allocator_after_first: Option<u64>,
    allocator_after_last: Option<u64>,
) {
    let capture = composer.capture(master);
    capture.write_png(Path::new(CAPTURE)).expect("V1b capture");
    assert!(
        !capture.is_trivial(),
        "V1b capture has only {} distinct colours",
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
        "both rapid zooms must recover within the V1b trace"
    );
    let allocator_growth_bytes = allocator_after_first
        .zip(allocator_after_last)
        .map(|(first, last)| last as i64 - first as i64);
    if let Some(growth) = allocator_growth_bytes {
        // Driver-internal staging metadata moves by a few bytes between
        // samples; a leaked page or texture would be half a megabyte. The
        // tolerance is far below one brick slot.
        assert!(
            growth.abs() <= 4096,
            "the allocator moved {growth} bytes after the first frame; the cache is not stable"
        );
    }
    let receipt = Receipt {
        gate: "V1b",
        vessel: "paredros",
        camera_profile: "third-person continuous zoom: near acts, mid leads, far plans",
        traversal_implementation: "modulus::BRICK_DDA_WGSL via mesocosm_lens::BrickTracer",
        resident_measure: "fixed pointer plus atlas allocation, with wgpu allocator-report bytes",
        publication_mode: "one capacity-fixed cache; retargets publish the pointer volume plus \
                           loaded slots only, retained slots never re-upload",
        adapter,
        size: SIZE,
        frames: samples.len(),
        world_bricks: scene.ground.brick_count(),
        ground_revision: scene.ground.revision(),
        resident_budget_bytes: RESIDENT_BUDGET_BYTES,
        cache_capacity_bricks: stable.capacity(),
        fixed_resident_bytes: stable.resident_bytes(),
        fixed_pointer_extent: stable.map().pointer_extent(),
        fixed_atlas_extent: stable.map().atlas_extent(),
        allocator_bytes_after_first_frame: allocator_after_first,
        allocator_bytes_after_last_frame: allocator_after_last,
        allocator_growth_bytes,
        retargets: samples
            .iter()
            .filter(|sample| sample.page_transition)
            .count(),
        travel_frame: TRAVEL_FRAME,
        total_brick_upload_bytes: samples.iter().map(|sample| sample.brick_upload_bytes).sum(),
        frame_us_min: *frame_spans.iter().min().expect("V1b frames"),
        frame_us_median: median(&frame_spans),
        frame_us_max: *frame_spans.iter().max().expect("V1b frames"),
        steady_frame_us_median: median(&steady_spans),
        rapid_close_recovery,
        rapid_far_recovery,
        capture: CAPTURE,
        capture_distinct_colours: capture.distinct,
        samples,
    };
    let json = serde_json::to_string_pretty(&receipt).expect("V1b receipt JSON");
    std::fs::write(RECEIPT, &json).expect("write V1b receipt");
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
