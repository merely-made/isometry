// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl ContactWorld {
    pub(super) fn step_body(&mut self, id: BodyId, input: Input) {
        let mut body = *self.bodies.get(&id).expect("accepted body exists");
        body.bracing = input.held.brace;
        if body.attack_cooldown > 0 {
            body.attack_cooldown -= 1;
        }
        if body.attack_remaining > 0 {
            body.attack_remaining -= 1;
            if body.attack_remaining == 0 {
                body.attack_cooldown = body.profile.attack_recovery_ticks;
                self.resolve_attack(id);
            }
        }
        if body.recovery_remaining > 0 {
            body.recovery_remaining -= 1;
            if body.recovery_remaining == 0 {
                body.impairment = None;
                self.effects.push(ContactEffect::Recovered { body: id });
            }
        }
        if input.triggered.recover && body.impairment.is_some() && body.recovery_remaining == 0 {
            body.recovery_remaining = body.profile.recover_ticks;
            self.effects
                .push(ContactEffect::RecoveryStarted { body: id });
        }
        let look = [input.held.look_x, 0.0, input.held.look_z];
        let move_ = [input.held.move_x, 0.0, input.held.move_z];
        if length_xz(look) > 0.001 {
            body.heading = unit_xz(look);
        } else if length_xz(move_) > 0.001 {
            body.heading = unit_xz(move_);
        }
        if input.triggered.release {
            self.release_board(id, body);
        } else if input.triggered.interact {
            self.toggle_board(id, &mut body);
        }
        if input.triggered.anchor {
            if body.anchored {
                body.anchored = false;
                body.anchor = None;
            } else if body.impairment != Some(Impairment::Grip) {
                body.anchor = self.anchor_point(body);
                body.anchored = body.anchor.is_some();
            }
            self.effects.push(if body.anchored {
                ContactEffect::Anchored { body: id }
            } else {
                ContactEffect::AnchorLost { body: id }
            });
        }
        if input.triggered.attack
            && body.attack_remaining == 0
            && body.attack_cooldown == 0
            && body.recovery_remaining == 0
        {
            body.attack_remaining = body.profile.attack_windup_ticks.max(1);
            self.effects.push(ContactEffect::AttackStarted { body: id });
        }
        let speed = if body.bracing {
            body.profile.brace_speed
        } else {
            body.profile.move_speed
        };
        let direction = if length_xz(move_) > 1.0 {
            unit_xz(move_)
        } else {
            move_
        };
        let desired = [
            direction[0] * speed * FIXED_DT_SECONDS,
            0.0,
            direction[2] * speed * FIXED_DT_SECONDS,
        ];
        self.advance_contact(&mut body, desired);
        self.bodies.insert(id, body);
        self.follow_held_board(id);
    }

    fn anchor_point(&self, body: BodyState) -> Option<Position> {
        self.solids
            .iter()
            .copied()
            .map(|solid| nearest_point(body.position, solid))
            .filter(|point| {
                let toward = sub(*point, body.position);
                point[1] > body.position[1] + 0.2
                    && (length_xz(toward) < 0.01 || dot_xz(body.heading, toward) > 0.0)
                    && distance(body.position, *point) <= body.profile.anchor_range
            })
            .min_by(|a, b| distance(body.position, *a).total_cmp(&distance(body.position, *b)))
    }
    pub(super) fn limit_tether(&mut self, body: BodyState, proposed: Position) -> Position {
        if !body.anchored {
            return proposed;
        }
        let Some(point) = body.anchor else {
            return proposed;
        };
        let offset = sub(proposed, point);
        let length = length(offset);
        if length <= body.profile.tether_length {
            proposed
        } else {
            let scale = body.profile.tether_length / length;
            [
                point[0] + offset[0] * scale,
                point[1] + offset[1] * scale,
                point[2] + offset[2] * scale,
            ]
        }
    }
    fn toggle_board(&mut self, id: BodyId, body: &mut BodyState) {
        let Some(mut board) = self.board else {
            return;
        };
        if self.board_holder == Some(id) {
            let mut position = add(
                body.position,
                [body.heading[0] * 2.4, 0.0, body.heading[2] * 2.4],
            );
            let candidate = board.collider.translated(position);
            let Some(top) = self
                .solids
                .iter()
                .filter(|s| {
                    horizontal_overlap(candidate, **s) && s.max[1] <= candidate.min[1] + 0.001
                })
                .map(|s| s.max[1])
                .max_by(f32::total_cmp)
            else {
                return;
            };
            position[1] = top - board.collider.min[1];
            let candidate = board.collider.translated(position);
            if !self
                .solids
                .iter()
                .copied()
                .any(|solid| candidate.overlaps(solid))
                && !self
                    .bodies
                    .values()
                    .any(|b| candidate.overlaps(body_box(*b, b.position)))
            {
                board.position = position;
                self.board = Some(board);
                self.board_holder = None;
                self.board_fall_velocity = None;
                self.effects
                    .push(ContactEffect::BoardPlaced { by: id, position });
            }
            return;
        }
        if self.board_holder.is_none()
            && body.profile.carry_capacity >= board.mass
            && body.impairment != Some(Impairment::Grip)
            && distance(
                body.position,
                nearest_point(body.position, board.collider.translated(board.position)),
            ) <= body.profile.reach
            && !self.solids.iter().any(|s| {
                board
                    .collider
                    .translated(carried_position(*body, body.position))
                    .overlaps(*s)
            })
        {
            self.board_holder = Some(id);
            self.board_fall_velocity = None;
            self.effects.push(ContactEffect::BoardHeld { by: id });
        }
    }
    fn follow_held_board(&mut self, id: BodyId) {
        if self.board_holder != Some(id) {
            return;
        }
        let body = self.bodies[&id];
        if let Some(board) = &mut self.board {
            board.position = carried_position(body, body.position);
        }
    }
    fn resolve_attack(&mut self, source: BodyId) {
        let attacker = *self.bodies.get(&source).expect("attacker exists");
        let reach = if attacker.impairment == Some(Impairment::Reach) {
            attacker.profile.reach * 0.5
        } else {
            attacker.profile.reach
        };
        let Some(target) = self
            .bodies
            .values()
            .filter(|other| {
                let delta = sub(other.position, attacker.position);
                let distance = length_xz(delta);
                other.id != source
                    && distance > 0.001
                    && distance <= reach
                    && delta[1].abs() <= 1.0
                    && dot_xz(attacker.heading, unit_xz(delta)) >= 0.5
                    && !self.solids.iter().copied().any(|solid| {
                        segment_hits(
                            add(
                                attacker.position,
                                [0., attacker.profile.half_extents[1], 0.],
                            ),
                            add(other.position, [0., other.profile.half_extents[1], 0.]),
                            solid,
                        )
                    })
            })
            .min_by(|a, b| {
                distance_xz(attacker.position, a.position)
                    .total_cmp(&distance_xz(attacker.position, b.position))
            })
            .copied()
        else {
            return;
        };
        let target = self.bodies.get_mut(&target.id).expect("target exists");
        let target_id = target.id;
        let braced = target.grounded && target.bracing;
        target.integrity = (target.integrity - if braced { 0.5 } else { 1.0 }).max(0.0);
        if braced {
            self.effects.push(ContactEffect::AttackBraced {
                source,
                target: target_id,
            });
            return;
        }
        let impairment = if target.kind == BodyKind::Climber {
            Impairment::Grip
        } else {
            Impairment::Reach
        };
        target.impairment = Some(impairment);
        if impairment == Impairment::Grip {
            target.anchored = false;
            target.anchor = None;
        }
        self.effects.push(ContactEffect::AttackHit {
            source,
            target: target_id,
            impairment,
        });
    }

    pub(super) fn configuration_open(&self) -> Result<(), ContactError> {
        if self.tick == 0 && self.inputs.is_empty() {
            Ok(())
        } else {
            Err(ContactError::ConfigurationLocked)
        }
    }
}
