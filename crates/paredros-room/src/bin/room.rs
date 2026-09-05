// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The headed room probe.
//!
//! A winit window presenting netrender's composed master: the renderling
//! room underneath, a vello chrome bar over it. With `ROOM_TRACE=1` the run
//! drives itself from the fixed trace, writes the screenshot receipt, prints
//! the hashes and the frame spans, and exits. Without it the window stays up
//! after the trace ends so a human can look at the room.
//! `PAREDROS_RG3_HEADED_PROBE=awaited|optimistic` injects one scoped validation
//! failure and records actual surface acquisition and presentation calls.
//! `PAREDROS_RG3_REBUILD_PROBE=1` latches a synthetic shared fault and proves
//! that every device client rebuilds before the preserved surface presents.

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Instant;
use std::{future::Future, pin::Pin};

use paredros_room::frame_health::{FrameDecision, FrameHealth, PresentationPolicy, SharedFault};
use paredros_room::gpu::{self, Composer, SIZE, Tenant};
#[cfg(feature = "r1-proof")]
use paredros_room::gpu::{BrickAbi, DdaTenant};
use paredros_room::room::SEED;
use paredros_room::{Probe, TICKS, scene};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

/// Where the receipt lands. The screenshots layout is per-repo under
/// `Code/testing/<repo>/`.
const CAPTURE: &str = r"C:\Users\mark_\Code\testing\paredros\s0_room.png";
const RG3B_AWAITED_RECEIPT: &str = r"C:\Users\mark_\Code\testing\paredros\rg3b_awaited_headed.json";
const RG3B_OPTIMISTIC_RECEIPT: &str =
    r"C:\Users\mark_\Code\testing\paredros\rg3b_optimistic_headed.json";
const RG3D_REBUILD_RECEIPT: &str =
    r"C:\Users\mark_\Code\testing\paredros\rg3d_rebuild_all_headed.json";
#[cfg(feature = "r1-proof")]
const RG3_RECEIPT: &str = r"C:\Users\mark_\Code\testing\paredros\rg3c_room.json";
#[cfg(feature = "r1-proof")]
const R1_CAPTURE: &str = r"C:\Users\mark_\Code\testing\paredros\r1_perspective.png";
#[cfg(feature = "r1-proof")]
const R1_RECEIPT: &str = r"C:\Users\mark_\Code\testing\paredros\r1_perspective.json";

fn main() {
    #[cfg(feature = "r1-proof")]
    let r1_mode = std::env::var("ROOM_R1").is_ok_and(|v| v != "0");
    #[cfg(not(feature = "r1-proof"))]
    let r1_mode = false;
    let headed_validation_probe = HeadedValidationProbe::from_env();
    let rebuild_probe = RebuildProbe::from_env();
    assert!(
        headed_validation_probe.is_none() || rebuild_probe.is_none(),
        "PAREDROS_RG3_HEADED_PROBE and PAREDROS_RG3_REBUILD_PROBE are mutually exclusive"
    );
    let trace_mode = r1_mode || std::env::var("ROOM_TRACE").is_ok_and(|v| v != "0");
    let policy = headed_validation_probe
        .as_ref()
        .map(|probe| probe.policy)
        .unwrap_or_else(PresentationPolicy::from_env);
    let probe = Probe::new(SEED).expect("the room probe grows its world");
    println!(
        "room at {:?}, stance {:?}, ground hash {:#018x}",
        probe.room().centre,
        probe.at(),
        probe.ground_hash()
    );

    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = RoomApp {
        instance: wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        ),
        probe,
        trace_mode,
        r1_mode,
        live: None,
        frames: 0,
        frame_attempts: 0,
        captured: false,
        frame_us: Vec::new(),
        #[cfg(feature = "r1-proof")]
        last_trace: None,
        policy,
        headed_validation_probe,
        rebuild_probe,
        device_generation: 0,
        rebuild_requested: None,
    };
    event_loop.run_app(&mut app).expect("winit run");
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    // wgpu 30 presents through the queue rather than the surface texture.
    queue: wgpu::Queue,
    tenant: Tenant,
    #[cfg(feature = "r1-proof")]
    dda: Option<DdaTenant>,
    composer: Composer,
    /// The room is meshed once. Only the torch changes, and the torch rides
    /// the eye, so the vertices are re-shaded per frame rather than remeshed.
    room: Vec<mesocosm_render::geometry::Vertex>,
    #[cfg(feature = "r1-proof")]
    adapter: String,
    health: Arc<Mutex<FrameHealth>>,
    pending_validation: VecDeque<PendingValidation>,
    device_generation: u64,
}

