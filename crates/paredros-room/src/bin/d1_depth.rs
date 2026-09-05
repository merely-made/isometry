// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The headed D1 receipt: raymarch depth composed with Renderling.
//!
//! The renderling tenant draws the body and three cyan witness pillars with
//! its depth stored; the shared brick traversal then draws the room's rock
//! into the same colour target against that depth. Occlusion must hold in
//! both directions: the pillar before the wall covers rock, the floor covers
//! the buried pillar base, and the pillar beyond the wall never appears.
//! The run replays the fixed S0 trace, judges the final frame, and writes
//! the capture and JSON receipt.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use paredros_room::gpu::{self, BrickAbi, Composer, JoinTenant, SIZE, Tenant};
use paredros_room::room::SEED;
use paredros_room::scene::Pillar;
use paredros_room::{Probe, TICKS, scene};
use renderling::glam::{Mat4, Vec4};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

const CAPTURE: &str = r"C:\Users\mark_\Code\testing\paredros\d1_depth.png";
const RECEIPT: &str = r"C:\Users\mark_\Code\testing\paredros\d1_depth.json";
/// The chrome bar's height plus a margin; pixel judgments stay below it.
const CHROME: u32 = 42;

fn main() {
    let probe = Probe::new(SEED).expect("the room probe grows its world");
    println!(
        "room at {:?}, stance {:?}, ground hash {:#018x}",
        probe.room().centre,
        probe.at(),
        probe.ground_hash()
    );

    let event_loop = EventLoop::new().expect("winit event loop");
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = DepthApp {
        instance: wgpu::Instance::new(
            wgpu::InstanceDescriptor::new_without_display_handle_from_env(),
        ),
        probe,
        live: None,
        frames: 0,
        captured: false,
        frame_us: Vec::new(),
        last_trace: None,
        judged: None,
    };
    event_loop.run_app(&mut app).expect("winit run");
}

struct Live {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    format: wgpu::TextureFormat,
    device: wgpu::Device,
    queue: wgpu::Queue,
    tenant: Tenant,
    join: JoinTenant,
    composer: Composer,
    pillars: [Pillar; 3],
    adapter: String,
}

struct DepthApp {
    instance: wgpu::Instance,
    probe: Probe,
    live: Option<Live>,
    frames: u64,
    captured: bool,
    frame_us: Vec<u64>,
    last_trace: Option<mesocosm_lens::BrickDiagnostics>,
    judged: Option<JudgedFrame>,
}

/// The frame the receipt judges: the first of the fixed trace whose camera
/// holds every probe well inside the picture with a clear sightline. The
/// trace's final corner frames the witnesses too steeply to keep their
/// floor junctions in view, so the frame is selected by the same predicate
/// the judgment needs rather than hand-picked.
struct JudgedFrame {
    tick: usize,
    clip: Mat4,
    capture: gpu::Capture,
}

impl ApplicationHandler for DepthApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.live.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Paredros D1: raymarch depth composed with Renderling")
            .with_inner_size(PhysicalSize::new(SIZE[0], SIZE[1]));
        let window = Arc::new(event_loop.create_window(attributes).expect("window"));
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("surface");

        let handles = gpu::boot(&self.instance, Some(&surface));
        let device = handles.device.clone();
        let queue = handles.queue.clone();
        let capabilities = surface.get_capabilities(&handles.adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(|f| !f.is_srgb())
            .unwrap_or(capabilities.formats[0]);

        let tenant = Tenant::new(&handles, SIZE);
        let join =
            JoinTenant::new(&handles, &self.probe.room().ground, SIZE).expect("D1 join tenant");
        let pillars = scene::d1_pillars(self.probe.room());
        let adapter = handles.adapter.get_info().name;
        let composer = Composer::new(handles, SIZE);

        let mut live = Live {
            window,
            surface,
            format,
            device,
            queue,
            tenant,
            join,
            composer,
            pillars,
            adapter,
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
            }
            WindowEvent::RedrawRequested => self.frame(event_loop),
            _ => {}
        }
    }
}

impl DepthApp {
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        let Some(live) = self.live.as_mut() else {
            return;
        };
        let began = Instant::now();
        self.probe.advance();

        let size = live.window.inner_size();
        let aspect = SIZE[0] as f32 / SIZE[1] as f32;
        let camera = scene::camera(
            self.probe.room(),
            self.probe.at(),
            self.probe.heading(),
            aspect,
        );

        // Raster half first, depth stored: the witnesses and the body, but
        // never the room's rock, which is the raymarch's to draw.
        live.tenant.look(camera.projection, camera.view);
        live.tenant
            .set_room(&scene::pillar_vertices(&live.pillars), camera.eye);
        live.tenant
            .set_body(&scene::body_vertices(self.probe.at()), camera.eye);
        let _ = live.tenant.draw();

