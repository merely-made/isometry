// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P3 and P2-live of mere's spatial compute plan, wing side.
//!
//! A resident padded-3D position buffer advanced by explicit-regime
//! kernels, published into renderling's slab by an adapter dispatch,
//! drawn as instanced lit geometry, and composed by netrender. Two
//! binaries share it: `ambience-lease` runs offscreen and measures,
//! `live` presents it in a window.
//!
//! The kernel shape is mere's resident-graph copied rather than shared,
//! per the plan: sharing is what promotion is for, and the copy is what
//! makes the neutrality claim testable at all.

pub mod compose;
pub mod lease;
pub mod tenant;
