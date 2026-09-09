// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{BodyDocument, Yaw, matter::Stock};

#[test]
fn every_graft_lowering_carries_donor_stock_without_changing_body_profile_bytes() {
    let mut body = BodyDocument::new(SpeciesId(1), crate::VolumeRef::from_tag(1), 100, [2, 2, 2]);
    let root = body.root;
    let limb = body
        .attach(
            crate::VolumeRef::from_tag(2),
            11,
            [1, 1, 1],
            Attachment {
                parent: root,
                offset: [3, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    let structural_bytes = crate::snapshot::encode(&body).unwrap();
    let mut donor = BodyPhenotype::seed(body);
    let mix = Stock::from_amounts([1, 3, 5, 2]);
    donor.replace_part_stock(limb, mix).unwrap();
    assert_eq!(
        crate::snapshot::encode(donor.body()).unwrap(),
        structural_bytes
    );
    let branch = donor.harvest(limb).unwrap();
    for lowering in [Lowering::Carried, Lowering::Adapted, Lowering::Regrown] {
        let mut recipient = BodyPhenotype::seed(BodyDocument::new(
            SpeciesId(2),
            crate::VolumeRef::from_tag(3),
            100,
            [2, 2, 2],
        ));
        let before = recipient.total_stock().unwrap();
        let graft = recipient
            .receive(
                crate::process::Registry::native(),
                &branch,
                Attachment {
                    parent: recipient.body().root,
                    offset: [3, 0, 0],
                    yaw: Yaw::Zero,
                },
                0,
                lowering,
            )
            .unwrap();
        assert_eq!(recipient.part_stock(graft.root), Some(&mix));
        assert_eq!(
            recipient.total_stock().unwrap(),
            before.checked_add(mix).unwrap()
        );
        let restored: BodyPhenotype =
            crate::snapshot::decode(&crate::snapshot::encode(&recipient).unwrap()).unwrap();
        assert_eq!(recipient, restored);
    }
    let original = donor.total_stock().unwrap();
    donor.sever(limb);
    assert_eq!(
        donor.total_stock().unwrap(),
        original.checked_sub(mix).unwrap()
    );
    assert_eq!(
        donor.part_stock(limb),
        Some(&mix),
        "severed history remains addressable"
    );
}

#[test]
fn a_hand_built_cutting_cannot_claim_more_stock_than_mass() {
    let mut donor = BodyPhenotype::seed(BodyDocument::new(
        SpeciesId(1),
        crate::VolumeRef::from_tag(1),
        100,
        [2, 2, 2],
    ));
    let root = donor.body().root;
    let limb = donor
        .attach(
            crate::VolumeRef::from_tag(2),
            11,
            [1, 1, 1],
            Attachment {
                parent: root,
                offset: [3, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    let mut branch = donor.harvest(limb).unwrap();
    branch.parts[0].stock = Stock::from_amounts([12, 0, 0, 0]);
    let before = donor.clone();
    assert!(
        donor
            .receive(
                crate::process::Registry::native(),
                &branch,
                Attachment {
                    parent: root,
                    offset: [-3, 0, 0],
                    yaw: Yaw::Zero
                },
                0,
                Lowering::Carried
            )
            .is_err()
    );
    assert_eq!(donor, before);
}
