// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The two directions of the bridge. (PD4)
//!
//! **Rust to Lua** builds the request table. Tables are built from sorted,
//! ordered data, so an author sees the same structure for the same request on
//! every host and in every run.
//!
//! **Lua to Rust** reads a proposal back. Every reader here is total: a script
//! may hand back anything at all, so a missing field, a wrong type, a table
//! nested past the policy or a list longer than the policy becomes a named
//! [`Refused`] rather than a panic inside somebody else's script.

use piccolo::{Context, Table, Value};

use super::request::{Ambient, Definition, PartView, Request, SiteView};
use super::{Expression, Policy, Proposal, Refused};

// ---------------------------------------------------------------------------
// Rust to Lua
// ---------------------------------------------------------------------------

/// Builds the one table a script is given.
pub(super) fn request_table<'gc>(ctx: Context<'gc>, request: &Request) -> Table<'gc> {
    let table = Table::new(&ctx);
    put_str(ctx, table, "trigger", request.trigger.word());
    put_str(ctx, table, "ruleset", &request.ruleset.hex());
    set(table, ctx, "revision", i64::from(request.revision));
    set(table, ctx, "material_mg", clamp(request.material_mg));

    let definitions = Table::new(&ctx);
    for (index, def) in request.definitions.iter().enumerate() {
        set(
            definitions,
            ctx,
            index as i64 + 1,
            definition_table(ctx, def),
        );
    }
    set(table, ctx, "definitions", definitions);

    let parts = Table::new(&ctx);
    for (index, part) in request.parts.iter().enumerate() {
        set(parts, ctx, index as i64 + 1, part_table(ctx, part));
    }
    set(table, ctx, "parts", parts);

    let candidates = Table::new(&ctx);
    for (index, id) in request.candidates.iter().enumerate() {
        set(candidates, ctx, index as i64 + 1, string(ctx, id));
    }
    set(table, ctx, "candidates", candidates);

    // Keyed by name as well as listed, because a script asking "how much is in
    // the ground" should not have to walk a list to find out.
    let conditions = Table::new(&ctx);
    for Ambient { name, value } in &request.conditions {
        set(conditions, ctx, string(ctx, name), *value);
    }
    set(table, ctx, "conditions", conditions);

    table
}

/// The host's draws, as plain Lua integers.
///
/// The high bit is cleared so each fits an `i64` without changing what the
/// recorded trace says was drawn — the same accommodation Isometry's runtime
/// makes, for the same reason.
pub(super) fn entropy_table<'gc>(ctx: Context<'gc>, draws: &[u64]) -> Table<'gc> {
    let table = Table::new(&ctx);
    for (index, draw) in draws.iter().enumerate() {
        set(table, ctx, index as i64 + 1, (draw >> 1) as i64);
    }
    table
}

fn definition_table<'gc>(ctx: Context<'gc>, def: &Definition) -> Table<'gc> {
    let table = Table::new(&ctx);
    put_str(ctx, table, "id", &def.id);
    put_str(ctx, table, "seeding", &def.seeding);
    let roles = Table::new(&ctx);
    for (index, word) in def.expressed_by.iter().enumerate() {
        set(roles, ctx, index as i64 + 1, string(ctx, word));
    }
    set(table, ctx, "expressed_by", roles);
    table
}

fn part_table<'gc>(ctx: Context<'gc>, part: &PartView) -> Table<'gc> {
    let table = Table::new(&ctx);
    set(table, ctx, "part", i64::from(part.part));
    put_str(ctx, table, "role", &part.role);
    set(table, ctx, "cells", i64::from(part.cells));
    set(table, ctx, "free", i64::from(part.free));
    set(table, ctx, "cell_mg", clamp(part.cell_mg));
    let sites = Table::new(&ctx);
    for (index, SiteView { process, cells }) in part.sites.iter().enumerate() {
        let site = Table::new(&ctx);
        put_str(ctx, site, "process", process);
        set(site, ctx, "cells", i64::from(*cells));
        set(sites, ctx, index as i64 + 1, site);
    }
    set(table, ctx, "sites", sites);
    table
}

// ---------------------------------------------------------------------------
// Lua to Rust
// ---------------------------------------------------------------------------

