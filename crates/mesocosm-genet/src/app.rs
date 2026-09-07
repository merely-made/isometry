// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The window, the surface, and the loop.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use mesocosm_core::{Kingdom, Organism, Signal, Stage};
use mesocosm_mesh::{BodyMesh, VolumeMap, VolumeSource, mesh_body};
use mesocosm_render::{SceneItem, deadened, kingdom_colour};
use mesocosm_runtime::Runtime;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

use crate::chrome::Chrome;
use crate::section::{self, Pan, Section, SectionFrame};

pub mod actions;
mod config;
mod content;
mod devtime;
mod devworld;
pub mod drive;
mod expression;
mod follow;
mod frame;
mod grafting;
mod inspection;
mod receipts;
mod setup;

pub use actions::{NEIGHBOUR_MIN_MG, PUMP_STEPS_PER_FRAME};
pub use config::HostConfig;
pub use devworld::DEV_PLACE_MG;

/// Recorded steps a replay frame drives. Exact, not throttled: the queue is
/// topped up with precisely this many intents and precisely this many steps
/// run, so a replay's trace is the recording's trace and nothing else.
const REPLAY_STEPS_PER_FRAME: usize = 4;

/// Steps the dev "step N" key runs at once (DT1). One named constant, per
/// the plan, rather than a literal at each call site. See [`devtime`] for
/// the rest of DT1's host-only time control.
pub const DEV_STEP_N: u64 = 10;

/// One drawable thing in the world: its geometry, where it sits, and how
/// brightly it reads.
type Placed = (BodyMesh, [i32; 3], f32, bool, [f32; 3], f32);

/// How an organism should read on screen.
///
/// Colour comes from its **guise**, never its kingdom, so a simulacrum is
/// drawn as the thing it pretends to be. Size tracks mass, so growth is
/// visible and a sapling does not look like a giant. The dead drain toward
/// grey.
pub(crate) fn look_of(organism: &Organism) -> ([f32; 3], f32) {
    let guise = match organism.guise {
        Kingdom::Producer => 0,
        Kingdom::Consumer => 1,
        Kingdom::Decomposer => 2,
    };
    let colour = kingdom_colour(guise);

    match organism.stage {
        // Growth you can see: a juvenile is visibly smaller than what it will
        // become, which is what makes waiting a decision.
        Stage::Juvenile => (
            [
                colour[0] * 0.85 + 0.1,
                colour[1] * 0.85 + 0.1,
                colour[2] * 0.85 + 0.1,
            ],
            0.55,
        ),
        Stage::Mature => (colour, 1.0),
        Stage::Carrion => (deadened(colour), 0.8),
        Stage::Spent => (deadened(colour), 0.3),
    }
}

pub struct Host {
    config: HostConfig,
    runtime: Runtime,
    volumes: VolumeMap,
    content: Option<mesocosm_mesh::content::ContentPack>,
    /// Where the section sits relative to the critter it follows. Presentation
    /// only; it never reaches an intent, so it cannot reach the trace.
    pan: Pan,
    habitat: Option<mesocosm_core::world::TerrariumHabitat>,
    window: Option<Arc<Window>>,
    gpu: Option<Gpu>,
    adapter: Option<wgpu::AdapterInfo>,
    last: Option<Instant>,
    frames: u32,
    steps: u64,
    /// How much of a replay's trace has been fed in.
    cursor: usize,
    finished: bool,
    /// Parts of the played body the lens's capsule budget could not carry on
    /// the last drawn frame. Nonzero means a truncated player, which the
    /// receipt says out loud — before DC3 it meant no player at all.
    body_capsules_dropped: u32,
    /// Which row of the trait board the cursor is on. (PE3b)
    ///
    /// Host state and nothing else: moving it sends no intent, so it cannot
    /// reach the trace or the hash. It resets whenever a review opens, because
    /// a cursor left pointing at a row from the last boundary would be pointing
    /// at a candidate this one may not offer.
    board_row: usize,
    /// Process exit code. Nonzero only when a replay landed on a different
    /// hash than the one its trace recorded.
    code: i32,
    /// Host-only time control (DT1); see `app::devtime`. None of it is in
    /// the snapshot, the trace, or the hash.
    dev_paused: bool,
    /// Index into devtime's speed ladder.
    dev_speed_idx: usize,
    /// Ticks taken through the dev step keys this session, shown on the dev
    /// lane. `steps` already counts them; this counts how many arrived by
    /// hand.
    dev_manual_steps: u64,
    /// The critter the section's slab is centred on, when it is not the one
    /// under the hand (DT2). `None` means the controlled critter, which is
    /// where the camera goes back to when its target dies or `M` is pressed.
    ///
    /// Host state, outside the snapshot: it moves the camera and nothing else,
    /// so it can no more reach the trace than the pan beside it. See
    /// `app::follow`.
    follow: Option<mesocosm_core::OrganismId>,
    /// A followed critter that stopped being one, kept so the tile can report
    /// it after follow has snapped back.
    follow_lost: Option<mesocosm_views::Lost>,
    inspection: inspection::Inspection,
    grafting: grafting::Grafting,
    /// The scenario driving this run, when `--scenario` gave it one. (DT4)
    ///
    /// `None` for an ordinary session at the keyboard, and for a bare
    /// `--replay`. While it is `Some`, the scenario decides when the run ends;
    /// see [`drive`].
    scenario: Option<genet_probe::Scenario>,
    /// Semantic events since the driver last drained them. Presentation of a
    /// sort: it is written from what the world already answered and read only
    /// by `assert event`, so nothing in it reaches an intent.
    events: Vec<String>,
    /// Scripted steps still to take, off the clock. (DT4)
    ///
    /// Where `--auto-eat` and `--record-demo` both went: a scenario asks for a
    /// stretch of hunting or a stretch of the recorded demo by name, and gets
    /// exactly the ticks it asked for rather than however many a frame rate
    /// happened to deliver. See [`actions::Pump`].
    pump: Option<actions::Pump>,
    /// Intents the driver has already turned into events, so a frame and an
    /// action cannot record the same outcome twice.
    noted: usize,
    /// The offspring the last accepted forced birth produced, so a scenario can
    /// follow what it just made. (DT4)
    last_child: Option<mesocosm_core::OrganismId>,
}

