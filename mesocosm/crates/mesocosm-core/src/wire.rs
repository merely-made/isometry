// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's compatibility name for the wing's raw interchange framing.
//!
//! Product schemas and validation remain local. The fixed header and refusal
//! ordering are shared so every v0 reader sees the same bytes and errors.

pub use wing_formats::{HEADER_LEN, WireError, frame, peek, unframe};
