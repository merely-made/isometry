// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Persistent Paredros world, embodied subjects, and recorded transitions.
//!
//! Mesocosm currently supplies verb-neutral generated topology and exact
//! material ground. This crate gives that structure Paredros meanings:
//! settlements, ruins, encounters, dungeons, inherited replacements, and
//! edits made by particular subjects. Saves carry only genesis facts and
//! ordered intents; restore regrows the base world before replaying them.
//!
//! A [`SlotId`] survives changes in what occupies it. This is the important
//! distinction for imported history: a fact replaces procedural content at
//! an existing structural address rather than adding a parallel world.
//! [`Movement`] records where a subject is and the accepted inputs that put it
//! there. [`Navigation`] is a disposable query over the same exact ground.
//! [`Bodies`] and [`Items`] own durable physical condition independently;
//! [`GameState`] composes those systems through one subject-addressed intent
//! grammar and regrows them from accepted inputs on restore.
//! [`Population`] records where named lives came from, [`Projects`] owns
//! durable goals, and [`Simulation`] advances every living subject without a
//! player, camera, or observer entering the scheduling contract.

mod anatomy;
mod bodies;
mod contact;
mod equipment;
pub mod fixtures;
mod items;
mod movement;
mod navigation;
mod population;
mod projects;
mod session;
mod simulation;
mod simulation_record;
mod sites;
mod state;
mod subject_sheet;
mod technique;
mod transitions;
mod world;

pub use anatomy::{
    Anatomies, AnatomyError, AnatomyRecord, MAX_ANATOMY_COORDINATE, MAX_ANATOMY_PARTS,
    MAX_ANATOMY_WORLD_COORDINATE,
};
pub use bodies::{
    Bodies, Body, BodyError, BodyProfile, MAX_NEED, MOBILITY_WOUND, Name, Needs, SAFE_FALL,
};
pub use contact::{
    BodyId, BodyKind, BodyProfile as ContactBodyProfile, BodyState, BoxCollider, ContactEffect,
    ContactError, ContactSave, ContactWorld, FIXED_DT_SECONDS, HeldInput, Impairment, Input,
    InputFrame, MAX_RECORDED_FRAMES, MovableBoard, Position, TriggeredInput,
};
pub use equipment::AttachmentView;
pub use items::{Item, ItemError, ItemId, ItemKind, ItemLocation, Items};
pub use movement::{Movement, MovementError, MovementEvent, MovementIntent, MovementSave};
pub use navigation::{Navigation, NavigationError};
pub use population::{
    Life, MAX_MIGRANTS_PER_ROUTE, MAX_RESIDENTS_PER_SITE, Migration, Population, PopulationConfig,
    PopulationError, PopulationOrigin,
};
pub use projects::{
    Project, ProjectError, ProjectEvent, ProjectGoal, ProjectId, ProjectIntent, ProjectSave,
    ProjectStatus, Projects,
};
pub use session::{
    MAX_CONTROL_INTENTS, MAX_GAME_INTENTS, MAX_SESSION_BYTES, SESSION_VERSION, Session,
    SessionError, SessionLimits, SessionSave,
};
pub use simulation::{Decision, LifeReport, Pursuit, Simulation, SimulationError};
pub use simulation_record::{SIMULATION_VERSION, SimulationSave};
pub use sites::{HistoryFactId, Layer, Site, SiteKind, SiteSource, SlotId, WorldMap};
pub use state::GameState;
pub use subject_sheet::{
    ActionRow, CapabilityRow, EquipmentRow, PartRow, ResourceRow, SubjectSheet, SubjectSheetInput,
};
pub use technique::{
    ActionBlocker, ActionQuery, AdhesiveResource, AdhesiveSurface, ArrestFallEnvironment,
    BindingBlocker, BindingKind, BindingQuery, EquipmentFunction, EquipmentProjection,
    PartCapability, PartFunction, ResourceCost, ResourceKind, ResourceReserve, SourceQuery,
    SubjectBody, TechniqueId, TechniqueInputs, TechniqueKnowledge, arrest_fall,
};
pub use transitions::{DeathCause, GAME_STATE_VERSION, GameError, GameEvent, GameIntent, GameSave};
pub use world::{
    GENERATOR_VERSION, World, WorldConfig, WorldError, WorldEvent, WorldIntent, WorldSave,
};
