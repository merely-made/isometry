use super::*;
use mesocosm_core::{Attachment, Provenance, SpeciesId, VolumeRef, Yaw};

fn body() -> BodyDocument {
    BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [1, 1, 1])
}

fn environment() -> ArrestFallEnvironment {
    ArrestFallEnvironment {
        support_present: true,
        support_distance_voxels: 3,
        arrest_load_mg: 6,
        support_load_capacity_mg: 6,
        adhesive_surface: AdhesiveSurface::Suitable,
        adhesive_resource: AdhesiveResource::Available,
    }
}

fn limbed_body() -> BodyDocument {
    let mut body = body();
    for offset in [[2, 0, 0], [-2, 0, 0]] {
        body.attach(
            VolumeRef::from_tag(2),
            100,
            [1, 1, 1],
            Attachment {
                parent: PartId(0),
                offset,
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .unwrap();
    }
    body
}

#[test]
fn stale_unlearned_subject_explains_all_three_bindings() {
    let body = body();
    let before = body.clone();
    let result = arrest_fall(
        &TechniqueKnowledge {
            subject: SubjectId(2),
            learned: vec![],
        },
        SubjectBody {
            subject: SubjectId(1),
            revision: BodyRevisionId(3),
            body: &body,
        },
        BodyRevisionId(4),
        &TechniqueInputs {
            occupied_parts: vec![PartId(0)],
            part_capabilities: vec![PartCapability {
                part: PartId(0),
                function: PartFunction::Grip,
                reach_voxels: 2,
                load_capacity_mg: 5,
            }],
            equipment: vec![],
            resources: vec![],
            environment: ArrestFallEnvironment {
                adhesive_surface: AdhesiveSurface::Unsuitable,
                adhesive_resource: AdhesiveResource::Exhausted,
                ..environment()
            },
        },
    );
    assert!(!result.available());
    assert_eq!(body, before);
    assert!(
        result
            .blockers
            .contains(&ActionBlocker::NotLearned(TechniqueId::ArrestFall))
    );
    assert!(
        result.bindings[0]
            .blockers
            .contains(&BindingBlocker::OccupiedPart(PartId(0)))
    );
    assert!(
        result.bindings[1]
            .blockers
            .contains(&BindingBlocker::UnsuitableSurface)
    );
    assert!(
        result
            .bindings
            .iter()
            .all(|binding| binding.blockers.contains(&BindingBlocker::ActionBlocked))
    );
}

#[test]
fn chooses_later_grip_and_line_when_earlier_sources_are_insufficient() {
    let body = limbed_body();
    let result = arrest_fall(
        &TechniqueKnowledge {
            subject: SubjectId(1),
            learned: vec![TechniqueId::ArrestFall],
        },
        SubjectBody {
            subject: SubjectId(1),
            revision: BodyRevisionId(1),
            body: &body,
        },
        BodyRevisionId(1),
        &TechniqueInputs {
            occupied_parts: vec![PartId(0)],
            part_capabilities: vec![
                PartCapability {
                    part: PartId(0),
                    function: PartFunction::Grip,
                    reach_voxels: 9,
                    load_capacity_mg: 9,
                },
                PartCapability {
                    part: PartId(1),
                    function: PartFunction::Grip,
                    reach_voxels: 2,
                    load_capacity_mg: 5,
                },
                PartCapability {
                    part: PartId(2),
                    function: PartFunction::Grip,
                    reach_voxels: 3,
                    load_capacity_mg: 6,
                },
            ],
            equipment: vec![
                EquipmentProjection {
                    item: ItemId(1),
                    carried_by: Some(SubjectId(1)),
                    attached_to: None,
                    function: EquipmentFunction::Line,
                    reach_voxels: 2,
                    load_capacity_mg: 5,
                },
                EquipmentProjection {
                    item: ItemId(2),
                    carried_by: Some(SubjectId(1)),
                    attached_to: None,
                    function: EquipmentFunction::Line,
                    reach_voxels: 3,
                    load_capacity_mg: 6,
                },
            ],
            resources: vec![],
            environment: environment(),
        },
    );
    assert!(result.bindings[0].available());
    assert_eq!(
        result.bindings[0].sources,
        vec![
            SourceQuery::Part(PartId(2)),
            SourceQuery::Equipment(ItemId(2))
        ]
    );
}

#[test]
fn absence_and_exhaustion_remain_explicit_when_all_bindings_fail() {
    let body = body();
    let result = arrest_fall(
        &TechniqueKnowledge {
            subject: SubjectId(1),
            learned: vec![TechniqueId::ArrestFall],
        },
        SubjectBody {
            subject: SubjectId(1),
            revision: BodyRevisionId(1),
            body: &body,
        },
        BodyRevisionId(1),
        &TechniqueInputs {
            occupied_parts: vec![],
            part_capabilities: vec![PartCapability {
                part: PartId(0),
                function: PartFunction::Adhesion,
                reach_voxels: 9,
                load_capacity_mg: 9,
            }],
            equipment: vec![],
            resources: vec![ResourceReserve {
                kind: ResourceKind::Adhesive,
                available_units: 0,
            }],
            environment: ArrestFallEnvironment {
                support_present: false,
                adhesive_resource: AdhesiveResource::Exhausted,
                ..environment()
            },
        },
    );
    assert!(
        result.bindings[0]
            .blockers
            .contains(&BindingBlocker::MissingEquipment(EquipmentFunction::Line))
    );
    assert!(
        result.bindings[1]
            .blockers
            .contains(&BindingBlocker::NoSupportPresent)
    );
    assert!(
        result.bindings[1]
            .blockers
            .contains(&BindingBlocker::ResourceExhausted)
    );
}

#[test]
fn short_line_blocks_a_long_grip() {
    let body = body();
    let result = arrest_fall(
        &TechniqueKnowledge {
            subject: SubjectId(1),
            learned: vec![TechniqueId::ArrestFall],
        },
        SubjectBody {
            subject: SubjectId(1),
            revision: BodyRevisionId(1),
            body: &body,
        },
        BodyRevisionId(1),
        &TechniqueInputs {
            occupied_parts: vec![],
            part_capabilities: vec![PartCapability {
                part: PartId(0),
                function: PartFunction::Grip,
                reach_voxels: 9,
                load_capacity_mg: 9,
            }],
            equipment: vec![EquipmentProjection {
                item: ItemId(1),
                carried_by: Some(SubjectId(1)),
                attached_to: None,
                function: EquipmentFunction::Line,
                reach_voxels: 2,
                load_capacity_mg: 9,
            }],
            resources: vec![],
            environment: environment(),
        },
    );
    assert!(
        result.bindings[0]
            .blockers
            .contains(&BindingBlocker::NoReachableSupport {
                reach_voxels: 2,
                distance_voxels: 3
            })
    );
}

#[test]
fn severing_blocks_expression_without_mutating_knowledge() {
    let mut body = limbed_body();
    body.sever(PartId(1));
    let knowledge = TechniqueKnowledge {
        subject: SubjectId(1),
        learned: vec![TechniqueId::ArrestFall],
    };
    let before = knowledge.clone();
    let result = arrest_fall(
        &knowledge,
        SubjectBody {
            subject: SubjectId(1),
            revision: BodyRevisionId(1),
            body: &body,
        },
        BodyRevisionId(1),
        &TechniqueInputs {
            occupied_parts: vec![],
            part_capabilities: vec![PartCapability {
                part: PartId(1),
                function: PartFunction::Grip,
                reach_voxels: 9,
                load_capacity_mg: 9,
            }],
            equipment: vec![EquipmentProjection {
                item: ItemId(1),
                carried_by: Some(SubjectId(1)),
                attached_to: None,
                function: EquipmentFunction::Line,
                reach_voxels: 9,
                load_capacity_mg: 9,
            }],
            resources: vec![],
            environment: environment(),
        },
    );
    assert_eq!(knowledge, before);
    assert!(
        result.bindings[0]
            .blockers
            .contains(&BindingBlocker::SeveredPart(PartId(1)))
    );
}
