// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a played session leaves behind, and what drives one without a player.
//!
//! A run is a seed, an organism count, admitted content, and an ordered trace;
//! the hash is what two runs compare. The trace file carries these so `--replay` can assert
//! rather than merely re-run, and so the assertion needs no second file.
//!
//! The recorded demo answers its own checkpoints, because a checkpoint holds the
//! world and a recording that ignored one would simply stop advancing. Those
//! answers are ordinary intents, so they sit in the trace beside the moves and
//! the meals and replay with them.
//!
//! # Where an unnamed run writes, and where it does not
//!
//! The trace, the receipt and the capture all default to a **scratch** name
//! under the headed-verify home — [`DEFAULT_STEM`] — and the golden
//! `ps1_played.*` fixture is written only when a path names it. Ruled by Mark
//! on 2026-09-02 and wired on 2026-09-04: the defaults *were* the fixture, so
//! the plainest possible run of this binary overwrote the file the scenario
//! driver checks a replay against, and every landing entry in the dev tools
//! plan had to pass explicit paths to stay out of its way. A discipline
//! somebody keeps is not a property of the program; this is.

use std::path::{Path, PathBuf};

use mesocosm_core::{Intent, Placement, World};
use mesocosm_mesh::VolumeMap;
use mesocosm_runtime::{Checkpoint, Occasion, Runtime};
use serde::{Deserialize, Serialize};

mod layout;
pub use layout::BodyLayout;

/// The host presentation/world fixture selected for a run.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SceneMode {
    #[default]
    Ecology,
    Terrarium,
}

impl SceneMode {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ecology => "ecology",
            Self::Terrarium => "terrarium",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "ecology" => Some(Self::Ecology),
            "terrarium" => Some(Self::Terrarium),
            _ => None,
        }
    }
}

/// Steps the recorded demo runs for.
///
/// **Long enough for the loop the game is named around** (PE1). It was 120 —
/// enough to grow a body, dig, deposit and spend a stretch on instincts, and
/// far short of a critter that lives long enough to breed. A demo that never
/// reaches a birth or a death cannot receipt the checkpoint, so the recording
/// now runs to the far side of both: three births under one hand, the parent's
/// own death at the end of its natural life, and the line continued through a
/// descendant. It still stops while somebody is alive, because a receipt of a
/// corpse shows neither the roster nor the vitals.
pub const DEMO_STEPS: u64 = 3_100;

/// The world the demo is recorded in.
///
/// **A fixture picks the world that shows the thing.** The interactive default
/// seed is untouched; this one is chosen because its played critter survives
/// its own lifespan and leaves exactly one living descendant behind, so a
/// recorded run reaches both halves of the individual checkpoint instead of
/// being eaten in the first twenty seconds (which is what most seeds do — a
/// real finding, and the terrarium dynamics plan's, not this one's). A replay
/// reads the seed out of the trace, so nothing downstream needs to know it.
pub const DEMO_SEED: u64 = 7;

/// Move deltas the demo walks, in the order WASD produces them.
const CARDINALS: [[i32; 3]; 4] = [[0, 0, -2], [0, 0, 2], [-2, 0, 0], [2, 0, 0]];

/// Steps the demo spends with its hands off the keys.
///
/// Longer than `INSTINCT_IDLE_TICKS`, deliberately: the recorded run has to
/// cross the threshold and keep going, so the receipt covers both halves of
/// TD4 — the hand holding a body, and the ecology taking it back. Four seconds
/// of nothing at the canonical tempo.
const HANDS_OFF: std::ops::Range<u64> = 300..340;

/// Steps the demo deliberately goes without eating. (PE2)
///
/// **The non-food discovery, scripted.** Every other route into the record
/// happens on its own — the demo eats constantly, and each meal is an
/// observation that unlocks nothing — but coming through a *stress* needs the
/// script to stop reaching for food and stay stopped: the condition is
/// `discovery::HUNGER_TICKS` consecutive ticks under the starved line with a
/// hand still on the body, and one meal in the middle of it resets the run.
///
/// It runs long, and after the ten meals the demo grows on, because a critter
/// that has never eaten is not enduring anything — it is simply small. The
/// window closes well before the first birth, so nothing downstream in the
/// recording depends on it.
const ENDURING: std::ops::Range<u64> = 120..340;

