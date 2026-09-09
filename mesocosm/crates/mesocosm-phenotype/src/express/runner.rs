// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The bounded Piccolo host. (PD4)
//!
//! One script, one entrypoint, one call, and a fixed budget for all of it.

use piccolo::{Closure, Executor, Fuel, IntoValue, Lua, StashedExecutor, Value};

use super::marshal::{entropy_table, proposal_of, request_table, string};
use super::{Entropy, Policy, Proposal, Refused, Request};

/// The declared entrypoint, per plan §4.
pub const ENTRYPOINT: &str = "express";

/// A loaded expression script, ready to be asked for proposals.
///
/// **Nothing mutable crosses into it.** The runner registers no host function,
/// holds no world and no body, and the only things it hands a script are two
/// tables built from owned copies of declared facts. What comes back is a
/// [`Proposal`] — data, on its way to the one validator. There is no method
/// here that changes anything outside this struct.
pub struct Runner {
    lua: Lua,
    policy: Policy,
}

impl Runner {
    /// Loads a script into a fresh sandbox.
    ///
    /// [`Lua::core`] rather than an ambient standard library (plan §4): no
    /// `io`, and piccolo 0.3 has no `os`, `require`, `dofile` or `loadfile` at
    /// all — so no network, filesystem, environment, wall clock or threads.
    ///
    /// **And then two more deletions the donor does not make.** `Lua::core()`
    /// installs `math.random` and `math.randomseed` over an RNG seeded from OS
    /// entropy, which is a script's own source of randomness and would make the
    /// same context produce different proposals on the same machine. Both are
    /// removed here. Entropy is host-owned, drawn before the call, and written
    /// down.
    pub fn load(script: &str, policy: Policy) -> Result<Self, Refused> {
        if policy.fuel <= 0 {
            return Err(Refused::Fuel { limit: policy.fuel });
        }
        let mut lua = Lua::core();
        lua.enter(|ctx| {
            if let Value::Table(math) = ctx.get_global("math") {
                for name in ["random", "randomseed"] {
                    math.set(ctx, name, Value::Nil)
                        .expect("a static name is a valid table key");
                }
            }
        });
        let executor = lua
            .try_enter(|ctx| {
                let closure = Closure::load(ctx, Some(ENTRYPOINT), script.as_bytes())?;
                Ok(ctx.stash(Executor::start(ctx, closure.into(), ())))
            })
            .map_err(|why| Refused::Script {
                why: why.to_string(),
            })?;
        // The chunk itself runs inside the same budget: a script whose top
        // level never finishes is exhausted fuel, not a hung host.
        run(&mut lua, &executor, policy.fuel)?;
        lua.try_enter(|ctx| {
            ctx.fetch(&executor).take_result::<()>(ctx)??;
            Ok(())
        })
        .map_err(|why| Refused::Script {
            why: why.to_string(),
        })?;
        Ok(Self { lua, policy })
    }

    /// The policy this runner enforces.
    pub fn policy(&self) -> Policy {
        self.policy
    }

    /// Asks the script for one proposal.
    ///
    /// `express(request, entropy) -> proposal`. The entropy is the host's
    /// pre-drawn tape, so the recorded trace is exactly what crossed the
    /// boundary and there is no draw the script could make that the record
    /// would miss.
    ///
    /// **Determinism is per runner, and that is enough.** A script may keep its
    /// own globals between calls on one runner, so a stateful author could make
    /// a second call differ from the first; a fresh [`Runner::load`] and the
    /// same context and seed always give the same answer, which is what the
    /// fixtures assert and what plan §6 needs. A replay never reruns a script
    /// at all — it applies the recorded validated result — so this cannot reach
    /// a saved world.
    pub fn propose(&mut self, request: &Request, entropy: &Entropy) -> Result<Proposal, Refused> {
        let policy = self.policy;
        let executor = self
            .lua
            .try_enter(|ctx| {
                let name = string(ctx, ENTRYPOINT);
                let Value::Function(function) = ctx.globals().get(ctx, name) else {
                    return Err("no entrypoint".into_value(ctx).into());
                };
                let arguments = (
                    request_table(ctx, request),
                    entropy_table(ctx, &entropy.draws),
                );
                Ok(ctx.stash(Executor::start(ctx, function, arguments)))
            })
            .map_err(|_| Refused::NoEntrypoint)?;

        run(&mut self.lua, &executor, policy.fuel)?;

        let proposal = self
            .lua
            .try_enter(|ctx| {
                let value = ctx.fetch(&executor).take_result::<Value>(ctx)??;
                Ok(proposal_of(ctx, value, policy))
            })
            .map_err(|why| Refused::Script {
                why: why.to_string(),
            })??;

        // **Measured after decoding, not on the wire.** The bound that matters
        // is how much the host is asked to carry into the game, and a script
        // cannot make that number smaller by compressing what it wrote.
        let bytes = serde_json::to_vec(&proposal)
            .map(|bytes| bytes.len())
            .unwrap_or(usize::MAX);
        if bytes > policy.max_output_bytes {
            return Err(Refused::Output {
                bytes,
                limit: policy.max_output_bytes,
            });
        }
        Ok(proposal)
    }
}

/// Steps one executor to completion on a finite total budget.
///
/// `Lua::execute` refuels internally, which is right for host formulas and
/// wrong for authored content: this steps the executor itself so the budget is
/// the whole of what a call may spend.
fn run(lua: &mut Lua, executor: &StashedExecutor, fuel: i32) -> Result<(), Refused> {
    let mut budget = Fuel::with(fuel);
    loop {
        if lua.enter(|ctx| ctx.fetch(executor).step(ctx, &mut budget)) {
            return Ok(());
        }
        if !budget.should_continue() {
            return Err(Refused::Fuel { limit: fuel });
        }
    }
}
