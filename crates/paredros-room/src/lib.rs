// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S0, the room probe: the first game code in this repository.
//!
//! One body, one room, a close camera, a fixed input trace, and a rendered
//! receipt. Everything load-bearing is consumed rather than grown here, per
//! the execution plan's stop rule:
//!
//! - the world and the room are `mesocosm-core`'s [`places`] tier: a grown
//!   world, brick ground, and a room carved into a hillside;
//! - movement is `near::step` exactly, with no kinematics of our own;
//! - geometry is `mesocosm-mesh`'s greedy mesher over the same bricks the
//!   walker collides against;
//! - the picture is renderling on netrender's device, composed into
//!   netrender's master frame as an external texture.
//!
//! What is Paredros's own is small and deliberate: which room, which trace,
//! where the camera sits, and the save/replay discipline the later gates
//! inherit.
//!
//! [`places`]: mesocosm_core::places

#[cfg(feature = "r1-proof")]
mod brick;
pub mod frame_health;
pub mod crossing;
pub mod gpu;
pub mod probe;
#[cfg(feature = "v1-proof")]
pub mod residency;
pub mod room;
pub mod scene;

pub use probe::{Probe, ProbeError, Save, TICKS, TRACE};
pub use room::{Room, RoomError};
