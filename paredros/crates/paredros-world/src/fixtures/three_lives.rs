// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Authored three-lives data for the borg-generation design receipt.
//!
//! This is deliberately fixture data. It is not a profession registry, a
//! historical generator, or a second body store. The bodies use Mesocosm's
//! stable `PartId` addresses and the lives use Paredros's continuing identity
//! types; the binding query receives projections of these facts from callers.

use crate::{
    AdhesiveResource, AdhesiveSurface, ArrestFallEnvironment, EquipmentFunction,
    EquipmentProjection, ItemId, PartCapability, PartFunction, ResourceKind, ResourceReserve,
    TechniqueId, TechniqueInputs, TechniqueKnowledge,
};
use mesocosm_core::{
    Attachment, BodyDocument, Origin, PartId, Provenance, SpeciesId, VolumeRef, Yaw,
};
use paredros_identity::{BodyRevisionId, SubjectId};

pub const WETLAND_LINE: SpeciesId = SpeciesId(71);
pub const NOW: u16 = 115;
pub const KEEPER: SubjectId = SubjectId(701);
pub const SURVEYOR: SubjectId = SubjectId(702);
pub const REPAIRER: SubjectId = SubjectId(703);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub id: &'static str,
    pub year: u16,
    pub place: &'static str,
    pub explains: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartingFact {
    pub description: &'static str,
    pub source_id: &'static str,
    pub acquired_at: u16,
    pub acquired_place: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Life {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub name: &'static str,
    pub ordinary_task: &'static str,
    pub danger_choices: &'static [&'static str],
    pub body: BodyDocument,
    pub sources: &'static [Source],
    pub facts: &'static [StartingFact],
}

pub const FLOOD_RECORD: Source = Source {
    id: "wetland-floods-018",
    year: 18,
    place: "reed delta",
    explains: "the line's gripping limbs, vibration mantle, and fibre gland",
};
pub const KEEPER_TEACHER: Source = Source {
    id: "keeper-teacher-094",
    year: 94,
    place: "south crossing",
    explains: "load sharing and rescue practice",
};
pub const KEEPER_HARNESS: Source = Source {
    id: "keeper-harness-101",
    year: 101,
    place: "south crossing",
    explains: "the supplied harness and line, and fibre-tensioning work",
};
pub const SURVEYOR_TRAVELS: Source = Source {
    id: "surveyor-travels-109",
    year: 109,
    place: "connected marshes",
    explains: "contact-probe use and attention practice",
};
pub const REPAIRER_INJURY: Source = Source {
    id: "repairer-injury-112",
    year: 112,
    place: "west sluice",
    explains: "the lost gripping limb and its stable injury record",
};
pub const REPAIRER_TEACHER: Source = Source {
    id: "repairer-teacher-108",
    year: 108,
    place: "south crossing",
    explains: "rescue principle taught before displacement",
};
pub const REPAIRER_SYMBIONT: Source = Source {
    id: "repairer-symbiont-114",
    year: 114,
    place: "west sluice",
    explains: "cultivated adhesive symbiont transmission and care",
};
pub const KEEPER_FACTS: &[StartingFact] = &[
    INHERITED_BODY,
    StartingFact {
        description: "load sharing and rescue practice",
        source_id: "keeper-teacher-094",
        acquired_at: 94,
        acquired_place: "south crossing",
    },
    StartingFact {
        description: "harness, line and fibre tensioning",
        source_id: "keeper-harness-101",
        acquired_at: 101,
        acquired_place: "south crossing",
    },
];
pub const SURVEYOR_FACTS: &[StartingFact] = &[StartingFact {
    description: "contact-probe attention practice",
    source_id: "surveyor-travels-109",
    acquired_at: 109,
    acquired_place: "connected marshes",
}];
pub const REPAIRER_FACTS: &[StartingFact] = &[
    INHERITED_BODY,
    StartingFact {
        description: "rescue principle and line supplied by a crossing teacher",
        source_id: "repairer-teacher-108",
        acquired_at: 108,
        acquired_place: "south crossing",
    },
    StartingFact {
        description: "rescue principle retained after limb loss",
        source_id: "repairer-injury-112",
        acquired_at: 112,
        acquired_place: "west sluice",
    },
    StartingFact {
        description: "cultivated adhesive symbiont",
        source_id: "repairer-symbiont-114",
        acquired_at: 114,
        acquired_place: "west sluice",
    },
];

