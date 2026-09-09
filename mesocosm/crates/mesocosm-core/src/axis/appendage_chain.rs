// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The per-segment appendage instructions carried by Recipe wire version 2.

use serde::{Deserialize, Serialize};

use crate::plan::Role;

/// The direction an appendage chain extends from its segment.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainFacing {
    Outward,
    Inward,
    Above,
    Below,
    Front,
    Back,
}

/// One appendage in a recipe's explicit serial chain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppendageStep {
    pub role: Role,
    pub shape: u8,
    pub facing: ChainFacing,
    pub distal: bool,
}