/// Steps the demo will try to take a branch off a carcass in. (P3)
///
/// **After the hunger window closes**, so it disturbs nothing the recording
/// already receipts: the critter has grown, gone hungry on purpose, come
/// through the discovery at tick 219, and put its hands back on. It is a window
/// rather than one step because whether a carcass with a branch is within reach
/// is the enclosure's business and not the script's — the attempt repeats until
/// one lands, and then never again.
///
/// It was tried at 40..120 first and the recording came out with no heir at the
/// death checkpoint: three extra parts early enough changes what the critter
/// eats for three thousand ticks. A fixture picks the window that shows the
/// thing, the same way `DEMO_SEED` picks the world.
const GRAFTING: std::ops::Range<u64> = 340..420;

/// What the script is keeping count of.
///
/// One value rather than a widening argument list: the demo's decisions depend
/// on what it has already done, and two of them do now.
#[derive(Clone, Copy, Debug, Default)]
pub struct Script {
    /// Meals taken, so the critter grows before it goes hungry on purpose.
    pub meals: u32,
    /// Branches taken. One is the receipt; a second would be a habit.
    pub grafts: u32,
}

/// A recorded run, complete enough to reproduce and to judge.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayedTrace {
    /// Native eating semantics. Unversioned traces predate typed intake.
    #[serde(default)]
    pub trophic_grammar: u32,
    #[serde(default, skip_serializing_if = "SceneMode::is_ecology")]
    pub scene: SceneMode,
    #[serde(
        default = "BodyLayout::axial",
        skip_serializing_if = "BodyLayout::is_axial"
    )]
    pub body_layout: BodyLayout,
    pub seed: u64,
    pub organisms: u32,
    pub steps: u64,
    /// What the recording run ended at. A replay that lands elsewhere is a
    /// determinism failure, which is the whole point of writing it down.
    pub state_hash: u64,
    pub intents: Vec<Intent>,
    /// Immutable palette and voxel bytes admitted when this world was founded.
    /// Absent in existing recordings, which retain their original fixtures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<mesocosm_mesh::content::ContentPack>,
}

impl SceneMode {
    pub const fn is_ecology(&self) -> bool {
        matches!(self, Self::Ecology)
    }
}

/// What a run says about itself on the way out.
#[derive(Clone, Debug, Serialize)]
pub struct PartSelectionReceipt {
    pub organism: u32,
    pub part: u32,
    pub revision: u64,
}

/// The last visible frame's opaque tenant participation in Netrender's graph.
#[derive(Clone, Debug, Serialize)]
pub struct FrameGraphReceipt {
    pub tenant_name: String,
    pub producer_path: String,
    pub fallback_count: u64,
    pub scene_op_boundary: usize,
    pub caller_reported_physical_submission_count: Option<u64>,
    pub logical_opaque_producer_boundaries: usize,
    pub graph_encoder_batches: usize,
    pub graph_submission_boundaries: usize,
    pub logical_plan_dump: String,
}