struct Gpu {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    /// The main view: the ruled side-on section over the live Ground.
    section: Section,
    /// Both chrome lanes and the device they share. `None` when netrender
    /// declined the device; the game runs chromeless rather than not at all.
    chrome: Option<Lanes>,
    /// The most recently completed Netrender tenant envelope. This is frame
    /// provenance for the played receipt, never simulation state.
    last_tenant_receipt: Option<netrender_graph::OpaqueTenantReceipt>,
}

/// The five chrome lanes over one shared-device Chrome and one blend pass: the
/// painted minimap, the cambium vitals panel, the individual checkpoint, the
/// trait board, and the dev lane. The checkpoint and the board draw only while
/// the world is holding at a question, and never both: a lineage checkpoint is
/// the board's. The dev lane draws only while `--dev` is set.
pub(crate) struct Lanes {
    device: Chrome,
    hud: crate::hud::Hud,
    vitals: crate::vitals::VitalsChrome,
    checkpoint: crate::succession::SuccessionChrome,
    board: crate::review::BoardChrome,
    grafting: crate::body_menu::BodyMenuChrome,
    dev: crate::dev::DevChrome,
}

/// The shipped pack's root, derived from this crate's own location rather than
/// a hardcoded path. (PE3b)
///
/// `<repo>/crates/mesocosm-genet` up two, then down into the pack the game
/// ships. What the host reads out of it is the review's second proposal source
/// and nothing else: the ruleset a world runs is still the core's own.
fn pack_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(|repo| repo.join("packs").join("mesocosm"))
        .unwrap_or_else(|| PathBuf::from("packs/mesocosm"))
}

impl Host {
    pub fn new(mut config: HostConfig) -> Self {
        if let Some(selection) = config.effective_start() {
            let (seed, organisms) = (selection.request.seed, selection.request.organisms);
            config.seed = seed;
            config.organisms = organisms;
        }
        let (runtime, content, volumes) = content::start(&config).unwrap_or_else(|why| {
            eprintln!("body content: {why}");
            std::process::exit(1);
        });
        // The pack's authored expressions, for the review. A pack that will not
        // load is a diagnostic and not a reason to refuse to run: the board
        // simply shows one proposal source per row, which is what it does
        // wherever no pack expression applies anyway.
        let runtime = match mesocosm_runtime::Authored::load(&pack_root()) {
            Ok(authored) => runtime.with_authored(authored),
            Err(why) => {
                eprintln!("pack: {}", why.words());
                runtime
            },
        };
        // A fixed follow target for an unattended capture run (DT2). It is
        // only where the camera starts: the ordinary keys move it from here,
        // and a target that is not alive is reported and dropped on the first
        // frame like any other.
        let follow = config.follow.map(mesocosm_core::OrganismId);
        // Parsed once, here, so a typo in a scenario stops the run before a
        // window opens rather than three verbs into it. (DT4)
        let scenario = match config.scenario.as_deref().map(genet_probe::Scenario::parse) {
            Some(Ok(scenario)) => Some(scenario),
            Some(Err(why)) => {
                eprintln!("scenario: {why}");
                std::process::exit(1);
            },
            None => None,
        };
        let habitat = (config.effective_scene() != crate::played::SceneMode::Ecology)
            .then(|| section::framed_habitat(runtime.world()));
        Self {
            habitat,
            config,
            runtime,
            volumes,
            content,
            pan: Pan::default(),
            window: None,
            gpu: None,
            adapter: None,
            last: None,
            frames: 0,
            steps: 0,
            cursor: 0,
            finished: false,
            body_capsules_dropped: 0,
            board_row: 0,
            code: 0,
            dev_paused: false,
            dev_speed_idx: devtime::DEV_SPEED_DEFAULT_IDX,
            dev_manual_steps: 0,
            follow,
            follow_lost: None,
            inspection: inspection::Inspection::default(),
            grafting: grafting::Grafting::default(),
            scenario,
            events: Vec::new(),
            pump: None,
            noted: 0,
            last_child: None,
        }
    }