        // Then the join, against the same matrix the raster projected with.
        // The depth view is fetched after the draw because the stage may
        // have replaced its depth texture during it.
        let clip = (camera.projection * camera.view).to_cols_array_2d();
        let depth_view = live.tenant.depth_view();
        let diagnostics = live
            .join
            .draw_over(&live.tenant.view, &depth_view, camera.trace(aspect), clip)
            .expect("D1 joined frame");
        self.last_trace = Some(diagnostics);

        let chrome = scene::chrome(SIZE, self.probe.tick_count(), TICKS);
        let master = live.composer.compose(&chrome, &live.tenant.view);

        use wgpu::CurrentSurfaceTexture as Acquired;
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
            }
            Acquired::Outdated | Acquired::Lost => configure(live),
            Acquired::Timeout | Acquired::Occluded => {}
            Acquired::Validation => panic!("surface acquisition failed validation"),
        }

        // The first frame whose camera can judge every probe becomes the
        // judged frame; the trace still runs to its end for the replay
        // hash and the span record. The capture stalls on readback, so its
        // frame span is an expected outlier.
        if self.judged.is_none() {
            let clip = camera.projection * camera.view;
            let probes = probe_defs(&live.pillars);
            let body = body_box(self.probe.at());
            if probes.iter().all(|probe| {
                probe.frames_well(clip, camera.eye.to_array(), &live.pillars, body)
            }) {
                self.judged = Some(JudgedFrame {
                    tick: self.probe.tick_count(),
                    clip,
                    capture: live.composer.capture(&master),
                });
            }
        }

        self.frame_us.push(began.elapsed().as_micros() as u64);
        self.frames += 1;

        if !self.probe.running() && !self.captured {
            self.captured = true;
            report(
                live,
                &self.probe,
                self.judged
                    .as_ref()
                    .expect("no frame of the fixed trace could hold every D1 probe"),
                &self.frame_us,
                self.last_trace,
            );
            event_loop.exit();
            return;
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
            color_space: wgpu::SurfaceColorSpace::Auto,
            desired_maximum_frame_latency: 2,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        },
    );
}

/// One judged sightline: a named world point on a witness surface and what
/// the pixel it projects to must show. Never a hand-tuned pixel.
struct ProbeDef {
    world: [f32; 3],
    expect_cyan: bool,
    why: &'static str,
    /// Which pillar owns the probed surface; its box is not an obstacle.
    owner: usize,
    /// For a visible-face probe, the face rectangle around the point, so
    /// framing can require the face to project large enough to hold the
    /// sample window. Covered probes need no such guard: the floor around
    /// them is what the window should see.
    face: Option<[[f32; 3]; 4]>,
}

/// The four sightlines, off the witness table. West faces the eye's side
/// of the room; the buried probes bracket the floor line on one face.
fn probe_defs(pillars: &[Pillar; 3]) -> [ProbeDef; 4] {
    let [front, buried, hidden] = pillars;
    let west = |pillar: &Pillar, y: f32| {
        [
            pillar.min[0] as f32,
            pillar.min[1] as f32 + y,
            pillar.min[2] as f32 + 0.5,
        ]
    };
    let west_face = |pillar: &Pillar, ylo: f32, yhi: f32| {
        let (x, zlo, zhi) = (
            pillar.min[0] as f32,
            pillar.min[2] as f32,
            (pillar.min[2] + pillar.extent[2]) as f32,
        );
        let (ylo, yhi) = (pillar.min[1] as f32 + ylo, pillar.min[1] as f32 + yhi);
        Some([
            [x, ylo, zlo],
            [x, ylo, zhi],
            [x, yhi, zlo],
            [x, yhi, zhi],
        ])
    };
    [
        ProbeDef {
            world: west(front, 1.5),
            expect_cyan: true,
            why: "the pillar before the wall must cover raymarched rock",
            owner: 0,
            face: west_face(front, 0.0, front.extent[1] as f32),
        },
        ProbeDef {
            world: west(buried, 2.0),
            expect_cyan: true,
            why: "the buried pillar's open span must stay visible (positive control)",
            owner: 1,
            face: west_face(buried, 1.0, buried.extent[1] as f32),
        },
        ProbeDef {
            world: west(buried, 0.8),
            expect_cyan: false,
            why: "the floor must cover the buried pillar base",
            owner: 1,
            face: None,
        },
        ProbeDef {
            world: [
                hidden.min[0] as f32 + 0.5,
                (hidden.min[1] + hidden.extent[1]) as f32,
                hidden.min[2] as f32 + 0.5,
            ],
            expect_cyan: false,
            why: "the wholly sunken pillar must stay invisible under the floor",
            owner: 2,
            face: None,
        },
    ]
}

