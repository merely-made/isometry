// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Discovery, admission, and lowering.
//!
//! # All or nothing
//!
//! A pack admits completely or not at all. Half a biology is not a smaller
//! biology, it is a different one: a body that cites a definition the partial
//! admission dropped would resolve `None` on every site it occupies, and a
//! world would run something nobody authored.
//!
//! # Deterministic, and independent of the disk
//!
//! Nothing here depends on directory iteration order. The manifest names every
//! file; files are read in the order it names them, lowered, and then sorted
//! into canonical order by [`Registry::admit`]. The one place a directory is
//! read — the undeclared-file check — sorts its entries before reporting one,
//! so two machines refuse the same pack with the same message.

use std::collections::BTreeSet;
use std::path::{Component, Path, PathBuf};

use mesocosm_core::{Process, ProcessDef, ProcessId, Registry};

use crate::pack::{Manifest, ProcessFile, SUPPORTED_ABI, role_of, seeding_of};

/// The manifest's own file name, and what [`discover`] looks for.
pub const MANIFEST: &str = "mesocosm-pack.json";

/// Why a pack was not admitted.
///
/// Every variant names the boundary that failed, and carries enough to say
/// which file and which word. The order the checks run in is part of the
/// contract for the same reason `Refusal`'s is (PD1b): two callers offering
/// the same bad pack must be told the same thing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Admission {
    /// No manifest at that root.
    NoManifest { root: String },
    /// A declared file could not be read.
    Unreadable { path: String, why: String },
    /// A file is not the shape the schema declares: a missing field, an
    /// unknown key, a wrong type, or not JSON at all.
    MalformedSchema { path: String, why: String },
    /// A pack written against a format this build does not read.
    UnknownAbi { found: u32, supported: u32 },
    /// A declared path leaves the pack root. **Refused before it is opened.**
    ///
    /// An absolute path, a drive prefix, a root component, or any `..` — and
    /// then, belt and braces, a resolved path that does not sit under the
    /// resolved root. A pack is a directory of data; a path out of it is a
    /// pack reaching into the machine.
    PathEscape { declared: String },
    /// A `.json` file sits in the pack that the manifest never declared.
    ///
    /// Refused rather than ignored (plan §5, "undeclared files are rejected"):
    /// a definition that is present and unlisted is either a rule someone
    /// meant to admit or a leftover of one, and admitting the directory's
    /// contents instead of the manifest's would make the ruleset depend on
    /// what happened to be lying about.
    UndeclaredFile { path: String },
    /// Two definitions claim one qualified id.
    DuplicateId { id: String },
    /// An empty namespace or name. A pack-qualified id is what stops a friendly
    /// name colliding, so half of one is refused.
    UnqualifiedId { path: String },
    /// A site requirement naming a shape this build does not hold.
    UnknownRole { path: String, word: String },
    /// A seeding rule this build does not hold.
    UnknownSeeding { path: String, word: String },
    /// A definition that no shape can express.
    NoSite { path: String },
    /// A pack declaring nothing.
    EmptyPack { root: String },
}

impl Admission {
    /// The refusal in the plain sentence a diagnostic prints.
    pub fn words(&self) -> String {
        match self {
            Admission::NoManifest { root } => format!("no {MANIFEST} in {root}"),
            Admission::Unreadable { path, why } => format!("{path} could not be read: {why}"),
            Admission::MalformedSchema { path, why } => format!("{path} is malformed: {why}"),
            Admission::UnknownAbi { found, supported } => {
                format!("pack format {found}, and this build reads {supported}")
            },
            Admission::PathEscape { declared } => {
                format!("{declared} leaves the pack root")
            },
            Admission::UndeclaredFile { path } => {
                format!("{path} is in the pack and the manifest does not declare it")
            },
            Admission::DuplicateId { id } => format!("two definitions claim {id}"),
            Admission::UnqualifiedId { path } => format!("{path} has no qualified id"),
            Admission::UnknownRole { path, word } => {
                format!("{path} expects a shape this world does not hold: {word}")
            },
            Admission::UnknownSeeding { path, word } => {
                format!("{path} names a seeding rule this world does not hold: {word}")
            },
            Admission::NoSite { path } => format!("{path} names no shape that could express it"),
            Admission::EmptyPack { root } => format!("{root} declares no definitions"),
        }
    }
}

/// The manifest at `root`, read and checked for format.
///
/// **Discovery is separate from admission** so a host can list what it found,
/// say what version and license each pack declares, and decide what to offer,
/// without having lowered anything into a ruleset yet.
pub fn discover(root: &Path) -> Result<Manifest, Admission> {
    let path = root.join(MANIFEST);
    if !path.is_file() {
        return Err(Admission::NoManifest {
            root: root.display().to_string(),
        });
    }
    let manifest: Manifest = read_json(&path)?;
    if manifest.abi != SUPPORTED_ABI {
        return Err(Admission::UnknownAbi {
            found: manifest.abi,
            supported: SUPPORTED_ABI,
        });
    }
    Ok(manifest)
}

/// Discovers and admits the pack at `root`.
///
/// The whole door in one call: what comes back is an ordinary
/// [`Registry`] the core runs, and its
/// [`digest`](Registry::digest) is what a world records as its
/// [`WorldRules`](mesocosm_core::WorldRules).
pub fn admit_dir(root: &Path) -> Result<Registry, Admission> {
    admit(root, &discover(root)?)
}