    /// Runs the host to completion. The `i32` is the process exit code.
    pub fn run(config: HostConfig) -> Result<i32, winit::error::EventLoopError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut host = Self::new(config);
        event_loop.run_app(&mut host)?;
        Ok(host.code)
    }

    /// Builds the scene the HUD's backdrop draws: the critter where it stands,
    /// plus every organism where it lies. Organisms out of reach are dimmed, so
    /// what can be eaten reads without a UI element.
    fn scene(&self) -> Option<(BodyMesh, Vec<Placed>)> {
        let world = self.runtime.world();
        let body = mesh_body(world.body()?, &self.volumes).ok()?;

        let mut loose = Vec::with_capacity(world.organisms.len());
        for organism in &world.organisms {
            let Some(volume) = self.volumes.volume(organism.volume()) else {
                continue;
            };
            // Ask the world rather than mirroring a constant. The host used to
            // keep its own copy of the reach rule, which was wrong the moment
            // reach became anatomy.
            let in_reach = world.in_reach(organism.position);
            let reach_tint = if in_reach { 1.0 } else { 0.45 };
            let (colour, scale) = look_of(organism);
            loose.push((
                BodyMesh::single(organism.volume(), volume),
                organism.position,
                reach_tint,
                organism.signal == Signal::Warning,
                colour,
                scale,
            ));
        }
        Some((body, loose))
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        if let Some(gpu) = &mut self.gpu {
            gpu.config.width = self.config.width;
            gpu.config.height = self.config.height;
            gpu.surface.configure(&gpu.device, &gpu.config);
            gpu.section.resize(self.config.width, self.config.height);
        }
    }

    /// Steps the world. A replay feeds recorded intents and ignores the clock;
    /// a played session converts elapsed wall time into whole fixed steps.
    /// Returns true when a replay has reached the end of its trace.
    fn advance(&mut self) -> bool {
        if self.grafting.open {
            self.last = None;
            return false;
        }
        if let Some(replay) = &self.config.replay {
            let end = (self.cursor + REPLAY_STEPS_PER_FRAME).min(replay.intents.len());
            let batch = end - self.cursor;
            for intent in &replay.intents[self.cursor..end] {
                self.runtime.queue(intent.clone());
            }
            self.cursor = end;
            // A checkpoint may hold part of a batch, so count what actually
            // ran. The recording answered its own questions, so the answer is
            // already the next intent in the queue and the hold lifts on the
            // step it is reached.
            let ran = self.runtime.step(batch as u64);
            self.steps += ran;
            // A trace that ends still holding at a question ends there, and the
            // frame that ends it is the question on screen.
            return self.cursor >= replay.intents.len()
                && (ran == 0 || self.runtime.queued_len() == 0);
        }

        // A scripted stretch, while a scenario is pumping one (DT4). Off the
        // clock and one intent per step, exactly as the headless recording
        // runs it — which is what lets the two reach the same hash, and what
        // makes a scripted run the same run at any frame rate.
        if self.pump.is_some() {
            self.pump_frame();
            return false;
        }

        let now = Instant::now();
        let elapsed_us = self
            .last
            .map(|then| now.duration_since(then).as_micros() as u64)
            .unwrap_or(0);
        self.last = Some(now);
        // Host-only time control (DT1): pause and speed applied to this
        // frame's elapsed time before the clock ever sees it. See `devtime`.
        let elapsed_us = self.dev_paced_elapsed(elapsed_us);

        self.steps += self.runtime.advance(elapsed_us);
        false
    }
}

impl ApplicationHandler for Host {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.boot(event_loop);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.finish(event_loop),

            WindowEvent::Resized(size) => self.resize(size.width, size.height),

            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => {
                match &event.logical_key {
                    key if self.grafting.open && !event.repeat => self.press_key(key),
                    _ if self.grafting.open => {},
                    Key::Named(NamedKey::Escape) => self.finish(event_loop),
                    // Panning and quarter turns are presentation controls.
                    Key::Named(NamedKey::ArrowLeft) => self.pan.x -= section::PAN_STEP,
                    Key::Named(NamedKey::ArrowRight) => self.pan.x += section::PAN_STEP,
                    Key::Named(NamedKey::ArrowUp) => self.pan.y += section::PAN_STEP,
                    Key::Named(NamedKey::ArrowDown) => self.pan.y -= section::PAN_STEP,
                    // A replay drives itself. Accepting a key would put an
                    // intent in the trace that the recording never had.
                    //
                    // `event.repeat` filters OS auto-repeat (see `input`'s
                    // module docs for the backlog it used to build): a held
                    // key contributes its initial keydown and nothing more
                    // while it stays down.
                    key if !event.repeat
                        && (self.config.replay.is_none()
                            || matches!(key, Key::Character(c) if matches!(c.as_str(), "z" | "Z" | "v" | "V"))) =>
                    {
                        self.press_key(key)
                    },
                    _ => {},
                }
            },

            WindowEvent::RedrawRequested => {
                self.frame(event_loop);
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },

            _ => {},
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}