impl ProbeDef {
    /// Whether this frame's camera can judge the probe: the point projects
    /// well inside the picture, the sightline from the eye crosses neither
    /// other pillar nor the body, and a visible face projects large enough
    /// to hold the sample window. Room rock needs no test here: the carved
    /// chamber is convex, so a segment between in-room points stays in
    /// air, and one to a sub-floor point crosses only the floor meant to
    /// cover it.
    fn frames_well(
        &self,
        clip: Mat4,
        eye: [f32; 3],
        pillars: &[Pillar; 3],
        body: ([f32; 3], [f32; 3]),
    ) -> bool {
        let Some(pixel) = project(clip, self.world) else {
            return false;
        };
        if pixel[0] < 6.0
            || pixel[0] > (SIZE[0] - 6) as f32
            || pixel[1] < (CHROME + 6) as f32
            || pixel[1] > (SIZE[1] - 6) as f32
        {
            return false;
        }
        let mut obstacles: Vec<([f32; 3], [f32; 3])> = pillars
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != self.owner)
            .map(|(_, pillar)| pillar_box(pillar))
            .collect();
        obstacles.push(body);
        if obstacles
            .iter()
            .any(|obstacle| segment_hits_box(eye, self.world, *obstacle))
        {
            return false;
        }
        if let Some(face) = self.face {
            let mut low = [f32::MAX, f32::MAX];
            let mut high = [f32::MIN, f32::MIN];
            for corner in face {
                let Some(at) = project(clip, corner) else {
                    return false;
                };
                low = [low[0].min(at[0]), low[1].min(at[1])];
                high = [high[0].max(at[0]), high[1].max(at[1])];
            }
            if (high[0] - low[0]).min(high[1] - low[1]) < 20.0 {
                return false;
            }
        }
        true
    }
}

fn project(clip: Mat4, world: [f32; 3]) -> Option<[f32; 2]> {
    let projected = clip * Vec4::new(world[0], world[1], world[2], 1.0);
    if projected.w <= 0.0 {
        return None;
    }
    Some([
        (projected.x / projected.w * 0.5 + 0.5) * SIZE[0] as f32,
        (1.0 - (projected.y / projected.w * 0.5 + 0.5)) * SIZE[1] as f32,
    ])
}

fn pillar_box(pillar: &Pillar) -> ([f32; 3], [f32; 3]) {
    (
        [
            pillar.min[0] as f32,
            pillar.min[1] as f32,
            pillar.min[2] as f32,
        ],
        [
            (pillar.min[0] + pillar.extent[0]) as f32,
            (pillar.min[1] + pillar.extent[1]) as f32,
            (pillar.min[2] + pillar.extent[2]) as f32,
        ],
    )
}

fn body_box(at: [i32; 3]) -> ([f32; 3], [f32; 3]) {
    (
        [at[0] as f32, at[1] as f32, at[2] as f32],
        [
            (at[0] + 1) as f32,
            at[1] as f32 + mesocosm_core::places::WALKER_HEIGHT as f32,
            (at[2] + 1) as f32,
        ],
    )
}

/// Whether the open segment from `from` toward `to` crosses the box before
/// reaching `to`. The endpoint itself may lie on the box.
fn segment_hits_box(from: [f32; 3], to: [f32; 3], (low, high): ([f32; 3], [f32; 3])) -> bool {
    let mut enter: f32 = 0.0;
    let mut exit: f32 = 0.999;
    for axis in 0..3 {
        let direction = to[axis] - from[axis];
        if direction.abs() < 1e-6 {
            if from[axis] < low[axis] || from[axis] > high[axis] {
                return false;
            }
            continue;
        }
        let a = (low[axis] - from[axis]) / direction;
        let b = (high[axis] - from[axis]) / direction;
        enter = enter.max(a.min(b));
        exit = exit.min(a.max(b));
    }
    enter <= exit
}

/// Whether a pixel can only be a witness pillar. Cyan-dominant survives the
/// torch falloff and renderling's tonemap; nothing else in the frame is in
/// the cyan family.
fn is_cyan(pixel: &[u8]) -> bool {
    let (r, g, b) = (pixel[0] as u32, pixel[1] as u32, pixel[2] as u32);
    g > 60 && g * 10 > r * 16 && b * 10 > r * 16
}

/// How many of the 3x3 window around a probe pixel are witness-coloured.
fn cyan_at(capture: &gpu::Capture, pixel: [u32; 2]) -> u32 {
    let mut cyan = 0;
    for y in pixel[1] - 1..=pixel[1] + 1 {
        for x in pixel[0] - 1..=pixel[0] + 1 {
            let at = ((y * capture.size[0] + x) * 4) as usize;
            if is_cyan(&capture.pixels[at..at + 4]) {
                cyan += 1;
            }
        }
    }
    cyan
}