impl Live {
    fn into_host_surface(self) -> (Arc<Window>, wgpu::Surface<'static>) {
        let Self {
            window, surface, ..
        } = self;
        (window, surface)
    }
}

struct PendingValidation {
    future: Pin<Box<dyn Future<Output = Option<wgpu::Error>>>>,
    tenant_name: String,
    producer_path: String,
    frame: u64,
}

struct HeadedValidationProbe {
    policy: PresentationPolicy,
    injected_attempt: Option<u64>,
    surface_acquire_attempts: Vec<u64>,
    surface_present_attempts: Vec<u64>,
    suppressed_attempts: Vec<u64>,
}

impl HeadedValidationProbe {
    fn from_env() -> Option<Self> {
        let value = std::env::var("PAREDROS_RG3_HEADED_PROBE").ok()?;
        let policy = match value.to_ascii_lowercase().as_str() {
            "awaited" | "awaited-diagnostic" => PresentationPolicy::AwaitedDiagnostic,
            "optimistic" => PresentationPolicy::Optimistic,
            _ => panic!("PAREDROS_RG3_HEADED_PROBE must be awaited or optimistic, got {value:?}"),
        };
        Some(Self {
            policy,
            injected_attempt: None,
            surface_acquire_attempts: Vec::new(),
            surface_present_attempts: Vec::new(),
            suppressed_attempts: Vec::new(),
        })
    }

    fn inject_once(&mut self, device: &wgpu::Device, attempt: u64) {
        if self.injected_attempt.is_some() {
            return;
        }
        self.injected_attempt = Some(attempt);
        let source = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RG3b headed validation source"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let target = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("RG3b headed validation target"),
            size: 4,
            usage: wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("RG3b disposable invalid tenant encoder"),
        });
        encoder.copy_buffer_to_buffer(&source, 0, &target, 0, 8);
        drop(encoder.finish());
    }

    fn record_suppressed(&mut self, attempt: u64) {
        self.suppressed_attempts.push(attempt);
    }

    fn completed_record<'a>(
        &self,
        health: &'a FrameHealth,
    ) -> Option<&'a paredros_room::frame_health::ValidationRecord> {
        let injected = self.injected_attempt?;
        let validation = health
            .validations()
            .iter()
            .find(|record| record.frame == injected)?;
        let suppressed = self.suppressed_attempts.first().copied()?;
        let healthy_presented = self
            .surface_present_attempts
            .iter()
            .any(|attempt| *attempt > suppressed);
        let policy_contract = match self.policy {
            PresentationPolicy::AwaitedDiagnostic => suppressed == injected,
            PresentationPolicy::Optimistic => {
                self.surface_present_attempts.contains(&injected) && suppressed > injected
            },
        };
        let suppressed_never_acquired = self
            .suppressed_attempts
            .iter()
            .all(|attempt| !self.surface_acquire_attempts.contains(attempt));
        let suppressed_never_presented = self
            .suppressed_attempts
            .iter()
            .all(|attempt| !self.surface_present_attempts.contains(attempt));
        (healthy_presented
            && policy_contract
            && suppressed_never_acquired
            && suppressed_never_presented)
            .then_some(validation)
    }

    fn write_receipt(&self, validation: &paredros_room::frame_health::ValidationRecord) {
        let (policy, path) = match self.policy {
            PresentationPolicy::AwaitedDiagnostic => ("awaited", RG3B_AWAITED_RECEIPT),
            PresentationPolicy::Optimistic => ("optimistic", RG3B_OPTIMISTIC_RECEIPT),
        };
        let injected = self.injected_attempt.expect("headed fault was injected");
        let receipt = format!(
            concat!(
                "{{\n",
                "  \"gate\": \"RG3b headed surface suppression\",\n",
                "  \"policy\": \"{}\",\n",
                "  \"injected_attempt\": {},\n",
                "  \"validation\": {{\n",
                "    \"tenant_name\": \"{}\",\n",
                "    \"producer_path\": \"{}\",\n",
                "    \"attempt\": {},\n",
                "    \"captured\": true\n",
                "  }},\n",
                "  \"surface_acquire_attempts\": {:?},\n",
                "  \"surface_present_attempts\": {:?},\n",
                "  \"suppressed_attempts\": {:?},\n",
                "  \"suppressed_attempts_never_acquired\": true,\n",
                "  \"suppressed_attempts_never_presented\": true,\n",
                "  \"healthy_frame_presented_after_suppression\": true,\n",
                "  \"shared_device\": true,\n",
                "  \"actual_surface_calls_observed\": true,\n",
                "  \"scope_limit\": \"native surface acquisition and presentation; no transactional rollback of renderer-internal bookkeeping\"\n",
                "}}\n"
            ),
            policy,
            injected,
            validation.tenant_name,
            validation.producer_path,
            validation.frame,
            self.surface_acquire_attempts,
            self.surface_present_attempts,
            self.suppressed_attempts,
        );
        let path = PathBuf::from(path);
        std::fs::create_dir_all(path.parent().expect("headed receipt directory"))
            .expect("create headed receipt directory");
        std::fs::write(&path, receipt).expect("write headed validation receipt");
        println!("RG3b headed {policy} receipt: {}", path.display());
    }
}

