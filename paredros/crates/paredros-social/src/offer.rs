// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What is asked, and on what terms.
//!
//! An offer is asked. Constructing one commits nobody to anything: it is a
//! proposal handed to [`crate::Society::consider`], and the answer that comes
//! back is theirs. There is no call anywhere in this crate that makes a
//! companion do a piece of work.

use paredros_identity::SubjectId;
use serde::{Deserialize, Serialize};

use crate::companion::Craft;

/// The work itself. Grade is what it takes; danger is what it costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Work {
    pub craft: Craft,
    pub grade: u8,
    pub danger: u8,
}

impl Work {
    pub fn new(craft: Craft, grade: u8, danger: u8) -> Self {
        Self {
            craft,
            grade,
            danger,
        }
    }
}

/// What the asker promises. `share` is the portion of what comes back;
/// `danger_cap` is the most danger they will be asked to bear under this
/// arrangement, which is the term a standing agreement is really made of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Terms {
    pub share: u8,
    pub danger_cap: u8,
}

impl Terms {
    pub fn new(share: u8, danger_cap: u8) -> Self {
        Self { share, danger_cap }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    pub asked_by: SubjectId,
    pub asked_of: SubjectId,
    pub work: Work,
    pub terms: Terms,
}

impl Offer {
    pub fn new(asked_by: SubjectId, asked_of: SubjectId, work: Work, terms: Terms) -> Self {
        Self {
            asked_by,
            asked_of,
            work,
            terms,
        }
    }

    /// The same ask, put to somebody else.
    pub fn to(&self, asked_of: SubjectId) -> Self {
        Self { asked_of, ..*self }
    }

    /// The same ask, on the terms they countered with.
    pub fn on(&self, terms: Terms) -> Self {
        Self { terms, ..*self }
    }
}