fn report(
    live: &Live,
    probe: &Probe,
    judged: &JudgedFrame,
    frame_us: &[u64],
    trace: Option<mesocosm_lens::BrickDiagnostics>,
) {
    let capture = &judged.capture;
    let path = PathBuf::from(CAPTURE);
    capture.write_png(&path).expect("write the D1 capture");
    assert!(
        !capture.is_trivial(),
        "the capture is one flat colour ({} distinct); nothing rendered",
        capture.distinct
    );

    // Judge the frame that framed every probe. What each probe sees is
    // settled by the depth join alone: framing already required clear
    // sightlines past the other witnesses and the body.
    let judge = |def: &ProbeDef| -> Judgment {
        let at = project(judged.clip, def.world).expect("a framed probe projects");
        let pixel = [at[0].round() as u32, at[1].round() as u32];
        let cyan = cyan_at(capture, pixel);
        assert!(
            if def.expect_cyan { cyan == 9 } else { cyan == 0 },
            "{}: probe {:?} at pixel {pixel:?} on tick {} saw {cyan}/9 cyan, expected {}",
            def.why,
            def.world,
            judged.tick,
            if def.expect_cyan { "9/9" } else { "0/9" },
        );
        Judgment {
            world: def.world,
            pixel,
            cyan_of_nine: cyan,
            expected_cyan: def.expect_cyan,
        }
    };
    let [front_def, open_def, base_def, hidden_def] = probe_defs(&live.pillars);
    let front_probe = judge(&front_def);
    let buried_open_probe = judge(&open_def);
    let buried_base_probe = judge(&base_def);
    let hidden_probe = judge(&hidden_def);
    let [front, buried, hidden] = live.pillars;

    let mut spans = frame_us.to_vec();
    spans.sort_unstable();
    let diagnostics = trace.expect("D1 trace diagnostics");
    let receipt = D1Receipt {
        gate: "D1",
        vessel: "paredros",
        mechanism: "renderling raster first with stored Depth32Float; \
                    modulus::BRICK_DDA_WGSL via mesocosm_lens::BrickTracer::encode_with_depth \
                    writes frag_depth from the shared clip_from_world under LessEqual",
        adapter: &live.adapter,
        size: SIZE,
        judged_tick: judged.tick,
        frames: spans.len(),
        frame_us_min: spans[0],
        frame_us_median: spans[spans.len() / 2],
        frame_us_max: *spans.last().expect("non-empty D1 spans"),
        tracer_cpu_prepare_us: diagnostics.cpu_prepare_us,
        steady_brick_upload_bytes: diagnostics.brick_upload_bytes,
        steady_uniform_upload_bytes: diagnostics.uniform_upload_bytes,
        steady_resource_creations: diagnostics.resource_creations,
        brick_abi: live.join.abi(),
        witnesses: [
            Witness {
                role: front.role,
                probes: vec![front_probe],
            },
            Witness {
                role: buried.role,
                probes: vec![buried_open_probe, buried_base_probe],
            },
            Witness {
                role: hidden.role,
                probes: vec![hidden_probe],
            },
        ],
        capture: path.display().to_string(),
        capture_distinct_colours: capture.distinct,
        ground_hash: format!("{:#018x}", probe.ground_hash()),
        position_log_hash: format!("{:#018x}", probe.hash()),
    };
    let json = serde_json::to_string_pretty(&receipt).expect("D1 receipt JSON");
    std::fs::write(RECEIPT, &json).expect("write D1 receipt");
    println!("{json}");
    print!("frame span final");
    match live.composer.net.last_frame_timings() {
        Some(timings) => println!(" | netrender total {:?}", timings.total),
        None => println!(" | netrender reported no timings"),
    }
}

#[derive(serde::Serialize)]
struct Witness {
    role: &'static str,
    probes: Vec<Judgment>,
}

#[derive(Clone, Copy, serde::Serialize)]
struct Judgment {
    world: [f32; 3],
    pixel: [u32; 2],
    cyan_of_nine: u32,
    expected_cyan: bool,
}

#[derive(serde::Serialize)]
struct D1Receipt<'a> {
    gate: &'static str,
    vessel: &'static str,
    mechanism: &'static str,
    adapter: &'a str,
    size: [u32; 2],
    /// The trace tick whose frame every probe fit inside and was judged on.
    judged_tick: usize,
    frames: usize,
    frame_us_min: u64,
    frame_us_median: u64,
    frame_us_max: u64,
    tracer_cpu_prepare_us: u64,
    steady_brick_upload_bytes: u64,
    steady_uniform_upload_bytes: u64,
    steady_resource_creations: u32,
    brick_abi: BrickAbi,
    witnesses: [Witness; 3],
    capture: String,
    capture_distinct_colours: usize,
    ground_hash: String,
    position_log_hash: String,
}
