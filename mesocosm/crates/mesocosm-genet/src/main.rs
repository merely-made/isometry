// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Runs Mesocosm in a window.
//!
//! ```text
//! cargo run -p mesocosm-genet
//! cargo run -p mesocosm-genet -- --replay <trace>
//! cargo run -p mesocosm-genet -- --scenario <scenario>
//! ```
//!
//! With nothing named on the command line the trace, receipt and capture go to
//! `<Code>/testing/mesocosm/scratch_played.*`. The golden `ps1_played.*`
//! fixture is written only when a flag names it, ruled 2026-09-02 — before
//! that the defaults were the fixture, and an unqualified run overwrote it.
//!
//! `--scenario` drives the run from a text scenario through genet-probe's
//! shared driver (DT4). It is where `--record-demo` and `--auto-eat` went: both
//! are now actions a scenario asks for by name. See [`mesocosm_genet::app::drive`]
//! for the verbs and [`mesocosm_genet::app::actions`] for the names.
//!
//! Controls: WASD moves, E or Space metabolizes what is in reach, Q deposits,
//! C digs, the arrow keys pan the section, Escape writes the receipts and
//! quits. At a checkpoint — a birth involving your critter, or its death — the
//! world stops and the keys narrow to Enter (carry on) and T (take the body on
//! offer). At the epoch boundary the trait board comes up instead: Tab moves
//! among the candidates, R commits the selected one, Enter goes back to the
//! terrarium.
//!
//! `--camera` turns the section without touching the world (DC4, Q9).
//! `oblique` is the shipped section, ruled the default on 2026-09-04: it tilts
//! twenty degrees off `-z` on both free rotations, so depth reads as a short
//! diagonal and a body's part budget can be counted. The two level arms the
//! measurement compared it against are still here — `side` looks straight down
//! `-z` and draws bodies end-on, `across` looks down `-x` so the segments
//! chain at full length. It is a sibling of `--slab` — framing, not rule — and
//! the receipt names which one drew.
//!
//! `--dev` adds a fifth chrome lane and twelve keys, live only while it is set.
//! DT1's five drive time: P pauses or unpauses the clock, `.` steps once and
//! `,` steps ten, both off the clock, and `[`/`]` move the clock's speed down
//! or up a rung. DT2's three drive the camera: N and B cycle the follow target
//! through the living roster in id order, and M snaps it back to the critter
//! under your hand. Following moves the camera and nothing else — control
//! stays where it is.
//!
//! DT3's four change the world, and are the only dev keys that do: X ends the
//! epoch now, F forces a birth from the followed critter, K kills it, and G
//! puts matter into the ground under it. Each queues an ordinary intent, so it
//! is in the trace, it replays, and the receipt counts it — a run that used one
//! prints as **assisted**. See [`mesocosm_genet::input`] for the exact mapping.

use std::path::PathBuf;

use mesocosm_genet::section::CameraMode;
use mesocosm_genet::{Host, HostConfig, played};

