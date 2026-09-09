// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Paredros-owned deterministic limb actions over the shared charge router.
//! The router only says whether charge can reach a supplied part. This module
//! owns elapsed ticks, anatomical bindings, interruption, and strike receipts.

use std::collections::{BTreeMap, BTreeSet};

use mesocosm_core::{PartId, snapshot};
use paredros_identity::{BodyRevisionId, SubjectId, Tick};
use serde::{Deserialize, Serialize};
use wing_functions::{
    EffectReport, EvaluationError, EvaluationRequest, FunctionalNetwork, NetworkError,
    NetworkSnapshot, Node, NodeId, NodeKind, Operator, PartRef, WorldRules,
};

mod routing;
use routing::route_open;

use crate::{GameEvent, GameIntent, Session, SessionError, SessionSave};

pub const TIMED_ACTION_VERSION: u32 = 1;
pub const MAX_CONTRIBUTORS: usize = 32;
pub const MAX_ELAPSED_TICKS: u64 = 600;
pub const MAX_TIMED_ACTION_BYTES: usize = crate::MAX_SESSION_BYTES + 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LimbBinding {
    pub part: PartId,
    pub revision: BodyRevisionId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContributionState {
    Charging,
    Paused,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contribution {
    pub binding: LimbBinding,
    pub node: NodeId,
    pub charge: u64,
    pub state: ContributionState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedAction {
    pub subject: SubjectId,
    pub direction: Direction,
    pub started_at: Tick,
    pub last_tick: Tick,
    pub contributors: BTreeMap<PartId, Contribution>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedActionRules {
    pub max_contributors: usize,
    pub max_elapsed_ticks: u64,
    pub charge_per_tick: u64,
    pub max_charge_per_limb: u64,
    pub evaluation: WorldRules,
}

impl TimedActionRules {
    pub fn validate(&self) -> Result<(), TimedActionError> {
        if self.max_contributors == 0
            || self.max_contributors > MAX_CONTRIBUTORS
            || self.max_elapsed_ticks == 0
            || self.max_elapsed_ticks > MAX_ELAPSED_TICKS
            || self.charge_per_tick == 0
            || self.max_charge_per_limb == 0
        {
            return Err(TimedActionError::InvalidRules);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrikeReceipt {
    pub binding: LimbBinding,
    pub direction: Direction,
    pub charge: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChargeOutcome {
    Charged {
        part: PartId,
        amount: u64,
        total: u64,
    },
    Paused {
        part: PartId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedActionSave {
    pub version: u32,
    pub session: SessionSave,
    pub network: NetworkSnapshot,
    pub rules: TimedActionRules,
    pub action: Option<TimedAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimedActionError {
    Session(SessionError),
    Network(NetworkError),
    Evaluation(EvaluationError),
    InvalidRules,
    NoCurrentAction,
    ActionAlreadyPrepared,
    WrongSubject(SubjectId),
    UnknownNode(NodeId),
    WrongTick { previous: Tick, next: Tick },
    ElapsedLimit,
    NoAvailableLimb,
    VersionDiverged { saved: u32, current: u32 },
    Encode,
    Decode,
}

impl From<SessionError> for TimedActionError {
    fn from(value: SessionError) -> Self {
        Self::Session(value)
    }
}
impl From<NetworkError> for TimedActionError {
    fn from(value: NetworkError) -> Self {
        Self::Network(value)
    }
}
impl From<EvaluationError> for TimedActionError {
    fn from(value: EvaluationError) -> Self {
        Self::Evaluation(value)
    }
}

/// Owns one authoritative session and does not expose it mutably. Game changes
/// go through `apply_game_batch`, so anatomy and action bindings reconcile at
/// the same accepted cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimedActionSession {
    session: Session,
    network: FunctionalNetwork,
    rules: TimedActionRules,
    action: Option<TimedAction>,
}

impl TimedActionSession {
    pub fn begin(
        session: Session,
        network: FunctionalNetwork,
        rules: TimedActionRules,
    ) -> Result<Self, TimedActionError> {
        rules.validate()?;
        network.validate()?;
        Ok(Self {
            session,
            network,
            rules,
            action: None,
        })
    }
    pub fn session(&self) -> &Session {
        &self.session
    }
    pub fn network(&self) -> &FunctionalNetwork {
        &self.network
    }
    pub fn action(&self) -> Option<&TimedAction> {
        self.action.as_ref()
    }

    /// The only supported network mutation. Closing a gate invalidates release
    /// only where that gate is still a route to the particular limb.
    pub fn set_gate(&mut self, node: NodeId, open: bool) -> Result<(), TimedActionError> {
        let Some(found) = self.network.nodes.get_mut(&node) else {
            return Err(TimedActionError::UnknownNode(node));
        };
        let NodeKind::Gate { open: state, .. } = &mut found.kind else {
            return Err(TimedActionError::InvalidRules);
        };
        *state = open;
        Ok(())
    }

    pub fn prepare(&mut self, direction: Direction) -> Result<&TimedAction, TimedActionError> {
        if self.action.is_some() {
            return Err(TimedActionError::ActionAlreadyPrepared);
        }
        let subject = self.session.control().played();
        let anatomy = self
            .session
            .game()
            .current_anatomy(subject)
            .map_err(SessionError::from)?;
        let mut contributors = BTreeMap::new();
        for (&node, value) in &self.network.nodes {
            let NodeKind::Effect { part } = value.kind else {
                continue;
            };
            if part.subject != subject.0 {
                continue;
            }
            let part = PartId(part.part);
            if anatomy
                .document
                .part(part)
                .is_some_and(|found| !found.severed)
            {
                contributors.insert(
                    part,
                    Contribution {
                        binding: LimbBinding {
                            part,
                            revision: anatomy.revision,
                        },
                        node,
                        charge: 0,
                        state: ContributionState::Charging,
                    },
                );
                break;
            }
        }
        if contributors.is_empty() {
            return Err(TimedActionError::NoAvailableLimb);
        }
        let now = self.session.game().next_tick();
        self.action = Some(TimedAction {
            subject,
            direction,
            started_at: now,
            last_tick: now,
            contributors,
        });
        Ok(self.action.as_ref().expect("just installed"))
    }

    /// Adds one explicitly selected current limb. A part can contribute once,
    /// even if a malformed network names several effects for it.
    pub fn join(&mut self, part: PartId) -> Result<(), TimedActionError> {
        let action = self
            .action
            .as_mut()
            .ok_or(TimedActionError::NoCurrentAction)?;
        if action.contributors.contains_key(&part)
            || action.contributors.len() == self.rules.max_contributors
        {
            return Err(TimedActionError::InvalidRules);
        }
        let anatomy = self
            .session
            .game()
            .current_anatomy(action.subject)
            .map_err(SessionError::from)?;
        if !anatomy
            .document
            .part(part)
            .is_some_and(|found| !found.severed)
        {
            return Err(TimedActionError::NoAvailableLimb);
        }
        let node = self.network.nodes.iter().find_map(|(&id, node)| matches!(node.kind, NodeKind::Effect { part: found } if found == PartRef { subject: action.subject.0, part: part.0 }).then_some(id)).ok_or(TimedActionError::NoAvailableLimb)?;
        action.contributors.insert(
            part,
            Contribution {
                binding: LimbBinding {
                    part,
                    revision: anatomy.revision,
                },
                node,
                charge: 0,
                state: ContributionState::Charging,
            },
        );
        Ok(())
    }

    pub fn charge(&mut self, tick: Tick) -> Result<Vec<ChargeOutcome>, TimedActionError> {
        let mut candidate = self.clone();
        let outcome = candidate.charge_inner(tick)?;
        *self = candidate;
        Ok(outcome)
    }

    fn charge_inner(&mut self, tick: Tick) -> Result<Vec<ChargeOutcome>, TimedActionError> {
        let mut action = self
            .action
            .take()
            .ok_or(TimedActionError::NoCurrentAction)?;
        if action.last_tick.0.checked_add(1) != Some(tick.0) {
            self.action = Some(action);
            return Err(TimedActionError::WrongTick {
                previous: self.action.as_ref().unwrap().last_tick,
                next: tick,
            });
        }
        if tick.0 - action.started_at.0 > self.rules.max_elapsed_ticks {
            self.action = Some(action);
            return Err(TimedActionError::ElapsedLimit);
        }
        let live = live_parts(self.session.game(), action.subject).map_err(SessionError::from)?;
        let mut candidate = self.network.clone();
        let mut outcomes = Vec::new();
        for contribution in action.contributors.values_mut() {
            if contribution.state == ContributionState::Cancelled
                || contribution.charge == self.rules.max_charge_per_limb
            {
                continue;
            }
            let amount = self
                .rules
                .charge_per_tick
                .min(self.rules.max_charge_per_limb - contribution.charge);
            let request = EvaluationRequest {
                operator: Operator::Strengthen,
                target: contribution.node,
                cost: amount,
                range: 0,
                max_hops: self.rules.evaluation.max_hops,
            };
            match candidate.evaluate(&request, &live, &self.rules.evaluation) {
                Ok(evaluation) => match evaluation.effect {
                    EffectReport::Strengthened { amount, .. } => {
                        contribution.charge += amount;
                        contribution.state = ContributionState::Charging;
                        outcomes.push(ChargeOutcome::Charged {
                            part: contribution.binding.part,
                            amount,
                            total: contribution.charge,
                        });
                    },
                    _ => unreachable!("strengthen result"),
                },
                Err(
                    EvaluationError::InsufficientCharge { .. }
                    | EvaluationError::ClosedGate { .. }
                    | EvaluationError::MissingPart(_),
                ) => {
                    contribution.state = ContributionState::Paused;
                    outcomes.push(ChargeOutcome::Paused {
                        part: contribution.binding.part,
                    });
                },
                Err(error) => {
                    self.action = Some(action);
                    return Err(error.into());
                },
            }
        }
        self.network = candidate;
        action.last_tick = tick;
        self.action = Some(action);
        Ok(outcomes)
    }

    /// Emits the charge already paid during `charge`; it never evaluates or debits a route.
    pub fn release(&mut self, tick: Tick) -> Result<Vec<StrikeReceipt>, TimedActionError> {
        let mut candidate = self.clone();
        let strikes = candidate.release_inner(tick)?;
        *self = candidate;
        Ok(strikes)
    }

    fn release_inner(&mut self, tick: Tick) -> Result<Vec<StrikeReceipt>, TimedActionError> {
        let action = self
            .action
            .take()
            .ok_or(TimedActionError::NoCurrentAction)?;
        if tick < action.last_tick || tick.0 - action.started_at.0 > self.rules.max_elapsed_ticks {
            self.action = Some(action);
            return Err(TimedActionError::ElapsedLimit);
        }
        let live = live_parts(self.session.game(), action.subject).map_err(SessionError::from)?;
        Ok(action
            .contributors
            .into_values()
            .filter(|c| {
                c.state != ContributionState::Cancelled
                    && c.charge > 0
                    && route_open(&self.network, c.node, &live, self.rules.evaluation.max_hops)
            })
            .map(|c| StrikeReceipt {
                binding: c.binding,
                direction: action.direction,
                charge: c.charge,
            })
            .collect())
    }

    /// Applies a whole injury/reconciliation cut before action repair. A failed
    /// member leaves both the session and action untouched.
    pub fn apply_game_batch(
        &mut self,
        intents: &[GameIntent],
    ) -> Result<Vec<GameEvent>, TimedActionError> {
        let mut candidate = self.clone();
        let mut events = Vec::new();
        for intent in intents {
            events.extend(candidate.session.apply_game(intent.clone())?);
        }
        candidate.reconcile_action()?;
        *self = candidate;
        Ok(events)
    }

    pub fn save_record(&self) -> Result<TimedActionSave, TimedActionError> {
        Ok(TimedActionSave {
            version: TIMED_ACTION_VERSION,
            session: self.session.save_record()?,
            network: NetworkSnapshot::new(self.network.clone()),
            rules: self.rules.clone(),
            action: self.action.clone(),
        })
    }
    pub fn save(&self) -> Result<Vec<u8>, TimedActionError> {
        let bytes = snapshot::encode(&self.save_record()?).map_err(|_| TimedActionError::Encode)?;
        if bytes.len() > MAX_TIMED_ACTION_BYTES {
            return Err(TimedActionError::Encode);
        }
        Ok(bytes)
    }
    pub fn restore(bytes: &[u8]) -> Result<Self, TimedActionError> {
        if bytes.len() > MAX_TIMED_ACTION_BYTES {
            return Err(TimedActionError::Decode);
        }
        let save = snapshot::decode(bytes).map_err(|_| TimedActionError::Decode)?;
        Self::restore_record(save)
    }
    pub fn restore_record(save: TimedActionSave) -> Result<Self, TimedActionError> {
        if save.version != TIMED_ACTION_VERSION {
            return Err(TimedActionError::VersionDiverged {
                saved: save.version,
                current: TIMED_ACTION_VERSION,
            });
        }
        save.rules.validate()?;
        save.network.validate()?;
        let mut restored = Self::begin(
            Session::restore_record(save.session)?,
            save.network.network,
            save.rules,
        )?;
        restored.action = save.action;
        restored.validate_action()?;
        Ok(restored)
    }

    fn reconcile_action(&mut self) -> Result<(), TimedActionError> {
        let Some(action) = &mut self.action else {
            return Ok(());
        };
        match self.session.game().current_anatomy(action.subject) {
            Ok(anatomy) => {
                for contribution in action.contributors.values_mut() {
                    if anatomy
                        .document
                        .part(contribution.binding.part)
                        .is_some_and(|part| !part.severed)
                    {
                        contribution.binding.revision = anatomy.revision;
                    } else {
                        contribution.state = ContributionState::Cancelled;
                    }
                }
            },
            Err(crate::GameError::Body(crate::BodyError::Dead(_))) => {
                for contribution in action.contributors.values_mut() {
                    contribution.state = ContributionState::Cancelled;
                }
            },
            Err(crate::GameError::Anatomy(crate::AnatomyError::StaleRevision { .. })) => {
                for contribution in action.contributors.values_mut() {
                    if contribution.state != ContributionState::Cancelled {
                        contribution.state = ContributionState::Paused;
                    }
                }
            },
            Err(error) => return Err(SessionError::from(error).into()),
        }
        Ok(())
    }
    fn validate_action(&mut self) -> Result<(), TimedActionError> {
        if let Some(action) = &self.action {
            if action.subject != self.session.control().played() {
                return Err(TimedActionError::WrongSubject(action.subject));
            }
            if action.contributors.is_empty()
                || action.contributors.len() > self.rules.max_contributors
                || action.last_tick < action.started_at
                || action.last_tick.0 - action.started_at.0 > self.rules.max_elapsed_ticks
            {
                return Err(TimedActionError::InvalidRules);
            }
            let anatomy = self.session.game().current_anatomy(action.subject);
            for (&part, contribution) in &action.contributors {
                if part != contribution.binding.part
                    || contribution.charge > self.rules.max_charge_per_limb
                {
                    return Err(TimedActionError::InvalidRules);
                }
                let Some(Node {
                    kind: NodeKind::Effect { part: effect },
                    ..
                }) = self.network.nodes.get(&contribution.node)
                else {
                    return Err(TimedActionError::InvalidRules);
                };
                if *effect
                    != (PartRef {
                        subject: action.subject.0,
                        part: part.0,
                    })
                {
                    return Err(TimedActionError::InvalidRules);
                }
                let valid = match &anatomy {
                    Ok(anatomy) => {
                        contribution.state == ContributionState::Cancelled
                            || (contribution.binding.revision == anatomy.revision
                                && anatomy
                                    .document
                                    .part(part)
                                    .is_some_and(|item| !item.severed))
                    },
                    Err(crate::GameError::Body(crate::BodyError::Dead(_))) => {
                        contribution.state == ContributionState::Cancelled
                    },
                    Err(crate::GameError::Anatomy(crate::AnatomyError::StaleRevision {
                        ..
                    })) => contribution.state != ContributionState::Charging,
                    Err(error) => return Err(SessionError::from(error.clone()).into()),
                };
                if !valid {
                    return Err(TimedActionError::InvalidRules);
                }
            }
        }
        Ok(())
    }
}

fn live_parts(
    game: &crate::GameState,
    subject: SubjectId,
) -> Result<BTreeSet<PartRef>, crate::GameError> {
    let anatomy = game.current_anatomy(subject)?;
    Ok(anatomy
        .document
        .parts
        .iter()
        .filter(|part| !part.severed)
        .map(|part| PartRef {
            subject: subject.0,
            part: part.id.0,
        })
        .collect())
}
