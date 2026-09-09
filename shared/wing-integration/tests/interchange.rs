//! Current Mesocosm producers crossing into the tabletop and back.
//!
//! This is intentionally a separate workspace. The tests depend on both
//! vessels only to exercise bytes at their public boundary; neither product
//! gains a dependency on the other.

use isometry_campaign::{Arrival, ChronicleError};
use isometry_voxel::{BakeParams, BodyError, BodyProfile as TabletopBodyProfile, bake_facing};
use mesocosm_core::{
    Attachment, BodyDocument, Chronicle, Consequence, Deed, Origin, PartId, Provenance, SpeciesId,
    VolumeRef, Yaw,
};
use mesocosm_mesh::{BodyProfile, Volume, VolumeMap};

/// The retained v0 artifact protects persisted data. The live producer test
/// below protects the producer-reader seam when either side changes.
const CRITTER_FIXTURE: &[u8] =
    include_bytes!("../../../crates/isometry-voxel/tests/fixtures/critter.body");

fn grown() -> (BodyDocument, VolumeMap) {
    let mut body = BodyDocument::new(SpeciesId(7), VolumeRef::from_tag(1), 4_000, [2, 3, 2]);
    body.attach(
        VolumeRef::from_tag(2),
        900,
        [1, 1, 3],
        Attachment {
            parent: body.root,
            offset: [3, 0, 0],
            yaw: Yaw::Zero,
        },
        Provenance {
            origin: Origin::Incorporated {
                from_species: SpeciesId(42),
                from_part: PartId(1),
            },
            epoch: 3,
        },
    )
    .expect("the limb attaches");
    body.attach(
        VolumeRef::from_tag(3),
        1_400,
        [2, 1, 1],
        Attachment {
            parent: body.root,
            offset: [0, 4, 0],
            yaw: Yaw::Quarter,
        },
        Provenance {
            origin: Origin::Incorporated {
                from_species: SpeciesId(11),
                from_part: PartId(0),
            },
            epoch: 7,
        },
    )
    .expect("the plate attaches");

    let mut volumes = VolumeMap::default();
    volumes.insert(VolumeRef::from_tag(1), Volume::solid([5, 7, 5], 1));
    volumes.insert(VolumeRef::from_tag(2), Volume::solid([3, 3, 7], 2));
    volumes.insert(VolumeRef::from_tag(3), Volume::solid([5, 3, 3], 3));
    (body, volumes)
}

fn live_profile_bytes() -> Vec<u8> {
    let (body, volumes) = grown();
    BodyProfile::of(&body, &volumes)
        .expect("the producer can flatten its body")
        .to_bytes()
        .expect("the producer frames the profile")
}

#[test]
fn current_body_producer_reaches_tabletop_reader_and_every_facing() {
    let bytes = live_profile_bytes();
    assert_eq!(
        bytes.as_slice(),
        CRITTER_FIXTURE,
        "the retained v0 artifact still records this producer output"
    );

    let body =
        TabletopBodyProfile::read(&bytes).expect("the tabletop reads current producer bytes");
    assert_eq!(body.species, 7);
    assert_eq!(body.size, [7, 9, 7]);
    assert_eq!(body.origin, [-2, -3, -3]);
    assert_eq!(body.cells.len(), 7 * 9 * 7);
    assert_eq!(body.attribution.len(), body.cells.len());
    assert_eq!(body.parts.len(), 3);
    assert_eq!(body.incorporated_parts(), 2);
    assert_eq!(body.parts[1].from_species, Some(42));
    assert_eq!(body.parts[1].from_part, Some(1));
    assert_eq!(body.parts[1].epoch, 3);
    assert_eq!(body.parts[2].from_species, Some(11));
    assert_eq!(body.parts[2].from_part, Some(0));
    assert_eq!(body.parts[2].epoch, 7);
    assert!(body.cells.iter().any(|cell| *cell != 0));
    assert!(
        body.cells
            .iter()
            .zip(&body.attribution)
            .all(|(cell, part)| (*cell != 0) == (*part != 0))
    );

    let voxels = body.voxels_by_part();
    let palette = body.origin_palette([90, 140, 60], [200, 120, 70]);
    for facing in 0..4 {
        let sheet = bake_facing(&voxels, &palette, facing, &BakeParams::default());
        assert!(sheet.w > 0 && sheet.h > 0, "facing {facing} has dimensions");
        assert!(sheet.opaque_pixels() > 0, "facing {facing} is visible");
    }
}