fn main() {
    let mut config = HostConfig::default();
    let mut args = std::env::args().skip(1);
    let mut replay: Option<PathBuf> = None;
    let mut trace = None;
    let mut receipt = None;
    let mut capture = None;
    let mut slab_explicit = false;
    let mut create = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--create" => create = true,
            "--draft" => {
                config.creator_draft = Some(PathBuf::from(args.next().unwrap_or_else(|| {
                    eprintln!("--draft requires a criteria JSON path");
                    std::process::exit(1);
                })));
                create = true;
            },
            "--start" => {
                let selection = read_start(args.next()).unwrap_or_else(|why| {
                    eprintln!("--start: {why}");
                    std::process::exit(1);
                });
                config.seed = selection.request.seed;
                config.organisms = selection.request.organisms;
                config.start = Some(selection);
            },
            "--frames" => config.frames = args.next().and_then(|v| v.parse().ok()),
            "--size" => {
                let size = args.next().unwrap_or_default();
                let parsed = size
                    .split_once('x')
                    .and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?)));
                let Some((width, height)) = parsed.filter(|(w, h)| *w > 0 && *h > 0) else {
                    eprintln!("--size wants positive WIDTHxHEIGHT in logical pixels");
                    std::process::exit(1);
                };
                config.width = width;
                config.height = height;
            },
            "--bodies" => {
                let named = args.next().unwrap_or_default();
                config.body_mode =
                    mesocosm_genet::section::BodyMode::parse(&named).unwrap_or_else(|| {
                        eprintln!("--bodies wants voxels or capsules");
                        std::process::exit(1);
                    });
            },
            "--body-budget" => {
                config.body_budget = args
                    .next()
                    .and_then(|v| v.parse::<usize>().ok())
                    .filter(|v| *v > 0)
                    .unwrap_or_else(|| {
                        eprintln!("--body-budget wants a positive integer");
                        std::process::exit(1);
                    });
            },
            "--body-content" => {
                config.generated_content = match args.next().as_deref() {
                    Some("generated") => true,
                    Some("fixtures") => false,
                    _ => {
                        eprintln!("--body-content wants generated or fixtures");
                        std::process::exit(1);
                    },
                };
            },
            "--body-layout" => {
                config.body_layout = args
                    .next()
                    .as_deref()
                    .and_then(played::BodyLayout::parse)
                    .unwrap_or_else(|| {
                        eprintln!(
                            "--body-layout wants spaced (default), jointed, branching or axial"
                        );
                        std::process::exit(1)
                    });
            },
            "--scene" => {
                let named = args.next().unwrap_or_default();
                config.scene =
                    mesocosm_genet::played::SceneMode::parse(&named).unwrap_or_else(|| {
                        eprintln!("--scene wants ecology, terrarium, graft-practice, expression-practice, family-practice, family-clearing or family-clearing-lean");
                        std::process::exit(1);
                    });
            },
            "--terrarium-pitch" => {
                config.terrarium_pitch = args
                    .next()
                    .and_then(|value| value.parse::<f32>().ok())
                    .filter(|value| value.is_finite() && (0.0..=45.0).contains(value))
                    .unwrap_or_else(|| {
                        eprintln!("--terrarium-pitch wants a finite angle from 0 to 45");
                        std::process::exit(1);
                    });
            },
            "--cutaway" => {
                let named = args.next().unwrap_or_default();
                config.cutaway =
                    mesocosm_genet::section::Cutaway::parse(&named).unwrap_or_else(|| {
                        eprintln!("--cutaway wants occupied, always or never");
                        std::process::exit(1);
                    });
            },
            "--terrain-style" => {
                config.terrain_style = args
                    .next()
                    .as_deref()
                    .and_then(mesocosm_genet::section::TerrainStyle::parse)
                    .unwrap_or_else(|| {
                        eprintln!("--terrain-style wants auto, classic or habitat");
                        std::process::exit(1);
                    });
            },
            "--capture" => capture = args.next().map(PathBuf::from),
            "--trace" => trace = args.next().map(PathBuf::from),
            "--receipt" => receipt = args.next().map(PathBuf::from),
            "--replay" => replay = args.next().map(PathBuf::from),
            // The scenario driver (DT4). It replaces `--record-demo` and
            // `--auto-eat`, which are now the `demo` and `hunt` actions.
            "--scenario" => {
                let Some(path) = args.next().map(PathBuf::from) else {
                    eprintln!("--scenario wants a path");
                    std::process::exit(1);
                };
                match std::fs::read_to_string(&path) {
                    Ok(text) => config.scenario = Some(text),
                    Err(error) => {
                        eprintln!("scenario: {}: {error}", path.display());
                        std::process::exit(1);
                    },
                }
            },
            // Which way the section looks (DC4, Q9). Presentation only: the
            // same golden trace replays to the same hash under all three, so
            // the ruled oblique default and the two level arms it was
            // measured against are interchangeable to the world.
            "--camera" => {
                let named = args.next().unwrap_or_default();
                match CameraMode::parse(&named) {
                    Some(mode) => {
                        config.camera = mode;
                        config.camera_explicit = true;
                    },
                    None => {
                        eprintln!(
                            "--camera wants one of oblique, side, across, terrarium-east, terrarium-south, terrarium-west, terrarium-north"
                        );
                        std::process::exit(1);
                    },
                }
            },
            // Presentation only; the default is ruled and this varies it.
            "--slab" => {
                slab_explicit = true;
                if let Some(half) = args.next().and_then(|v| v.parse::<f32>().ok()) {
                    config.slab_half_height = half;
                }
            },
            "--seed" => {
                if let Some(seed) = args.next().and_then(|v| v.parse().ok()) {
                    config.seed = seed;
                }
            },
            // The dev lane and its keys (DT1). Off by default; recorded in
            // the receipt either way.
            "--dev" => config.dev = true,
            // Where the camera starts (DT2). Presentation only, and only a
            // starting point: the follow keys move it from here.
            "--follow" => config.follow = args.next().and_then(|v| v.parse().ok()),
            "--help" | "-h" => {
                println!("{}", HELP);
                return;
            },
            other => eprintln!("ignoring unknown argument: {other}"),
        }
    }

    // A scratch name under the workspace's headed-verify home, unless a flag
    // says otherwise. **Scratch, deliberately** (ruled 2026-09-02): these
    // defaulted to `ps1_played.*` until 2026-09-04, which is the golden fixture
    // `--replay` is checked against, so running this binary with no arguments
    // destroyed it. See `played::DEFAULT_STEM`.
    let trace_path = trace.unwrap_or_else(played::default_trace_path);
    config.capture = Some(capture.unwrap_or_else(played::default_capture_path));
    config.receipt = Some(receipt.unwrap_or_else(played::default_receipt_path));

    if let Some(path) = replay {
        match played::read_trace(&path) {
            Ok(recorded) => {
                // The replay is the recording's run, so it is the recording's
                // seed and roster too; a flag that disagreed would be a
                // different world wearing the same trace.
                config.seed = recorded.seed;
                config.organisms = recorded.organisms;
                config.replay = Some(recorded);
            },
            Err(error) => {
                eprintln!("replay: {error}");
                std::process::exit(1);
            },
        }
    } else {
        config.trace = Some(trace_path);
    }

    if config.effective_scene() != mesocosm_genet::played::SceneMode::Ecology
        && !config.camera_explicit
    {
        config.camera = if config.effective_scene().is_family_clearing() {
            CameraMode::TerrariumSouth
        } else {
            CameraMode::TerrariumEast
        };
    }

    if config.effective_scene() != mesocosm_genet::played::SceneMode::Ecology && !slab_explicit {
        config.slab_half_height = 18.0;
    }

    if create {
        if config.replay.is_some() || config.effective_scene() != played::SceneMode::Ecology {
            eprintln!("--create requires a fresh ecology run");
            std::process::exit(1);
        }
        if config.creator_draft.is_some() && config.start.is_some() {
            eprintln!("choose --draft for criteria or --start for an admitted selection");
            std::process::exit(1);
        }
        let saved = config.creator_draft.as_ref().and_then(|path| {
            mesocosm_genet::creator_draft::load(path).unwrap_or_else(|why| {
                eprintln!("--draft {}: {why}", path.display());
                std::process::exit(1);
            })
        });
        let request = saved
            .or_else(|| config.start.as_ref().map(|s| s.request.clone()))
            .unwrap_or_else(|| mesocosm_core::world::generation::Request {
                seed: config.seed,
                ..Default::default()
            });
        config.creator_request = Some(request);
    }
    match Host::run(config) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("host failed: {error}");
            std::process::exit(1);
        },
    }
}