struct RebuildProbe {
    injected_attempt: Option<u64>,
    fault_generation: Option<u64>,
    rebuilt_generation: Option<u64>,
    surface_acquire_attempts: Vec<u64>,
    surface_present_attempts: Vec<u64>,
    suppressed_attempts: Vec<u64>,
}

impl RebuildProbe {
    fn from_env() -> Option<Self> {
        let value = std::env::var("PAREDROS_RG3_REBUILD_PROBE").ok()?;
        match value.to_ascii_lowercase().as_str() {
            "" | "0" | "false" => return None,
            "1" | "true" => {},
            _ => panic!("PAREDROS_RG3_REBUILD_PROBE must be 0/false or 1/true, got {value:?}"),
        }
        Some(Self {
            injected_attempt: None,
            fault_generation: None,
            rebuilt_generation: None,
            surface_acquire_attempts: Vec::new(),
            surface_present_attempts: Vec::new(),
            suppressed_attempts: Vec::new(),
        })
    }

    fn inject_once(&mut self, live: &Live, attempt: u64) {
        if self.injected_attempt.is_some() {
            return;
        }
        self.injected_attempt = Some(attempt);
        self.fault_generation = Some(live.device_generation);
        live.health
            .lock()
            .expect("frame health lock")
            .latch_uncaptured_error("synthetic RG3d shared-device fault");
    }

    fn record_rebuild(&mut self, fault_attempt: u64, rebuilt_generation: u64) {
        self.suppressed_attempts.push(fault_attempt);
        self.rebuilt_generation = Some(rebuilt_generation);
    }

    fn complete(&self, current_generation: u64) -> bool {
        let Some(injected) = self.injected_attempt else {
            return false;
        };
        let Some(fault_generation) = self.fault_generation else {
            return false;
        };
        let Some(rebuilt_generation) = self.rebuilt_generation else {
            return false;
        };
        rebuilt_generation > fault_generation
            && current_generation == rebuilt_generation
            && self.suppressed_attempts == [injected]
            && !self.surface_acquire_attempts.contains(&injected)
            && !self.surface_present_attempts.contains(&injected)
            && self
                .surface_present_attempts
                .iter()
                .any(|attempt| *attempt > injected)
    }

