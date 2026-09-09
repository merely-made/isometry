// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The admitted ruleset: every process definition this world holds. (PD3)
//!
//! Split out of `process.rs` at the 600-line ceiling when PD3 widened
//! [`ProcessId`] to owned strings and gave admission a door.
//!
//! # Canonical, so declaration order is not a rule
//!
//! A registry's definitions are sorted by qualified id and no id appears
//! twice. That is what lets a pack's manifest list its files in any order and
//! still lower to the same ruleset: the order a pack was written in is an
//! authoring convenience, and [`Registry::digest`] is order-independent on top
//! of that, folding the *sorted* definition digests rather than the vector's.
//!
//! The native table below is written in the same canonical order, so a pack
//! that encodes exactly these definitions admits to a registry that is `==` to
//! [`Registry::native`] rather than merely digest-equal. That equality is
//! PD3's parity receipt.

use std::sync::LazyLock;

use super::{DefinitionDigest, Process, ProcessDef, ProcessId, ProcessRef, Seeding};
use crate::plan::Role;
use crate::rules::RulesetDigest;

/// The pack format ABI this build admits.
///
/// Not a digest input: a ruleset is what its definitions say, and the ABI is
/// the gate that decides whether this build can read the file at all.
/// [`crate::rules::WorldRules`] therefore records the digest, and admission
/// records the refusal.
pub const NATIVE_ABI: u32 = 1;

/// The registry: every admitted process definition, in canonical order.
///
/// Deterministic by construction. The native table is fixed; a pack is
/// admitted through [`Registry::admit`], never around it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Registry {
    defs: Vec<ProcessDef>,
}

/// The definitions this build ships.
///
/// **Sorted by qualified id**, which is the canonical order every admitted
/// registry is in. `contract, fix, intake, secrete, sense` therefore reads
/// alphabetically rather than in the order the four natives were built; the
/// order is not rule-bearing (see [`Registry::digest`]) and no rule reads a
/// position, so this is a canonicalisation rather than a change.
fn native_defs() -> Vec<ProcessDef> {
    vec![
        ProcessDef {
            id: ProcessId::new("mesocosm", "contract"),
            native: Some(Process::Contract),
            expressed_by: vec![Role::Limb],
            seeding: Seeding::Geometry,
        },
        ProcessDef {
            id: ProcessId::new("mesocosm", "fix"),
            native: Some(Process::Fix),
            expressed_by: vec![Role::Plate],
            seeding: Seeding::Geometry,
        },
        ProcessDef {
            id: ProcessId::new("mesocosm", "intake"),
            native: Some(Process::Intake),
            expressed_by: vec![Role::Mass],
            seeding: Seeding::Geometry,
        },
        ProcessDef {
            id: ProcessId::new("mesocosm", "secrete"),
            native: Some(Process::Secrete),
            // **The same shape that fixes.** Area against the world is what a
            // toxin surface needs too, which is why a nettle's sting is on its
            // leaf; and it puts the tradeoff where PD1a wanted it, inside one
            // organ. A plate on a consumer is armour rather than a frond, so
            // the same rule lets an animal arm its shell without becoming a
            // plant.
            expressed_by: vec![Role::Plate],
            // **Nothing grows a gland.** This is the whole of PD2's first
            // done-condition: expressing it is an act with a record, so a body
            // that has one was given one.
            seeding: Seeding::Acquired,
        },
        ProcessDef {
            id: ProcessId::new("mesocosm", "sense"),
            native: Some(Process::Sense),
            expressed_by: vec![Role::Sensor],
            seeding: Seeding::Geometry,
        },
    ]
}

static NATIVE: LazyLock<Registry> = LazyLock::new(|| {
    Registry::admit(native_defs()).expect("the native table is canonical and free of collisions")
});

impl Registry {
    /// The ruleset this build ships.
    ///
    /// Borrowed and built once: PD3's owned [`ProcessId`] means a registry
    /// carries allocations, and this is read on every validation.
    pub fn native() -> &'static Registry {
        &NATIVE
    }

    /// Admits a set of definitions as one ruleset.
    ///
    /// **The only door.** Sorting is done here rather than trusted from the
    /// caller, so admission order cannot leak into the ruleset; a repeated
    /// qualified id is returned rather than silently collapsed, because two
    /// definitions claiming one name is exactly the collision a pack must be
    /// refused for.
    pub fn admit(mut defs: Vec<ProcessDef>) -> Result<Self, ProcessId> {
        defs.sort_by(|a, b| a.id.cmp(&b.id));
        if let Some(pair) = defs.windows(2).find(|pair| pair[0].id == pair[1].id) {
            return Err(pair[0].id.clone());
        }
        Ok(Self { defs })
    }

    pub fn all(&self) -> impl Iterator<Item = &ProcessDef> {
        self.defs.iter()
    }

    pub fn len(&self) -> usize {
        self.defs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.defs.is_empty()
    }

    pub fn get(&self, id: &ProcessId) -> Option<&ProcessDef> {
        self.defs.iter().find(|def| &def.id == id)
    }

    /// The definition a stored reference names, or `None` when this world's
    /// ruleset does not hold it.
    ///
    /// `None` is a real answer and must never be substituted for a similar
    /// local definition (plan §6, missing packs). Allocation refuses rather
    /// than guessing.
    pub fn resolve(&self, reference: ProcessRef) -> Option<&ProcessDef> {
        self.defs
            .iter()
            .find(|def| def.digest() == reference.definition)
    }

    /// The definition a native binding resolves to. Total for the natives
    /// by construction; the bijection is receipted in `process/tests.rs`.
    pub fn of_native(&self, process: Process) -> &ProcessDef {
        self.defs
            .iter()
            .find(|def| def.native == Some(process))
            .expect("every native process is registered")
    }

    /// **The seeding rule**: the definitions growing this shape expresses.
    ///
    /// Not the same question as [`ProcessDef::admits`], which is the site
    /// requirement a proposal must satisfy. A plate is admitted for two
    /// definitions and grows one.
    pub fn seeds(&self, role: Role) -> impl Iterator<Item = &ProcessDef> {
        self.defs
            .iter()
            .filter(move |def| def.seeded() && def.expressed_by.contains(&role))
    }

    /// Digest over the whole admitted ruleset. **Order-independent.**
    ///
    /// The rule-bearing content of a ruleset is the *set* of definitions it
    /// holds, each already reduced to its own digest. Folding the sorted
    /// digests rather than the vector's own order is what makes "the manifest
    /// listed its files the other way round" provably not a rule change, while
    /// one flipped role or seeding byte in any one definition still moves this.
    pub fn digest(&self) -> RulesetDigest {
        let mut digests: Vec<DefinitionDigest> = self.defs.iter().map(|def| def.digest()).collect();
        digests.sort_unstable();
        let mut bytes = (digests.len() as u64).to_le_bytes().to_vec();
        for digest in digests {
            bytes.extend_from_slice(&digest.0.to_le_bytes());
        }
        RulesetDigest(crate::snapshot::hash_bytes(&bytes))
    }
}
