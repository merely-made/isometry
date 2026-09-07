// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's windowed host.
//!
//! Owns the window, the device, and the frame loop. Holds **no game state**:
//! input becomes intents, the shared runtime steps the world at a fixed rate,
//! and the world comes back out to be drawn. If a rule ever appears in this
//! crate, it is in the wrong crate.
//!
//! The loop is the standard one, and the reason it is safe is elsewhere:
//! `mesocosm-runtime` converts however much wall time actually elapsed into
//! whole fixed steps, so a stuttering window cannot change what happens.
//!
//! The main view is the lens brick tracer's side-on terrarium section over the
//! live world (PS1). Camera motion is presentation only and never enters the
//! trace, which is what lets a recorded session replay to the same hash.

//! Five chrome lanes ride the frame, all through [`chrome`]: the painted
//! minimap ([`hud`]), the cambium vitals panel ([`vitals`], landed 2026-08-29),
//! the individual checkpoint ([`succession`], PE1), the trait board
//! ([`review`], PE3b), and the dev lane ([`dev`], DT1). The checkpoint and the
//! board draw only while the driver is holding the world at a question, and
//! never both at once; the dev lane draws only while `--dev` is set. None of
//! them touches the world, so none can reach the trace.

//! **The harness is genet-probe's** (DT4). This host implements `Automatable`
//! and `Driveable`, so a text `genet_probe::Scenario` drives a run: see
//! [`app::drive`] for what each verb means here and [`app::actions`] for what
//! `act` can ask for. The four cambium lanes above hand the driver the retained
//! trees they draw from, so an `assert text` is a claim about what is on
//! screen. `--record-demo`, `--auto-eat` and the DT3 example are gone; they are
//! scenario actions now.

pub mod app;
pub mod body_menu;
pub mod chrome;
pub mod dev;
pub mod fixture;
pub mod hud;
pub mod input;
pub mod played;
pub mod review;
pub mod section;
pub mod succession;
pub mod vitals;

pub use app::{Host, HostConfig};
pub use body_menu::BodyMenuChrome;
pub use played::{PlayedReceipt, PlayedTrace};
