// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::{DevelopmentError, Founding, PartPalette, World};

use super::Runtime;

impl Runtime {
    /// A recorded family origin in the clearing, cave and tunnel scene.
    pub fn family_clearing(
        seed: u64,
        ticks_per_second: u32,
        founding: Founding,
        palette: PartPalette,
        reserve_assisted: bool,
    ) -> Result<Self, DevelopmentError> {
        let world = World::family_clearing(seed, founding, palette, reserve_assisted)?;
        let organisms = world.organisms.len().saturating_sub(1) as u32;
        let mut runtime = Self::from_world(world, seed, organisms, ticks_per_second);
        let origin = runtime.world.drain_events();
        runtime.history.record_all(origin);
        Ok(runtime)
    }

    /// Drives the authored single-critter expression practice scene.
    pub fn expression_practice(
        seed: u64,
        ticks_per_second: u32,
        founding: Founding,
        palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        let world = World::expression_practice(seed, founding, palette)?;
        let organisms = u32::try_from(world.organisms.len().saturating_sub(1))
            .expect("the expression-practice population fits a trace count");
        Ok(Self::from_world(world, seed, organisms, ticks_per_second))
    }

    /// Drives the authored family-practice scene.
    pub fn family_practice(
        seed: u64,
        ticks_per_second: u32,
        founding: Founding,
        palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        let world = World::family_practice(seed, founding, palette)?;
        let organisms = u32::try_from(world.organisms.len().saturating_sub(1))
            .expect("the family-practice population fits a trace count");
        let mut runtime = Self::from_world(world, seed, organisms, ticks_per_second);
        let origin_events = runtime.world.drain_events();
        runtime.history.record_all(origin_events);
        Ok(runtime)
    }
}
