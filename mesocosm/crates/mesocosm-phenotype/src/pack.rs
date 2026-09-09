// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The on-disk schema: exactly what a pack file may say.
//!
//! `deny_unknown_fields` on both records is deliberate and is half of what
//! "malformed schema is refused" means. A typo in a key is not a comment: a
//! definition whose `expresed_by` was silently ignored would admit a process
//! nothing can express, and the world would be a different world for a reason
//! nobody chose. The other half is that every word in a rule-bearing field is
//! a closed set — [`role_of`] and [`seeding_of`] answer `None` rather than
//! guessing.

use serde::{Deserialize, Serialize};

use mesocosm_core::{NATIVE_ABI, Role, Seeding};

/// The pack format ABI this build reads. One, and it is the core's own.
pub const SUPPORTED_ABI: u32 = NATIVE_ABI;

/// `mesocosm-pack.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// The pack's own id. Not the definitions' namespace — a pack may declare
    /// definitions in more than one namespace, and a namespace may be spoken
    /// for by more than one pack version.
    pub pack: String,
    /// The pack's friendly version. **Not rule-bearing**: a world records the
    /// ruleset digest, not a version string, precisely so that editing a pack
    /// without bumping its version cannot pass for the old biology.
    pub version: String,
    /// The format ABI. An admission gate, not a digest input.
    pub abi: u32,
    /// SPDX metadata. Recorded, never used to relicense anything.
    pub license: String,
    /// Author-facing prose. Outside rule authority, plan §3.
    #[serde(default)]
    pub note: String,
    /// Every declared definition file, relative to the pack root.
    pub processes: Vec<String>,
    /// Every declared expression script, relative to the pack root. (PD4)
    ///
    /// The `expression/` arm plan §5 sketched and left for this gate. Declared
    /// rather than discovered, for the same reason a definition is: a host may
    /// open exactly the scripts the manifest names, so a file that happens to
    /// be lying in the directory is not code this game will run.
    #[serde(default)]
    pub expression: Vec<String>,
    /// Every declared fixture, relative to the pack root. (PD4)
    ///
    /// A fixture is a claim about what a script does, so an undeclared one is
    /// refused by the same rule an undeclared definition is: admission would
    /// otherwise depend on what was left in the directory.
    #[serde(default)]
    pub fixtures: Vec<String>,
}

/// One `processes/<name>.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessFile {
    pub namespace: String,
    pub name: String,
    /// The shapes that may express it: the site requirement. Rule-bearing.
    pub expressed_by: Vec<String>,
    /// `geometry` or `acquired`. Rule-bearing — a world whose plates grew
    /// glands is a different world.
    pub seeding: String,
    /// The plain word a panel uses. Outside rule authority.
    #[serde(default)]
    pub label: String,
    /// Author-facing explanation. Outside rule authority.
    #[serde(default)]
    pub note: String,
}

/// The role a pack's word names, or `None` for a word this build does not
/// hold.
///
/// A closed set on purpose. A pack cannot mint a shape: the roles are what
/// [`classify`](mesocosm_core::classify) produces out of geometry, and a
/// definition that could name a fifth would be a site requirement no part
/// could ever satisfy.
pub fn role_of(word: &str) -> Option<Role> {
    match word {
        "mass" => Some(Role::Mass),
        "limb" => Some(Role::Limb),
        "plate" => Some(Role::Plate),
        "sensor" => Some(Role::Sensor),
        _ => None,
    }
}

/// The seeding rule a pack's word names, or `None`.
pub fn seeding_of(word: &str) -> Option<Seeding> {
    match word {
        "geometry" => Some(Seeding::Geometry),
        "acquired" => Some(Seeding::Acquired),
        _ => None,
    }
}
