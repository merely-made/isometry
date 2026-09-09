// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Walking toward somewhere, on mesocosm's kinematics.
//!
//! Nothing here moves a body except `near::step`, per the stop rule. What
//! this module owns is only which step to ask for: the signum heading toward
//! a goal, and one deterministic shoulder-around when the terrain refuses it.
//! `near::step` slides only on diagonal headings, so a cardinal march into a
//! two-voxel riser would hold forever; trying the two perpendiculars in a
//! fixed order is that slide rule lifted one level, and just as replayable.

use mesocosm_core::places::{Ground, step};

/// One walking tick toward a goal in x and z. Returns where the body ends
/// up, which is where it started when everything in that direction refuses.
///
/// `shoulder` is the walker's memory of which way it last shouldered
/// around an obstacle: +1 clockwise, -1 counter, 0 free. Without it, a
/// wall that blocks the direct heading bounces the walker between the two
/// perpendiculars forever; with it, the walker follows the wall one way
/// until the direct heading opens again. One integer of state, and as
/// replayable as the step itself.
pub fn toward(ground: &Ground, at: [i32; 3], goal: [i32; 2], shoulder: &mut i8) -> [i32; 3] {
    let heading = [(goal[0] - at[0]).signum(), (goal[1] - at[2]).signum()];
    if heading == [0, 0] {
        return at;
    }
    let attempt =
        |from: [i32; 3], h: [i32; 2]| step(ground, from, [from[0] + h[0], from[1], from[2] + h[1]]);

    let direct = attempt(at, heading);
    if direct != at {
        *shoulder = 0;
        return direct;
    }
    let side = |sign: i8| {
        if sign >= 0 {
            [heading[1], -heading[0]]
        } else {
            [-heading[1], heading[0]]
        }
    };
    let first = if *shoulder == 0 { 1 } else { *shoulder };
    let along = attempt(at, side(first));
    if along != at {
        *shoulder = first;
        return along;
    }
    let other = attempt(at, side(-first));
    if other != at {
        *shoulder = -first;
        return other;
    }
    at
}

/// Whether a body is standing on the goal column.
pub fn arrived(at: [i32; 3], goal: [i32; 2]) -> bool {
    at[0] == goal[0] && at[2] == goal[1]
}

/// Chebyshev distance in the walking plane.
pub fn apart(a: [i32; 3], b: [i32; 3]) -> i32 {
    (a[0] - b[0]).abs().max((a[2] - b[2]).abs())
}

/// A standing spot on the open surface of a column, if the column has one.
pub fn stand(ground: &Ground, x: i32, z: i32) -> Option<[i32; 3]> {
    use mesocosm_core::places::WALKER_HEIGHT;
    let top = ground.surface(x, z)?;
    let at = [x, top + 1, z];
    ground.stands(at, WALKER_HEIGHT).then_some(at)
}
