// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::snapshot::{encode, hash_bytes};
use paredros_identity::{ControlIntent, SubjectId, Tick};
use paredros_world::{
    GameIntent, GameState, ItemLocation, Name, Session, SessionError, SessionLimits, SessionSave,
    World, WorldConfig,
};

const FIRST: SubjectId = SubjectId(1);
const SECOND: SubjectId = SubjectId(2);

fn session(second_dead: bool) -> Session {
    let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at) => Some(at),
            _ => None,
        })
        .unwrap();
    for subject in [FIRST, SECOND] {
        game.apply(GameIntent::Generate {
            tick: game.next_tick(),
            subject,
            body_seed: subject.0,
            at,
        })
        .unwrap();
        game.apply(GameIntent::Name {
            tick: game.next_tick(),
            subject,
            name: Name::new(format!("Life {}", subject.0)).unwrap(),
        })
        .unwrap();
    }
    if second_dead {
        game.apply(GameIntent::Fall {
            tick: game.next_tick(),
            subject: SECOND,
            distance: 20,
        })
        .unwrap();
    }
    Session::begin(game, FIRST).unwrap()
}

fn kill(session: &mut Session) {
    session
        .apply_game(GameIntent::Fall {
            tick: session.game().next_tick(),
            subject: FIRST,
            distance: 20,
        })
        .unwrap();
}

fn rechecksum(mut save: SessionSave) -> SessionSave {
    save.expected_hash =
        hash_bytes(&encode(&(save.game.expected_hash, save.control.as_slice())).unwrap());
    save
}

#[test]
fn byte_and_record_limits_are_symmetric_at_the_boundary() {
    let session = session(false);
    let bytes = session.save().unwrap();
    let exact = SessionLimits {
        max_bytes: bytes.len(),
        max_game_intents: session.game().intents().len(),
        max_control_intents: session.control().log().len(),
    };
    assert_eq!(session.save_with_limits(exact).unwrap(), bytes);
    assert_eq!(
        Session::restore_with_limits(&bytes, exact).unwrap(),
        session
    );
    for limits in [
        SessionLimits {
            max_bytes: exact.max_bytes - 1,
            ..exact
        },
        SessionLimits {
            max_game_intents: exact.max_game_intents - 1,
            ..exact
        },
        SessionLimits {
            max_control_intents: 0,
            ..exact
        },
    ] {
        assert_eq!(
            session.save_with_limits(limits),
            Err(SessionError::TooLarge)
        );
        assert_eq!(
            Session::restore_with_limits(&bytes, limits),
            Err(SessionError::TooLarge)
        );
    }
    for limits in [
        SessionLimits {
            max_game_intents: 0,
            ..exact
        },
        SessionLimits {
            max_control_intents: 0,
            ..exact
        },
    ] {
        assert_eq!(
            Session::restore_record_with_limits(session.save_record().unwrap(), limits),
            Err(SessionError::TooLarge)
        );
    }
}

#[test]
fn invalid_control_cuts_are_refused_even_with_matching_checksums() {
    let mut session = session(false);
    let begin = session.game().next_tick();
    kill(&mut session);
    session.succeed_existing(SECOND).unwrap();
    for (at, expected) in [
        (begin, SessionError::HomeStillAlive(FIRST)),
        (
            Tick(begin.0 - 1),
            SessionError::ControlOutOfOrder {
                previous: begin,
                next: Tick(begin.0 - 1),
            },
        ),
        (
            Tick(begin.0 + 2),
            SessionError::ControlCutOutOfRange {
                at: Tick(begin.0 + 2),
                intents: begin.0 + 1,
            },
        ),
    ] {
        let mut save = session.save_record().unwrap();
        save.control[1] = ControlIntent::Succeed { to: SECOND, at };
        assert_eq!(Session::restore_record(rechecksum(save)), Err(expected));
    }
}

#[test]
fn succession_cannot_select_missing_or_already_dead_lives() {
    let mut session = session(true);
    kill(&mut session);
    let before = session.clone();
    for target in [SECOND, SubjectId(99)] {
        let expected = SessionError::Ineligible { subject: target };
        assert_eq!(session.succeed_existing(target), Err(expected.clone()));
        assert_eq!(session, before);
        let mut save = session.save_record().unwrap();
        save.control.push(ControlIntent::Succeed {
            to: target,
            at: session.game().next_tick(),
        });
        assert_eq!(Session::restore_record(rechecksum(save)), Err(expected));
    }
    let mut restored = Session::restore(&session.save().unwrap()).unwrap();
    assert!(
        restored
            .apply_game(GameIntent::Wait {
                tick: restored.game().next_tick(),
                subject: FIRST,
            })
            .is_err()
    );
    assert_eq!(restored, before);
}
