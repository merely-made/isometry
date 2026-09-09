// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Throwaway PD2 staging probe.
use mesocosm_core::{Role, World, classify};

fn main() {
    let world = World::new(7, 916);
    let me = world.controlled().unwrap();
    let mut plated = 0;
    let mut eligible = Vec::new();
    for o in world.living() {
        let plates: Vec<_> = o
            .body()
            .living()
            .filter(|p| classify(p.half_extent) == Role::Plate)
            .map(|p| (p.id.0, p.half_extent))
            .collect();
        if plates.is_empty() {
            continue;
        }
        plated += 1;
        if world.is_eligible(o.id) && eligible.len() < 6 {
            let d = (0..3)
                .map(|a| (o.position[a] - me.position[a]).abs())
                .max()
                .unwrap();
            let canopy: Vec<_> = o.body().canopy_parts().map(|p| p.0).collect();
            eligible.push((o.id.0, o.kingdom(), plates.len(), plates[0], canopy, d));
        }
    }
    println!("living: {}  plated: {plated}", world.living().count());
    println!("frontier {}", world.frontier());
    for e in &eligible {
        println!("  eligible plated: {e:?}");
    }
}