fn read_start(path: Option<String>) -> Result<mesocosm_core::world::generation::Selection, String> {
    let path = path.ok_or("expected a selection JSON path")?;
    let bytes = std::fs::read(&path).map_err(|why| format!("{path}: {why}"))?;
    let selection: mesocosm_core::world::generation::Selection =
        serde_json::from_slice(&bytes).map_err(|why| format!("{path}: {why}"))?;
    selection
        .request
        .validate()
        .map_err(|why| format!("{why:?}"))?;
    Ok(selection)
}

const HELP: &str = "\
mesocosm-genet: run Mesocosm in a window

  --frames N      run N frames and exit
  --size WxH      initial window size in logical pixels (default 960x540)
  --bodies MODE   voxels (default) or capsules (comparison), presentation only
  --body-budget N maximum detailed bodies in the section (default 41)
  --body-content MODE generated (default) or fixtures for new worlds; replay uses saved content
  --body-layout MODE  spaced (default), jointed, branching or axial; replay uses the recorded set
  --capture PATH  write the final frame as a PNG
  --trace PATH    write (or, with --replay, read) the intent trace
  --receipt PATH  write the run's receipt
  --replay PATH   drive the run from a recorded trace and assert its hash
  --scenario PATH drive the run from a text scenario and exit 1 if it fails
  --seed N        world seed
  --create        compare generated starting lives before play (arrows, R/N/P/C/M, +/-)
  --draft PATH    reopen criteria or begin a new draft; implies --create; S saves
                  K retains selected role/organs/segment count; U clears these filters
  --start PATH    enter a generated selection JSON (recorded for replay)
  --scene MODE    ecology (default), terrarium, or authored graft-practice/expression-practice/family-practice/family-clearing/family-clearing-lean
  --terrarium-pitch DEG  shallow camera pitch, 0..45 degrees (default 12)
  --cutaway MODE  occupied (default), always (expose interior), or never
  --terrain-style MODE  auto (habitat in terrarium, classic in ecology),
                        classic (retro comparison), or habitat (earth/sky contrast)
  --slab H        section slab half-height in voxels (presentation only, default 28, terrarium 18)
  --camera MODE   which way the section looks: oblique (the shipped section,
                  tilted 20 degrees so depth reads as a short diagonal; the
                  default), side (straight down -z, bodies end-on) or across
                  (turned a quarter, so bodies chain across the view).
                  Presentation only: it picks rays, never a rule, so a replay
                  lands on the same hash under all three
  --dev           enable the dev lane and its keys (DT1, DT2, DT3); off by
                  default
  --follow ID     start the camera on this critter (DT2; needs --dev to be
                  worth anything, presentation only)