/// What a run says about itself on the way out.
#[derive(Clone, Debug, Serialize)]
pub struct PlayedReceipt {
    pub trophic_grammar: u32,
    pub scene: &'static str,
    pub habitat_bounds: Option<[[i32; 3]; 2]>,
    pub terrarium_pitch: Option<f32>,
    pub burrow_occupied: Option<bool>,
    pub body_layout: &'static str,
    pub body_content: &'static str,
    pub inspecting: bool,
    pub selected_part: Option<PartSelectionReceipt>,
    /// `played` for a session at the keyboard, `replay` for a driven one.
    pub mode: &'static str,
    pub seed: u64,
    pub organisms: u32,
    pub steps: u64,
    pub state_hash: u64,
    /// The hash the driving trace recorded. `None` for a played session,
    /// which is the run that establishes it.
    pub expected_state_hash: Option<u64>,
    pub state_hash_matches: Option<bool>,
    pub adapter: String,
    pub backend: String,
    pub frames: u32,
    pub trace_len: usize,
    /// Advances with every carve and deposit that changed ground, so a
    /// receipt shows whether the section had anything to re-upload.
    pub ground_revision: u64,
    pub body_parts: usize,
    /// Parts of the played body the lens's capsule budget could not carry on
    /// the last drawn frame.
    ///
    /// **This number exists because a body used to vanish instead.** The host
    /// swallowed the lens's refusal with `.ok()`, so a critter that grew past
    /// the cap simply stopped being drawn while alive. Since DC3 the widest
    /// capsules are kept, the body is drawn truncated, and the loss is
    /// reported here rather than shown as an absence.
    pub body_capsules_dropped: u32,
    /// Bodies the last traced frame drew beside the played one: every alive
    /// organism the slab window held, capped by the lens's roster limit.
    pub section_roster: u32,
    /// Capsules the drawn roster members carried past their own smaller
    /// budget, widest kept.
    pub roster_capsules_dropped: u32,
    /// How much world the section framed, in voxels of slab half-height. A
    /// capture that does not say what it framed cannot be compared with the
    /// next one. (S1; the number was ruled at 28 on 2026-08-29.)
    pub slab_half_height: f32,
    /// Which way the section looked, by name (DC4, Q9); `oblique` unless the
    /// run asked for one of the two level arms. **Presentation, and the
    /// receipt says so**: the camera picks rays and never a rule, so a replay
    /// under `side` or `across` lands on the same state hash the ruled
    /// `oblique` default does. It is written down because captures of one
    /// tick are only comparable if each says which arm it is.
    pub camera: &'static str,
    pub cutaway: &'static str,
    pub terrain_style: &'static str,
    pub bodies: &'static str,
    pub body_budget: usize,
    pub body_projection: crate::section::BodyFrameStats,
    /// Absent only when Netrender declined the shared device and the
    /// chromeless direct-surface fallback drew the frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_graph: Option<FrameGraphReceipt>,
    pub trace: Option<String>,
    pub capture: Option<String>,
    /// Whether `--dev` was set for this run (DT1, ruled 2026-09-02). A played
    /// receipt says so out loud, so a playtest cannot be quietly assisted by
    /// time control without it showing.
    pub dev: bool,
    /// World-changing dev intents this run applied. (DT3)
    ///
    /// **The flag above says the tools were available; this says they were
    /// used.** Zero for every run without `--dev`, and for most runs with it —
    /// pause, step, speed and follow are host-only and change nothing, so a
    /// nonzero count means the epoch was ended, a birth forced, something
    /// killed, or matter placed. A run with any of those is labelled
    /// **assisted** in the line the host prints (dev tools plan §2, principle
    /// 5).
    ///
    /// Refused ones are not counted: nothing was applied. Counted by the
    /// driver off its own trace, so a replay of an assisted trace reports the
    /// same number the recording did.
    pub dev_intents: u64,
}

/// `Code/testing/<repo>/`, the workspace's headed-verify home. Derived from
/// this crate's own location rather than a hardcoded user path; every file
/// below it is overridable from the command line.
pub fn default_out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .map(|code| code.join("testing").join("mesocosm"))
        .unwrap_or_else(|| PathBuf::from("testing/mesocosm"))
}

/// The stem every default output file below shares. (Ruled 2026-09-02.)
///
/// **Not `ps1_played`, and that is the whole point.** The defaults used to be
/// `ps1_played.trace.json`, `ps1_played.json` and `ps1_played.png`, which are
/// the golden fixture the scenario driver replays against — so an unqualified
/// run of the headed binary, the most ordinary thing anybody does with it,
/// overwrote the very files the receipt is checked against. Every DT1-DT4
/// landing entry in the dev tools plan records passing explicit non-default
/// paths for exactly that reason, which is a discipline somebody has to keep
/// rather than a property of the program.
///
/// So a run with nothing on the command line writes scratch, and the golden
/// fixture is written only when somebody names it, which the test
/// `defaults_do_not_name_the_golden_fixture` asserts rather than leaving to
/// this comment.
pub const DEFAULT_STEM: &str = "scratch_played";

/// The fixture the defaults must never be. Named here so the test that keeps
/// them apart reads the same string this module is defined against.
pub const GOLDEN_STEM: &str = "ps1_played";

pub fn default_trace_path() -> PathBuf {
    default_out_dir().join(format!("{DEFAULT_STEM}.trace.json"))
}

pub fn default_receipt_path() -> PathBuf {
    default_out_dir().join(format!("{DEFAULT_STEM}.json"))
}

