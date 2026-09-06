// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Authored dry crossing. These boxes are both the contact input and the mesh;
//! this is a playable fixture, not generated terrain or an Isometry actor model.

mod draw;

use mesocosm_render::geometry::Vertex;
use netrender::Scene;
pub use paredros_world::{BodyId, BodyKind, ContactWorld, HeldInput, Input, TriggeredInput};
use paredros_world::{BoxCollider, MovableBoard};
use renderling::glam::{Mat4, Vec3};

pub fn new_world(kind: BodyKind) -> (ContactWorld, BodyId) {
    let mut solids = vec![
        bounds([-12., -2., -7.], [-1.3, 0., 7.]),
        bounds([1.3, -2., -7.], [12., 0., 7.]),
        bounds([-1.3, -2., -7.], [1.3, -1.4, 7.]),
        bounds([-12.3, -2., -7.3], [-12., 2., 7.3]),
        bounds([12., -2., -7.3], [12.3, 2., 7.3]),
        bounds([-12., -2., -7.3], [12., 2., -7.]),
        bounds([-12., -2., 7.], [12., 2., 7.3]),
        // A low overhead hold. Its underside clears both intact bodies.
        bounds([-2.2, 1.9, -2.2], [2.2, 2.2, -1.8]),
        bounds([-2.5, 0., -2.3], [-2.2, 2.2, -1.7]),
        bounds([2.2, 0., -2.3], [2.5, 2.2, -1.7]),
    ];
    // Broad, shallow steps make an unsuccessful crossing recoverable by foot.
    for step in 0..7 {
        solids.push(bounds(
            [-1.3, -1.4, 2.8 + step as f32 * 0.6],
            [1.3, -1.2 + step as f32 * 0.2, 3.4 + step as f32 * 0.6],
        ));
    }
    let mut world = ContactWorld::new(solids);
    let player = world.add_body(kind, [-6., 0., 0.]).expect("fixture player");
    // A stationary practice body, not a inhabitant with social authority.
    world
        .add_body(BodyKind::Climber, [5., 0., 0.])
        .expect("practice body");
    world
        .set_board(MovableBoard {
            collider: bounds([-1.8, 0., -0.6], [1.8, 0.2, 0.6]),
            position: [-3.5, 0., 0.],
            mass: 4.,
        })
        .expect("fixture board");
    (world, player)
}

fn bounds(min: [f32; 3], max: [f32; 3]) -> BoxCollider {
    BoxCollider { min, max }
}

pub fn geometry(world: &ContactWorld) -> Vec<Vertex> {
    let mut mesh = Vec::new();
    for (i, solid) in world.solids().iter().enumerate() {
        let color = match i {
            0 | 1 => [0.55, 0.65, 0.63],
            2 => [0.28, 0.40, 0.45],
            3..=6 => [0.32, 0.42, 0.46],
            7..=9 => [0.40, 0.85, 0.84],
            _ => [0.66, 0.72, 0.61],
        };
        draw::cuboid(&mut mesh, solid.min, solid.max, color);
    }
    // Paint lies on the existing surface. It does not pretend to be a collider.
    for side in [-1., 1.] {
        for z in -6..=6 {
            let x = side * 1.48;
            draw::cuboid(
                &mut mesh,
                [x - 0.09, 0.002, z as f32 - 0.18],
                [x + 0.09, 0.008, z as f32 + 0.18],
                [0.93, 0.66, 0.22],
            );
        }
    }
    if let Some(board) = world.board() {
        let min = std::array::from_fn(|i| board.collider.min[i] + board.position[i]);
        let max = std::array::from_fn(|i| board.collider.max[i] + board.position[i]);
        draw::cuboid(&mut mesh, min, max, [0.90, 0.62, 0.30]);
    }
    mesh
}

pub fn body_geometry(world: &ContactWorld) -> Vec<Vertex> {
    let mut mesh = Vec::new();
    for body in world.bodies() {
        let p = body.position;
        let h = body.profile.half_extents;
        let color = if body.impairment.is_some() {
            [0.96, 0.29, 0.28]
        } else if body.id == BodyId(0) {
            [0.96, 0.81, 0.37]
        } else {
            [0.65, 0.54, 0.91]
        };
        draw::cuboid(
            &mut mesh,
            [p[0] - h[0], p[1], p[2] - h[2]],
            [p[0] + h[0], p[1] + 2. * h[1], p[2] + h[2]],
            color,
        );
        // Heading marker stays within the body's occupied volume.
        let face = Vec3::from(p)
            + Vec3::new(0., h[1] * 1.5, 0.)
            + Vec3::from(body.heading) * (h[0].min(h[2]) * 0.85);
        draw::cuboid(
            &mut mesh,
            (face - Vec3::splat(0.06)).to_array(),
            (face + Vec3::splat(0.06)).to_array(),
            [0.99, 0.99, 1.],
        );
        if let Some(anchor) = body.anchor {
            draw::line(
                &mut mesh,
                [p[0], p[1] + h[1] * 1.5, p[2]],
                anchor,
                [0.4, 1., 0.88],
            );
        }
    }
    mesh
}

