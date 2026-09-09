// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Small owned world fixture for the body-sheet equipment controls.
//!
//! The session only dispatches recorded game intents. Its sheet is rebuilt
//! from the admitted anatomy after each read, so presentation never repairs
//! or mutates the durable game facts.

use mesocosm_core::PartId;
use paredros_identity::{BodyRevisionId, SubjectId};
use paredros_world::fixtures::three_lives::wetland_body;
use paredros_world::{
    AdhesiveResource, AdhesiveSurface, ArrestFallEnvironment, GameIntent, GameState, Item, ItemId,
    ItemKind, ItemLocation, Name, SubjectSheet, SubjectSheetInput, TechniqueInputs,
    TechniqueKnowledge, World, WorldConfig,
};

const SUBJECT: SubjectId = SubjectId(1);
const INITIAL_INTENTS: usize = 5;

/// A fixed, owned subject and a real world-state transition path for one HUD.
pub struct EquipmentSession {
    game: GameState,
    subject: SubjectId,
    status: String,
}

impl EquipmentSession {
    /// Creates the named keeper with two carried dressings and an admitted body.
    pub fn new() -> Result<Self, String> {
        let mut game = GameState::new(
            World::generate(4242, WorldConfig::default()).map_err(|error| format!("{error:?}"))?,
        );
        let at = game
            .items()
            .all()
            .find_map(|item| match item.location {
                ItemLocation::At(at)
                    if item.kind == ItemKind::Dressing
                        && game
                            .world()
                            .ground()
                            .stands(at, mesocosm_core::places::WALKER_HEIGHT) =>
                {
                    Some(at)
                },
                _ => None,
            })
            .ok_or_else(|| "no accessible dressing location".to_owned())?;
        let dressings: Vec<_> = game
            .items()
            .at(at)
            .filter(|item| item.kind == ItemKind::Dressing)
            .map(|item| item.id)
            .take(2)
            .collect();
        if dressings.len() != 2 {
            return Err("fewer than two dressings at accessible location".to_owned());
        }

        let tick = game.next_tick();
        apply_new(
            &mut game,
            GameIntent::Generate {
                tick,
                subject: SUBJECT,
                body_seed: 7,
                at,
            },
        )?;
        let tick = game.next_tick();
        apply_new(
            &mut game,
            GameIntent::Name {
                tick,
                subject: SUBJECT,
                name: Name::new("Keeper").map_err(|error| format!("{error:?}"))?,
            },
        )?;
        let tick = game.next_tick();
        apply_new(
            &mut game,
            GameIntent::AdmitAnatomy {
                tick,
                subject: SUBJECT,
                revision: BodyRevisionId(0),
                document: Box::new(wetland_body()),
            },
        )?;
        for item in dressings {
            let tick = game.next_tick();
            apply_new(
                &mut game,
                GameIntent::Take {
                    tick,
                    subject: SUBJECT,
                    item,
                },
            )?;
        }

        Ok(Self {
            game,
            subject: SUBJECT,
            status: "ready".to_owned(),
        })
    }

    pub fn game(&self) -> &GameState {
        &self.game
    }