--capture, --trace and --receipt default to scratch names under the workspace's
headed-verify home: <Code>/testing/mesocosm/scratch_played.png, .trace.json and
.json. They are never the golden ps1_played.* fixture, which is written only
when one of those flags names it.

controls: Z/V turn the terrarium left/right; WASD move along world axes, E/Space eat, Q deposit, C dig, arrows pan, Esc quit
Y opens Inspect and consume tissue (world paused): arrows or J/L select; Enter consumes; Esc cancels
O opens Express discovery (world paused): arrows or J/L select; Enter expresses; Esc cancels
H opens Graft tissue beside a carcass (world paused):
  arrows or J/L select a branch; Tab switches keep/regrow; Enter confirms
  Esc cancels the menu; Z/V rotates the preview. Only confirmation enters the trace.
in family-clearing and family-clearing-lean, Enter waits one tick and X opens lineage review.
  X remains an explicit assisted epoch boundary on the receipt.
at a checkpoint the world stops and the keys narrow:
  Enter  carry on unchanged
  T      take the body on offer (the newborn, or your eldest descendant)
at the epoch boundary the trait board comes up instead:
  Tab    move among the candidates
  R      commit the selected candidate to your line
  Enter  back to the terrarium
with --dev, eight host-only keys are live (none of them ever reaches the
trace, so a replay's hash cannot move because of one):
  P      pause or unpause the clock
  .      step once, off the clock
  ,      step ten, off the clock
  [      one rung slower on the speed ladder (1/4, 1/2, 1, 2, 4)
  ]      one rung faster
  N      follow the next living critter in id order, wrapping
  B      follow the previous one
  M      snap the camera back to the critter under your hand
following moves the camera and nothing else: control stays where it is
  I      open/close body-part inspection (dev ground truth)
  J/L    previous/next drawn part of the followed critter
  U      clear the selected part
inspection consumes gameplay and world-edit keys until I closes it;
time, follow and camera pan remain available. Amber marks the selected part.
and four world-changing ones, which queue ordinary intents and so do enter
the trace, replay with it, and are counted on the receipt:
  X      end the epoch now (refused where the world's epoch rule says so)
  F      force a birth from the followed critter
  K      kill the followed critter
  G      put matter into the ground under the followed critter
a run that applied any of the four prints as assisted

a scenario is one verb a line (blank lines and # comments skipped):
  act NAME        one of the key letters above (w, e, x, ...), or one of five
                  host actions: follow ID, follow-nearest, follow-child,
                  hunt EVERY, demo STEPS
  settle N        pump N frames
  wait [CAP]      hold until the host reports quiet (a replay spent, a demo or
                  hunt finished, the queue empty); CAP is a hang-stop
  assert text S   S is on a chrome lane that is on screen
  assert snap F OP V   a run field: hash, expected, matches, mode, tick, steps,
                  frames, epoch, dev, dev-intents, assisted, queued, controlled,
                  follow, living, checkpoint, boundary, paused. OP is == >= <= ~
                  (assisted reads 'unassisted' or 'assisted (N dev intents)')
  assert event S  S is in what the world answered
  capture NAME    a PNG, at NAME if it is a path or beside the fixtures if not
  log WORDS       into the run's log
the process exits 1 if any assertion fails or the scenario runs out of frames";
