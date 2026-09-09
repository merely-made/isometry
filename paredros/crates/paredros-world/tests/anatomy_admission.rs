// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::{BodyDocument, Origin, PartId, SpeciesId, VolumeRef, snapshot};
use paredros_identity::{BodyRevisionId, SubjectId, Tick};
use paredros_world::fixtures::three_lives as fixture;
use paredros_world::{
    AnatomyError, BodyError, GAME_STATE_VERSION, GameError, GameEvent, GameIntent, GameState, Name,
    Navigation, SiteKind, World, WorldConfig,
};

const SEED: u64 = 42_424;
const SUBJECT: SubjectId = SubjectId(1);

fn state() -> GameState {
    let world = World::generate(SEED, WorldConfig::default()).unwrap();
    let at = world
        .map()
        .slots_of_kind(SiteKind::Settlement)
        .next()
        .map(|slot| Navigation::default().surface_stance(&world, slot).unwrap())
        .unwrap();
    let mut state = GameState::new(world);
    state
        .apply(GameIntent::Generate {
            tick: state.next_tick(),
            subject: SUBJECT,
            body_seed: 7,
            at,
        })
        .unwrap();
    state
        .apply(GameIntent::Name {
            tick: state.next_tick(),
            subject: SUBJECT,
            name: Name::new("Anatomist").unwrap(),
        })
        .unwrap();
    state
}

fn unnamed_state() -> GameState {
    let world = World::generate(SEED, WorldConfig::default()).unwrap();
    let at = world
        .map()
        .slots_of_kind(SiteKind::Settlement)
        .next()
        .map(|slot| Navigation::default().surface_stance(&world, slot).unwrap())
        .unwrap();
    let mut state = GameState::new(world);
    state
        .apply(GameIntent::Generate {
            tick: state.next_tick(),
            subject: SUBJECT,
            body_seed: 7,
            at,
        })
        .unwrap();
    state
}

fn admit(state: &mut GameState, document: BodyDocument, revision: BodyRevisionId) {
    let tick = state.next_tick();
    let events = state
        .apply(GameIntent::AdmitAnatomy {
            tick: state.next_tick(),
            subject: SUBJECT,
            revision,
            document: Box::new(document),
        })
        .unwrap();
    assert_eq!(
        events,
        vec![GameEvent::AnatomyAdmitted {
            tick,
            subject: SUBJECT,
            revision
        }]
    );
}

fn unchanged(state: &GameState, before: (u64, Tick, usize, usize)) {
    assert_eq!(state.state_hash().unwrap(), before.0);
    assert_eq!(state.next_tick(), before.1);
    assert_eq!(state.intents().len(), before.2);
    assert_eq!(state.events().len(), before.3);
}

#[test]
fn admission_preserves_fixture_provenance_and_severed_parts() {
    let life = fixture::three_lives()[2].clone(); // Mend, with the severed limb.
    let mut state = state();
    admit(&mut state, life.body.clone(), BodyRevisionId(0));

    let record = state.anatomies().get(SUBJECT).unwrap();
    assert_eq!(record.subject, SUBJECT);
    assert_eq!(record.revision, BodyRevisionId(0));
    assert_eq!(record.document, life.body);
    assert!(!record.document.is_living(PartId(1)));
    assert_eq!(
        record
            .document
            .part(fixture::ADHESIVE_SYMBIONT)
            .unwrap()
            .provenance
            .origin,
        Origin::Incorporated {
            from_species: fixture::symbiont_donor().species,
            from_part: fixture::symbiont_donor().root,
        }
    );
}

#[test]
fn admission_roundtrips_through_save_restore_and_snapshot_codec() {
    let document = fixture::three_lives()[0].body.clone();
    let mut state = state();
    admit(&mut state, document, BodyRevisionId(0));
    assert_eq!(
        state.current_anatomy(SUBJECT).unwrap().revision,
        BodyRevisionId(0)
    );

    let save = state.save_record().unwrap();
    let bytes = snapshot::encode(&save).unwrap();
    let decoded = snapshot::decode(&bytes).unwrap();
    let restored = GameState::restore_record(decoded).unwrap();
    assert_eq!(restored, state);
    assert_eq!(GameState::restore(&state.save().unwrap()).unwrap(), state);
}

#[test]
fn duplicate_and_invalid_admissions_are_atomic() {
    let document = fixture::three_lives()[0].body.clone();
    let mut state = state();
    admit(&mut state, document.clone(), BodyRevisionId(0));

    let before = (
        state.state_hash().unwrap(),
        state.next_tick(),
        state.intents().len(),
        state.events().len(),
    );
    assert!(
        state
            .apply(GameIntent::AdmitAnatomy {
                tick: state.next_tick(),
                subject: SUBJECT,
                revision: BodyRevisionId(0),
                document: Box::new(document.clone()),
            })
            .is_err()
    );
    unchanged(&state, before);

    for (subject, revision, document) in [
        (SubjectId(999), BodyRevisionId(0), document.clone()),
        (SUBJECT, BodyRevisionId(99), document.clone()),
        (SUBJECT, BodyRevisionId(0), document),
    ] {
        let before = (
            state.state_hash().unwrap(),
            state.next_tick(),
            state.intents().len(),
            state.events().len(),
        );
        assert!(
            state
                .apply(GameIntent::AdmitAnatomy {
                    tick: state.next_tick(),
                    subject,
                    revision,
                    document: Box::new(document),
                })
                .is_err()
        );
        unchanged(&state, before);
    }
}

