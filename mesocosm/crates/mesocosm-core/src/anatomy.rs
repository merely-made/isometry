// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The body as a tree: descent, depth, and loss.
//!
//! [`BodyDocument`] could always be climbed — `world_pivot` walks parents to
//! find where a part sits — but nothing could descend it. That made the wing's
//! anatomy rule inexpressible: **lose an arm, lose the hand, lose the
//! fingers**. You cannot cascade a loss through a tree you can only climb.
//!
//! # Severing tombstones rather than removes
//!
//! `PartId` is an index into [`BodyDocument::parts`], so deleting an entry
//! would renumber every part after it and silently reassign identities that
//! provenance, attribution grids, and other games' records all point at.
//!
//! So a severed part stays in the document with [`Part::severed`] set. Three
//! things fall out, and all three are wanted:
//!
//! - Ids stay stable, so a chronicle written before an injury still names the
//!   same parts afterwards.
//! - **What was lost is still on the record.** Law B wants a small number of
//!   very loud signatures, and a missing arm is exactly that. A body that
//!   forgot its injuries would be quietly less interesting.
//! - Regrowth can tell an empty socket from a place nothing ever grew.
//!
//! The cost is that every consumer of `parts` must skip the severed, which is
//! why [`BodyDocument::living`] exists and why the geometry passes use it.

use crate::body::{BodyDocument, Part, PartId};

impl BodyDocument {
    /// Parts that are still attached, in document order.
    ///
    /// The iterator every consumer of anatomy wants. Iterating `parts`
    /// directly includes the severed, which is right for a record and wrong
    /// for anything drawing, weighing, or placing a body.
    pub fn living(&self) -> impl Iterator<Item = &Part> {
        self.parts.iter().filter(|part| !part.severed)
    }

    /// Whether a part is present and attached.
    pub fn is_living(&self, id: PartId) -> bool {
        self.part(id).is_some_and(|part| !part.severed)
    }

