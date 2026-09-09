// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{FunctionalNetwork, NetworkError, NodeId, NodeKind, PartRef};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum Operator {
    Strengthen,
    Project,
    Store,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldRules {
    pub allowed_operators: BTreeSet<Operator>,
    pub allowed_costs: BTreeSet<u64>,
    pub max_range: u32,
    pub max_hops: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct EvaluationRequest {
    pub operator: Operator,
    pub target: NodeId,
    pub cost: u64,
    pub range: u32,
    pub max_hops: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SupplyDebit {
    pub source: NodeId,
    pub amount: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum EffectReport {
    Strengthened {
        effect: PartRef,
        amount: u64,
    },
    Projected {
        effect: PartRef,
        amount: u64,
        range: u32,
    },
    Stored {
        store: PartRef,
        amount: u64,
        charge_after: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Evaluation {
    pub operator: Operator,
    pub target: NodeId,
    pub cost: u64,
    pub supplied_by: Vec<SupplyDebit>,
    pub effect: EffectReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluationError {
    InvalidNetwork(NetworkError),
    ZeroCost,
    ChainTooLong {
        actual: usize,
        maximum: usize,
    },
    OperatorForbidden(Operator),
    CostForbidden(u64),
    RangeExceeded {
        requested: u32,
        maximum: u32,
    },
    HopsExceeded {
        requested: u32,
        maximum: u32,
    },
    UnknownTarget(NodeId),
    WrongTarget {
        operator: Operator,
        target: NodeId,
    },
    MissingPart(PartRef),
    ClosedGate {
        node: NodeId,
        part: PartRef,
    },
    InsufficientCharge {
        needed: u64,
        available: u64,
    },
    StoreOverflow {
        node: NodeId,
        capacity: u64,
        charge: u64,
        incoming: u64,
    },
    ArithmeticOverflow,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNetwork(error) => write!(f, "invalid network: {error}"),
            Self::ZeroCost => write!(f, "zero-cost operations are unavailable"),
            Self::ChainTooLong { actual, maximum } => {
                write!(f, "chain has {actual} operations; maximum is {maximum}")
            },
            Self::OperatorForbidden(op) => {
                write!(f, "operator {op:?} is unavailable in this world")
            },
            Self::CostForbidden(cost) => write!(f, "cost {cost} is unavailable in this world"),
            Self::RangeExceeded { requested, maximum } => {
                write!(f, "range {requested} exceeds world maximum {maximum}")
            },
            Self::HopsExceeded { requested, maximum } => {
                write!(f, "hop bound {requested} exceeds world maximum {maximum}")
            },
            Self::UnknownTarget(id) => write!(f, "target node {} does not exist", id.0),
            Self::WrongTarget { operator, target } => {
                write!(f, "{operator:?} cannot target node {}", target.0)
            },
            Self::MissingPart(part) => {
                write!(f, "body part {}:{} is unavailable", part.subject, part.part)
            },
            Self::ClosedGate { node, part } => write!(
                f,
                "gate {} on body part {}:{} is closed",
                node.0, part.subject, part.part
            ),
            Self::InsufficientCharge { needed, available } => write!(
                f,
                "need {needed} charge but selected routes deliver only {available}"
            ),
            Self::StoreOverflow {
                node,
                capacity,
                charge,
                incoming,
            } => write!(
                f,
                "store {} cannot add {incoming} to {charge}/{capacity}",
                node.0
            ),
            Self::ArithmeticOverflow => write!(f, "charge arithmetic overflow"),
        }
    }
}
impl std::error::Error for EvaluationError {}

pub const MAX_CHAIN: usize = 32;

impl FunctionalNetwork {
    pub fn preview(
        &self,
        request: &EvaluationRequest,
        live_parts: &BTreeSet<PartRef>,
        rules: &WorldRules,
    ) -> Result<Evaluation, EvaluationError> {
        let mut clone = self.clone();
        clone.evaluate(request, live_parts, rules)
    }

    pub fn evaluate(
        &mut self,
        request: &EvaluationRequest,
        live_parts: &BTreeSet<PartRef>,
        rules: &WorldRules,
    ) -> Result<Evaluation, EvaluationError> {
        let mut clone = self.clone();
        let result = clone.evaluate_inner(request, live_parts, rules)?;
        *self = clone;
        Ok(result)
    }

    pub fn evaluate_chain(
        &mut self,
        requests: &[EvaluationRequest],
        live_parts: &BTreeSet<PartRef>,
        rules: &WorldRules,
    ) -> Result<Vec<Evaluation>, EvaluationError> {
        self.validate().map_err(EvaluationError::InvalidNetwork)?;
        if requests.len() > MAX_CHAIN {
            return Err(EvaluationError::ChainTooLong {
                actual: requests.len(),
                maximum: MAX_CHAIN,
            });
        }
        let mut clone = self.clone();
        let mut results = Vec::with_capacity(requests.len());
        for request in requests {
            results.push(clone.evaluate_inner(request, live_parts, rules)?);
        }
        *self = clone;
        Ok(results)
    }

    fn evaluate_inner(
        &mut self,
        request: &EvaluationRequest,
        live: &BTreeSet<PartRef>,
        rules: &WorldRules,
    ) -> Result<Evaluation, EvaluationError> {
        self.validate().map_err(EvaluationError::InvalidNetwork)?;
        if request.cost == 0 {
            return Err(EvaluationError::ZeroCost);
        }
        if !rules.allowed_operators.contains(&request.operator) {
            return Err(EvaluationError::OperatorForbidden(request.operator));
        }
        if !rules.allowed_costs.contains(&request.cost) {
            return Err(EvaluationError::CostForbidden(request.cost));
        }
        if request.range > rules.max_range {
            return Err(EvaluationError::RangeExceeded {
                requested: request.range,
                maximum: rules.max_range,
            });
        }
        if request.max_hops > rules.max_hops {
            return Err(EvaluationError::HopsExceeded {
                requested: request.max_hops,
                maximum: rules.max_hops,
            });
        }
        let target = self
            .nodes
            .get(&request.target)
            .ok_or(EvaluationError::UnknownTarget(request.target))?;
        if !live.contains(&target.kind.part()) {
            return Err(EvaluationError::MissingPart(target.kind.part()));
        }
        let valid = matches!(
            (&request.operator, &target.kind),
            (Operator::Store, NodeKind::Store { .. })
                | (Operator::Strengthen, NodeKind::Effect { .. })
                | (Operator::Project, NodeKind::Effect { .. })
        );
        if !valid {
            return Err(EvaluationError::WrongTarget {
                operator: request.operator,
                target: request.target,
            });
        }
        if let NodeKind::Store {
            capacity, charge, ..
        } = target.kind
        {
            if charge
                .checked_add(request.cost)
                .ok_or(EvaluationError::ArithmeticOverflow)?
                > capacity
            {
                return Err(EvaluationError::StoreOverflow {
                    node: request.target,
                    capacity,
                    charge,
                    incoming: request.cost,
                });
            }
        }
        let supplies = self.route_charge(request.target, request.cost, request.max_hops, live)?;
        if let NodeKind::Store { charge, .. } = &mut self
            .nodes
            .get_mut(&request.target)
            .expect("validated target")
            .kind
        {
            *charge = charge
                .checked_add(request.cost)
                .ok_or(EvaluationError::ArithmeticOverflow)?;
        }
        let part = self
            .nodes
            .get(&request.target)
            .expect("validated target")
            .kind
            .part();
        let effect = match request.operator {
            Operator::Strengthen => EffectReport::Strengthened {
                effect: part,
                amount: request.cost,
            },
            Operator::Project => EffectReport::Projected {
                effect: part,
                amount: request.cost,
                range: request.range,
            },
            Operator::Store => match self
                .nodes
                .get(&request.target)
                .expect("validated target")
                .kind
            {
                NodeKind::Store { charge, .. } => EffectReport::Stored {
                    store: part,
                    amount: request.cost,
                    charge_after: charge,
                },
                _ => unreachable!(),
            },
        };
        Ok(Evaluation {
            operator: request.operator,
            target: request.target,
            cost: request.cost,
            supplied_by: supplies,
            effect,
        })
    }

    fn route_charge(
        &mut self,
        target: NodeId,
        mut needed: u64,
        hops: u32,
        live: &BTreeSet<PartRef>,
    ) -> Result<Vec<SupplyDebit>, EvaluationError> {
        let mut used = vec![0_u64; self.edges.len()];
        let mut debits = Vec::new();
        while needed > 0 {
            let Some((source, path)) = self.find_path(target, hops, live, &used) else {
                break;
            };
            let source_charge = match self.nodes[&source].kind {
                NodeKind::Source { charge, .. } | NodeKind::Store { charge, .. } => charge,
                _ => unreachable!(),
            };
            let edge_room = path
                .iter()
                .try_fold(u64::MAX, |room, edge| {
                    self.edges[*edge]
                        .capacity
                        .checked_sub(used[*edge])
                        .map(|left| room.min(left))
                })
                .ok_or(EvaluationError::ArithmeticOverflow)?;
            let amount = needed.min(source_charge).min(edge_room);
            if amount == 0 {
                break;
            }
            for edge in path {
                used[edge] = used[edge]
                    .checked_add(amount)
                    .ok_or(EvaluationError::ArithmeticOverflow)?;
            }
            match &mut self.nodes.get_mut(&source).expect("path source").kind {
                NodeKind::Source { charge, .. } | NodeKind::Store { charge, .. } => {
                    *charge -= amount
                },
                _ => unreachable!(),
            }
            needed -= amount;
            if let Some(last) = debits
                .last_mut()
                .filter(|debit: &&mut SupplyDebit| debit.source == source)
            {
                last.amount = last
                    .amount
                    .checked_add(amount)
                    .ok_or(EvaluationError::ArithmeticOverflow)?;
            } else {
                debits.push(SupplyDebit { source, amount });
            }
        }
        if needed == 0 {
            return Ok(debits);
        }
        if debits.is_empty() {
            if let Some(error) = self.first_blocker(target, hops, live) {
                return Err(error);
            }
        }
        let available = debits
            .iter()
            .try_fold(0_u64, |total, debit| total.checked_add(debit.amount))
            .ok_or(EvaluationError::ArithmeticOverflow)?;
        Err(EvaluationError::InsufficientCharge {
            needed: needed
                .checked_add(available)
                .ok_or(EvaluationError::ArithmeticOverflow)?,
            available,
        })
    }

    fn find_path(
        &self,
        target: NodeId,
        hops: u32,
        live: &BTreeSet<PartRef>,
        used: &[u64],
    ) -> Option<(NodeId, Vec<usize>)> {
        let mut queue = VecDeque::new();
        let mut seen = BTreeSet::new();
        for (&id, node) in &self.nodes {
            let charged_supply = matches!(node.kind, NodeKind::Source { charge, .. } | NodeKind::Store { charge, .. } if charge > 0);
            if id != target && charged_supply && live.contains(&node.kind.part()) {
                queue.push_back((id, Vec::new()));
                seen.insert(id);
            }
        }
        while let Some((node, path)) = queue.pop_front() {
            if node == target && !path.is_empty() {
                let source = self.edge_source(path[0]);
                return Some((source, path));
            }
            if path.len() >= hops as usize {
                continue;
            }
            for (edge_index, edge) in self.outgoing(node) {
                if used[edge_index] >= edge.capacity || seen.contains(&edge.to) {
                    continue;
                }
                let next = &self.nodes[&edge.to];
                if !live.contains(&next.kind.part()) {
                    continue;
                }
                if matches!(next.kind, NodeKind::Gate { open: false, .. }) {
                    continue;
                }
                let mut next_path = path.clone();
                next_path.push(edge_index);
                seen.insert(edge.to);
                queue.push_back((edge.to, next_path));
            }
        }
        None
    }

    fn first_blocker(
        &self,
        target: NodeId,
        hops: u32,
        live: &BTreeSet<PartRef>,
    ) -> Option<EvaluationError> {
        let mut queue: VecDeque<(NodeId, u32)> = self
            .nodes
            .iter()
            .filter_map(|(&id, node)| {
                (matches!(node.kind, NodeKind::Source { .. } | NodeKind::Store { .. })
                    && self.can_reach_target(id, target, hops))
                .then_some((id, 0))
            })
            .collect();
        let mut seen = BTreeSet::new();
        while let Some((node, depth)) = queue.pop_front() {
            if !seen.insert(node) || depth >= hops {
                continue;
            }
            let current = &self.nodes[&node];
            if !live.contains(&current.kind.part()) {
                return Some(EvaluationError::MissingPart(current.kind.part()));
            }
            for (_, edge) in self.outgoing(node) {
                let next = &self.nodes[&edge.to];
                if !self.can_reach_target(edge.to, target, hops - depth - 1) {
                    continue;
                }
                if !live.contains(&next.kind.part()) {
                    return Some(EvaluationError::MissingPart(next.kind.part()));
                }
                if let NodeKind::Gate { part, open: false } = next.kind {
                    return Some(EvaluationError::ClosedGate {
                        node: edge.to,
                        part,
                    });
                }
                queue.push_back((edge.to, depth + 1));
            }
        }
        None
    }

    fn can_reach_target(&self, from: NodeId, target: NodeId, hops: u32) -> bool {
        let mut queue = VecDeque::from([(from, 0_u32)]);
        let mut seen = BTreeSet::new();
        while let Some((node, depth)) = queue.pop_front() {
            if node == target {
                return true;
            }
            if !seen.insert(node) || depth >= hops {
                continue;
            }
            for (_, edge) in self.outgoing(node) {
                queue.push_back((edge.to, depth + 1));
            }
        }
        false
    }

    fn edge_source(&self, edge: usize) -> NodeId {
        self.edges[edge].from
    }
}
