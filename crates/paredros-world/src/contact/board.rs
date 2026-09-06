// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl ContactWorld {
    pub(super) fn release_board(&mut self, id: BodyId, body: BodyState) {
        if self.board_holder != Some(id) {
            return;
        }
        let board = self.board.expect("held board exists");
        // Releasing removes the grip, not a placement precondition. Preserve
        // the last carried position, including when carry clearance blocked us.
        self.board_holder = None;
        self.board_fall_velocity = Some(body.vertical_velocity.min(0.));
        self.effects.push(ContactEffect::BoardReleased {
            by: id,
            position: board.position,
        });
    }

    pub(super) fn advance_released_board(&mut self) {
        let (Some(mut board), Some(velocity)) = (self.board, self.board_fall_velocity) else {
            return;
        };
        // Fixture-only vertical gravity and swept landing, not a rigid-body
        // simulation. Check every tick so a body supporting it can move away.
        let mut velocity = (velocity - 9.8 * FIXED_DT_SECONDS).max(-20.);
        let bounds = board.collider.translated(board.position);
        let bottom = bounds.min[1] + velocity * FIXED_DT_SECONDS;
        let top = self
            .solids
            .iter()
            .copied()
            .chain(self.bodies.values().map(|b| {
                let mut support = body_box(*b, b.position);
                // Leave room for the character controller's 0.002 skin;
                // exact head contact would obstruct walking out underneath.
                support.max[1] += 0.004;
                support
            }))
            .filter(|s| {
                horizontal_overlap(bounds, *s)
                    && s.max[1] <= bounds.min[1] + 0.001
                    && s.max[1] >= bottom
            })
            .map(|s| s.max[1])
            .max_by(f32::total_cmp);
        if let Some(top) = top {
            board.position[1] = top - board.collider.min[1];
            velocity = 0.;
        } else {
            board.position[1] += velocity * FIXED_DT_SECONDS;
        }
        self.board = Some(board);
        self.board_fall_velocity = Some(velocity);
    }
}
