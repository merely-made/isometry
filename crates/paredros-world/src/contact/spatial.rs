// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Lower fixture geometry into the existing shared character controller.
//! This small query world is reconstructed from product state per move; it is
//! derived, never a second save authority. Retention is a measured future gate.

use super::*;
use conatus::{
    BodyDesc, BodyWorld, CharacterAutostep, CharacterConfig, ColliderDesc, ColliderId,
    ColliderShape, Transform,
};

impl ContactWorld {
    pub(super) fn advance_contact(&mut self, body: &mut BodyState, horizontal: Position) {
        body.vertical_velocity = (body.vertical_velocity - 9.8 * FIXED_DT_SECONDS).max(-20.);
        let intended = add(
            body.position,
            [
                horizontal[0],
                body.vertical_velocity * FIXED_DT_SECONDS,
                horizontal[2],
            ],
        );
        let constrained = self.limit_tether(*body, intended);
        let requested = sub(constrained, body.position);
        let mut spatial = BodyWorld::new([0.; 3]);
        for solid in self
            .solids
            .iter()
            .copied()
            .chain(
                self.board
                    .into_iter()
                    .filter(|_| self.board_holder != Some(body.id))
                    .map(|b| b.collider.translated(b.position)),
            )
            .chain(
                self.bodies
                    .values()
                    .filter(|b| b.id != body.id)
                    .map(|b| body_box(*b, b.position)),
            )
        {
            spatial
                .spawn(box_desc(solid, false))
                .expect("admitted fixture solid");
        }
        let id = spatial
            .spawn(box_desc(body_box(*body, body.position), true))
            .expect("admitted fixture body");
        // Populate the shared query acceleration structure before the cast.
        spatial.step(FIXED_DT_SECONDS).expect("fixed query step");
        let movement = spatial
            .move_character(
                ColliderId::new(id, 0),
                requested,
                FIXED_DT_SECONDS,
                CharacterConfig {
                    offset: 0.002,
                    autostep: body.grounded.then_some(CharacterAutostep {
                        max_height: 0.25,
                        min_width: 0.1,
                        include_dynamic_bodies: false,
                    }),
                    snap_to_ground: Some(0.02),
                    ..CharacterConfig::default()
                },
            )
            .expect("admitted character request");
        let mut at = add(body.position, movement.applied);
        if self.board_holder == Some(body.id) {
            let board = self
                .board
                .expect("held board")
                .collider
                .translated(carried_position(*body, at));
            if self.solids.iter().any(|s| board.overlaps(*s)) {
                // Carry clearance is a product action precondition, not a body solver.
                at = body.position;
            }
        }
        if horizontal != [0.; 3]
            && (at[0] - intended[0]).abs() + (at[2] - intended[2]).abs() > 0.005
        {
            self.effects.push(ContactEffect::Blocked { body: body.id });
        }
        body.position = at;
        body.grounded = movement.grounded;
        if body.grounded || (body.anchored && constrained[1] > intended[1] + 0.0001) {
            body.vertical_velocity = 0.;
        }
    }
}

fn box_desc(bounds: BoxCollider, moving: bool) -> BodyDesc {
    let center = std::array::from_fn(|i| (bounds.min[i] + bounds.max[i]) * 0.5);
    let half_extents = std::array::from_fn(|i| (bounds.max[i] - bounds.min[i]) * 0.5);
    let desc = if moving {
        BodyDesc::kinematic_position()
    } else {
        BodyDesc::fixed()
    };
    desc.at(Transform::from_translation(center))
        .with_collider(ColliderDesc::new(ColliderShape::Box { half_extents }))
}
