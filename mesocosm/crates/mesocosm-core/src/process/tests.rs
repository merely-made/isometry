// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reach, refusals, and the registry parity receipts.
//!
//! Split out of `process.rs` at the 600-line ceiling when PD1b's allocation
//! half added process references and definition digests.

use super::*;
use crate::body::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};

/// A bulk root, with an optional long limb reaching out along +x.
fn critter(limb: bool) -> (BodyDocument, Option<PartId>) {
    let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2]);
    let root = body.root;
    if !limb {
        return (body, None);
    }
    let arm = body
        .attach(
            VolumeRef::from_tag(2),
            200,
            // Long in one axis only, so `classify` reads it as a limb.
            [6, 1, 1],
            Attachment {
                parent: root,
                offset: [8, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("attaches");
    (body, Some(arm))
}

#[test]
fn a_parts_processes_come_from_its_shape() {
    let (body, arm) = critter(true);
    assert_eq!(
        body.processes(body.root),
        &[Process::Intake],
        "a bulk root admits"
    );
    assert_eq!(
        body.processes(arm.unwrap()),
        &[Process::Contract],
        "a long part acts"
    );
}

#[test]
fn a_body_without_an_actuator_reaches_only_its_own_bulk() {
    let (body, _) = critter(false);
    assert!(!body.performs(Process::Contract));
    assert_eq!(
        body.reach(),
        BULK_REACH + 2,
        "its own half-extent, and no further"
    );
}

#[test]
fn growing_a_limb_extends_reach() {
    // The first embodied consequence. Two bodies, different reach, and no
    // capability number was written anywhere.
    let (bare, _) = critter(false);
    let (limbed, _) = critter(true);

    assert!(
        limbed.reach() > bare.reach(),
        "{} vs {}",
        limbed.reach(),
        bare.reach()
    );
}

#[test]
fn severing_the_limb_takes_the_reach_with_it() {
    // The other half: a capability that came from anatomy leaves with it.
    let (mut body, arm) = critter(true);
    let reached = body.reach();

    body.sever(arm.unwrap());

    assert!(body.reach() < reached, "the reach went with the arm");
    assert_eq!(body.reach(), BULK_REACH + 2, "back to bulk");
    assert!(
        !body.performs(Process::Contract),
        "and nothing acts any more"
    );
}

#[test]
fn a_longer_limb_reaches_further_than_a_short_one() {
    let mut short = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1]);
    let root = short.root;
    let mut long = short.clone();

    short
        .attach(
            VolumeRef::from_tag(2),
            50,
            [3, 1, 1],
            Attachment {
                parent: root,
                offset: [4, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    long.attach(
        VolumeRef::from_tag(2),
        50,
        [9, 1, 1],
        Attachment {
            parent: root,
            offset: [10, 0, 0],
            yaw: Yaw::Zero,
        },
        Provenance::founding(),
    )
    .unwrap();

    assert!(long.reach() > short.reach(), "length is the reach");
}

#[test]
fn a_failure_says_which_requirement_was_unmet() {
    // "No arm" and "arm too short" are different problems and a receipt
    // has to be able to say which.
    let (bare, _) = critter(false);
    assert_eq!(
        bare.can_reach(50),
        Err(Unmet::NoProcess {
            capability: Capability::Reach,
            needs: Process::Contract
        })
    );

    let (limbed, _) = critter(true);
    let reach = limbed.reach();
    assert_eq!(
        limbed.can_reach(50),
        Err(Unmet::TooFar {
            reach,
            distance: 50
        })
    );
    assert_eq!(
        limbed.can_reach(reach),
        Ok(()),
        "and what it can do, it can do"
    );
}

#[test]
fn the_registry_and_the_native_view_agree() {
    // PD1b slice 1's load-bearing receipt: expression is defined by
    // registry data, and the enum fast-path may never drift from it.
    let registry = Registry::native();
    for role in Role::ALL {
        let via_registry: Vec<Process> =
            registry.seeds(role).filter_map(|def| def.native).collect();
        assert_eq!(
            via_registry,
            role.processes().to_vec(),
            "{role:?} grows differently in data and in code"
        );
    }
    // The binding is a bijection: every native resolves, and ids are
    // qualified and distinct.
    let mut ids = std::collections::BTreeSet::new();
    for process in Process::ALL {
        let def = registry.of_native(process);
        assert_eq!(def.native, Some(process));
        assert!(ids.insert(def.id.clone()), "duplicate id {:?}", def.id);
        assert_eq!(def.id.namespace, "mesocosm");
    }
    assert_eq!(
        registry.all().count(),
        Process::ALL.len(),
        "the registry and the native list hold different numbers of processes"
    );
}

#[test]
fn nothing_grows_a_gland() {
    // PD2's first done-condition, as a property of the registry rather than a
    // hope about worldgen: the acquired definition is admitted on a shape and
    // seeded by none, so no body arrives with one and every body that has one
    // was given it by a development that is on the record.
    let registry = Registry::native();
    let gland = registry.of_native(Process::Secrete);
    assert!(!gland.seeded(), "a gland is acquired, never grown");
    assert!(
        gland.admits(Role::Plate),
        "and a plate is the shape that carries it"
    );
    for role in Role::ALL {
        assert!(
            !registry
                .seeds(role)
                .any(|def| def.native == Some(Process::Secrete)),
            "{role:?} grows a gland"
        );
    }
    // The two questions are genuinely different for exactly one shape today.
    // If this ever reads equal again, the seeding split has been undone.
    let admitted = registry.all().filter(|def| def.admits(Role::Plate)).count();
    let grown = registry.seeds(Role::Plate).count();
    assert_eq!((admitted, grown), (2, 1));
}

#[test]
fn a_reference_resolves_to_the_definition_it_addresses() {
    // PD1b's identity claim, both ways: what a phenotype stores is a content
    // address, it resolves to exactly the definition it came from, and an
    // address this ruleset does not hold answers `None` rather than the
    // nearest local process.
    let registry = Registry::native();
    for process in Process::ALL {
        let def = registry.of_native(process);
        let stored = def.reference();
        assert_eq!(
            registry.resolve(stored).map(|found| &found.id),
            Some(&def.id),
            "{process:?} does not resolve to itself"
        );
    }
    let foreign = ProcessRef {
        definition: DefinitionDigest(0),
    };
    assert!(
        registry.resolve(foreign).is_none(),
        "a missing definition must refuse, never substitute"
    );
}

#[test]
fn a_rule_bearing_byte_changes_the_digest() {
    let registry = Registry::native();
    let contract = registry.of_native(Process::Contract);
    // Same identity, different expression rule: a different definition.
    let tampered = ProcessDef {
        expressed_by: vec![Role::Limb, Role::Plate],
        ..contract.clone()
    };
    assert_ne!(contract.digest(), tampered.digest());
    // And the PD2 byte is rule-bearing too: a world whose plates grew glands
    // is a different world, so it cannot answer to the same address.
    let ungrown = ProcessDef {
        seeding: crate::process::Seeding::Acquired,
        ..contract.clone()
    };
    assert_ne!(contract.digest(), ungrown.digest());
    // And the ruleset digest is stable across constructions, moves when any
    // one definition's rule-bearing bytes do, and does not move when the
    // declaration order does. (PD3)
    assert_eq!(Registry::native().digest(), Registry::native().digest());

    let mut moved: Vec<ProcessDef> = registry.all().cloned().collect();
    moved
        .iter_mut()
        .find(|def| def.native == Some(Process::Contract))
        .expect("registered")
        .expressed_by = vec![Role::Limb, Role::Plate];
    assert_ne!(
        Registry::admit(moved).expect("no collision").digest(),
        registry.digest(),
        "one definition's site requirement is the whole ruleset's business"
    );

    let mut reordered: Vec<ProcessDef> = registry.all().cloned().collect();
    reordered.reverse();
    assert_eq!(
        Registry::admit(reordered).expect("no collision").digest(),
        registry.digest(),
        "declaration order is not a rule"
    );
}

#[test]
fn a_plate_is_not_an_actuator() {
    // Armour resists; it does not reach. Without this a body could grow
    // reach by growing anything at all, and shape would stop mattering.
    let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 100, [1, 1, 1]);
    let root = body.root;
    body.attach(
        VolumeRef::from_tag(2),
        50,
        // Wide and flat: two long axes, one short.
        [4, 4, 1],
        Attachment {
            parent: root,
            offset: [6, 0, 0],
            yaw: Yaw::Zero,
        },
        Provenance::founding(),
    )
    .unwrap();

    assert_eq!(classify([4, 4, 1]), Role::Plate);
    assert!(!body.performs(Process::Contract));
    assert_eq!(body.reach(), BULK_REACH + 1, "a plate bought no reach");
    // It bought the other thing, which is the DC1.5 half: area against the
    // world is what fixing is, and it is still not an arm.
    assert!(body.performs(Process::Fix), "and a plate fixes");
}