    fn write_receipt(&self) {
        let receipt = format!(
            concat!(
                "{{\n",
                "  \"gate\": \"RG3d shared-device rebuild-all\",\n",
                "  \"fault_injection\": \"synthetic host-latched uncaptured error\",\n",
                "  \"fault_attempt\": {},\n",
                "  \"fault_generation\": {},\n",
                "  \"rebuilt_generation\": {},\n",
                "  \"surface_acquire_attempts\": {:?},\n",
                "  \"surface_present_attempts\": {:?},\n",
                "  \"suppressed_attempts\": {:?},\n",
                "  \"fault_attempt_never_acquired\": true,\n",
                "  \"fault_attempt_never_presented\": true,\n",
                "  \"healthy_frame_presented_after_rebuild\": true,\n",
                "  \"preserved_host_resources\": [\"window\", \"surface\"],\n",
                "  \"recreated_shared_device_clients\": [\"adapter/device/queue\", \"renderling tenant\", \"optional DDA tenant\", \"netrender composer\", \"frame health and callbacks\"],\n",
                "  \"scope_limit\": \"proves the rebuild lifecycle after a synthetic shared fault; does not manufacture physical device loss\"\n",
                "}}\n"
            ),
            self.injected_attempt.expect("shared fault was injected"),
            self.fault_generation
                .expect("fault generation was recorded"),
            self.rebuilt_generation
                .expect("rebuilt generation was recorded"),
            self.surface_acquire_attempts,
            self.surface_present_attempts,
            self.suppressed_attempts,
        );
        let path = PathBuf::from(RG3D_REBUILD_RECEIPT);
        std::fs::create_dir_all(path.parent().expect("rebuild receipt directory"))
            .expect("create rebuild receipt directory");
        std::fs::write(&path, receipt).expect("write rebuild receipt");
        println!("RG3d headed rebuild-all receipt: {}", path.display());
    }
}

struct RoomApp {
    instance: wgpu::Instance,
    probe: Probe,
    trace_mode: bool,
    r1_mode: bool,
    live: Option<Live>,
    frames: u64,
    frame_attempts: u64,
    captured: bool,
    frame_us: Vec<u64>,
    #[cfg(feature = "r1-proof")]
    last_trace: Option<mesocosm_lens::BrickDiagnostics>,
    policy: PresentationPolicy,
    headed_validation_probe: Option<HeadedValidationProbe>,
    rebuild_probe: Option<RebuildProbe>,
    device_generation: u64,
    rebuild_requested: Option<(u64, SharedFault)>,
}

impl ApplicationHandler for RoomApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title("Paredros S0: the room probe")
            .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1]));
        let window = Arc::new(event_loop.create_window(attributes).expect("window"));
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("surface");
        let mut live = self.build_live(window, surface);
        configure(&mut live);
        live.window.request_redraw();
        self.live = Some(live);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.live.as_ref().is_none_or(|live| live.window.id() != id) {
            return;
        }
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

impl RoomApp {
    fn build_live(&mut self, window: Arc<Window>, surface: wgpu::Surface<'static>) -> Live {
        // One device for both tenants, chosen against the surface the window
        // actually got, so the picture and the presentation cannot end up on
        // different adapters. Initial boot and rebuild share this exact path.
        let handles = gpu::boot(&self.instance, Some(&surface));
        self.device_generation += 1;
        let device_generation = self.device_generation;
        let health = Arc::new(Mutex::new(FrameHealth::new(self.policy)));
        let uncaptured_health = Arc::clone(&health);
        handles.device.on_uncaptured_error(Arc::new(move |error| {
            uncaptured_health
                .lock()
                .expect("frame health lock")
                .latch_uncaptured_error(format!("{error:?}"));
        }));
        let lost_health = Arc::clone(&health);
        handles
            .device
            .set_device_lost_callback(move |reason, message| {
                lost_health
                    .lock()
                    .expect("frame health lock")
                    .latch_device_lost(format!("{reason:?}"), message);
            });
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let capabilities = surface.get_capabilities(&handles.adapter);
        // Prefer a linear format: the master is already encoded, and letting
        // the swapchain apply a second srgb transfer washes the room out.
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let tenant = Tenant::new(&handles, SIZE);
        #[cfg(feature = "r1-proof")]
        let dda = self
            .r1_mode
            .then(|| DdaTenant::new(&handles, &self.probe.room().ground, SIZE))
            .transpose()
            .expect("R1 brick map and tracer");
        let room = scene::room_vertices(self.probe.room());
        println!(
            "room mesh: {} triangles (device generation {device_generation})",
            room.len() / 3
        );
        #[cfg(feature = "r1-proof")]
        let adapter = handles.adapter.get_info().name;
        let composer = Composer::new(handles, SIZE);

        Live {
            window,
            surface,
            format,
            device,
            queue,
            tenant,
            #[cfg(feature = "r1-proof")]
            dda,
            composer,
            room,
            #[cfg(feature = "r1-proof")]
            adapter,
            health,
            pending_validation: VecDeque::new(),
            device_generation,
        }
    }

