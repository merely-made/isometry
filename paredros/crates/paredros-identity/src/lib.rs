// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The identity prerequisites of the execution plan's §1, in minimal form.
//!
//! Four facts kept four, so that tag-in, succession, chassis replacement, and
//! descent stay four different operations rather than one muddle:
//!
//! - [`SubjectId`], the continuing person, across bodies and across control;
//! - [`BodyRevisionId`], which body at which revision;
//! - [`Facets`], character and faction facets kept separate, because a role is
//!   held and not been;
//! - [`Control`], who the player is currently being, as recorded state.
//!
//! This crate exists rather than a module because of the plan's two-owners
//! rule. Social willingness and combat execution are separate owners that never
//! read each other, and both need to name a person. A foundation crate that
//! knows nothing about either is the wall; putting `SubjectId` in one owner
//! would force the other to depend on it.
//!
//! [`BodyRevisionId`] is opaque here on purpose. The wing phenotype contract
//! owns what a body revision means; Paredros only needs to point at one.

pub mod control;
pub mod facets;

pub use control::{Control, ControlIntent};
pub use facets::{Facets, Holding, Office};

use serde::{Deserialize, Serialize};

/// The continuing person. Survives a new body, a change of office, a change
/// of who is controlling them, and their own death as far as the record goes.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct SubjectId(pub u64);

/// Which body, at which revision. Opaque here: nothing in this repository
/// interprets the number, and the wing phenotype contract owns its semantics.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct BodyRevisionId(pub u64);

/// A faction. A faction is a relationship rather than a property, which is why
/// membership is stored as a pair in [`Facets`] and not as a field on a person.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct FactionId(pub u64);

/// An office: a role that exists apart from whoever holds it.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct OfficeId(pub u64);

/// The shared clock. Both owners stamp their records the same way, which is
/// what lets a deed and a body revision be put in one order later.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Tick(pub u64);

impl Tick {
    pub fn after(self, ticks: u64) -> Self {
        Self(self.0.saturating_add(ticks))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IdentityError {
    /// An office was founded twice under the same id.
    OfficeExists(OfficeId),
    NoSuchOffice(OfficeId),
    /// Someone else already holds it. Offices are vacated, not overwritten.
    OfficeHeld(OfficeId, SubjectId),
    /// Nobody holds it, so there is nothing to vacate.
    OfficeVacant(OfficeId),
    /// A control log began twice.
    AlreadyBegun,
    /// A control log that does not open with [`ControlIntent::Begin`].
    NotBegun,
    /// Tag-in while already tagged in, which would be two bodies at once.
    AlreadyTaggedIn,
    /// Tag-out while not tagged in.
    NotTaggedIn,
}