pub const INHERITED_BODY: StartingFact = StartingFact {
    description: "inherited four-limb, mantle and gland body plan",
    source_id: "wetland-floods-018",
    acquired_at: 18,
    acquired_place: "reed delta",
};

pub fn symbiont_donor() -> BodyDocument {
    BodyDocument::new(SpeciesId(72), VolumeRef::from_tag(7), 800, [1, 1, 1])
}

pub fn wetland_body() -> BodyDocument {
    let mut body = BodyDocument::new(WETLAND_LINE, VolumeRef::from_tag(71), 48_000, [2, 2, 2]);
    for (tag, offset) in [
        (1, [3, 0, 0]),
        (2, [-3, 0, 0]),
        (3, [0, 0, 3]),
        (4, [0, 0, -3]),
    ] {
        body.attach(
            VolumeRef::from_tag(tag),
            2_000,
            [1, 1, 1],
            Attachment {
                parent: body.root,
                offset,
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("authored limb attaches");
    }
    body.attach(
        VolumeRef::from_tag(5),
        3_000,
        [2, 1, 1],
        Attachment {
            parent: body.root,
            offset: [0, 2, 0],
            yaw: Yaw::Zero,
        },
        Provenance::founding(),
    )
    .expect("authored mantle attaches");
    body.attach(
        VolumeRef::from_tag(6),
        1_000,
        [1, 1, 1],
        Attachment {
            parent: body.root,
            offset: [0, -2, 0],
            yaw: Yaw::Zero,
        },
        Provenance::founding(),
    )
    .expect("authored gland attaches");
    body
}

pub fn three_lives() -> [Life; 3] {
    let body = wetland_body();
    let donor = symbiont_donor();
    let mut repairer_body = body.clone();
    repairer_body.sever(PartId(1));
    repairer_body
        .attach(
            VolumeRef::from_tag(7),
            800,
            [1, 1, 1],
            Attachment {
                parent: repairer_body.root,
                offset: [0, 0, 2],
                yaw: Yaw::Zero,
            },
            Provenance {
                origin: Origin::Incorporated {
                    from_species: donor.species,
                    from_part: donor.root,
                },
                epoch: 114,
            },
        )
        .expect("cultivated symbiont attaches");
    [
        Life {
            subject: KEEPER,
            revision: BodyRevisionId(1),
            name: "Sedge",
            ordinary_task: "keep the south crossing open and haul stranded neighbours",
            danger_choices: &[
                "brace and catch a falling traveller",
                "withdraw and mark a flood route",
            ],
            body: body.clone(),
            sources: &[FLOOD_RECORD, KEEPER_TEACHER, KEEPER_HARNESS],
            facts: KEEPER_FACTS,
        },
        Life {
            subject: SURVEYOR,
            revision: BodyRevisionId(1),
            name: "Tremor",
            ordinary_task: "survey connected mud and find a route around deep water",
            danger_choices: &[
                "follow a quiet vibration path",
                "wait when disconnected ground hides an approach",
            ],
            body: body.clone(),
            sources: &[FLOOD_RECORD, SURVEYOR_TRAVELS],
            facts: SURVEYOR_FACTS,
        },
        Life {
            subject: REPAIRER,
            revision: BodyRevisionId(2),
            name: "Mend",
            ordinary_task: "repair sluice gear and maintain adhesive rescue lines",
            danger_choices: &[
                "adhere to a wet wall and arrest a fall",
                "release and retreat when the symbiont is exhausted",
            ],
            body: repairer_body,
            sources: &[
                FLOOD_RECORD,
                REPAIRER_TEACHER,
                REPAIRER_INJURY,
                REPAIRER_SYMBIONT,
            ],
            facts: REPAIRER_FACTS,
        },
    ]
}

pub fn source_order_is_valid(life: &Life) -> bool {
    life.sources
        .windows(2)
        .all(|pair| pair[0].year <= pair[1].year)
        && life
            .sources
            .iter()
            .all(|source| !source.id.is_empty() && !source.place.is_empty() && source.year <= NOW)
}

pub fn facts_have_sources(life: &Life) -> bool {
    fact_has_source(life, &INHERITED_BODY)
        && life.facts.iter().all(|fact| fact_has_source(life, fact))
}

pub fn fact_has_source(life: &Life, fact: &StartingFact) -> bool {
    fact.acquired_at <= NOW
        && life.sources.iter().any(|source| {
            source.id == fact.source_id
                && source.year <= fact.acquired_at
                && source.place == fact.acquired_place
        })
}

pub fn knowledge(life: &Life) -> TechniqueKnowledge {
    let taught = life.facts.iter().any(|fact| {
        [KEEPER_TEACHER.id, REPAIRER_TEACHER.id].contains(&fact.source_id)
            && fact_has_source(life, fact)
    });
    TechniqueKnowledge {
        subject: life.subject,
        learned: if taught {
            vec![TechniqueId::ArrestFall]
        } else {
            Vec::new()
        },
    }
}

pub fn inputs(life: &Life) -> TechniqueInputs {
    let mut capabilities: Vec<_> = limb_parts(&life.body)
        .into_iter()
        .map(|part| PartCapability {
            part,
            function: PartFunction::Grip,
            reach_voxels: 5,
            load_capacity_mg: 60_000,
        })
        .collect();
    if life.subject == REPAIRER {
        capabilities.push(PartCapability {
            part: ADHESIVE_SYMBIONT,
            function: PartFunction::Adhesion,
            reach_voxels: 4,
            load_capacity_mg: 60_000,
        });
    }
    let equipment = match life.subject {
        KEEPER => vec![
            EquipmentProjection {
                item: ItemId(701),
                carried_by: Some(life.subject),
                attached_to: None,
                function: EquipmentFunction::Line,
                reach_voxels: 5,
                load_capacity_mg: 60_000,
            },
            EquipmentProjection {
                item: ItemId(702),
                carried_by: Some(life.subject),
                attached_to: Some(MANTLE),
                function: EquipmentFunction::Harness,
                reach_voxels: 5,
                load_capacity_mg: 60_000,
            },
        ],
        REPAIRER => vec![EquipmentProjection {
            item: ItemId(703),
            carried_by: Some(life.subject),
            attached_to: None,
            function: EquipmentFunction::Line,
            reach_voxels: 4,
            load_capacity_mg: 60_000,
        }],
        _ => Vec::new(),
    };
    TechniqueInputs {
        occupied_parts: if life.subject == KEEPER {
            vec![PartId(1)]
        } else {
            Vec::new()
        },
        part_capabilities: capabilities,
        equipment,
        resources: vec![ResourceReserve {
            kind: ResourceKind::Adhesive,
            available_units: if life.subject == REPAIRER { 1 } else { 0 },
        }],
        environment: ArrestFallEnvironment {
            support_present: true,
            support_distance_voxels: 3,
            arrest_load_mg: 60_000,
            support_load_capacity_mg: 60_000,
            adhesive_surface: AdhesiveSurface::Suitable,
            adhesive_resource: AdhesiveResource::Available,
        },
    }
}

pub fn limb_parts(_body: &BodyDocument) -> [PartId; 4] {
    [PartId(1), PartId(2), PartId(3), PartId(4)]
}

pub const MANTLE: PartId = PartId(5);
pub const FIBRE_GLAND: PartId = PartId(6);
pub const ADHESIVE_SYMBIONT: PartId = PartId(7);

/// Stable labels used by the native sheet inspector. This is fixture naming,
/// not a production anatomy vocabulary.
pub const PART_NAMES: &[(PartId, &str)] = &[
    (PartId(0), "body root"),
    (PartId(1), "gripping limb A"),
    (PartId(2), "gripping limb B"),
    (PartId(3), "gripping limb C"),
    (PartId(4), "gripping limb D"),
    (MANTLE, "vibration mantle"),
    (FIBRE_GLAND, "fibre gland"),
    (ADHESIVE_SYMBIONT, "adhesive symbiont"),
];