    fn rebuild_live(&mut self, fault_attempt: u64, fault: SharedFault) {
        let prior = self.live.take().expect("rebuild requires a live GPU stack");
        let old_generation = prior.device_generation;
        let (window, surface) = prior.into_host_surface();
        eprintln!("rebuilding shared GPU stack after generation {old_generation} fault: {fault:?}");
        let mut live = self.build_live(window, surface);
        assert!(live.device_generation > old_generation);
        configure(&mut live);
        if let Some(probe) = self.rebuild_probe.as_mut() {
            probe.record_rebuild(fault_attempt, live.device_generation);
        }
        live.window.request_redraw();
        self.live = Some(live);
    }

    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        if let Some((fault_attempt, fault)) = self.rebuild_requested.take() {
            self.rebuild_live(fault_attempt, fault);
        }
        let Some(live) = self.live.as_mut() else {
            return;
        };
        self.frame_attempts += 1;
        let frame_number = self.frame_attempts;
        if let Err(error) = live.device.poll(wgpu::PollType::Poll) {
            live.health
                .lock()
                .expect("frame health lock")
                .latch_poll_failure(error.to_string());
        }
        let mut pending = std::mem::take(&mut live.pending_validation);
        let waker = Waker::noop();
        let mut context = Context::from_waker(waker);
        let mut retained = VecDeque::with_capacity(pending.len());
        while let Some(mut pending_validation) = pending.pop_front() {
            match pending_validation.future.as_mut().poll(&mut context) {
                Poll::Ready(validation) => {
                    live.health
                        .lock()
                        .expect("frame health lock")
                        .finish_validation(
                            pending_validation.tenant_name,
                            pending_validation.producer_path,
                            pending_validation.frame,
                            validation.map(|error| format!("{error:?}")),
                        );
                },
                Poll::Pending => retained.push_back(pending_validation),
            }
        }
        live.pending_validation = retained;
        if let Some(probe) = self.rebuild_probe.as_mut() {
            probe.inject_once(live, frame_number);
        }
        match live
            .health
            .lock()
            .expect("frame health lock")
            .begin_frame(frame_number)
        {
            FrameDecision::Proceed => {},
            FrameDecision::Suppress => {
                if let Some(probe) = self.headed_validation_probe.as_mut() {
                    probe.record_suppressed(frame_number);
                }
                live.window.request_redraw();
                return;
            },
            FrameDecision::RebuildAll(fault) => {
                eprintln!("shared GPU fault requires rebuild-all: {fault:?}");
                self.rebuild_requested = Some((frame_number, fault));
                live.window.request_redraw();
                return;
            },
        }
        let began = Instant::now();

        // One tick of the fixed trace per frame. The trace is the input;
        // there is no other.
        self.probe.advance();

