// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Read-only, body-addressed technique explanations.
//!
//! This is deliberately one concrete action rather than a general ability
//! language. Callers own the current body admission, learned facts, equipment
//! projection, and environmental observations. The result explains whether a
//! known `ArrestFall` principle can use each of its three implementations.

use mesocosm_core::{BodyDocument, PartId};
use paredros_identity::{BodyRevisionId, SubjectId};
use serde::{Deserialize, Serialize};

use crate::ItemId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TechniqueId {
    ArrestFall,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechniqueKnowledge {
    pub subject: SubjectId,
    pub learned: Vec<TechniqueId>,
}

#[derive(Clone, Copy, Debug)]
pub struct SubjectBody<'a> {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub body: &'a BodyDocument,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartFunction {
    Grip,
    Adhesion,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartCapability {
    pub part: PartId,
    pub function: PartFunction,
    /// Maximum support distance in body-scale voxel units.
    pub reach_voxels: u32,
    /// Maximum arrestable load in milligrams.
    pub load_capacity_mg: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipmentFunction {
    Line,
    Harness,
}

/// A caller-supplied reading of an item, not a second owner of `Items`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentProjection {
    pub item: ItemId,
    pub carried_by: Option<SubjectId>,
    /// Required for a harness. A line has no body attachment in this slice.
    pub attached_to: Option<PartId>,
    pub function: EquipmentFunction,
    /// Maximum support distance in body-scale voxel units.
    pub reach_voxels: u32,
    pub load_capacity_mg: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdhesiveSurface {
    Suitable,
    Unsuitable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdhesiveResource {
    Available,
    Exhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    Adhesive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceReserve {
    pub kind: ResourceKind,
    /// Fixture-scale units. The owning inventory system decides their storage.
    pub available_units: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceCost {
    pub kind: ResourceKind,
    /// Fixture-scale units consumed on a successful use.
    pub units: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArrestFallEnvironment {
    pub support_present: bool,
    pub support_distance_voxels: u32,
    /// Fixture-scale arrest demand in milligrams, supplied by the action
    /// authority. This query does not model collision or fall dynamics.
    pub arrest_load_mg: u32,
    pub support_load_capacity_mg: u32,
    pub adhesive_surface: AdhesiveSurface,
    pub adhesive_resource: AdhesiveResource,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TechniqueInputs {
    pub occupied_parts: Vec<PartId>,
    pub part_capabilities: Vec<PartCapability>,
    pub equipment: Vec<EquipmentProjection>,
    pub resources: Vec<ResourceReserve>,
    pub environment: ArrestFallEnvironment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingKind {
    GripLine,
    Adhesion,
    HarnessLine,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceQuery {
    Part(PartId),
    Equipment(ItemId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionBlocker {
    SubjectMismatch {
        body: SubjectId,
        knowledge: SubjectId,
    },
    StaleRevision {
        known: BodyRevisionId,
        current: BodyRevisionId,
    },
    NotLearned(TechniqueId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BindingBlocker {
    /// A global identity, revision, or knowledge refusal makes every source
    /// unusable. The exact reason remains in [`ActionQuery::blockers`].
    ActionBlocked,
    MissingCapability(PartFunction),
    MissingPart(PartId),
    SeveredPart(PartId),
    OccupiedPart(PartId),
    MissingEquipment(EquipmentFunction),
    NotCarriedEquipment(ItemId),
    HarnessUnattached(ItemId),
    NoReachableSupport {
        reach_voxels: u32,
        distance_voxels: u32,
    },
    NoSupportPresent,
    InsufficientLoadCapacity {
        capacity_mg: u32,
        load_mg: u32,
    },
    UnsuitableSurface,
    ResourceExhausted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingQuery {
    pub kind: BindingKind,
    pub sources: Vec<SourceQuery>,
    pub costs: Vec<ResourceCost>,
    pub blockers: Vec<BindingBlocker>,
}

impl BindingQuery {
    pub fn available(&self) -> bool {
        self.blockers.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionQuery {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub technique: TechniqueId,
    pub blockers: Vec<ActionBlocker>,
    pub bindings: Vec<BindingQuery>,
}

impl ActionQuery {
    pub fn available(&self) -> bool {
        self.blockers.is_empty() && self.bindings.iter().any(BindingQuery::available)
    }
}

/// Explains the three supported ways one learned subject can arrest a fall.
pub fn arrest_fall(
    knowledge: &TechniqueKnowledge,
    body: SubjectBody<'_>,
    current_revision: BodyRevisionId,
    inputs: &TechniqueInputs,
) -> ActionQuery {
    let mut blockers = Vec::new();
    if knowledge.subject != body.subject {
        blockers.push(ActionBlocker::SubjectMismatch {
            body: body.subject,
            knowledge: knowledge.subject,
        });
    }
    if body.revision != current_revision {
        blockers.push(ActionBlocker::StaleRevision {
            known: body.revision,
            current: current_revision,
        });
    }
    if !knowledge.learned.contains(&TechniqueId::ArrestFall) {
        blockers.push(ActionBlocker::NotLearned(TechniqueId::ArrestFall));
    }

    ActionQuery {
        subject: body.subject,
        revision: current_revision,
        technique: TechniqueId::ArrestFall,
        blockers,
        bindings: vec![
            grip_line(body, inputs),
            adhesion(body, inputs),
            harness_line(body, inputs),
        ],
    }
    .with_action_blockers()
}

impl ActionQuery {
    fn with_action_blockers(mut self) -> Self {
        if !self.blockers.is_empty() {
            for binding in &mut self.bindings {
                binding.blockers.push(BindingBlocker::ActionBlocked);
            }
        }
        self
    }
}

fn part_for(
    body: SubjectBody<'_>,
    inputs: &TechniqueInputs,
    function: PartFunction,
) -> (
    Vec<SourceQuery>,
    Vec<BindingBlocker>,
    Option<PartCapability>,
) {
    let candidates: Vec<_> = inputs
        .part_capabilities
        .iter()
        .copied()
        .filter(|capability| capability.function == function)
        .collect();
    if candidates.is_empty() {
        return (
            Vec::new(),
            vec![BindingBlocker::MissingCapability(function)],
            None,
        );
    }
    let capability = candidates
        .iter()
        .copied()
        .find(|capability| capability_usable(body, inputs, *capability))
        .unwrap_or(candidates[0]);
    let mut blockers = Vec::new();
    match body.body.part(capability.part) {
        None => blockers.push(BindingBlocker::MissingPart(capability.part)),
        Some(part) if part.severed => blockers.push(BindingBlocker::SeveredPart(capability.part)),
        Some(_) if inputs.occupied_parts.contains(&capability.part) => {
            blockers.push(BindingBlocker::OccupiedPart(capability.part));
        },
        Some(_) => {},
    }
    (
        vec![SourceQuery::Part(capability.part)],
        blockers,
        Some(capability),
    )
}

fn capability_usable(
    body: SubjectBody<'_>,
    inputs: &TechniqueInputs,
    capability: PartCapability,
) -> bool {
    body.body.is_living(capability.part)
        && !inputs.occupied_parts.contains(&capability.part)
        && inputs.environment.support_present
        && capability.reach_voxels >= inputs.environment.support_distance_voxels
        && capability
            .load_capacity_mg
            .min(inputs.environment.support_load_capacity_mg)
            >= inputs.environment.arrest_load_mg
}

fn carried_equipment(
    inputs: &TechniqueInputs,
    subject: SubjectId,
    body: SubjectBody<'_>,
    function: EquipmentFunction,
) -> (
    Vec<SourceQuery>,
    Vec<BindingBlocker>,
    Option<EquipmentProjection>,
) {
    let candidates: Vec<_> = inputs
        .equipment
        .iter()
        .copied()
        .filter(|equipment| equipment.function == function)
        .collect();
    if candidates.is_empty() {
        return (
            Vec::new(),
            vec![BindingBlocker::MissingEquipment(function)],
            None,
        );
    }
    let equipment = candidates
        .iter()
        .copied()
        .find(|equipment| equipment_usable(*equipment, subject, body, inputs.environment))
        .unwrap_or(candidates[0]);
    let mut blockers = Vec::new();
    if equipment.carried_by != Some(subject) {
        blockers.push(BindingBlocker::NotCarriedEquipment(equipment.item));
    }
    (
        vec![SourceQuery::Equipment(equipment.item)],
        blockers,
        Some(equipment),
    )
}

fn equipment_usable(
    equipment: EquipmentProjection,
    subject: SubjectId,
    body: SubjectBody<'_>,
    environment: ArrestFallEnvironment,
) -> bool {
    equipment.carried_by == Some(subject)
        && equipment
            .load_capacity_mg
            .min(environment.support_load_capacity_mg)
            >= environment.arrest_load_mg
        && (equipment.function != EquipmentFunction::Line
            || (environment.support_present
                && equipment.reach_voxels >= environment.support_distance_voxels))
        && (equipment.function != EquipmentFunction::Harness
            || equipment
                .attached_to
                .is_some_and(|part| body.body.is_living(part)))
}

fn reach_and_load(
    blockers: &mut Vec<BindingBlocker>,
    reach_voxels: u32,
    capacity_mg: u32,
    environment: ArrestFallEnvironment,
) {
    if !environment.support_present {
        blockers.push(BindingBlocker::NoSupportPresent);
    } else if reach_voxels < environment.support_distance_voxels {
        blockers.push(BindingBlocker::NoReachableSupport {
            reach_voxels,
            distance_voxels: environment.support_distance_voxels,
        });
    }
    if capacity_mg < environment.arrest_load_mg {
        blockers.push(BindingBlocker::InsufficientLoadCapacity {
            capacity_mg,
            load_mg: environment.arrest_load_mg,
        });
    }
}

fn grip_line(body: SubjectBody<'_>, inputs: &TechniqueInputs) -> BindingQuery {
    let (mut sources, mut blockers, grip) = part_for(body, inputs, PartFunction::Grip);
    let (line_sources, line_blockers, line) =
        carried_equipment(inputs, body.subject, body, EquipmentFunction::Line);
    sources.extend(line_sources);
    blockers.extend(line_blockers);
    if let (Some(grip), Some(line)) = (grip, line) {
        reach_and_load(
            &mut blockers,
            grip.reach_voxels.min(line.reach_voxels),
            grip.load_capacity_mg
                .min(line.load_capacity_mg)
                .min(inputs.environment.support_load_capacity_mg),
            inputs.environment,
        );
    }
    BindingQuery {
        kind: BindingKind::GripLine,
        sources,
        costs: Vec::new(),
        blockers,
    }
}

fn adhesion(body: SubjectBody<'_>, inputs: &TechniqueInputs) -> BindingQuery {
    let (sources, mut blockers, adhesion) = part_for(body, inputs, PartFunction::Adhesion);
    if inputs.environment.adhesive_surface == AdhesiveSurface::Unsuitable {
        blockers.push(BindingBlocker::UnsuitableSurface);
    }
    let adhesive_cost = ResourceCost {
        kind: ResourceKind::Adhesive,
        units: 1,
    };
    let adhesive_available = inputs
        .resources
        .iter()
        .filter(|reserve| reserve.kind == adhesive_cost.kind)
        .map(|reserve| reserve.available_units)
        .fold(0u16, |total, available| total.saturating_add(available));
    if inputs.environment.adhesive_resource == AdhesiveResource::Exhausted
        || adhesive_available < adhesive_cost.units
    {
        blockers.push(BindingBlocker::ResourceExhausted);
    }
    if let Some(adhesion) = adhesion {
        reach_and_load(
            &mut blockers,
            adhesion.reach_voxels,
            adhesion
                .load_capacity_mg
                .min(inputs.environment.support_load_capacity_mg),
            inputs.environment,
        );
    }
    BindingQuery {
        kind: BindingKind::Adhesion,
        sources,
        costs: vec![adhesive_cost],
        blockers,
    }
}

fn harness_line(body: SubjectBody<'_>, inputs: &TechniqueInputs) -> BindingQuery {
    let (mut sources, mut blockers, harness) =
        carried_equipment(inputs, body.subject, body, EquipmentFunction::Harness);
    let (line_sources, line_blockers, line) =
        carried_equipment(inputs, body.subject, body, EquipmentFunction::Line);
    sources.extend(line_sources);
    blockers.extend(line_blockers);
    if let Some(harness) = harness {
        match harness.attached_to {
            None => blockers.push(BindingBlocker::HarnessUnattached(harness.item)),
            Some(part) => match body.body.part(part) {
                None => blockers.push(BindingBlocker::MissingPart(part)),
                Some(found) if found.severed => blockers.push(BindingBlocker::SeveredPart(part)),
                Some(_) => {},
            },
        }
    }
    if let Some(harness) = harness {
        if let Some(part) = harness.attached_to {
            sources.push(SourceQuery::Part(part));
        }
    }
    if let (Some(harness), Some(line)) = (harness, line) {
        reach_and_load(
            &mut blockers,
            line.reach_voxels,
            harness
                .load_capacity_mg
                .min(line.load_capacity_mg)
                .min(inputs.environment.support_load_capacity_mg),
            inputs.environment,
        );
    }
    BindingQuery {
        kind: BindingKind::HarnessLine,
        sources,
        costs: Vec::new(),
        blockers,
    }
}

#[cfg(test)]
#[path = "technique/tests.rs"]
mod tests;
