// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Product-free, deterministic charge routing over consumer-supplied body parts.
//! Each operation uses first-fit, breadth-first routes in `NodeId` order; it is
//! deliberately bounded, not a maximum-flow solver. Edge capacity resets for
//! each operation, including sequential operations in an atomic chain.
//! The crate records functional receipts; callers own body state and adjudication.

pub mod generation;
mod network;
mod operators;

pub use generation::*;
pub use network::{
    Edge, FunctionalNetwork, MAX_EDGES, MAX_NODES, NETWORK_SCHEMA_VERSION, NetworkError,
    NetworkSnapshot, Node, NodeId, NodeKind, PartRef,
};
pub use operators::{
    EffectReport, Evaluation, EvaluationError, EvaluationRequest, MAX_CHAIN, Operator, SupplyDebit,
    WorldRules,
};

#[cfg(test)]
mod tests;