        let size = live.window.inner_size();
        let aspect = SIZE[0] as f32 / SIZE[1] as f32;
        let camera = scene::camera(
            self.probe.room(),
            self.probe.at(),
            self.probe.heading(),
            aspect,
        );
        let validation_scope = live.device.push_error_scope(wgpu::ErrorFilter::Validation);
        if let Some(probe) = self.headed_validation_probe.as_mut() {
            probe.inject_once(&live.device, frame_number);
        }
        let (master, opaque_receipt, scope_tenant, scope_path) = {
            #[cfg(feature = "r1-proof")]
            if let Some(dda) = live.dda.as_mut() {
                let pose = scene::body_pose(self.probe.at());
                let diagnostics = dda
                    .draw(camera.trace(aspect), &pose)
                    .expect("Paredros R1 DDA frame");
                self.last_trace = Some(diagnostics);
                (
                    live.composer.compose(
                        &scene::chrome(SIZE, self.probe.tick_count(), TICKS),
                        &dda.view,
                    ),
                    None,
                    "paredros-room",
                    "paredros::DdaTenant::draw",
                )
            } else {
                live.tenant.look(camera.projection, camera.view);
                live.tenant.set_room(&live.room, camera.eye);
                live.tenant
                    .set_body(&scene::body_vertices(self.probe.at()), camera.eye);
                let tenant_report = live.tenant.draw();
                let chrome = scene::chrome(SIZE, self.probe.tick_count(), TICKS);
                let (master, receipt) =
                    live.composer
                        .compose_opaque_tenant(&chrome, &live.tenant, tenant_report);
                (
                    master,
                    Some(receipt),
                    "paredros-room",
                    "renderling::Stage::encode_into (opaque)",
                )
            }
            #[cfg(not(feature = "r1-proof"))]
            {
                live.tenant.look(camera.projection, camera.view);
                live.tenant.set_room(&live.room, camera.eye);
                live.tenant
                    .set_body(&scene::body_vertices(self.probe.at()), camera.eye);
                let tenant_report = live.tenant.draw();
                let chrome = scene::chrome(SIZE, self.probe.tick_count(), TICKS);
                let (master, receipt) =
                    live.composer
                        .compose_opaque_tenant(&chrome, &live.tenant, tenant_report);
                (
                    master,
                    Some(receipt),
                    "paredros-room",
                    "renderling::Stage::encode_into (opaque)",
                )
            }
        };
        let validation_future = Box::pin(validation_scope.pop());
        if live.health.lock().expect("frame health lock").policy()
            == PresentationPolicy::AwaitedDiagnostic
        {
            let validation = pollster::block_on(validation_future);
            let decision = live
                .health
                .lock()
                .expect("frame health lock")
                .finish_validation(
                    scope_tenant,
                    scope_path,
                    frame_number,
                    validation.map(|error| format!("{error:?}")),
                );
            match decision {
                FrameDecision::Proceed => {},
                FrameDecision::Suppress => {
                    // Netrender's internal master bookkeeping has already
                    // run; this gate promises only native surface
                    // suppression, not a transactional rollback of it.
                    if let Some(probe) = self.headed_validation_probe.as_mut() {
                        probe.record_suppressed(frame_number);
                    }
                    live.window.request_redraw();
                    return;
                },
                FrameDecision::RebuildAll(fault) => {
                    eprintln!("shared GPU fault requires rebuild-all: {fault:?}");
                    self.rebuild_requested = Some((frame_number, fault));
                    live.window.request_redraw();
                    return;
                },
            }
        } else {
            live.pending_validation.push_back(PendingValidation {
                future: validation_future,
                tenant_name: scope_tenant.to_owned(),
                producer_path: scope_path.to_owned(),
                frame: frame_number,
            });
        }

        use wgpu::CurrentSurfaceTexture as Acquired;
        if let Some(probe) = self.headed_validation_probe.as_mut() {
            probe.surface_acquire_attempts.push(frame_number);
        }
        if let Some(probe) = self.rebuild_probe.as_mut() {
            probe.surface_acquire_attempts.push(frame_number);
        }
        match live.surface.get_current_texture() {
            Acquired::Success(frame) | Acquired::Suboptimal(frame) => {
                let target = frame.texture.create_view(&Default::default());
                live.composer.present(
                    &master,
                    &target,
                    live.format,
                    [size.width.max(1), size.height.max(1)],
                );
                live.window.pre_present_notify();
                live.queue.present(frame);
                if let Some(probe) = self.headed_validation_probe.as_mut() {
                    probe.surface_present_attempts.push(frame_number);
                }
                if let Some(probe) = self.rebuild_probe.as_mut() {
                    probe.surface_present_attempts.push(frame_number);
                }
            },
            Acquired::Outdated | Acquired::Lost => configure(live),
            Acquired::Timeout | Acquired::Occluded => {},
            Acquired::Validation => panic!("surface acquisition failed validation"),
        }