    /// A geometry-preserving anatomy projection with no learned functions.
    ///
    /// The detailed document remains inspectable after a summary-body revision
    /// or death. It is evidence about the last admitted revision, never a
    /// substitute for the current body or a permission to equip against it.
    pub fn sheet(&self) -> SubjectSheet {
        let record = self
            .game
            .anatomies()
            .get(self.subject)
            .expect("session admits its fixture anatomy");
        let body = self
            .game
            .bodies()
            .get(self.subject)
            .expect("session generated its subject");
        let knowledge = TechniqueKnowledge {
            subject: self.subject,
            learned: Vec::new(),
        };
        let inputs = TechniqueInputs {
            occupied_parts: Vec::new(),
            part_capabilities: Vec::new(),
            equipment: Vec::new(),
            resources: Vec::new(),
            environment: ArrestFallEnvironment {
                support_present: false,
                support_distance_voxels: 0,
                arrest_load_mg: 0,
                support_load_capacity_mg: 0,
                adhesive_surface: AdhesiveSurface::Unsuitable,
                adhesive_resource: AdhesiveResource::Exhausted,
            },
        };
        let mut sheet = SubjectSheet::from_input(SubjectSheetInput {
            subject: self.subject,
            revision: record.revision,
            current_revision: body.revision,
            body: &record.document,
            knowledge: &knowledge,
            inputs: &inputs,
            part_names: paredros_world::fixtures::three_lives::PART_NAMES,
            selected_part: None,
        });
        sheet.actions.clear();
        sheet.resources.clear();
        sheet.global_blockers.clear();
        if record.revision != body.revision {
            sheet.global_blockers.push(format!(
                "Detailed anatomy revision {} is stale; current body revision is {}. Reconcile before attaching equipment.",
                record.revision.0, body.revision.0
            ));
        }
        if !body.alive() {
            sheet
                .global_blockers
                .push("Keeper is dead; this sheet is read-only.".to_owned());
        }
        sheet
    }

    pub fn items(&self) -> Vec<Item> {
        self.game
            .items()
            .carried_by(self.subject)
            .copied()
            .collect()
    }

    pub const fn subject(&self) -> SubjectId {
        self.subject
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }

    pub fn save_bytes(&self) -> Result<Vec<u8>, String> {
        self.game.save().map_err(|error| format!("{error:?}"))
    }

    /// Restores only this fixed equipment fixture, preserving the old game on
    /// every decode, replay, or fixture validation failure.
    pub fn load_bytes(&mut self, bytes: &[u8]) -> Result<(), String> {
        let expected_prefix = self
            .game
            .intents()
            .get(..INITIAL_INTENTS)
            .ok_or_else(|| "session lacks its genesis intents".to_owned())?
            .to_vec();
        let restored = match GameState::restore(bytes) {
            Ok(game) => game,
            Err(error) => return self.reject_load(format!("{error:?}")),
        };
        if let Err(error) = validate_loaded(&restored, &expected_prefix, self.game.world()) {
            return self.reject_load(error);
        }
        self.game = restored;
        self.status = "loaded equipment session".to_owned();
        Ok(())
    }

    pub fn attach(&mut self, item: ItemId, part: PartId) {
        let revision = self
            .game
            .bodies()
            .get(self.subject)
            .expect("session generated its subject")
            .revision;
        self.apply(
            GameIntent::AttachItem {
                tick: self.game.next_tick(),
                subject: self.subject,
                item,
                part,
                revision,
            },
            format!("attached item {} to part {}", item.0, part.0),
        );
    }

    pub fn detach(&mut self, item: ItemId) {
        self.apply(
            GameIntent::DetachItem {
                tick: self.game.next_tick(),
                subject: self.subject,
                item,
            },
            format!("detached item {}", item.0),
        );
    }

    fn apply(&mut self, intent: GameIntent, success: String) {
        self.status = match self.game.apply(intent) {
            Ok(_) => success,
            Err(error) => format!("{success} rejected: {error:?}"),
        };
    }

    fn reject_load(&mut self, error: String) -> Result<(), String> {
        self.status = format!("load rejected: {error}");
        Err(error)
    }
}

fn apply_new(game: &mut GameState, intent: GameIntent) -> Result<(), String> {
    game.apply(intent)
        .map(|_| ())
        .map_err(|error| format!("{error:?}"))
}

