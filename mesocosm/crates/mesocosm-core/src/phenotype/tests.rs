// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD1b's allocation receipts, over a hand-built body.
//!
//! The world-level half — parity with the geometry readings across a whole
//! founded roster, and severing under the live meal path — lives in
//! `tests/embodied.rs`, which is where the P2 claims it extends already are.

use super::*;
use crate::body::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};
use crate::plan::Role;
use crate::process::{IntakePort, NisKind, Process, Registry};

/// A bulk root `[2, 2, 2]`, a long limb `[7, 1, 1]`, and a frond `[4, 4, 1]`
/// held above: one part of three different roles, so every seeded process in
/// the native vocabulary but `Sense` is present.
fn critter() -> (BodyPhenotype, [PartId; 3]) {
    let mut body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2]);
    let root = body.root;
    let limb = body
        .attach(
            VolumeRef::from_tag(2),
            200,
            [7, 1, 1],
            Attachment {
                parent: root,
                offset: [9, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("attaches");
    let frond = body
        .attach(
            VolumeRef::from_tag(3),
            200,
            [4, 4, 1],
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("attaches");
    (BodyPhenotype::seed(body), [root, limb, frond])
}

fn reference(process: Process) -> ProcessRef {
    Registry::native().of_native(process).reference()
}

#[test]
fn geometry_seeds_the_allocation_it_used_to_only_answer() {
    let (phenotype, [root, limb, frond]) = critter();
    for (part, process) in [
        (root, Process::Intake),
        (limb, Process::Contract),
        (frond, Process::Fix),
    ] {
        let mosaic = phenotype.mosaic(part).expect("a living part has a mosaic");
        let sites: Vec<ProcessRef> = mosaic.sites().iter().map(|site| site.process).collect();
        assert_eq!(sites, vec![reference(process)], "{part:?}");
        assert_eq!(
            mosaic.sites()[0].cause,
            Expressed::Geometry,
            "and its shape is why"
        );
    }
}

#[test]
fn a_declared_port_waits_for_its_supporting_process_to_be_expressed() {
    let (mut phenotype, [_, _, frond]) = critter();
    phenotype.declare_port(
        frond,
        IntakePort::live(NisKind::Producer).supported_by(reference(Process::Fix)),
    );
    assert!(
        phenotype.part_port(frond).is_some(),
        "the fixing frond is active"
    );

    let cells = phenotype.mosaic(frond).unwrap().sites()[0].cells.clone();
    let secrete = AllocationProposal {
        expect: phenotype.digest(),
        source: Arrangement::Direct,
        parts: vec![frond],
        sites: vec![ProposedSite {
            part: frond,
            process: reference(Process::Secrete),
            cells: cells.clone(),
        }],
    };
    phenotype
        .develop(Registry::native(), &secrete)
        .expect("a gland is admitted on a plate");
    assert!(
        phenotype.part_port(frond).is_none(),
        "a gland does not keep a fixing port active"
    );

    let restore = AllocationProposal {
        expect: phenotype.digest(),
        source: Arrangement::Direct,
        parts: vec![frond],
        sites: vec![ProposedSite {
            part: frond,
            process: reference(Process::Fix),
            cells,
        }],
    };
    phenotype
        .develop(Registry::native(), &restore)
        .expect("the frond can be expressed again");
    assert!(
        phenotype.part_port(frond).is_some(),
        "the declaration survived re-expression"
    );
}

#[test]
fn a_seeded_part_arrives_fully_committed() {
    // The honest lowering of "this shape does this thing": the seeded site
    // takes every cell, so a second process has to be paid for out of the
    // first rather than out of free tissue nobody had to earn.
    let (phenotype, _) = critter();
    for (_, mosaic) in phenotype.allocations() {
        assert!(mosaic.capacity() > 0);
        assert_eq!(mosaic.occupied(), mosaic.capacity());
        assert_eq!(mosaic.free(), 0);
    }
}

#[test]
fn every_living_allocation_names_a_living_part() {
    let (mut phenotype, [_, limb, _]) = critter();
    phenotype.sever(limb);
    for (part, _) in phenotype.allocations() {
        assert!(
            phenotype.body().is_living(part),
            "{part:?} allocates without being alive"
        );
    }
    assert!(phenotype.conserves());
}

#[test]
fn a_mosaic_conserves_its_capacity() {
    let (phenotype, _) = critter();
    for (part, mosaic) in phenotype.allocations() {
        assert!(mosaic.conserves(), "{part:?}");
        assert_eq!(mosaic.occupied() + mosaic.free(), mosaic.capacity());
        assert!(mosaic.capacity() <= MAX_CELLS);
    }
}

#[test]
fn a_lattice_follows_the_shape_it_grew_from() {
    // Not renderer voxels: a chain for a limb, a sheet for a plate, one cell
    // for a sensor. The graph is what a part *is*, coarsely.
    let (phenotype, [root, limb, frond]) = critter();
    assert_eq!(phenotype.mosaic(root).unwrap().dims(), [2, 2, 2]);
    assert_eq!(phenotype.mosaic(limb).unwrap().dims(), [4, 1, 1]);
    assert_eq!(phenotype.mosaic(frond).unwrap().dims(), [3, 3, 1]);

    // And adjacency is the graph's, not a coordinate's: the limb is a chain.
    let limb = phenotype.mosaic(limb).unwrap();
    assert_eq!(limb.neighbours(CellId(0)), vec![CellId(1)]);
    assert_eq!(limb.neighbours(CellId(1)), vec![CellId(0), CellId(2)]);
}

#[test]
fn attaching_seeds_the_new_part_in_the_same_operation() {
    let (mut phenotype, [root, ..]) = critter();
    let before = phenotype.allocations().count();
    let eye = phenotype
        .attach(
            VolumeRef::from_tag(4),
            10,
            [1, 1, 1],
            Attachment {
                parent: root,
                offset: [0, 0, -3],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("attaches");

    assert_eq!(phenotype.allocations().count(), before + 1);
    let mosaic = phenotype.mosaic(eye).expect("seeded with the part");
    assert_eq!(mosaic.capacity(), 1, "a sensor is one cell of tissue");
    assert_eq!(
        mosaic.sites()[0].process,
        reference(Process::Sense),
        "and it senses because of its shape"
    );
    assert!(phenotype.conserves());
}

#[test]
fn severing_removes_the_allocation_and_its_consequence_together() {
    let (mut phenotype, [_, limb, _]) = critter();
    assert!(phenotype.expresses(reference(Process::Contract)));

    let lost = phenotype.sever(limb);

    assert_eq!(lost, vec![limb]);
    assert!(
        !phenotype.expresses(reference(Process::Contract)),
        "the allocation left with the arm"
    );
    assert!(
        !phenotype.body().performs(Process::Contract),
        "and so did the anatomy reading"
    );
    // History survives the loss, because a receipt has to be able to say what
    // the branch used to do.
    let explained = phenotype.explain(limb).expect("still addressable");
    assert!(!explained.living);
    assert_eq!(explained.sites.len(), 1);
    assert_eq!(
        explained.sites[0].named.as_ref().map(|id| id.name.as_str()),
        Some("contract")
    );
}

#[test]
fn a_stale_proposal_is_refused_and_moves_nothing() {
    let (mut phenotype, _) = critter();
    let mut proposal = arrange(&phenotype, Aim::Spare);
    proposal.expect ^= 1;
    let before = crate::snapshot::encode(&phenotype).unwrap();

    let refusal = phenotype
        .develop(Registry::native(), &proposal)
        .unwrap_err();

    assert!(matches!(refusal, Refusal::Stale { .. }));
    assert_eq!(
        crate::snapshot::encode(&phenotype).unwrap(),
        before,
        "a refusal leaves the wrapper byte-identical"
    );
    assert_eq!(phenotype.revision(), 0);
}

#[test]
fn direct_and_automatic_lower_the_same_candidate_the_same_way() {
    // **The one-validator receipt.** Same candidate, two authors: the same
    // instruction, and byte-identical state afterwards. The source rides along
    // as diagnostic metadata and the validator never reads it.
    let (mut automatic, _) = critter();
    let mut direct = automatic.clone();

    let by_game = arrange(&automatic, Aim::Spare);
    let by_hand = AllocationProposal {
        source: Arrangement::Direct,
        ..by_game.clone()
    };
    assert_eq!(by_game.source, Arrangement::Automatic);

    let there = automatic
        .develop(Registry::native(), &by_game)
        .expect("valid");
    let here = direct.develop(Registry::native(), &by_hand).expect("valid");

    assert_eq!(there.instruction, here.instruction, "one instruction");
    assert_ne!(there.source, here.source, "two authors");
    assert_eq!(
        crate::snapshot::encode(&automatic).unwrap(),
        crate::snapshot::encode(&direct).unwrap(),
        "and one committed state"
    );
    assert!(there.instruction.cost_cells > 0, "it actually moved tissue");
}

#[test]
fn direct_and_automatic_earn_the_same_refusal() {
    let (mut automatic, [_, limb, _]) = critter();
    let mut direct = automatic.clone();

    // A cell the limb does not have. Invalid whoever drew it.
    let mut by_game = arrange(&automatic, Aim::Spare);
    for site in by_game.sites.iter_mut().filter(|site| site.part == limb) {
        site.cells = vec![CellId(99)];
    }
    let by_hand = AllocationProposal {
        source: Arrangement::Direct,
        ..by_game.clone()
    };

    assert_eq!(
        automatic.develop(Registry::native(), &by_game).unwrap_err(),
        direct.develop(Registry::native(), &by_hand).unwrap_err(),
        "the same invalid candidate earns the same refusal"
    );
    assert_eq!(
        automatic.develop(Registry::native(), &by_game).unwrap_err(),
        Refusal::NoSuchCell {
            part: limb,
            cell: CellId(99)
        }
    );
}

#[test]
fn a_part_cannot_acquire_a_capability_by_editing_a_number() {
    // Contraction on a plate. There is no number to raise, so the only way to
    // make a frond an actuator is to make it a limb — which is a different
    // shape, and a different part.
    let (mut phenotype, [_, _, frond]) = critter();
    let mut proposal = arrange(&phenotype, Aim::Spare);
    for site in proposal.sites.iter_mut().filter(|site| site.part == frond) {
        site.process = reference(Process::Contract);
    }

    assert_eq!(
        phenotype
            .develop(Registry::native(), &proposal)
            .unwrap_err(),
        Refusal::SiteMismatch {
            part: frond,
            process: reference(Process::Contract)
        }
    );
    assert!(
        !phenotype
            .expressing(reference(Process::Contract))
            .any(|p| p == frond),
        "and the frond still does not contract"
    );
}

#[test]
fn every_refusal_names_the_boundary_that_failed() {
    // A receipt that only says "refused" cannot tell a player what to change.
    // Each of these is a different thing to fix, and the whole roster is
    // checked in one place so a new boundary cannot arrive unnamed.
    let (mut phenotype, [root, limb, frond]) = critter();
    let sound = arrange(&phenotype, Aim::Spare);
    let refuse = |phenotype: &mut BodyPhenotype, edit: &dyn Fn(&mut AllocationProposal)| {
        let mut proposal = sound.clone();
        edit(&mut proposal);
        proposal.expect = phenotype.digest();
        phenotype
            .develop(Registry::native(), &proposal)
            .unwrap_err()
    };

    assert_eq!(
        refuse(&mut phenotype, &|p| p.parts.clear()),
        Refusal::NothingProposed
    );
    assert_eq!(
        refuse(&mut phenotype, &|p| p.parts = vec![frond, root]),
        Refusal::UnorderedParts,
        "a desired state has one canonical spelling"
    );
    assert_eq!(
        refuse(&mut phenotype, &|p| p.parts.push(PartId(99))),
        Refusal::NoSuchPart(PartId(99))
    );
    assert_eq!(
        refuse(&mut phenotype, &|p| p.parts.retain(|part| *part != limb)),
        Refusal::UnclaimedPart(limb),
        "a site cannot touch a part the proposal did not claim"
    );
    assert_eq!(
        refuse(&mut phenotype, &|p| p.sites[0].cells.clear()),
        Refusal::EmptySite(root)
    );
    assert_eq!(
        refuse(&mut phenotype, &|p| p.sites[0].cells =
            vec![CellId(2), CellId(0)]),
        Refusal::UnorderedCells(root)
    );
    // Two sites on one part claiming the same cell. Occupancy is disjoint or
    // it is not occupancy.
    assert_eq!(
        refuse(&mut phenotype, &|p| {
            let doubled = p.sites[0].clone();
            p.sites.push(doubled);
        }),
        Refusal::Overlap {
            part: root,
            cell: CellId(0)
        }
    );
    // And nothing above landed.
    assert_eq!(phenotype.revision(), 0);
}

#[test]
fn a_lost_cell_leaves_the_graph_and_stays_addressable() {
    let (phenotype, [_, limb, _]) = critter();
    let mut mosaic = phenotype.mosaic(limb).unwrap().clone();
    mosaic.tombstone(&[CellId(3)]);
    assert!(!mosaic.is_living(CellId(3)));
    assert!(mosaic.holds(CellId(3)), "still addressable as history");
    assert_eq!(
        mosaic.neighbours(CellId(2)),
        vec![CellId(1)],
        "and the graph closed over the gap"
    );
}

#[test]
fn an_invalid_multipart_development_leaves_everything_unchanged() {
    // Atomicity: the first part is fine and the third is not, so nothing at
    // all lands. Partial acceptance would make a receipt ambiguous.
    let (mut phenotype, [_, _, frond]) = critter();
    let mut proposal = arrange(&phenotype, Aim::Spare);
    for site in proposal.sites.iter_mut().filter(|site| site.part == frond) {
        site.cells = vec![CellId(0), CellId(8)];
    }
    let before = crate::snapshot::encode(&phenotype).unwrap();

    assert_eq!(
        phenotype
            .develop(Registry::native(), &proposal)
            .unwrap_err(),
        Refusal::Disconnected(frond),
        "corner to corner is not one organ"
    );
    assert_eq!(crate::snapshot::encode(&phenotype).unwrap(), before);
}

#[test]
fn an_unknown_definition_is_refused_rather_than_substituted() {
    let (mut phenotype, _) = critter();
    let mut proposal = arrange(&phenotype, Aim::Spare);
    let foreign = ProcessRef {
        definition: crate::process::DefinitionDigest(0xdead_beef),
    };
    proposal.sites[0].process = foreign;

    assert_eq!(
        phenotype
            .develop(Registry::native(), &proposal)
            .unwrap_err(),
        Refusal::UnknownProcess(foreign)
    );
}

#[test]
fn a_severed_part_cannot_be_rearranged() {
    let (mut phenotype, [_, limb, _]) = critter();
    let proposal = arrange(&phenotype, Aim::Spare);
    phenotype.sever(limb);
    // Staleness catches it first, which is the point of the expected digest;
    // re-authoring against the injured body then names the real boundary.
    assert!(matches!(
        phenotype
            .develop(Registry::native(), &proposal)
            .unwrap_err(),
        Refusal::Stale { .. }
    ));
    let mut reauthored = arrange(&phenotype, Aim::Spare);
    reauthored.parts.push(limb);
    reauthored.parts.sort_unstable();
    assert_eq!(
        phenotype
            .develop(Registry::native(), &reauthored)
            .unwrap_err(),
        Refusal::SeveredPart(limb)
    );
}

#[test]
fn rearrangement_is_ordered_and_on_the_record() {
    let (mut phenotype, _) = critter();
    assert_eq!(phenotype.revision(), 0);

    let spare = arrange(&phenotype, Aim::Spare);
    let first = phenotype
        .develop(Registry::native(), &spare)
        .expect("valid");
    assert_eq!(first.instruction.revision, 1);
    assert_eq!(phenotype.revision(), 1);
    assert_eq!(
        first.instruction.digest,
        phenotype.digest(),
        "the record names the arrangement it created"
    );
    for (_, mosaic) in phenotype.allocations() {
        for site in mosaic.sites() {
            assert_eq!(site.cause, Expressed::Arranged { revision: 1 });
        }
    }

    // Free tissue exists now, and expressing it again is a second event.
    assert!(phenotype.allocations().any(|(_, m)| m.free() > 0));
    let express = arrange(&phenotype, Aim::Express);
    let second = phenotype
        .develop(Registry::native(), &express)
        .expect("valid");
    assert_eq!(second.instruction.revision, 2);
    assert!(phenotype.allocations().all(|(_, m)| m.free() == 0));
    assert_ne!(
        first.instruction.digest, second.instruction.digest,
        "and two events are two arrangements"
    );
}

#[test]
fn a_phenotype_round_trips() {
    let (mut phenotype, _) = critter();
    phenotype
        .develop(Registry::native(), &arrange(&phenotype, Aim::Spare))
        .expect("valid");
    let bytes = crate::snapshot::encode(&phenotype).unwrap();
    let restored: BodyPhenotype = crate::snapshot::decode(&bytes).unwrap();

    assert_eq!(restored, phenotype);
    assert_eq!(restored.digest(), phenotype.digest());
    assert!(restored.conserves());
}

#[test]
fn the_mosaic_and_the_geometry_reading_agree() {
    // The seeding rule and the allocation it seeds are the same claim written
    // twice, and this is what keeps them the same claim. PD2 makes the
    // allocation the authority; until then they must not drift.
    let (phenotype, _) = critter();
    for process in Process::ALL {
        assert_eq!(
            phenotype.expresses(reference(process)),
            phenotype.body().performs(process),
            "{process:?}"
        );
    }
    for role in Role::ALL {
        let expected: Vec<ProcessRef> = Registry::native()
            .seeds(role)
            .map(|def| def.reference())
            .collect();
        for part in phenotype.body().living() {
            if crate::plan::classify(part.half_extent) != role {
                continue;
            }
            let sites: Vec<ProcessRef> = phenotype
                .mosaic(part.id)
                .unwrap()
                .sites()
                .iter()
                .map(|site| site.process)
                .collect();
            assert_eq!(sites, expected, "{role:?} on {:?}", part.id);
        }
    }
}

#[test]
fn irreversible_loss_takes_capacity_and_the_site_with_it() {
    // Not reached by play yet; the rule lives with the mosaic so the first
    // caller does not have to invent it.
    let (phenotype, [_, limb, _]) = critter();
    let mut mosaic = phenotype.mosaic(limb).unwrap().clone();
    assert_eq!(mosaic.capacity(), 4);

    mosaic.tombstone(&[CellId(1)]);

    assert_eq!(mosaic.capacity(), 3, "a lost cell is not capacity");
    assert!(mosaic.conserves());
    assert!(
        mosaic.sites().is_empty(),
        "and a site cut in two owns no connected region"
    );
}

#[test]
fn an_explanation_names_the_definition_the_tissue_expresses() {
    let (phenotype, [root, ..]) = critter();
    let reading = phenotype.explain(root).expect("a living part");

    assert!(reading.living);
    assert_eq!(reading.capacity, 8);
    assert_eq!(reading.free, 0);
    assert_eq!(reading.sites.len(), 1);
    assert_eq!(reading.sites[0].cells, 8);
    assert_eq!(
        reading.sites[0]
            .named
            .as_ref()
            .map(|id| (id.namespace.as_str(), id.name.as_str())),
        Some(("mesocosm", "intake"))
    );
}