pub fn camera(
    world: &ContactWorld,
    player: BodyId,
    yaw: f32,
    distance: f32,
) -> crate::scene::Camera {
    let body = world.body(player).expect("fixture player");
    let target = Vec3::from(body.position) + Vec3::new(0., body.profile.half_extents[1], 0.);
    let direction = Vec3::new(-yaw.cos(), 0.65, -yaw.sin()).normalize();
    let desired = distance.clamp(3., 12.);
    let mut clear = 0.5;
    // Camera clearance is view-local and does not alter simulation authority.
    for step in 1..=120 {
        let d = desired * step as f32 / 120.;
        let p = target + direction * d;
        if world
            .solids()
            .iter()
            .any(|s| (0..3).all(|i| p[i] >= s.min[i] - 0.12 && p[i] <= s.max[i] + 0.12))
        {
            break;
        }
        clear = d;
    }
    let eye = target + direction * clear;
    crate::scene::Camera {
        projection: Mat4::perspective_rh(60_f32.to_radians(), 1280. / 720., 0.05, 80.),
        view: Mat4::look_at_rh(eye, target, Vec3::Y),
        eye,
        target,
    }
}

pub struct Hud {
    text: draw::Text,
}

impl Hud {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            text: draw::Text::new()?,
        })
    }

    pub fn scene(&mut self, world: &ContactWorld, player: BodyId) -> Scene {
        let body = world.body(player).expect("fixture player");
        let mut scene = Scene::new(1280, 720);
        scene.push_rect(14., 12., 1266., 115., [0.025, 0.045, 0.06, 0.94]);
        scene.push_rect(14., 565., 1266., 708., [0.025, 0.045, 0.06, 0.94]);
        let ink = [0.92, 0.95, 0.95, 1.];
        self.text.label(
            &mut scene,
            "PAREDROS / DAMAGED CROSSING / dry contact fixture",
            [28., 20.],
            23.,
            ink,
        );
        let action = if body.attack_remaining > 0 {
            "Strike: winding up"
        } else if body.attack_cooldown > 0 {
            "Strike: recovering"
        } else if body.recovery_remaining > 0 {
            "Repairing capability"
        } else {
            "Ready"
        };
        self.text.label(&mut scene, action, [980., 25.], 18., ink);
        let status = format!(
            "{:?}  |  integrity {:.0}/{:.0}  |  {:?}  |  {}  |  position {:.1}, {:.1}, {:.1} m",
            body.kind,
            body.integrity,
            body.profile.max_integrity,
            body.impairment,
            if body.anchored {
                "tethered"
            } else if body.bracing {
                "braced"
            } else if body.grounded {
                "grounded"
            } else {
                "falling"
            },
            body.position[0],
            body.position[1],
            body.position[2]
        );
        self.text.label(&mut scene, &status, [28., 55.], 18., ink);
        let task = if world.board_holder() == Some(player) {
            "Holding board: approach the cut, face across, then E to place."
        } else if body.position[0] > 2. {
            "Across. Purple is a stationary practice body: face it and test a timed strike."
        } else if body.position[1] < -0.4 {
            "Recoverable fall: follow the channel toward +Z to the pale exit steps."
        } else {
            "Try the loose amber board, or the cyan overhead hold. The channel has an exit stair."
        };
        self.text
            .label(&mut scene, task, [28., 83.], 17., [0.98, 0.79, 0.4, 1.]);
        if let Some(target) = world.bodies().find(|b| b.id != player) {
            self.text.label(&mut scene, &format!("Practice body: integrity {:.0}/{:.0}; impairment {:?}. Its decisions are not simulated.",
                target.integrity, target.profile.max_integrity, target.impairment), [28., 573.], 17., [0.8, 0.74, 1., 1.]);
        }
        self.text.label(
            &mut scene,
            "WASD move   |   arrows turn camera   |   wheel zoom   |   Shift brace",
            [28., 601.],
            18.,
            ink,
        );
        self.text.label(
            &mut scene,
            "E board   |   Q tether   |   Space strike   |   F practice counterstrike   |   R recover",
            [28., 630.],
            18.,
            ink,
        );
        self.text.label(&mut scene, "1 restart crawler   |   2 restart climber   |   Esc quit   |   authored test world; no NPC decisions", [28., 659.], 17., ink);
        scene
    }
}
