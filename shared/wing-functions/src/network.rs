// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PartRef {
    pub subject: u64,
    pub part: u32,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct NodeId(pub u32);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Source {
        part: PartRef,
        capacity: u64,
        charge: u64,
    },
    Store {
        part: PartRef,
        capacity: u64,
        charge: u64,
    },
    Gate {
        part: PartRef,
        open: bool,
    },
    Effect {
        part: PartRef,
    },
}

impl NodeKind {
    pub fn part(&self) -> PartRef {
        match self {
            Self::Source { part, .. }
            | Self::Store { part, .. }
            | Self::Gate { part, .. }
            | Self::Effect { part } => *part,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub capacity: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FunctionalNetwork {
    pub nodes: BTreeMap<NodeId, Node>,
    pub edges: Vec<Edge>,
}

/// Versioned durable envelope. Consumers choose their own storage and migration policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NetworkSnapshot {
    pub schema_version: u16,
    pub network: FunctionalNetwork,
}

pub const NETWORK_SCHEMA_VERSION: u16 = 1;
pub const MAX_NODES: usize = 256;
pub const MAX_EDGES: usize = 1_024;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkError {
    DuplicateNode(NodeId),
    DanglingEdge(Edge),
    InvalidEdgeCapacity(Edge),
    InvalidCharge(NodeId),
    NodeKeyMismatch { key: NodeId, node: NodeId },
    UnsupportedSchema(u16),
    TooManyNodes { actual: usize, maximum: usize },
    TooManyEdges { actual: usize, maximum: usize },
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateNode(id) => write!(f, "duplicate node {}", id.0),
            Self::DanglingEdge(edge) => write!(
                f,
                "edge {} -> {} has a missing endpoint",
                edge.from.0, edge.to.0
            ),
            Self::InvalidEdgeCapacity(edge) => {
                write!(f, "edge {} -> {} has zero capacity", edge.from.0, edge.to.0)
            },
            Self::InvalidCharge(id) => write!(f, "node {} has charge above capacity", id.0),
            Self::NodeKeyMismatch { key, node } => {
                write!(f, "node map key {} disagrees with node {}", key.0, node.0)
            },
            Self::UnsupportedSchema(version) => {
                write!(f, "network schema version {version} is unsupported")
            },
            Self::TooManyNodes { actual, maximum } => {
                write!(f, "network has {actual} nodes; maximum is {maximum}")
            },
            Self::TooManyEdges { actual, maximum } => {
                write!(f, "network has {actual} edges; maximum is {maximum}")
            },
        }
    }
}
impl std::error::Error for NetworkError {}

impl FunctionalNetwork {
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>) -> Result<Self, NetworkError> {
        let mut indexed = BTreeMap::new();
        for node in nodes {
            let id = node.id;
            if indexed.insert(id, node).is_some() {
                return Err(NetworkError::DuplicateNode(id));
            }
        }
        let network = Self {
            nodes: indexed,
            edges,
        };
        network.validate()?;
        Ok(network)
    }

    /// Rechecks persisted or externally deserialized state before use.
    pub fn validate(&self) -> Result<(), NetworkError> {
        if self.nodes.len() > MAX_NODES {
            return Err(NetworkError::TooManyNodes {
                actual: self.nodes.len(),
                maximum: MAX_NODES,
            });
        }
        if self.edges.len() > MAX_EDGES {
            return Err(NetworkError::TooManyEdges {
                actual: self.edges.len(),
                maximum: MAX_EDGES,
            });
        }
        for edge in &self.edges {
            if !self.nodes.contains_key(&edge.from) || !self.nodes.contains_key(&edge.to) {
                return Err(NetworkError::DanglingEdge(*edge));
            }
            if edge.capacity == 0 {
                return Err(NetworkError::InvalidEdgeCapacity(*edge));
            }
        }
        for (&key, node) in &self.nodes {
            if key != node.id {
                return Err(NetworkError::NodeKeyMismatch { key, node: node.id });
            }
            match node.kind {
                NodeKind::Source {
                    capacity, charge, ..
                }
                | NodeKind::Store {
                    capacity, charge, ..
                } if charge > capacity => return Err(NetworkError::InvalidCharge(node.id)),
                _ => {},
            }
        }
        Ok(())
    }

    pub(crate) fn outgoing(&self, from: NodeId) -> impl Iterator<Item = (usize, Edge)> + '_ {
        self.edges
            .iter()
            .copied()
            .enumerate()
            .filter(move |(_, edge)| edge.from == from)
    }
}

impl NetworkSnapshot {
    pub fn new(network: FunctionalNetwork) -> Self {
        Self {
            schema_version: NETWORK_SCHEMA_VERSION,
            network,
        }
    }

    pub fn validate(&self) -> Result<(), NetworkError> {
        if self.schema_version != NETWORK_SCHEMA_VERSION {
            return Err(NetworkError::UnsupportedSchema(self.schema_version));
        }
        self.network.validate()
    }
}