        let span = began.elapsed();
        self.frame_us.push(span.as_micros() as u64);
        self.frames += 1;

        if let Some(probe) = self.headed_validation_probe.as_ref() {
            let health = live.health.lock().expect("frame health lock");
            if let Some(validation) = probe.completed_record(&health) {
                probe.write_receipt(validation);
                event_loop.exit();
                return;
            }
            assert!(
                frame_number < 240,
                "RG3b headed probe did not complete after {frame_number} attempts"
            );
        }
        if let Some(probe) = self.rebuild_probe.as_ref() {
            if probe.complete(live.device_generation) {
                probe.write_receipt();
                event_loop.exit();
                return;
            }
            assert!(
                frame_number < 240,
                "RG3d headed rebuild probe did not complete after {frame_number} attempts"
            );
        }

        if self.trace_mode && !self.probe.running() && !self.captured {
            self.captured = true;
            report(
                &live.composer,
                &master,
                &self.probe,
                span,
                self.frames,
                ReportEvidence {
                    r1_mode: self.r1_mode,
                    frame_us: &self.frame_us,
                    #[cfg(feature = "r1-proof")]
                    trace: self.last_trace,
                    #[cfg(feature = "r1-proof")]
                    abi: live.dda.as_ref().map(DdaTenant::abi),
                    #[cfg(feature = "r1-proof")]
                    adapter: &live.adapter,
                    opaque_receipt: opaque_receipt.as_ref(),
                },
            );
            event_loop.exit();
            return;
        }
        if self.frames.is_multiple_of(60) {
            report_spans(&live.composer, span);
        }
        live.window.request_redraw();
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
            // wgpu 30 made surface color space explicit; Auto keeps the
            // pre-30 platform-chosen behavior.
            color_space: wgpu::SurfaceColorSpace::Auto,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
}

/// The receipt: the capture, its variety, the replay hash, and the spans.
struct ReportEvidence<'a> {
    r1_mode: bool,
    frame_us: &'a [u64],
    #[cfg(feature = "r1-proof")]
    trace: Option<mesocosm_lens::BrickDiagnostics>,
    #[cfg(feature = "r1-proof")]
    abi: Option<BrickAbi>,
    #[cfg(feature = "r1-proof")]
    adapter: &'a str,
    opaque_receipt: Option<&'a netrender::OpaqueTenantReceipt>,
}