#[test]
fn unnamed_dead_and_stale_subjects_are_rejected_atomically() {
    let mut unnamed = unnamed_state();
    let before = (
        unnamed.state_hash().unwrap(),
        unnamed.next_tick(),
        unnamed.intents().len(),
        unnamed.events().len(),
    );
    assert_eq!(
        unnamed.apply(GameIntent::AdmitAnatomy {
            tick: unnamed.next_tick(),
            subject: SUBJECT,
            revision: BodyRevisionId(0),
            document: Box::new(fixture::three_lives()[0].body.clone()),
        }),
        Err(GameError::Body(BodyError::Unnamed(SUBJECT)))
    );
    unchanged(&unnamed, before);

    let mut dead = state();
    dead.apply(GameIntent::Fall {
        tick: dead.next_tick(),
        subject: SUBJECT,
        distance: 20,
    })
    .unwrap();
    assert!(!dead.bodies().get(SUBJECT).unwrap().alive());
    let before = (
        dead.state_hash().unwrap(),
        dead.next_tick(),
        dead.intents().len(),
        dead.events().len(),
    );
    assert_eq!(
        dead.apply(GameIntent::AdmitAnatomy {
            tick: dead.next_tick(),
            subject: SUBJECT,
            revision: BodyRevisionId(1),
            document: Box::new(fixture::three_lives()[0].body.clone()),
        }),
        Err(GameError::Body(BodyError::Dead(SUBJECT)))
    );
    unchanged(&dead, before);

    let mut stale = state();
    admit(
        &mut stale,
        fixture::three_lives()[0].body.clone(),
        BodyRevisionId(0),
    );
    stale
        .apply(GameIntent::Fall {
            tick: stale.next_tick(),
            subject: SUBJECT,
            distance: 9,
        })
        .unwrap();
    assert!(stale.bodies().get(SUBJECT).unwrap().alive());
    assert_eq!(
        stale.bodies().get(SUBJECT).unwrap().revision,
        BodyRevisionId(1)
    );
    assert_eq!(
        stale.current_anatomy(SUBJECT),
        Err(GameError::Anatomy(AnatomyError::StaleRevision {
            known: BodyRevisionId(0),
            current: BodyRevisionId(1)
        }))
    );
    assert!(stale.anatomies().get(SUBJECT).is_some());
    let restored = GameState::restore(&stale.save().unwrap()).unwrap();
    assert_eq!(restored, stale);
    assert!(restored.current_anatomy(SUBJECT).is_err());
}

#[test]
fn malformed_graph_is_rejected_before_storage_and_old_versions_are_rejected() {
    let mut malformed = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1, [1, 1, 1]);
    malformed.parts.clear();
    let mut state = state();
    let before = (
        state.state_hash().unwrap(),
        state.next_tick(),
        state.intents().len(),
        state.events().len(),
    );
    assert!(
        state
            .apply(GameIntent::AdmitAnatomy {
                tick: state.next_tick(),
                subject: SUBJECT,
                revision: BodyRevisionId(0),
                document: Box::new(malformed),
            })
            .is_err()
    );
    unchanged(&state, before);

    let mut save = state.save_record().unwrap();
    save.version = 1;
    assert_eq!(
        GameState::restore_record(save),
        Err(GameError::VersionDiverged {
            saved: 1,
            current: GAME_STATE_VERSION
        })
    );
}

#[test]
fn admission_is_subject_scoped_and_does_not_replace_summary_condition() {
    let mut state = state();
    let peer = SubjectId(2);
    let at = state.movement().position(SUBJECT).unwrap();
    state
        .apply(GameIntent::Generate {
            tick: state.next_tick(),
            subject: peer,
            body_seed: 9,
            at,
        })
        .unwrap();
    state
        .apply(GameIntent::Name {
            tick: state.next_tick(),
            subject: peer,
            name: Name::new("Peer").unwrap(),
        })
        .unwrap();
    let bodies = state.bodies().clone();
    let items = state.items().clone();
    admit(
        &mut state,
        fixture::three_lives()[2].body.clone(),
        BodyRevisionId(0),
    );
    assert_eq!(
        state.current_anatomy(peer),
        Err(GameError::Anatomy(AnatomyError::Missing(peer)))
    );
    state
        .apply(GameIntent::AdmitAnatomy {
            tick: state.next_tick(),
            subject: peer,
            revision: BodyRevisionId(0),
            document: Box::new(fixture::three_lives()[0].body.clone()),
        })
        .unwrap();
    assert_eq!(
        state.current_anatomy(SUBJECT).unwrap().document.parts.len(),
        8
    );
    assert_eq!(state.current_anatomy(peer).unwrap().document.parts.len(), 7);
    assert_eq!(state.bodies(), &bodies);
    assert_eq!(state.items(), &items);
    assert_eq!(GameState::restore(&state.save().unwrap()).unwrap(), state);
}