/// Admits a manifest already discovered at `root`.
pub fn admit(root: &Path, manifest: &Manifest) -> Result<Registry, Admission> {
    if manifest.processes.is_empty() {
        return Err(Admission::EmptyPack {
            root: root.display().to_string(),
        });
    }

    let mut declared: BTreeSet<PathBuf> = BTreeSet::new();
    let mut defs = Vec::with_capacity(manifest.processes.len());
    for relative in &manifest.processes {
        let path = inside(root, relative)?;
        let file: ProcessFile = read_json(&path)?;
        defs.push(lower(relative, &file)?);
        declared.insert(path);
    }
    // The PD4 arms. Not lowered into the ruleset — a script proposes and a
    // fixture checks, and neither is a rule — but path-checked and counted as
    // declared, so a pack cannot reach out of itself and an undeclared file
    // beside them is still refused.
    for relative in manifest.expression.iter().chain(&manifest.fixtures) {
        declared.insert(inside(root, relative)?);
    }

    // Every definition file in the pack is declared, or the ruleset depends on
    // what is lying about rather than on what was written down.
    undeclared(root, &declared)?;

    Registry::admit(defs).map_err(|id: ProcessId| Admission::DuplicateId { id: id.qualified() })
}

/// One declared asset's path inside the pack. (PD4)
///
/// **The only way to open a pack file.** A relative path the manifest did not
/// declare is refused as `UndeclaredFile` and a path that leaves the root as
/// `PathEscape`, so a host cannot be talked into reading a sibling of the pack
/// after admission has validated it.
pub fn asset(root: &Path, manifest: &Manifest, relative: &str) -> Result<PathBuf, Admission> {
    if !manifest
        .processes
        .iter()
        .chain(&manifest.expression)
        .chain(&manifest.fixtures)
        .any(|declared| declared == relative)
    {
        return Err(Admission::UndeclaredFile {
            path: relative.to_string(),
        });
    }
    inside(root, relative)
}

/// Resolves a declared path and refuses one that leaves the root.
///
/// Two checks, because either alone has a gap. The component walk refuses
/// `..`, a root, and a drive prefix without touching the filesystem, so a path
/// that does not exist is still refused for the right reason; the resolved
/// comparison then catches a symlink out, which no amount of reading the
/// string can.
fn inside(root: &Path, declared: &str) -> Result<PathBuf, Admission> {
    let escape = || Admission::PathEscape {
        declared: declared.to_string(),
    };
    let relative = Path::new(declared);
    for component in relative.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {},
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(escape());
            },
        }
    }
    let joined = root.join(relative);
    // `canonicalize` needs the file to exist; a declared file that does not is
    // an unreadable file rather than an escape, and reads as one below.
    match (joined.canonicalize(), root.canonicalize()) {
        (Ok(resolved), Ok(base)) if !resolved.starts_with(&base) => Err(escape()),
        _ => Ok(joined),
    }
}

/// Refuses a `.json` under the pack root that the manifest never named.
fn undeclared(root: &Path, declared: &BTreeSet<PathBuf>) -> Result<(), Admission> {
    let mut found: Vec<PathBuf> = Vec::new();
    walk(root, &mut found);
    found.sort();
    for path in found {
        if path.file_name().is_some_and(|name| name == MANIFEST) {
            continue;
        }
        let resolved = path.canonicalize().unwrap_or_else(|_| path.clone());
        if declared
            .iter()
            .any(|one| one.canonicalize().unwrap_or_else(|_| one.clone()) == resolved)
        {
            continue;
        }
        return Err(Admission::UndeclaredFile {
            path: path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string(),
        });
    }
    Ok(())
}

fn walk(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, found);
        } else if path.extension().is_some_and(|ext| ext == "json") {
            found.push(path);
        }
    }
}

/// Lowers one checked file into the core's own definition record.
///
/// The **native binding is recovered here, not declared** — a pack is data,
/// and which engine fast path a definition happens to have is the core's
/// index rather than an author's claim. It is not rule-bearing, so a pack that
/// mints something new lowers with `None` and runs through exactly the same
/// validator.
fn lower(relative: &str, file: &ProcessFile) -> Result<ProcessDef, Admission> {
    if file.namespace.is_empty() || file.name.is_empty() {
        return Err(Admission::UnqualifiedId {
            path: relative.to_string(),
        });
    }
    if file.expressed_by.is_empty() {
        return Err(Admission::NoSite {
            path: relative.to_string(),
        });
    }
    let mut expressed_by = Vec::with_capacity(file.expressed_by.len());
    for word in &file.expressed_by {
        let Some(role) = role_of(word) else {
            return Err(Admission::UnknownRole {
                path: relative.to_string(),
                word: word.clone(),
            });
        };
        expressed_by.push(role);
    }
    let Some(seeding) = seeding_of(&file.seeding) else {
        return Err(Admission::UnknownSeeding {
            path: relative.to_string(),
            word: file.seeding.clone(),
        });
    };

    let id = ProcessId::new(&file.namespace, &file.name);
    let native = Process::ALL.into_iter().find(|process| process.id() == &id);
    Ok(ProcessDef {
        id,
        native,
        expressed_by,
        seeding,
    })
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, Admission> {
    let bytes = std::fs::read(path).map_err(|error| Admission::Unreadable {
        path: path.display().to_string(),
        why: error.to_string(),
    })?;
    serde_json::from_slice(&bytes).map_err(|error| Admission::MalformedSchema {
        path: path.display().to_string(),
        why: error.to_string(),
    })
}