pub fn default_capture_path() -> PathBuf {
    default_out_dir().join(format!("{DEFAULT_STEM}.png"))
}

/// What a run's assistance reads as, in one place. (DT3, moved here by DT4.)
///
/// Empty for an unaided run, and `assisted (N dev intents)` for one that ended
/// an epoch, forced a birth, killed something or placed matter. Written once
/// because two things say it now: the line the host prints on the way out, and
/// the `assisted` field a scenario asserts against ([`crate::drive`]). A label
/// that could differ between the two would let a scenario pass a run the
/// receipt calls assisted.
pub fn assisted_label(dev_intents: u64) -> String {
    if dev_intents > 0 {
        format!("assisted ({dev_intents} dev intents)")
    } else {
        String::new()
    }
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    ensure_parent(path)?;
    let json = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn read_trace(path: &Path) -> Result<PlayedTrace, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let trace: PlayedTrace =
        serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    trace.validate_rules()?;
    Ok(trace)
}

impl PlayedTrace {
    pub fn validate_rules(&self) -> Result<(), String> {
        let current = mesocosm_core::TROPHIC_GRAMMAR_REVISION;
        if self.trophic_grammar != current {
            return Err(format!(
                "recording uses trophic grammar revision {}, this build uses {current}; replay it with its original build or record a new run",
                self.trophic_grammar
            ));
        }
        Ok(())
    }
}

pub fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), String> {
    ensure_parent(path)?;
    let file =
        std::fs::File::create(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(pixels))
        .map_err(|error| error.to_string())
}

/// **One step of the demo, and the only place it is driven.** (DT4)
///
/// A checkpoint holds the world, so a recording that ignored one would simply
/// stop advancing. Answering it is the demo's whole point: the choice is an
/// ordinary intent, so it lands in the trace beside the moves and the meals and
/// replays with them.
///
/// One intent per step, off the clock: a recording has no frame rate. That is
/// what lets the headed scenario driver pump this from a frame loop
/// ([`crate::drive`]'s `demo` action) and reach the same hash the headless
/// [`record_demo`] loop below reaches — the two are one driver with two pumps,
/// rather than a script written twice.
pub fn demo_step(runtime: &mut Runtime, volumes: &VolumeMap, step: u64, script: &mut Script) {
    let intent = match runtime.checkpoint() {
        Some(checkpoint) => answer(checkpoint),
        None => demo_intent(runtime.world(), volumes, step, script),
    };
    let grafting = matches!(intent, Intent::Graft { .. });
    runtime.queue(intent);
    runtime.step(1);
    if grafting && runtime.world().last_graft().is_some() {
        script.grafts += 1;
    }
}

/// Builds a trace headlessly, so the headed receipt runs with nobody at the
/// keyboard. Every verb the slice claims appears in it: moves in each
/// direction, metabolize, deposit, carve — and a stretch of doing nothing at
/// all, which since TD4 is a verb too.
pub fn record_demo(seed: u64, organisms: u32, ticks_per_second: u32, steps: u64) -> PlayedTrace {
    // This historical fixture helper retains tag-backed founding. The real
    // host records its generated pack alongside intents in app/receipts.rs.
    let volumes = crate::fixture::volumes();
    let mut runtime = Runtime::new(seed, organisms, ticks_per_second);
    let mut script = Script::default();
    for step in 0..steps {
        demo_step(&mut runtime, &volumes, step, &mut script);
    }
    PlayedTrace {
        trophic_grammar: mesocosm_core::TROPHIC_GRAMMAR_REVISION,
        scene: SceneMode::Ecology,
        body_layout: BodyLayout::Axial,
        seed,
        organisms,
        steps,
        state_hash: runtime.state_hash(),
        intents: runtime.trace().to_vec(),
        content: None,
    }
}

