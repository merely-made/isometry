// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! S3, the joint receipt: one sortie and return.
//!
//! The execution plan keeps social willingness and combat execution as
//! separate owners that never read each other; this crate is the one place
//! they meet. The world and kinematics come from `paredros-room` and
//! mesocosm's near tier exactly as S0 consumes them; willingness comes from
//! `paredros-social` exactly as S1 and S2 built it; who the player is being
//! comes from `paredros-identity`'s control pointer. What is new here is
//! only the composition: parts negotiated at departure, drivers that read
//! agreements instead of orders, the wound law over the terrain's own
//! falls, the pact that governs tag-in, and the deeds a sortie leaves
//! behind that later explain an answer.
//!
//! **Nothing here commands a companion.** [`Sortie::advance`] takes no
//! input at all: the played body walks toward the leg's goal, companions
//! walk by their agreed parts, and a companion whose agreement never
//! formed stands at home for the whole run, unmovable by anything in this
//! crate.

pub mod march;
pub mod party;
pub mod scene;
pub mod sortie;

pub use party::{Departure, Pact, Part};
pub use sortie::{
    LEAD, MAX_TICKS, Salvage, Sortie, SortieEvent, TEND_DROP, TEND_REACH, TRAIL, Wound,
};
