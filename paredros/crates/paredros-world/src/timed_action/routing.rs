// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

pub(super) fn route_open(
    network: &FunctionalNetwork,
    target: NodeId,
    live: &BTreeSet<PartRef>,
    max_hops: u32,
) -> bool {
    let mut queue: std::collections::VecDeque<(NodeId, u32)> = network
        .nodes
        .iter()
        .filter_map(|(&id, node)| {
            matches!(node.kind, NodeKind::Source { .. } | NodeKind::Store { .. }).then_some((id, 0))
        })
        .collect();
    let mut seen = BTreeSet::new();
    while let Some((id, hops)) = queue.pop_front() {
        if !seen.insert(id) {
            continue;
        }
        let Some(node) = network.nodes.get(&id) else {
            continue;
        };
        if !live.contains(&node.kind.part())
            || matches!(node.kind, NodeKind::Gate { open: false, .. })
        {
            continue;
        }
        if id == target {
            return true;
        }
        if hops == max_hops {
            continue;
        }
        queue.extend(
            network
                .edges
                .iter()
                .filter(|edge| edge.from == id)
                .map(|edge| (edge.to, hops + 1)),
        );
    }
    false
}
