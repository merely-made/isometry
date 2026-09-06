// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn carried_position(body: BodyState, position: Position) -> Position {
    add(
        position,
        [
            body.heading[0] * 0.8,
            body.profile.half_extents[1] * 2.0 + 0.1,
            body.heading[2] * 0.8,
        ],
    )
}
pub(super) fn body_box(body: BodyState, position: Position) -> BoxCollider {
    BoxCollider {
        min: [
            position[0] - body.profile.half_extents[0],
            position[1],
            position[2] - body.profile.half_extents[2],
        ],
        max: [
            position[0] + body.profile.half_extents[0],
            position[1] + body.profile.half_extents[1] * 2.0,
            position[2] + body.profile.half_extents[2],
        ],
    }
}
pub(super) fn add(a: Position, b: Position) -> Position {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub(super) fn sub(a: Position, b: Position) -> Position {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub(super) fn length_xz(a: Position) -> f32 {
    (a[0] * a[0] + a[2] * a[2]).sqrt()
}
pub(super) fn length(a: Position) -> f32 {
    (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt()
}
pub(super) fn distance_xz(a: Position, b: Position) -> f32 {
    length_xz(sub(a, b))
}
pub(super) fn distance(a: Position, b: Position) -> f32 {
    length(sub(a, b))
}
pub(super) fn unit_xz(a: Position) -> Position {
    let length = length_xz(a);
    [a[0] / length, 0.0, a[2] / length]
}
pub(super) fn dot_xz(a: Position, b: Position) -> f32 {
    a[0] * b[0] + a[2] * b[2]
}
pub(super) fn segment_hits(from: Position, to: Position, box_: BoxCollider) -> bool {
    let delta = sub(to, from);
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    for (origin, direction, min, max) in [
        (from[0], delta[0], box_.min[0], box_.max[0]),
        (from[1], delta[1], box_.min[1], box_.max[1]),
        (from[2], delta[2], box_.min[2], box_.max[2]),
    ] {
        if direction.abs() < f32::EPSILON {
            if origin < min || origin > max {
                return false;
            }
        } else {
            let a = (min - origin) / direction;
            let b = (max - origin) / direction;
            enter = enter.max(a.min(b));
            exit = exit.min(a.max(b));
        }
    }
    enter <= exit && exit >= 0.0 && enter <= 1.0
}
pub(super) fn horizontal_overlap(a: BoxCollider, b: BoxCollider) -> bool {
    a.min[0] < b.max[0] && a.max[0] > b.min[0] && a.min[2] < b.max[2] && a.max[2] > b.min[2]
}
pub(super) fn nearest_point(point: Position, box_: BoxCollider) -> Position {
    [
        point[0].clamp(box_.min[0], box_.max[0]),
        point[1].clamp(box_.min[1], box_.max[1]),
        point[2].clamp(box_.min[2], box_.max[2]),
    ]
}
