// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's host-neutral runtime.
//!
//! Sits between [`mesocosm_core`] and any host. A host owns the window, the
//! device, and the frame loop; it hands elapsed time in, queues intents, and
//! reads the world back to draw it.
//!
//! # Why this is shared rather than per-host
//!
//! The host probe compares a custom Genet lane against an engine lane. If each
//! host wrote its own stepping, a difference between them could be a
//! difference in *stepping* rather than in the host, and the comparison would
//! measure the wrong thing. Both hosts drive the world through this crate, so
//! a divergence is attributable.
//!
//! This is also the extraction candidate named in the body pipeline plan: if
//! Paredros ever wants it, it gets renamed and lifted with two real consumers
//! justifying the move. Until then it stays here and stays small.
//!
//! # Frame delivery does not reach the simulation
//!
//! [`Clock`] converts elapsed microseconds into whole fixed steps and keeps
//! the remainder. The number of steps for a given total elapsed time is fixed,
//! however raggedly that time arrives, and a step cap defers work rather than
//! dropping it. Time is integer microseconds for the same reason the core is
//! integer-only.

pub mod clock;
pub mod readings;
pub mod review;
pub mod runtime;
pub mod succession;
pub mod tactile;

pub use clock::{Advance, Clock};
pub use readings::{FlowWindows, JUDGEMENT_TICKS, RETENTION_TICKS};
pub use review::{Authored, Proposed, Review, Row, Source};
pub use runtime::{DEFAULT_MAX_STEPS_PER_ADVANCE, Receipt, Replayed, Runtime};
pub use succession::{Birth, Boundary, Checkpoint, Loss, Occasion};
pub use tactile::{TactileCapsule, TactileError, TactileHit, TactilePick, TactileWorld};
