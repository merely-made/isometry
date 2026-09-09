// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The body plan: the slow-changing rules that decide where growth goes.
//!
//! **Two timescales.** Parts fill in during an epoch, automatically, because a
//! player in the middle of an action game should not be placing geometry. The
//! *plan* changes between epochs, deliberately, by spending the bank. So the
//! player never places a part; they shape the rules that place it. That is
//! partial authorship as a mechanic: a critter you specified is a possession,
//! a critter grown by rules you chose is somebody.
//!
//! It is also genotype and phenotype separated properly. The plan is the
//! heritable thing; the parts are somatic. A lineage carries its plan.
//!
//! Prior art: Karl Sims' 1994 evolved virtual creatures encoded morphology as
//! a directed graph of nodes and connections, growing a body of jointed boxes
//! from a root. `BodyDocument` is already that graph. What this adds is the
//! patterning rules that turn a heap into a body.

use serde::{Deserialize, Serialize};

/// Which way a socket faces, in body space.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Facing {
    Front,
    Back,
    Left,
    Right,
    Above,
    Below,
}

impl Facing {
    /// The axis index and sign this facing moves along.
    pub fn axis(self) -> (usize, i32) {
        match self {
            Facing::Right => (0, 1),
            Facing::Left => (0, -1),
            Facing::Above => (1, 1),
            Facing::Below => (1, -1),
            Facing::Back => (2, 1),
            Facing::Front => (2, -1),
        }
    }

    /// The mirrored facing across the lateral plane. Only left and right
    /// differ; a body's front stays its front when mirrored.
    pub fn mirrored(self) -> Self {
        match self {
            Facing::Left => Facing::Right,
            Facing::Right => Facing::Left,
            other => other,
        }
    }

    pub fn is_lateral(self) -> bool {
        matches!(self, Facing::Left | Facing::Right)
    }

    pub const ALL: [Facing; 6] = [
        Facing::Front,
        Facing::Back,
        Facing::Left,
        Facing::Right,
        Facing::Above,
        Facing::Below,
    ];
}

/// What a part is for, read from its shape.
///
/// This is the anti-Spore mechanism doing real work. Spore had 228 catalogue
/// parts that resolved to stat icons, and the simulation never read their
/// form. Here a part's actual geometry decides what it can be, and the physics
/// reads that geometry. It also makes the world's contents matter: **if
/// nothing long and thin lives in your world, you cannot grow limbs.**
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Role {
    /// Roughly cubic. Mass, gut, armour.
    Mass,
    /// One long axis. Limbs, stalks, tails.
    Limb,
    /// Two long axes and one short. Fins, plates, leaves — the shape that
    /// presents area to the world, and so the one that fixes. (DC1.5)
    Plate,
    /// Small in every axis. Sensors, nodes, detail.
    Sensor,
}

impl Role {
    pub const ALL: [Role; 4] = [Role::Mass, Role::Limb, Role::Plate, Role::Sensor];

    fn index(self) -> usize {
        match self {
            Role::Mass => 0,
            Role::Limb => 1,
            Role::Plate => 2,
            Role::Sensor => 3,
        }
    }
}

/// Reads a part's role from its half-extents.
///
/// Deterministic and integer-only, like everything the core decides.
pub fn classify(half_extent: [i32; 3]) -> Role {
    let e: [i32; 3] = [
        half_extent[0].abs().max(1),
        half_extent[1].abs().max(1),
        half_extent[2].abs().max(1),
    ];
    let max = e[0].max(e[1]).max(e[2]);
    let min = e[0].min(e[1]).min(e[2]);

    if max <= 1 {
        return Role::Sensor;
    }
    // How many axes are within a hair of the longest?
    let long = e.iter().filter(|d| **d * 2 >= max).count();
    match long {
        1 => Role::Limb,
        2 if min * 2 <= max => Role::Plate,
        _ => Role::Mass,
    }
}

/// How a body repeats itself.
///
/// Symmetry is the cheapest legibility there is: a mirrored heap reads as a
/// creature and an unmirrored one does not.
///
/// **Geometry only, since DC1.5.** It used to be a kingdom signature as well,
/// and `Kingdom::from_symmetry` made the two a bijection — so a body's whole
/// trophic life was decided by a field that decides where a limb's twin goes.
/// A kingdom is now read from feeding anatomy
/// ([`Kingdom::of_body`](crate::organism::Kingdom::of_body)); this says which
/// growth mirrors, and nothing else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Symmetry {
    /// Paired left and right, so lateral growth grows a twin.
    #[default]
    Bilateral,
    /// Repeated around the vertical axis. Nothing mirrors.
    Radial,
    /// Networked, no plane of symmetry. Nothing mirrors.
    None,
}

