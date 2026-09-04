//! The swatch the view and the host both read.
//!
//! One `GraphCanvasSwatch` describing the party's discovered overmap: node
//! identity, order, and layout. The view builds its hit targets from it and
//! the host paints its retained leaf from it, so the two project through one
//! identical layout.
//!
//! Split out of `overmap.rs` on 2026-09-04; unchanged.

use super::*;

/// Build the graph-canvas swatch for the party's discovered overmap. Both the
/// view (native hit targets + the `custom_leaf`) and the host (the registered
/// `paint_leaf`) call this, so they agree on node identity, order, and layout.
/// Returns `None` when the party has discovered nothing (no leaf to paint).
pub fn overmap_swatch(ui: &UiState) -> Option<GraphCanvasSwatch<String, OvermapNodeKind>> {
    // Whose party? The viewer's; the DM (no viewer) watches the "dm" party.
    let party = ui.viewer.as_deref().unwrap_or("dm");
    // Only what the party has discovered (E6): the unfound map is not drawn.
    let world = ui.overmap_source_world();
    let overmap = world.overmap_for(party);
    if overmap.nodes.is_empty() {
        return None;
    }
    let here = world.party_at(party).map(str::to_owned);

    // Shared score -> scene realization; the final viewport fit is local to the
    // Cambium swatch and does not alter campaign or scene data.
    let placed = overmap_positions(&overmap);
    let nodes: Vec<GraphCanvasNode<String, OvermapNodeKind>> = overmap
        .nodes
        .iter()
        .map(|node| {
            let position = ui
                .overmap_position_overrides
                .get(&node.id)
                .copied()
                .or_else(|| placed.get(&node.id).copied())
                .unwrap_or((0.5, 0.5));
            let kind = if here.as_deref() == Some(node.id.as_str()) {
                OvermapNodeKind::Here
            } else {
                OvermapNodeKind::Elsewhere
            };
            GraphCanvasNode {
                id: node.id.clone(),
                kind,
                position,
                label: node.name.clone(),
                key: Some(node.id.clone()),
            }
        })
        .collect();
    let relations: Vec<GraphCanvasRelation<String>> = overmap
        .edges
        .iter()
        .enumerate()
        .map(|(index, edge)| {
            let id = overmap_relation_id(index, edge);
            let from = overmap
                .node(&edge.from)
                .map(|node| node.name.clone())
                .unwrap_or_else(|| edge.from.clone());
            let to = overmap
                .node(&edge.to)
                .map(|node| node.name.clone())
                .unwrap_or_else(|| edge.to.clone());
            GraphCanvasRelation {
                id: id.clone(),
                from: edge.from.clone(),
                to: edge.to.clone(),
                kind: if edge.directed {
                    "One-way route".to_owned()
                } else {
                    "Route".to_owned()
                },
                label: format!("{from} to {to}, travel weight {}", edge.weight),
                route: Vec::new(),
                visible: !ui.overmap_hidden_relations.contains(&id),
                emphasized: ui.overmap_selected_relation.as_deref() == Some(id.as_str())
                    || ui.overmap_hovered_relation.as_deref() == Some(id.as_str()),
            }
        })
        .collect();

    let mut swatch = GraphCanvasSwatch::new(
        OVERMAP_LEAF_KEY,
        GraphCanvasSubgraph {
            nodes,
            edges: Vec::new(),
        },
    )
    .with_relations(relations)
    .with_size(OVERMAP_CANVAS.0, OVERMAP_CANVAS.1)
    .with_label("Overmap")
    // The overmap *is* the full view; there is nothing to expand into. The
    // chip used to render and get hidden with `display: none`, which left a
    // focusable, screen-reader-announced control that did nothing.
    .with_expand(false)
    // A pointcrawl is unreadable as unnamed dots, and since cambium 0.3.2 the
    // visible labels carry each node's `selected` / `hovered` state, so "here"
    // can be emphasized without a parallel hand-rolled label layer.
    .with_node_labels(true);
    // A larger ring for a full-panel canvas than the card default.
    swatch.node_radius = 6.0;
    swatch.edge_width = 1.5;
    // The party's place reads as the selected node (its emphasis ring); the node
    // under the pointer reads as hovered.
    swatch.selected = here;
    swatch.hovered = ui.overmap_hover.clone();
    Some(swatch)
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_core::OvermapNode;

    fn node(id: &str, at: (i32, i32)) -> OvermapNode {
        OvermapNode {
            id: id.to_owned(),
            name: id.to_owned(),
            at,
            site: None,
        }
    }

    #[test]
    fn authored_overmap_uses_geographic_score_and_viewport_fit() {
        let mut overmap = Overmap::new("shore");
        overmap.nodes = vec![node("west", (0, 4)), node("east", (10, 0))];
        let score = overmap_score(&overmap);
        assert!(matches!(score.arrangement, Arrangement::Geographic(_)));
        let positions = overmap_positions(&overmap);
        assert_eq!(positions["west"], (0.08, 0.92));
        assert_eq!(positions["east"], (0.92, 0.08));
    }

    #[test]
    fn uniform_overmap_uses_portable_spiral_and_product_free_types() {
        let mut overmap = Overmap::new("unplaced");
        overmap.nodes = vec![node("a", (0, 0)), node("b", (0, 0)), node("c", (0, 0))];
        let score = overmap_score(&overmap);
        assert!(matches!(score.arrangement, Arrangement::Spiral(_)));
        let scene = scenomise::solve(&score);
        assert!(std::any::type_name::<Score>().starts_with("sceno::"));
        assert!(std::any::type_name::<sceno::Scene>().starts_with("sceno::"));
        let score_json = serde_json::to_string(&score).unwrap();
        let scene_json = serde_json::to_string(&scene).unwrap();
        assert_eq!(serde_json::from_str::<Score>(&score_json).unwrap(), score);
        assert_eq!(
            serde_json::from_str::<sceno::Scene>(&scene_json).unwrap(),
            scene
        );
        assert_eq!(overmap_positions(&overmap).len(), 3);
    }

    /// The Expand chip is off at the model, not hidden at the sheet. The old
    /// `display: none` left a real button in the tree: tabbable, announced as
    /// "Expand graph", and wired to a closure that did nothing. Switching it
    /// back on without also wiring a full-canvas route re-creates that, and CSS
    /// would no longer hide the evidence.
    #[test]
    fn overmap_renders_no_expand_affordance() {
        let swatch = overmap_swatch(&discovered_one_place()).expect("a discovered place draws");
        assert!(
            !swatch.show_expand,
            "the overmap has no fuller view to expand into; wire the route before re-enabling"
        );
    }

    /// The names come from the component, not a parallel layer. Isometry drew its
    /// own `overmap-label*` layer until cambium 0.3.2 taught visible labels the
    /// node's state; turning `show_labels` back off would render an unnamed
    /// pointcrawl rather than fall back to anything, since the hand-rolled layer
    /// is deleted. Cambium's own suite pins that the labels carry `selected` /
    /// `hovered`, which is the emphasis "here" needs.
    #[test]
    fn the_component_draws_the_place_names() {
        let ui = discovered_one_place();
        let swatch = overmap_swatch(&ui).expect("a discovered place draws");
        assert!(
            swatch.show_labels,
            "the pointcrawl reads as unnamed dots without the component's labels"
        );
        assert_eq!(
            swatch.graph.nodes[0].label, "Here",
            "the label the component renders is the place's name"
        );
        assert_eq!(
            swatch.selected.as_deref(),
            Some("here"),
            "the party's place is the selected node, which is what emphasizes its label"
        );
    }

    #[test]
    fn overmap_drag_is_local_curation_and_consumes_its_release_click() {
        let mut ui = discovered_one_place();
        let source_position = ui.world.places["here"].position.clone();

        ui.drag_overmap_node(cambium::GraphCanvasNodeDrag {
            id: "here".to_owned(),
            phase: cambium::PointerPhase::Down,
            position: (0.5, 0.5),
        });
        ui.drag_overmap_node(cambium::GraphCanvasNodeDrag {
            id: "here".to_owned(),
            phase: cambium::PointerPhase::Move,
            position: (0.78, 0.22),
        });
        ui.drag_overmap_node(cambium::GraphCanvasNodeDrag {
            id: "here".to_owned(),
            phase: cambium::PointerPhase::Up,
            position: (0.78, 0.22),
        });

        let swatch = overmap_swatch(&ui).expect("a discovered place draws");
        assert_eq!(swatch.graph.nodes[0].position, (0.78, 0.22));
        assert_eq!(
            ui.world.places["here"].position, source_position,
            "pulling a Swatch node does not edit campaign geography"
        );

        ui.activate_overmap_node("here".to_owned());
        assert!(
            ui.overmap_travel_request.is_none(),
            "the click following a drag is consumed rather than becoming travel"
        );
        ui.activate_overmap_node("here".to_owned());
        assert_eq!(ui.overmap_travel_request.as_deref(), Some("here"));
    }

    #[test]
    fn parallel_route_cells_remain_independently_selectable_and_hideable() {
        let first = OvermapEdge {
            from: "here".to_owned(),
            to: "there".to_owned(),
            weight: 1,
            directed: false,
        };
        let second = OvermapEdge {
            weight: 3,
            ..first.clone()
        };
        let first_id = overmap_relation_id(0, &first);
        let second_id = overmap_relation_id(1, &second);
        assert_ne!(first_id, second_id, "parallel routes are distinct cells");

        let mut ui = discovered_one_place();
        ui.select_overmap_relation(first_id.clone());
        ui.toggle_overmap_relation_visibility(first_id.clone());
        assert_eq!(
            ui.overmap_selected_relation.as_deref(),
            Some(first_id.as_str())
        );
        assert!(ui.overmap_hidden_relations.contains(&first_id));
        assert!(
            !ui.overmap_hidden_relations.contains(&second_id),
            "hiding one relation cell never hides its parallel sibling"
        );
        ui.toggle_overmap_relation_visibility(first_id.clone());
        assert!(
            !ui.overmap_hidden_relations.contains(&first_id),
            "show restores only the local view cell"
        );
    }

    fn discovered_one_place() -> UiState {
        let mut ui = UiState::new(isometry_core::MapDocument::new("t", 2, 2));
        ui.world.places.insert(
            "here".to_owned(),
            isometry_campaign::WorldPlace {
                id: "here".to_owned(),
                name: "Here".to_owned(),
                tags: Vec::new(),
                map: None,
                position: None,
            },
        );
        ui.world.reveal("dm", "here");
        ui.world
            .apply(&isometry_campaign::WorldEvent::PartyMoved {
                party: "dm".to_owned(),
                node: "here".to_owned(),
            })
            .expect("the party stands on a discovered place");
        ui
    }
}
