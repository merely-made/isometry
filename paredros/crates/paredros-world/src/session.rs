// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The durable product boundary for one controlled life.
//!
//! This coordinator owns only control history and its save cut. `GameState`
//! remains the sole owner of world, body, movement, inventory, and injury.
//! A control change is stamped at a game-intent cut, so restore can validate
//! who was alive when they were selected rather than guessing from final state.

use mesocosm_core::snapshot::{self, hash_bytes};
use paredros_identity::{Control, ControlIntent, IdentityError, SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::{GAME_STATE_VERSION, GameError, GameEvent, GameIntent, GameSave, GameState, World};

pub const SESSION_VERSION: u32 = 1;
pub const MAX_SESSION_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_GAME_INTENTS: usize = 1_000_000;
pub const MAX_CONTROL_INTENTS: usize = 1_000_000;

/// Host resource limits for session archives. They are not world rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SessionLimits {
    pub max_bytes: usize,
    pub max_game_intents: usize,
    pub max_control_intents: usize,
}

impl Default for SessionLimits {
    fn default() -> Self {
        Self {
            max_bytes: MAX_SESSION_BYTES,
            max_game_intents: MAX_GAME_INTENTS,
            max_control_intents: MAX_CONTROL_INTENTS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    game: GameState,
    control: Control,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionSave {
    pub version: u32,
    pub game: GameSave,
    pub control: Vec<ControlIntent>,
    pub expected_hash: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SessionError {
    Game(GameError),
    Control(IdentityError),
    NotControlled {
        expected: SubjectId,
        actual: SubjectId,
    },
    Ineligible {
        subject: SubjectId,
    },
    SuccessorIsCurrent(SubjectId),
    HomeStillAlive(SubjectId),
    UnsupportedControlIntent,
    ControlOutOfOrder {
        previous: Tick,
        next: Tick,
    },
    ControlCutOutOfRange {
        at: Tick,
        intents: u64,
    },
    VersionDiverged {
        saved: u32,
        current: u32,
    },
    StateDiverged {
        saved: u64,
        restored: u64,
    },
    TooLarge,
    Encode,
    Decode,
}

impl From<GameError> for SessionError {
    fn from(error: GameError) -> Self {
        Self::Game(error)
    }
}

impl From<IdentityError> for SessionError {
    fn from(error: IdentityError) -> Self {
        Self::Control(error)
    }
}

impl Session {
    /// Starts control at the current accepted game-intent cut.
    pub fn begin(game: GameState, subject: SubjectId) -> Result<Self, SessionError> {
        eligible(&game, subject)?;
        Ok(Self {
            control: Control::begin(subject, game.next_tick()),
            game,
        })
    }

    pub fn game(&self) -> &GameState {
        &self.game
    }

    pub fn control(&self) -> &Control {
        &self.control
    }

    /// Applies one ordinary game action for the one currently controlled body.
    pub fn apply_game(&mut self, intent: GameIntent) -> Result<Vec<GameEvent>, SessionError> {
        if intent.subject() != self.control.played() {
            return Err(SessionError::NotControlled {
                expected: self.control.played(),
                actual: intent.subject(),
            });
        }
        Ok(self.game.apply(intent)?)
    }

    /// Continues after the current home body died through another existing life.
    /// This does not transfer property or alter either body's history.
    pub fn succeed_existing(&mut self, successor: SubjectId) -> Result<(), SessionError> {
        let home = self.control.home();
        if self.control.played() != home {
            return Err(SessionError::Control(IdentityError::AlreadyTaggedIn));
        }
        if successor == home {
            return Err(SessionError::SuccessorIsCurrent(successor));
        }
        let home_body = self
            .game
            .bodies()
            .get(home)
            .ok_or(SessionError::Ineligible { subject: home })?;
        if home_body.alive() {
            return Err(SessionError::HomeStillAlive(home));
        }
        eligible(&self.game, successor)?;
        self.control.apply(ControlIntent::Succeed {
            to: successor,
            at: self.game.next_tick(),
        })?;
        Ok(())
    }

    pub fn state_hash(&self) -> Result<u64, SessionError> {
        let game = self.game.state_hash()?;
        snapshot::encode(&(game, self.control.log()))
            .map(|bytes| hash_bytes(&bytes))
            .map_err(|_| SessionError::Encode)
    }

    pub fn save_record(&self) -> Result<SessionSave, SessionError> {
        Ok(SessionSave {
            version: SESSION_VERSION,
            game: self.game.save_record()?,
            control: self.control.log().to_vec(),
            expected_hash: self.state_hash()?,
        })
    }

    pub fn save(&self) -> Result<Vec<u8>, SessionError> {
        self.save_with_limits(SessionLimits::default())
    }

    pub fn save_with_limits(&self, limits: SessionLimits) -> Result<Vec<u8>, SessionError> {
        if self.game.intents().len() > limits.max_game_intents
            || self.control.log().len() > limits.max_control_intents
        {
            return Err(SessionError::TooLarge);
        }
        let save = self.save_record()?;
        let bytes = snapshot::encode(&save).map_err(|_| SessionError::Encode)?;
        if bytes.len() > limits.max_bytes {
            return Err(SessionError::TooLarge);
        }
        Ok(bytes)
    }

    pub fn restore(bytes: &[u8]) -> Result<Self, SessionError> {
        Self::restore_with_limits(bytes, SessionLimits::default())
    }

    pub fn restore_with_limits(bytes: &[u8], limits: SessionLimits) -> Result<Self, SessionError> {
        if bytes.len() > limits.max_bytes {
            return Err(SessionError::TooLarge);
        }
        let save: SessionSave = snapshot::decode(bytes).map_err(|_| SessionError::Decode)?;
        Self::restore_record_with_limits(save, limits)
    }

    /// Restores into a candidate. Callers replace their active session only
    /// after this complete replay and validation succeeds.
    pub fn restore_record(save: SessionSave) -> Result<Self, SessionError> {
        Self::restore_record_with_limits(save, SessionLimits::default())
    }

    pub fn restore_record_with_limits(
        save: SessionSave,
        limits: SessionLimits,
    ) -> Result<Self, SessionError> {
        if save.version != SESSION_VERSION {
            return Err(SessionError::VersionDiverged {
                saved: save.version,
                current: SESSION_VERSION,
            });
        }
        if save.game.version != GAME_STATE_VERSION {
            return Err(SessionError::Game(GameError::VersionDiverged {
                saved: save.game.version,
                current: GAME_STATE_VERSION,
            }));
        }
        if save.game.intents.len() > limits.max_game_intents
            || save.control.len() > limits.max_control_intents
        {
            return Err(SessionError::TooLarge);
        }
        let world = World::restore_record(save.game.world.clone()).map_err(GameError::from)?;
        let mut game = GameState::new(world);
        let mut control = None;
        let mut cursor = 0usize;
        let intent_count = save.game.intents.len() as u64;

        for cut in 0..=save.game.intents.len() {
            let at = Tick(cut as u64);
            while let Some(intent) = save.control.get(cursor).copied() {
                if control_at(intent) < at {
                    return Err(SessionError::ControlOutOfOrder {
                        previous: at,
                        next: control_at(intent),
                    });
                }
                if control_at(intent) > at {
                    break;
                }
                apply_control_at_cut(&mut control, &game, intent)?;
                cursor += 1;
            }
            if cut < save.game.intents.len() {
                let intent = save.game.intents[cut].clone();
                if let Some(control) = &control
                    && intent.subject() != control.played()
                {
                    return Err(SessionError::NotControlled {
                        expected: control.played(),
                        actual: intent.subject(),
                    });
                }
                game.apply(intent)?;
            }
        }
        if let Some(intent) = save.control.get(cursor).copied() {
            return Err(SessionError::ControlCutOutOfRange {
                at: control_at(intent),
                intents: intent_count,
            });
        }
        let control = control.ok_or(SessionError::Control(IdentityError::NotBegun))?;
        let game_hash = game.state_hash()?;
        if game_hash != save.game.expected_hash {
            return Err(SessionError::Game(GameError::StateDiverged {
                saved: save.game.expected_hash,
                restored: game_hash,
            }));
        }
        let session = Self { game, control };
        let restored = session.state_hash()?;
        if restored != save.expected_hash {
            return Err(SessionError::StateDiverged {
                saved: save.expected_hash,
                restored,
            });
        }
        Ok(session)
    }
}

fn eligible(game: &GameState, subject: SubjectId) -> Result<(), SessionError> {
    let Some(body) = game.bodies().get(subject) else {
        return Err(SessionError::Ineligible { subject });
    };
    if body.alive() {
        Ok(())
    } else {
        Err(SessionError::Ineligible { subject })
    }
}

fn control_at(intent: ControlIntent) -> Tick {
    match intent {
        ControlIntent::Begin { at, .. } | ControlIntent::Succeed { at, .. } => at,
        ControlIntent::TagIn { at, .. } | ControlIntent::TagOut { at } => at,
    }
}

fn apply_control_at_cut(
    control: &mut Option<Control>,
    game: &GameState,
    intent: ControlIntent,
) -> Result<(), SessionError> {
    match (control.as_mut(), intent) {
        (None, ControlIntent::Begin { subject, at }) => {
            if at != game.next_tick() {
                return Err(SessionError::ControlCutOutOfRange {
                    at,
                    intents: game.intents().len() as u64,
                });
            }
            eligible(game, subject)?;
            *control = Some(Control::begin(subject, at));
            Ok(())
        },
        (None, _) => Err(SessionError::Control(IdentityError::NotBegun)),
        (Some(_), ControlIntent::Begin { .. }) => {
            Err(SessionError::Control(IdentityError::AlreadyBegun))
        },
        (Some(_), ControlIntent::TagIn { .. } | ControlIntent::TagOut { .. }) => {
            Err(SessionError::UnsupportedControlIntent)
        },
        (Some(control), ControlIntent::Succeed { to, at }) => {
            if at != game.next_tick() {
                return Err(SessionError::ControlCutOutOfRange {
                    at,
                    intents: game.intents().len() as u64,
                });
            }
            let home = control.home();
            if control.played() != home {
                return Err(SessionError::Control(IdentityError::AlreadyTaggedIn));
            }
            if to == home {
                return Err(SessionError::SuccessorIsCurrent(to));
            }
            let home_body = game
                .bodies()
                .get(home)
                .ok_or(SessionError::Ineligible { subject: home })?;
            if home_body.alive() {
                return Err(SessionError::HomeStillAlive(home));
            }
            eligible(game, to)?;
            control.apply(ControlIntent::Succeed { to, at })?;
            Ok(())
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ItemLocation, Name, WorldConfig};

    const FIRST: SubjectId = SubjectId(10);
    const SECOND: SubjectId = SubjectId(11);
    const THIRD: SubjectId = SubjectId(12);

    fn game() -> GameState {
        let mut game = GameState::new(World::generate(7, WorldConfig::default()).unwrap());
        let at = game
            .items()
            .all()
            .find_map(|item| match item.location {
                ItemLocation::At(at) => Some(at),
                _ => None,
            })
            .unwrap();
        for (subject, seed, name) in [
            (FIRST, 1, "First"),
            (SECOND, 2, "Second"),
            (THIRD, 3, "Third"),
        ] {
            let tick = game.next_tick();
            game.apply(GameIntent::Generate {
                tick,
                subject,
                body_seed: seed,
                at,
            })
            .unwrap();
            let tick = game.next_tick();
            game.apply(GameIntent::Name {
                tick,
                subject,
                name: Name::new(name).unwrap(),
            })
            .unwrap();
        }
        game
    }

    fn kill(session: &mut Session) {
        let tick = session.game().next_tick();
        session
            .apply_game(GameIntent::Fall {
                tick,
                subject: session.control().played(),
                distance: 20,
            })
            .unwrap();
    }

    fn rechecksum(mut save: SessionSave) -> SessionSave {
        let world = World::restore_record(save.game.world.clone()).unwrap();
        let mut game = GameState::new(world);
        for intent in &save.game.intents {
            game.apply(intent.clone()).unwrap();
        }
        save.game.expected_hash = game.state_hash().unwrap();
        save.expected_hash = snapshot::encode(&(save.game.expected_hash, save.control.as_slice()))
            .map(|bytes| hash_bytes(&bytes))
            .unwrap();
        save
    }

    #[test]
    fn control_gates_actions_without_copying_other_lives() {
        let game = game();
        let before = game.bodies().get(SECOND).unwrap().clone();
        let mut session = Session::begin(game, FIRST).unwrap();
        let tick = session.game().next_tick();
        assert_eq!(
            session.apply_game(GameIntent::Wait {
                tick,
                subject: SECOND
            }),
            Err(SessionError::NotControlled {
                expected: FIRST,
                actual: SECOND
            })
        );
        assert_eq!(session.game().bodies().get(SECOND).unwrap(), &before);
    }

    #[test]
    fn begin_rejects_missing_unnamed_and_dead_subjects() {
        let state = game();
        assert_eq!(
            Session::begin(state.clone(), SubjectId(99)),
            Err(SessionError::Ineligible {
                subject: SubjectId(99)
            })
        );
        let at = state.movement().position(FIRST).unwrap();
        let mut unnamed = state.clone();
        unnamed
            .apply(GameIntent::Generate {
                tick: unnamed.next_tick(),
                subject: SubjectId(99),
                body_seed: 99,
                at,
            })
            .unwrap();
        assert!(matches!(
            Session::begin(unnamed, SubjectId(99)),
            Err(SessionError::Ineligible { .. })
        ));
        let mut dead = state;
        dead.apply(GameIntent::Fall {
            tick: dead.next_tick(),
            subject: FIRST,
            distance: 20,
        })
        .unwrap();
        assert!(matches!(
            Session::begin(dead, FIRST),
            Err(SessionError::Ineligible { .. })
        ));
    }

    #[test]
    fn death_succeeds_to_an_existing_life_and_replays() {
        let mut session = Session::begin(game(), FIRST).unwrap();
        let at = session.game().movement().position(FIRST).unwrap();
        let item = session.game().items().at(at).next().unwrap().id;
        let tick = session.game().next_tick();
        session
            .apply_game(GameIntent::Take {
                tick,
                subject: FIRST,
                item,
            })
            .unwrap();
        kill(&mut session);
        session.succeed_existing(SECOND).unwrap();
        assert_eq!(session.control().played(), SECOND);
        assert!(!session.game().bodies().get(FIRST).unwrap().alive());
        assert_eq!(
            session.game().items().get(item).unwrap().location,
            ItemLocation::Carried(FIRST)
        );
        let restored = Session::restore(&session.save().unwrap()).unwrap();
        assert_eq!(restored, session);
        assert_eq!(
            restored.game().items().get(item).unwrap().location,
            ItemLocation::Carried(FIRST)
        );
    }

    #[test]
    fn historical_cuts_allow_a_later_dead_successor() {
        let mut session = Session::begin(game(), FIRST).unwrap();
        kill(&mut session);
        session.succeed_existing(SECOND).unwrap();
        kill(&mut session);
        session.succeed_existing(THIRD).unwrap();
        assert_eq!(session.control().played(), THIRD);
        assert_eq!(Session::restore(&session.save().unwrap()).unwrap(), session);
    }

    #[test]
    fn malformed_control_history_is_rejected_even_with_a_recomputed_hash() {
        let session = Session::begin(game(), FIRST).unwrap();
        let mut save = session.save_record().unwrap();
        save.control.push(ControlIntent::Succeed {
            to: SECOND,
            at: Tick(6),
        });
        let save = rechecksum(save);
        assert!(matches!(
            Session::restore_record(save),
            Err(SessionError::HomeStillAlive(FIRST))
        ));
    }

    #[test]
    fn forged_post_begin_foreign_action_is_rejected_with_a_valid_checksum() {
        let session = Session::begin(game(), FIRST).unwrap();
        let mut save = session.save_record().unwrap();
        save.game.intents.push(GameIntent::Wait {
            tick: Tick(save.game.intents.len() as u64),
            subject: SECOND,
        });
        let save = rechecksum(save);
        assert!(matches!(
            Session::restore_record(save),
            Err(SessionError::NotControlled {
                expected: FIRST,
                actual: SECOND
            })
        ));
    }

    #[test]
    fn corrupt_or_version_diverged_candidates_leave_a_live_session_unchanged() {
        let session = Session::begin(game(), FIRST).unwrap();
        let before = session.clone();
        assert_eq!(Session::restore(&[0, 1, 2]), Err(SessionError::Decode));
        let mut save = session.save_record().unwrap();
        save.version = 9;
        assert!(matches!(
            Session::restore_record(save),
            Err(SessionError::VersionDiverged { .. })
        ));
        assert_eq!(session, before);
    }

    #[test]
    fn dead_controlled_subject_can_be_saved_while_awaiting_successor() {
        let mut session = Session::begin(game(), FIRST).unwrap();
        kill(&mut session);
        let restored = Session::restore(&session.save().unwrap()).unwrap();
        assert_eq!(restored, session);
        assert!(!restored.game().bodies().get(FIRST).unwrap().alive());
        assert_eq!(restored.control().played(), FIRST);
    }
}
