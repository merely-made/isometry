// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The pack door: authored biology, admitted as data. (PD3)
//!
//! **The dependency runs one way.** This crate depends on `mesocosm-core`; the
//! core does not depend on it. The core is deterministic, integer-only and
//! free of I/O, so reading a directory and deciding whether what is in it may
//! be admitted cannot live there. What crosses back is a
//! [`Registry`](mesocosm_core::Registry) — ordinary native data the core
//! already knows how to run.
//!
//! # Data only
//!
//! A pack is a manifest plus one JSON file per definition. There is no code in
//! it, no path outside it, and no way to name a rule the core does not already
//! evaluate: a definition declares an id, the shapes that may express it, and
//! whether growing one of those shapes grows it. That is the whole vocabulary,
//! and it is exactly what [`ProcessDef`](mesocosm_core::ProcessDef) holds.
//! Piccolo authoring (PD4) arrives beside this, never underneath it.
//!
//! ```text
//! packs/mesocosm/
//!   mesocosm-pack.json      manifest: pack id, version, abi, license, files
//!   processes/
//!     secrete.json          one definition
//!     ...
//! ```
//!
//! # What is rule-bearing
//!
//! A definition's `namespace`, `name`, `expressed_by` and `seeding`, and
//! nothing else. `label` and `note` are the author-facing text plan §3 puts
//! outside rule authority; the manifest's `version`, `license` and `note` are
//! metadata; the order files are listed in is an authoring convenience,
//! because admission sorts by qualified id and
//! [`Registry::digest`](mesocosm_core::Registry::digest) folds the sorted
//! definition digests. JSON whitespace and key order cannot reach the digest
//! at all, since the digest is taken over the lowered definitions rather than
//! over file bytes.
//!
//! # What is refused
//!
//! Every refusal is a named [`Admission`] variant, so a diagnostic says which
//! boundary failed rather than that something failed. Path escape, malformed
//! schema, an unreadable or undeclared file, an unknown format ABI, an
//! unqualified or colliding id, an unknown role or seeding word, and an empty
//! pack. Admission is all-or-nothing: a pack with one bad file admits nothing.
//!
//! # Authored development (PD4)
//!
//! [`express`] is the second door, and it sits *beside* admission rather than
//! underneath it. A pack may also declare Lua that proposes what a body should
//! express; the sandbox is bounded, its entropy is the host's, and what comes
//! back is a proposal the core's one validator accepts or refuses. Lua reaches
//! no world, holds no handle, and registers no host function — see that
//! module's own note on why that is structural rather than a promise.

mod admit;
pub mod express;
mod pack;

pub use admit::{Admission, MANIFEST, admit, admit_dir, asset, discover};
pub use pack::{Manifest, ProcessFile, SUPPORTED_ABI};