/// Reads a returned value as a [`Proposal`], inside the policy's bounds.
pub(super) fn proposal_of<'gc>(
    ctx: Context<'gc>,
    value: Value<'gc>,
    policy: Policy,
) -> Result<Proposal, Refused> {
    let table = as_table(value, "proposal")?;
    depth_ok(table, 0, policy)?;

    let sites = match table.get(ctx, "sites") {
        Value::Nil => return Ok(Proposal::default()),
        found => as_table(found, "proposal.sites")?,
    };
    let entries = sites.length();
    if entries > policy.max_entries as i64 {
        return Err(Refused::Collection {
            entries: entries.max(0) as usize,
            limit: policy.max_entries,
        });
    }

    let mut out = Vec::new();
    for index in 1..=entries {
        let site = as_table(sites.get(ctx, index), "proposal.sites[]")?;
        out.push(Expression {
            part: u32::try_from(integer(ctx, site, "part")?).map_err(|_| Refused::Malformed {
                why: "part is not a part address".to_owned(),
            })?,
            process: text(ctx, site, "process")?,
            cells: u32::try_from(integer(ctx, site, "cells")?).map_err(|_| Refused::Malformed {
                why: "cells is not a cell count".to_owned(),
            })?,
        });
    }
    Ok(Proposal { sites: out })
}

/// Walks the returned table to prove it does not nest past the policy.
///
/// Bounded twice over: the recursion stops at `max_depth`, and each level reads
/// at most `max_entries`, so a script cannot make the host walk a structure it
/// did not agree to walk.
fn depth_ok(table: Table<'_>, depth: usize, policy: Policy) -> Result<(), Refused> {
    if depth > policy.max_depth {
        return Err(Refused::Depth {
            limit: policy.max_depth,
        });
    }
    let mut seen = 0usize;
    for (_, value) in table.iter() {
        seen += 1;
        if seen > policy.max_entries {
            return Err(Refused::Collection {
                entries: seen,
                limit: policy.max_entries,
            });
        }
        if let Value::Table(nested) = value {
            depth_ok(nested, depth + 1, policy)?;
        }
    }
    Ok(())
}

fn as_table<'gc>(value: Value<'gc>, what: &str) -> Result<Table<'gc>, Refused> {
    match value {
        Value::Table(table) => Ok(table),
        other => Err(Refused::Malformed {
            why: format!("{what} is a {}, not a table", other.type_name()),
        }),
    }
}

fn integer<'gc>(ctx: Context<'gc>, table: Table<'gc>, key: &'static str) -> Result<i64, Refused> {
    table
        .get(ctx, key)
        .to_integer()
        .ok_or_else(|| Refused::Malformed {
            why: format!("{key} is not an integer"),
        })
}

fn text<'gc>(ctx: Context<'gc>, table: Table<'gc>, key: &'static str) -> Result<String, Refused> {
    match table.get(ctx, key) {
        Value::String(found) => {
            String::from_utf8(found.as_bytes().to_vec()).map_err(|_| Refused::Malformed {
                why: format!("{key} is not UTF-8"),
            })
        },
        other => Err(Refused::Malformed {
            why: format!("{key} is a {}, not a string", other.type_name()),
        }),
    }
}

// ---------------------------------------------------------------------------
// Small shared helpers
// ---------------------------------------------------------------------------

pub(super) fn string<'gc>(ctx: Context<'gc>, value: &str) -> piccolo::String<'gc> {
    piccolo::String::from_slice(&ctx, value.as_bytes())
}

fn put_str<'gc>(ctx: Context<'gc>, table: Table<'gc>, key: &'static str, value: &str) {
    set(table, ctx, key, string(ctx, value));
}

/// Lua integers are signed, and none of these quantities is ever near the
/// boundary; saturating is still stated rather than assumed.
fn clamp(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn set<'gc, K: piccolo::IntoValue<'gc>, V: piccolo::IntoValue<'gc>>(
    table: Table<'gc>,
    ctx: Context<'gc>,
    key: K,
    value: V,
) {
    table
        .set(ctx, key, value)
        .expect("a request key is a string or a positive index, never nil or NaN");
}
