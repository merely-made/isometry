// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Authored development: a typed request in, a typed proposal out. (PD4)
//!
//! The pack door admits *definitions*; this admits *decisions about them*. An
//! author writes one Lua function, the host hands it a frozen picture of a body
//! and its situation, and what comes back is a [`Proposal`] — never a change.
//! The proposal is lowered to the ordinary
//! [`AllocationProposal`](mesocosm_core::AllocationProposal) and offered to the
//! one validator, which accepts or refuses it exactly as it would a hand-drawn
//! or an automatic one.
//!
//! # Lua cannot mutate anything, and that is structural
//!
//! Not a discipline, not a review note — a property of what the runner is able
//! to hand across:
//!
//! - **No host functions.** The runner registers nothing in the globals. There
//!   is no callback into Rust, so there is no Rust value a script could reach
//!   to change. Its whole world is the tables it was given and what
//!   [`Lua::core`](piccolo::Lua::core) provides.
//! - **No I/O in the sandbox.** `Lua::core()` deliberately omits the `io`
//!   library, and piccolo 0.3 has no `os`, `require`, `dofile` or `loadfile` at
//!   all — so there is no network, filesystem, environment or wall clock to
//!   reach.
//! - **No randomness of its own.** `Lua::core()` *does* install `math.random`
//!   and `math.randomseed`, seeded from OS entropy, so [`Runner::load`] deletes
//!   both. Entropy is host-owned, drawn before the call, handed in as plain
//!   integers, and recorded (see [`Entropy`]).
//! - **The Rust side is by value.** The runner takes a `&Request` — a frozen
//!   copy of declared facts — and returns a `Proposal`. It never holds a
//!   `&mut World`, a `&mut BodyPhenotype`, or any handle to one. There is
//!   nothing mutable in this module's public API for a script to be pointed at.
//!
//! What survives all of that is the only power an author has: *saying what the
//! body should express*, and being told no.
//!
//! # The entrypoint
//!
//! ```text
//! function express(request, entropy) -> proposal end
//! ```
//!
//! `request` is a table (see [`Request`]); `entropy` is an array of
//! [`DRAWS`] integers the host drew and wrote down; `proposal` is
//! `{ sites = { { part = <id>, process = "<namespace>:<name>", cells = <n> } } }`.
//!
//! # Refusals name the boundary
//!
//! Plan §4 asks that every refusal say which boundary failed. [`Refused`] is
//! that list at this door — unknown id, invalid part, excessive output,
//! exhausted fuel, and the rest — and the ones the *validator* owns arrive
//! whole, as [`Refused::Validator`], rather than being restated here. There is
//! one developmental authority and this is a proposal source over it.

mod fixture;
mod marshal;
mod proposal;
mod request;
mod runner;

pub use fixture::Fixture;
pub use proposal::{Expression, Proposal, lower};
pub use request::{Ambient, Definition, PartView, Request, SiteView, Trigger};
pub use runner::Runner;

use mesocosm_core::{PartId, Refusal};

/// Host policy for one expression call.
///
/// **Host policy, not pack metadata** — a script may do less work than this
/// allows and cannot ask for more. The numbers are small because the job is
/// small: this decides where a handful of cells go on one organ, and a script
/// that needs a hundred thousand VM steps to say so is not doing that job.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    /// Total VM fuel for one call, script load included. Exhausting it is
    /// [`Refused::Fuel`].
    pub fuel: i32,
    /// The largest proposal, measured after decoding, in bytes.
    pub max_output_bytes: usize,
    /// How deeply a returned table may nest.
    pub max_depth: usize,
    /// The longest list the host will read out of a returned table.
    pub max_entries: usize,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            fuel: 8_192,
            max_output_bytes: 4_096,
            max_depth: 8,
            max_entries: 64,
        }
    }
}

/// How many numbers the host draws and hands to a script, per call.
///
/// Fixed, and drawn *before* the call rather than on demand. A draw-on-demand
/// callback would be a host function in the script's globals, which is the one
/// thing this runner does not have; and a fixed tape makes the recorded trace
/// exactly reproducible without asking the script to co-operate.
pub const DRAWS: usize = 4;

/// The numbers a script was given, and the seed they came from.
///
/// **Host-owned.** The script cannot draw, reseed, or observe anything the host
/// did not put here, so "the same context and entropy produce the same
/// proposal" is a property of the arrangement rather than a hope about what a
/// pack does.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Entropy {
    pub seed: u64,
    /// The draw trace: exactly what crossed into Lua, in order.
    pub draws: Vec<u64>,
}

impl Entropy {
    /// Draws this call's tape from a seed.
    ///
    /// The core's own [`Rng`](mesocosm_core::Rng) — SplitMix64, the stream
    /// every other seeded decision in this game comes out of. No second
    /// generator was invented for this door.
    pub fn from_seed(seed: u64) -> Self {
        let mut rng = mesocosm_core::Rng::from_seed(seed);
        Self {
            seed,
            draws: (0..DRAWS).map(|_| rng.next_u64()).collect(),
        }
    }
}

/// Why an authored expression did not become a development.
///
/// Every variant names the boundary that failed (plan §4). The variants a
/// *validator* owns are not restated here: they arrive whole in
/// [`Refused::Validator`], because direct, automatic and authored arrangement
/// are three proposal sources over one developmental authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    /// The script does not declare `express`.
    NoEntrypoint,
    /// The script did not finish inside its fuel budget.
    Fuel { limit: i32 },
    /// The decoded proposal is larger than the host will read.
    Output { bytes: usize, limit: usize },
    /// A returned table nests deeper than the host will walk.
    Depth { limit: usize },
    /// A returned list is longer than the host will read.
    Collection { entries: usize, limit: usize },
    /// The returned value is not the declared proposal shape.
    Malformed { why: String },
    /// The script itself raised, or failed to load.
    Script { why: String },
    /// A proposal names a definition this world's ruleset does not hold.
    /// **Never substituted** with the nearest local one.
    UnknownProcess { id: String },
    /// A proposal names a part this body does not have.
    UnknownPart { part: PartId },
    /// A proposal asks for more tissue than the part has living.
    TooMuchTissue {
        part: PartId,
        asked: u32,
        living: u32,
    },
    /// The one validator refused it.
    Validator(Refusal),
}

impl Refused {
    /// The refusal in the plain sentence a diagnostic prints.
    pub fn words(&self) -> String {
        match self {
            Refused::NoEntrypoint => "the script declares no express(request, entropy)".to_owned(),
            Refused::Fuel { limit } => format!("the script did not finish within {limit} fuel"),
            Refused::Output { bytes, limit } => {
                format!("the proposal is {bytes} bytes and the host reads {limit}")
            },
            Refused::Depth { limit } => format!("the proposal nests deeper than {limit}"),
            Refused::Collection { entries, limit } => {
                format!("the proposal lists {entries} entries and the host reads {limit}")
            },
            Refused::Malformed { why } => format!("the proposal is not the declared shape: {why}"),
            Refused::Script { why } => format!("the script failed: {why}"),
            Refused::UnknownProcess { id } => {
                format!("this world's ruleset does not hold {id}")
            },
            Refused::UnknownPart { part } => format!("this body has no part {}", part.0),
            Refused::TooMuchTissue {
                part,
                asked,
                living,
            } => format!(
                "part {} has {living} living cells and the proposal asks for {asked}",
                part.0
            ),
            Refused::Validator(refusal) => format!("the validator refused it: {refusal:?}"),
        }
    }
}