fn validate_loaded(
    game: &GameState,
    expected_prefix: &[GameIntent],
    expected_world: &World,
) -> Result<(), String> {
    if game.world() != expected_world {
        return Err("different generated-world fixture".to_owned());
    }
    let intents = game.intents();
    if intents.get(..INITIAL_INTENTS) != Some(expected_prefix) {
        return Err("different equipment-session genesis".to_owned());
    }
    if !intents[INITIAL_INTENTS..].iter().all(|intent| {
        matches!(
            intent,
            GameIntent::AttachItem { subject, .. } | GameIntent::DetachItem { subject, .. }
                if *subject == SUBJECT
        )
    }) {
        return Err("equipment session permits only Keeper attachment changes".to_owned());
    }
    let body = game
        .bodies()
        .get(SUBJECT)
        .ok_or_else(|| "loaded session has no Keeper".to_owned())?;
    if !body.alive() || body.revision != BodyRevisionId(0) {
        return Err("loaded Keeper is not the original living revision".to_owned());
    }
    if body.name.as_ref().map(Name::as_str) != Some("Keeper") {
        return Err("loaded subject is not named Keeper".to_owned());
    }
    let anatomy = game
        .current_anatomy(SUBJECT)
        .map_err(|error| format!("loaded anatomy is not current: {error:?}"))?;
    if anatomy.revision != BodyRevisionId(0) || anatomy.document != wetland_body() {
        return Err("loaded anatomy is not the wetland fixture".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attaches_and_detaches_through_game_intents() {
        let mut session = EquipmentSession::new().unwrap();
        let item = session.items()[0].id;
        session.attach(item, PartId(1));
        assert_eq!(
            session.game().items().get(item).unwrap().location,
            ItemLocation::Attached {
                subject: session.subject(),
                part: PartId(1),
            }
        );
        assert_eq!(
            session.status(),
            format!("attached item {} to part 1", item.0)
        );
        session.detach(item);
        assert_eq!(
            session.game().items().get(item).unwrap().location,
            ItemLocation::Carried(session.subject())
        );
        assert_eq!(session.status(), format!("detached item {}", item.0));
    }

    #[test]
    fn rejected_attachment_is_atomic() {
        let mut session = EquipmentSession::new().unwrap();
        let item = session.items()[0].id;
        let before = session.game().save().unwrap();
        session.attach(item, PartId(99));
        assert_eq!(session.game().save().unwrap(), before);
        assert!(session.status().contains("rejected"));
        assert_eq!(
            session.game().items().get(item).unwrap().location,
            ItemLocation::Carried(session.subject())
        );
    }

    #[test]
    fn save_replay_and_subject_identity_stay_stable() {
        let mut session = EquipmentSession::new().unwrap();
        let item = session.items()[0].id;
        session.attach(item, PartId(1));
        let restored = GameState::restore(&session.game().save().unwrap()).unwrap();
        assert_eq!(restored, session.game().clone());
        assert_eq!(session.subject(), SUBJECT);
    }

    #[test]
    fn loading_restores_attached_equipment_and_a_safe_projection() {
        let mut session = EquipmentSession::new().unwrap();
        let item = session.items()[0].id;
        session.attach(item, PartId(1));
        let saved = session.save_bytes().unwrap();
        session.detach(item);
        session.load_bytes(&saved).unwrap();
        assert_eq!(
            session.game().items().get(item).unwrap().location,
            ItemLocation::Attached {
                subject: SUBJECT,
                part: PartId(1),
            }
        );
        assert!(
            session
                .sheet()
                .parts
                .iter()
                .all(|part| part.bounds.is_some())
        );
    }

    #[test]
    fn rejected_loads_leave_the_current_session_unchanged() {
        let mut session = EquipmentSession::new().unwrap();
        let before = session.save_bytes().unwrap();
        assert!(session.load_bytes(&[1, 2, 3]).is_err());
        assert_eq!(session.save_bytes().unwrap(), before);

        let mut wrong_version = session.game().save_record().unwrap();
        wrong_version.version = 0;
        let wrong_version = mesocosm_core::snapshot::encode(&wrong_version).unwrap();
        assert!(session.load_bytes(&wrong_version).is_err());
        assert_eq!(session.save_bytes().unwrap(), before);

        let mut stale = session.game().clone();
        let mut foreign = session.game().save_record().unwrap();
        if let GameIntent::Name { name, .. } = &mut foreign.intents[1] {
            *name = Name::new("Another").unwrap();
        }
        let mut foreign_game = GameState::new(session.game().world().clone());
        for intent in foreign.intents {
            foreign_game.apply(intent).unwrap();
        }
        let foreign = foreign_game.save().unwrap();
        assert!(session.load_bytes(&foreign).is_err());
        assert_eq!(session.save_bytes().unwrap(), before);
        stale
            .apply(GameIntent::Fall {
                tick: stale.next_tick(),
                subject: SUBJECT,
                distance: 5,
            })
            .unwrap();
        assert!(session.load_bytes(&stale.save().unwrap()).is_err());
        assert_eq!(session.save_bytes().unwrap(), before);
        assert!(session.status().contains("load rejected"));
    }

    #[test]
    fn sheet_keeps_anatomy_geometry_without_invented_abilities() {
        let session = EquipmentSession::new().unwrap();
        let sheet = session.sheet();
        assert_eq!(sheet.subject, session.subject());
        assert_eq!(sheet.parts.len(), wetland_body().parts.len());
        assert!(sheet.parts.iter().all(|part| part.bounds.is_some()));
        assert!(sheet.parts.iter().all(|part| part.capabilities.is_empty()));
        assert!(sheet.learned.is_empty());
        assert!(sheet.actions.is_empty());
        assert!(sheet.resources.is_empty());
        assert!(sheet.global_blockers.is_empty());
    }

    #[test]
    fn stale_anatomy_stays_inspectable_but_blocks_new_attachments() {
        let mut session = EquipmentSession::new().unwrap();
        let item = session.items()[0].id;
        session
            .game
            .apply(GameIntent::Fall {
                tick: session.game.next_tick(),
                subject: SUBJECT,
                distance: 5,
            })
            .unwrap();

        let before = session.game.state_hash().unwrap();
        let sheet = session.sheet();
        assert_eq!(session.game.state_hash().unwrap(), before);
        assert!(sheet.parts.iter().all(|part| part.bounds.is_some()));
        assert!(
            sheet
                .global_blockers
                .iter()
                .any(|line| line.contains("stale"))
        );

        session.attach(item, PartId(1));
        assert!(session.status().contains("rejected"));
        assert_eq!(session.game.state_hash().unwrap(), before);
    }

    #[test]
    fn reconciliation_severs_the_sheet_part_and_releases_its_dressing() {
        let mut session = EquipmentSession::new().unwrap();
        let item = session.items()[0].id;
        session.attach(item, PartId(1));
        session
            .game
            .apply(GameIntent::Fall {
                tick: session.game.next_tick(),
                subject: SUBJECT,
                distance: 5,
            })
            .unwrap();
        session
            .game
            .apply(GameIntent::ReconcileAnatomy {
                tick: session.game.next_tick(),
                subject: SUBJECT,
                from_revision: BodyRevisionId(0),
                revision: BodyRevisionId(1),
                severed_parts: vec![PartId(1)],
            })
            .unwrap();

        let at = session.game.movement().position(SUBJECT).unwrap();
        assert_eq!(
            session.game.items().get(item).unwrap().location,
            ItemLocation::At(at)
        );
        let before = session.game.state_hash().unwrap();
        let sheet = session.sheet();
        assert_eq!(session.game.state_hash().unwrap(), before);
        assert!(
            sheet
                .parts
                .iter()
                .any(|part| part.id == PartId(1) && part.severed)
        );
        assert!(sheet.global_blockers.is_empty());
    }

    #[test]
    fn dead_keeper_remains_inspectable_and_cannot_attach() {
        let mut session = EquipmentSession::new().unwrap();
        let item = session.items()[0].id;
        session
            .game
            .apply(GameIntent::Fall {
                tick: session.game.next_tick(),
                subject: SUBJECT,
                distance: 20,
            })
            .unwrap();

        let before = session.game.state_hash().unwrap();
        let sheet = session.sheet();
        assert_eq!(session.game.state_hash().unwrap(), before);
        assert!(!sheet.parts.is_empty());
        assert!(
            sheet
                .global_blockers
                .iter()
                .any(|line| line.contains("dead"))
        );
        assert!(
            sheet
                .global_blockers
                .iter()
                .any(|line| line.contains("stale"))
        );

        session.attach(item, PartId(1));
        assert!(session.status().contains("rejected"));
        assert_eq!(session.game.state_hash().unwrap(), before);
    }
}