/// How the recorded demo answers a checkpoint: **continue at a birth, succeed
/// at a death.**
///
/// Continuing is the world default and it is what a birth checkpoint is for —
/// the offspring is alive in the enclosure either way, and the body the run
/// spent nine hundred ticks growing is not thrown away by a script. A death is
/// the other half of the same choice, and taking a descendant there is the
/// whole claim PE1 makes: the line goes on, and the session does not end.
fn answer(checkpoint: &Checkpoint) -> Intent {
    match checkpoint.occasion {
        Occasion::Birth(_) => checkpoint.default_answer(),
        Occasion::Loss(_) => match checkpoint.heir() {
            Some(organism) => Intent::TakeControl { organism },
            // Nothing survived to carry it. Declining is still an answer, and
            // it is the honest one.
            None => checkpoint.default_answer(),
        },
        // **Back to the terrarium** (PE3a). The recorded demo resumes rather
        // than revising: choosing among candidates is the review, and the
        // review is PE3b. What the demo receipts is that the boundary happens,
        // that the unplayed lines take their turns at it, and that a run
        // through one replays to the same hash.
        Occasion::Epoch(_) => checkpoint.default_answer(),
    }
}

fn demo_intent(world: &World, volumes: &VolumeMap, step: u64, script: &mut Script) -> Intent {
    // Open with a lap in every direction, so each movement key is in the
    // trace whether or not the hunt below happens to use it.
    if step < 16 {
        return Intent::Move {
            delta: CARDINALS[(step / 4) as usize % 4],
        };
    }
    // Walking away, and staying away. Checked before everything below so the
    // run of idles is unbroken — a single carve in the middle of it would
    // reset the world's idle count and put the hand straight back on.
    if HANDS_OFF.contains(&step) {
        return Intent::Idle;
    }
    // **Going hungry on purpose** (PE2). Simply not eating was not enough: by
    // this point the demo's critter has grown a canopy and earns about what it
    // spends, so its budget sits flat and it never crosses the starved line at
    // all. So it spends the reserve where a player can — into the ground under
    // it — until the budget sits about half a horizon short, and then holds
    // there while its own income keeps it alive. No new verb, and no scripted
    // world change: `Deposit` is a key the host already has.
    if ENDURING.contains(&step)
        && let Some(me) = world.controlled()
    {
        let target = me.upkeep_mg() * mesocosm_core::STARVED_UPKEEP_TICKS / 2;
        if me.energy_mg > target {
            return Intent::Deposit {
                mass_mg: me.energy_mg - target,
            };
        }
    }
    // **Taking a branch** (P3). One, and only where the enclosure has actually
    // left a carcass carrying one within reach — which it does, because things
    // die here and what they leave behind still has a shape. The crossing is
    // chosen the way a player would choose it: carry the tissue where this
    // world's affinity permits, and regrow it here where it does not.
    if GRAFTING.contains(&step)
        && script.grafts == 0
        && let Some((donor, part, _)) = crate::fixture::branch_donor(world)
    {
        return Intent::Graft {
            organism: donor,
            part,
            crossing: crate::fixture::crossing_for(world, donor),
        };
    }
    // Digging at your own feet: reach is anatomy, and one voxel down is
    // inside the shortest reach a starting critter has.
    if step % 40 == 19
        && let Some(at) = world.position()
    {
        return Intent::Carve {
            at: [at[0], at[1] - 1, at[2]],
            radius: 1,
        };
    }
    if step % 40 == 39 {
        return Intent::Deposit { mass_mg: 60 };
    }
    // One verb, no route. Whether these meals burn or build is the demo's
    // *budget* talking, not its script — which is the point of recording it:
    // the trace exercises the decision without carrying it.
    //
    // Ten meals to grow on, then whenever the budget says so. Growing raises
    // the rent, which is what pushes a well-fed critter back over the starved
    // line; a script that stopped eating at its quota simply starved to death
    // partway through the run, and a receipt of a corpse shows nothing.
    if !ENDURING.contains(&step)
        && (script.meals < 10 || world.is_starved())
        && let Some(target) = crate::fixture::reachable(world)
    {
        script.meals += 1;
        return crate::fixture::metabolize(world, target, volumes, Placement::Planned);
    }
    if let Some(delta) = crate::fixture::toward_prey(world) {
        return Intent::Move { delta };
    }
    // **A march, not a shuffle.** This used to turn every three steps, which
    // walks a critter in a small circle: it re-crossed ground it had already
    // eaten and starved inside a couple of hundred ticks in every seed
    // measured. Holding a heading covers new enclosure, which is what finding
    // the next meal actually takes.
    Intent::Move {
        delta: CARDINALS[(step / 200) as usize % 4],
    }
}

#[cfg(test)]
mod tests;