/// The heritable rules that shape a body.
///
/// Deliberately small. It has to be **legible** — a plan nobody can perceive
/// is procedural noise — and it has to be **mutable**, because the adaptation
/// phase spends the bank on changing exactly this.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyPlan {
    pub symmetry: Symmetry,
    /// Where each role prefers to attach, indexed by [`Role`].
    preferences: [Facing; 4],
    /// How many facings growth may try in total, counting the preferred one.
    /// Zero means only the preferred facing (and, when bilateral, its mirror);
    /// a part that fits nowhere there is refused rather than forced.
    pub tolerance: u8,
}

impl Default for BodyPlan {
    fn default() -> Self {
        // A consumer's default: mass at the core, limbs out to the sides,
        // plates on the back, sensors forward. Recognisably an animal.
        Self {
            symmetry: Symmetry::Bilateral,
            preferences: [Facing::Below, Facing::Right, Facing::Above, Facing::Front],
            tolerance: 2,
        }
    }
}

impl BodyPlan {
    pub fn preference(&self, role: Role) -> Facing {
        self.preferences[role.index()]
    }

    /// The adaptation phase's edit: where a role grows from now on.
    pub fn set_preference(&mut self, role: Role, facing: Facing) {
        self.preferences[role.index()] = facing;
    }

    /// Facings to try for a role, preferred first, then outward by tolerance.
    pub fn candidates(&self, role: Role) -> Vec<Facing> {
        let preferred = self.preference(role);
        let mut out = vec![preferred];
        if self.symmetry == Symmetry::Bilateral && preferred.is_lateral() {
            out.push(preferred.mirrored());
        }
        for facing in Facing::ALL {
            // Checked before pushing, so tolerance 0 admits the preferred
            // facing (and its mirror) and nothing else.
            if out.len() > self.tolerance as usize {
                break;
            }
            if !out.contains(&facing) {
                out.push(facing);
            }
        }
        out
    }

    /// Whether a part landing on `facing` should also grow its mirror.
    ///
    /// One meal grows a pair, and the mass is split between them: the
    /// incorporated organism's *pattern* is expressed twice while its
    /// *substance* is divided, which is both how development works and the
    /// only version that keeps the metabolic budget honest.
    pub fn mirrors(&self, facing: Facing) -> bool {
        self.symmetry == Symmetry::Bilateral && facing.is_lateral()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_reads_as_role() {
        assert_eq!(classify([1, 1, 1]), Role::Sensor);
        assert_eq!(classify([6, 1, 1]), Role::Limb);
        assert_eq!(classify([5, 1, 5]), Role::Plate);
        assert_eq!(classify([3, 3, 3]), Role::Mass);
    }

    #[test]
    fn a_long_thing_is_a_limb_on_any_axis() {
        assert_eq!(classify([1, 8, 1]), Role::Limb);
        assert_eq!(classify([1, 1, 8]), Role::Limb);
    }

    #[test]
    fn lateral_facings_mirror_and_others_do_not() {
        assert_eq!(Facing::Left.mirrored(), Facing::Right);
        assert_eq!(Facing::Front.mirrored(), Facing::Front);
        assert!(Facing::Left.is_lateral());
        assert!(!Facing::Above.is_lateral());
    }

    #[test]
    fn bilateral_plans_mirror_lateral_growth_only() {
        let plan = BodyPlan::default();
        assert!(plan.mirrors(Facing::Right));
        assert!(!plan.mirrors(Facing::Above));
    }

    #[test]
    fn other_symmetries_do_not_mirror() {
        let plan = BodyPlan {
            symmetry: Symmetry::None,
            ..Default::default()
        };
        assert!(!plan.mirrors(Facing::Right));
    }

    #[test]
    fn candidates_lead_with_the_preference_then_its_mirror() {
        let plan = BodyPlan::default();
        let candidates = plan.candidates(Role::Limb);
        assert_eq!(candidates[0], Facing::Right, "the plan's preference wins");
        assert_eq!(candidates[1], Facing::Left, "then its mirror");
    }

    #[test]
    fn tolerance_bounds_how_far_growth_wanders() {
        let strict = BodyPlan {
            tolerance: 0,
            ..Default::default()
        };
        assert_eq!(strict.candidates(Role::Mass).len(), 1);
        let loose = BodyPlan {
            tolerance: 5,
            ..Default::default()
        };
        assert!(loose.candidates(Role::Mass).len() > 1);
    }

    #[test]
    fn editing_a_preference_changes_where_growth_goes() {
        let mut plan = BodyPlan::default();
        plan.set_preference(Role::Limb, Facing::Above);
        assert_eq!(plan.candidates(Role::Limb)[0], Facing::Above);
    }
}