fn report(
    composer: &Composer,
    master: &wgpu::Texture,
    probe: &Probe,
    span: std::time::Duration,
    frames: u64,
    evidence: ReportEvidence<'_>,
) {
    #[cfg(not(feature = "r1-proof"))]
    let _ = (evidence.r1_mode, evidence.frame_us, evidence.opaque_receipt);
    let capture = composer.capture(master);
    #[cfg(feature = "r1-proof")]
    let path = PathBuf::from(if evidence.r1_mode {
        R1_CAPTURE
    } else {
        CAPTURE
    });
    #[cfg(not(feature = "r1-proof"))]
    let path = PathBuf::from(CAPTURE);
    capture.write_png(&path).expect("write the receipt");
    assert!(
        !capture.is_trivial(),
        "the capture is one flat colour ({} distinct); nothing rendered",
        capture.distinct
    );

    println!("--- S0 room probe ---");
    println!("ticks: {} over {frames} frames", probe.tick_count());
    println!("final position: {:?}", probe.at());
    println!("position-log hash: {:#018x}", probe.hash());
    println!("ground hash: {:#018x}", probe.ground_hash());
    println!(
        "capture: {} ({}x{}, {} distinct colours)",
        path.display(),
        capture.size[0],
        capture.size[1],
        capture.distinct
    );
    report_spans(composer, span);
    #[cfg(feature = "r1-proof")]
    if let Some(receipt) = evidence.opaque_receipt {
        let rg3 = Rg3cReceipt {
            gate: "RG3c",
            tenant_name: &receipt.tenant_name,
            producer_path: &receipt.producer_path,
            fallback_count: receipt.fallback_count,
            scene_op_boundary: receipt.scene_op_boundary,
            shared_device: true,
            tenant_format: "Rgba8UnormSrgb",
            master_format: "Rgba8Unorm",
            logical_opaque_producer_boundaries: receipt.logical_opaque_producer_boundaries,
            graph_encoder_batches: receipt.graph_encoder_batches,
            graph_submission_boundaries: receipt.graph_submission_boundaries,
            caller_reported_physical_submission_count: receipt
                .caller_reported_physical_submission_count,
            capture_size: capture.size,
            capture_distinct_colours: capture.distinct,
            capture: path.display().to_string(),
            dependency_provenance: "paredros-room -> netrender RG3a d08f713d8",
            plan_dump: &receipt.logical_plan_dump,
        };
        let json = serde_json::to_string_pretty(&rg3).expect("RG3c receipt JSON");
        std::fs::write(RG3_RECEIPT, &json).expect("write RG3c receipt");
        println!("{json}");
    }
    #[cfg(feature = "r1-proof")]
    if evidence.r1_mode {
        let mut spans = evidence.frame_us.to_vec();
        spans.sort_unstable();
        let diagnostics = evidence.trace.expect("R1 trace diagnostics");
        let receipt = R1Receipt {
            gate: "R1",
            vessel: "paredros",
            camera_profile: "close-perspective-room",
            traversal_implementation: "modulus::BRICK_DDA_WGSL via mesocosm_lens::BrickTracer",
            adapter: evidence.adapter,
            size: SIZE,
            frames: spans.len(),
            frame_us_min: spans[0],
            frame_us_median: spans[spans.len() / 2],
            frame_us_max: *spans.last().expect("non-empty R1 spans"),
            tracer_cpu_prepare_us: diagnostics.cpu_prepare_us,
            steady_brick_upload_bytes: diagnostics.brick_upload_bytes,
            steady_uniform_upload_bytes: diagnostics.uniform_upload_bytes,
            brick_abi: evidence.abi.expect("R1 brick ABI"),
            capture: path.display().to_string(),
            capture_distinct_colours: capture.distinct,
            ground_hash: format!("{:#018x}", probe.ground_hash()),
            position_log_hash: format!("{:#018x}", probe.hash()),
        };
        let json = serde_json::to_string_pretty(&receipt).expect("R1 receipt JSON");
        std::fs::write(R1_RECEIPT, &json).expect("write R1 receipt");
        println!("{json}");
    }
}

#[cfg(feature = "r1-proof")]
#[derive(serde::Serialize)]
struct R1Receipt<'a> {
    gate: &'static str,
    vessel: &'static str,
    camera_profile: &'static str,
    traversal_implementation: &'static str,
    adapter: &'a str,
    size: [u32; 2],
    frames: usize,
    frame_us_min: u64,
    frame_us_median: u64,
    frame_us_max: u64,
    tracer_cpu_prepare_us: u64,
    steady_brick_upload_bytes: u64,
    steady_uniform_upload_bytes: u64,
    brick_abi: BrickAbi,
    capture: String,
    capture_distinct_colours: usize,
    ground_hash: String,
    position_log_hash: String,
}

#[cfg(feature = "r1-proof")]
#[derive(serde::Serialize)]
struct Rg3cReceipt<'a> {
    gate: &'static str,
    tenant_name: &'a str,
    producer_path: &'a str,
    fallback_count: u64,
    scene_op_boundary: usize,
    shared_device: bool,
    tenant_format: &'static str,
    master_format: &'static str,
    logical_opaque_producer_boundaries: usize,
    graph_encoder_batches: usize,
    graph_submission_boundaries: usize,
    caller_reported_physical_submission_count: Option<u64>,
    capture_size: [u32; 2],
    capture_distinct_colours: usize,
    capture: String,
    dependency_provenance: &'static str,
    plan_dump: &'a str,
}

fn report_spans(composer: &Composer, span: std::time::Duration) {
    print!("frame span: probe_frame {:?}", span);
    match composer.net.last_frame_timings() {
        Some(timings) => {
            print!(" | netrender total {:?}", timings.total);
            for named in &timings.spans {
                print!(" | {} {:?}", named.name, named.duration);
            }
        },
        None => print!(" | netrender reported no timings"),
    }
    println!();
}