    /// The parts attached directly to `id`.
    pub fn children(&self, id: PartId) -> impl Iterator<Item = PartId> + '_ {
        self.parts
            .iter()
            .filter(move |part| !part.severed && part.attachment.is_some_and(|at| at.parent == id))
            .map(|part| part.id)
    }

    /// `id` and everything hanging off it, parents before children.
    ///
    /// The subtree a severing takes. Breadth-first so a caller that stops
    /// early gets the parts nearest the wound.
    pub fn descendants(&self, id: PartId) -> Vec<PartId> {
        if !self.is_living(id) {
            return Vec::new();
        }
        let mut found = vec![id];
        let mut next = 0;
        while next < found.len() {
            let current = found[next];
            next += 1;
            for child in self.children(current) {
                // A malformed document could name a cycle; constructors
                // prevent it, but a deserialized one is not this code's to
                // trust, and an unbounded walk here would hang a host.
                if !found.contains(&child) {
                    found.push(child);
                }
            }
        }
        found
    }

    /// How many joints lie between `id` and the root. The root is zero.
    ///
    /// This is what makes a limb-end different from a trunk attachment, which
    /// is the property Qud gets from named slots and this game gets from
    /// geometry for free.
    pub fn depth(&self, id: PartId) -> Option<u32> {
        let mut depth = 0;
        let mut current = self.part(id)?;
        while let Some(attachment) = current.attachment {
            current = self.part(attachment.parent)?;
            depth += 1;
            if depth > self.parts.len() as u32 {
                return None; // a cycle, in a document we did not build
            }
        }
        Some(depth)
    }

    /// Severs `id` and everything below it, returning what was lost.
    ///
    /// The root cannot be severed: a body without a root is not an injured
    /// creature, it is no creature. Returns an empty list in that case and
    /// when the part is already gone, so severing twice is harmless.
    pub fn sever(&mut self, id: PartId) -> Vec<PartId> {
        if id == self.root || !self.is_living(id) {
            return Vec::new();
        }
        let lost = self.descendants(id);
        for part in lost.iter() {
            self.parts[part.0 as usize].severed = true;
        }
        lost
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};

    /// root -> arm -> hand -> finger, plus a plate on the root.
    fn limbed() -> (BodyDocument, [PartId; 4]) {
        let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2]);
        // Long in one axis, so `classify` reads these as limbs and they carry
        // a Contract process. A cube would be a sensor and buy no reach.
        let link = |body: &mut BodyDocument, parent: PartId, offset: [i32; 3]| {
            body.attach(
                VolumeRef::from_tag(2),
                100,
                [3, 1, 1],
                Attachment {
                    parent,
                    offset,
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .expect("attaches")
        };
        let root = body.root;
        let arm = link(&mut body, root, [3, 0, 0]);
        let hand = link(&mut body, arm, [2, 0, 0]);
        let finger = link(&mut body, hand, [2, 0, 0]);
        let plate = link(&mut body, root, [0, 3, 0]);
        (body, [arm, hand, finger, plate])
    }

    #[test]
    fn children_are_the_parts_hanging_directly_off_one() {
        let (body, [arm, hand, _, plate]) = limbed();
        let root_children: Vec<_> = body.children(body.root).collect();
        assert_eq!(root_children, vec![arm, plate]);
        assert_eq!(body.children(arm).collect::<Vec<_>>(), vec![hand]);
    }

    #[test]
    fn a_subtree_is_the_part_and_everything_under_it() {
        let (body, [arm, hand, finger, _]) = limbed();
        assert_eq!(body.descendants(arm), vec![arm, hand, finger]);
        assert_eq!(
            body.descendants(finger),
            vec![finger],
            "a leaf is its own subtree"
        );
    }

    #[test]
    fn losing_an_arm_loses_the_hand_and_the_fingers() {
        // The ruling, in one test.
        let (mut body, [arm, hand, finger, plate]) = limbed();
        let lost = body.sever(arm);

        assert_eq!(lost, vec![arm, hand, finger]);
        assert!(!body.is_living(arm) && !body.is_living(hand) && !body.is_living(finger));
        assert!(body.is_living(plate), "an unrelated limb is untouched");
        assert!(body.is_living(body.root));
    }

    #[test]
    fn the_lost_stay_on_the_record() {
        // Tombstoned, not deleted. Law B wants the injury to be visible, and
        // stable ids are what let another game's record still mean something.
        let (mut body, [arm, _, _, _]) = limbed();
        let before = body.parts.len();
        body.sever(arm);

        assert_eq!(body.parts.len(), before, "nothing was removed");
        assert_eq!(body.living().count(), before - 3);
        assert!(body.part(arm).is_some(), "the arm is still nameable");
        assert_eq!(body.part(arm).map(|p| p.severed), Some(true));
    }

    #[test]
    fn severing_is_idempotent_and_the_root_is_safe() {
        let (mut body, [arm, _, _, _]) = limbed();
        assert_eq!(body.sever(arm).len(), 3);
        assert!(
            body.sever(arm).is_empty(),
            "severing twice loses nothing more"
        );
        assert!(
            body.sever(body.root).is_empty(),
            "a body without a root is not a body"
        );
        assert!(body.is_living(body.root));
    }

    #[test]
    fn depth_distinguishes_a_limb_end_from_the_trunk() {
        let (body, [arm, hand, finger, plate]) = limbed();
        assert_eq!(body.depth(body.root), Some(0));
        assert_eq!(body.depth(arm), Some(1));
        assert_eq!(body.depth(plate), Some(1));
        assert_eq!(body.depth(hand), Some(2));
        assert_eq!(body.depth(finger), Some(3));
    }

    #[test]
    fn mass_and_reach_are_folds_over_what_survives() {
        // Capability as a consequence, not a stored number: the same body
        // answers differently after an injury, with nothing recomputed by hand.
        let (mut body, [arm, _, _, _]) = limbed();
        let reach_before = body.reach();
        let mass_before = body.total_mass_mg();

        body.sever(arm);

        assert!(body.reach() < reach_before, "the long limb was the reach");
        assert_eq!(
            body.total_mass_mg(),
            mass_before - 300,
            "and three parts' worth of mass went with it"
        );
    }

    #[test]
    fn severing_a_leaf_takes_only_itself() {
        let (mut body, [arm, hand, finger, _]) = limbed();
        assert_eq!(body.sever(finger), vec![finger]);
        assert!(body.is_living(arm) && body.is_living(hand));
    }
}