#[test]
fn current_body_bytes_refuse_corruption_and_an_unknown_version() {
    let bytes = live_profile_bytes();
    assert!(matches!(
        TabletopBodyProfile::read(&bytes[..bytes.len() / 2]),
        Err(BodyError::Malformed | BodyError::Inconsistent)
    ));

    let mut foreign = bytes.clone();
    foreign[..8].copy_from_slice(b"NOTABODY");
    assert_eq!(
        TabletopBodyProfile::read(&foreign),
        Err(BodyError::NotABody)
    );

    let mut newer = bytes;
    newer[8..10].copy_from_slice(&1u16.to_le_bytes());
    assert_eq!(
        TabletopBodyProfile::read(&newer),
        Err(BodyError::UnknownVersion {
            found: 1,
            expected: 0
        })
    );
}

#[test]
fn producer_chronicle_carries_foreign_fact_and_tabletop_loss_back_to_mesocosm() {
    let (body, _) = grown();
    let producer_deed = Deed::detailed("mesocosm", "survived-ashfall", 41, vec![0, 3, 255, 9]);
    let mut producer = Chronicle::of(&body);
    producer.append(producer_deed.clone());

    let mut tabletop = Arrival::read(
        &producer
            .to_bytes()
            .expect("the producer frames the chronicle"),
    )
    .expect("the tabletop reads current producer chronicle bytes");
    assert_eq!(tabletop.species(), 7);
    assert_eq!(tabletop.incorporated_parts(), 2);
    let foreign: Vec<_> = tabletop.foreign().collect();
    assert_eq!(foreign.len(), 1);
    assert_eq!(foreign[0].vessel, producer_deed.vessel);
    assert_eq!(foreign[0].verb, producer_deed.verb);
    assert_eq!(foreign[0].at, producer_deed.at);
    assert_eq!(foreign[0].detail, producer_deed.detail);

    tabletop.record_loss(1, 73);
    let returned = Chronicle::from_bytes(
        &tabletop
            .to_bytes()
            .expect("the tabletop re-frames the chronicle"),
    )
    .expect("Mesocosm reads the returned chronicle");
    assert_eq!(returned.species, 7);
    assert_eq!(returned.parts.len(), 3);
    assert_eq!(
        returned.deeds[0], producer_deed,
        "foreign detail returns byte-for-byte"
    );
    assert_eq!(
        returned.deeds.len(),
        2,
        "the tabletop appends without replacing history"
    );
    assert!(matches!(
        returned.read().nth(0),
        Some((_, Consequence::Unread))
    ));
    assert!(matches!(
        returned.read().nth(1),
        Some((_, Consequence::LostPart { part: 1 }))
    ));
}

#[test]
fn current_chronicle_bytes_refuse_foreign_schema_and_newer_version() {
    let (body, _) = grown();
    let bytes = Chronicle::of(&body)
        .to_bytes()
        .expect("the producer frames the chronicle");

    let mut foreign = bytes.clone();
    foreign[..8].copy_from_slice(b"MESOBODY");
    assert_eq!(Arrival::read(&foreign), Err(ChronicleError::WrongSchema));

    let mut newer = bytes;
    newer[8..10].copy_from_slice(&1u16.to_le_bytes());
    assert_eq!(
        Arrival::read(&newer),
        Err(ChronicleError::UnknownVersion {
            found: 1,
            expected: 0
        })
    );
}
