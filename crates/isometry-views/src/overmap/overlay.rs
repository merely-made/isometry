//! The surface itself: the drawn overlay and the controls around it.
//!
//! Native hit targets over the painted leaf, plus the pace, stance, and
//! source-time chrome. Every control here only *asks*: the host rolls the
//! navigation, spends the time, and moves the party.
//!
//! Split out of `overmap.rs` on 2026-09-04; unchanged.

use super::*;

pub fn overmap_overlay(ui: &UiState) -> Option<UiChild> {
    if !ui.overmap_open {
        return None;
    }
    let party = ui.viewer.as_deref().unwrap_or("dm");
    let world = ui.overmap_source_world();
    let overmap = world.overmap_for(party);
    let here = world.party_at(party).map(str::to_owned);
    let historical = ui.overmap_is_historical();

    let mut actions: Vec<UiChild> = Vec::new();
    if historical {
        actions.push(Box::new(clickable(
            el::<_, UiState, ()>("span", text("return live")).attr("class", "btn btn-mini"),
            |ui: &mut UiState, _| ui.return_overmap_to_live(),
        )));
    } else {
        actions.push(Box::new(clickable(
            el::<_, UiState, ()>("span", text("study map")).attr("class", "btn btn-mini"),
            |ui: &mut UiState, _| ui.request_map_read(),
        )));
    }
    actions.push(Box::new(clickable(
        el::<_, UiState, ()>("span", text("close")).attr("class", "btn btn-mini"),
        |ui: &mut UiState, _| ui.close_overmap(),
    )));

    let mut body: Vec<UiChild> = Vec::new();

    if let Some(label) = ui.overmap_source_time_label() {
        if ui.overmap_source_time_available() {
            body.push(Box::new(
                el(
                    "section",
                    (
                        el("div", text(label)).attr("class", "source-time-label"),
                        lens(
                            |state: &mut Slider| slider(state),
                            |ui: &mut UiState| &mut ui.overmap_source_slider,
                        ),
                    ),
                )
                .attr("class", "source-time"),
            ));
        }
    }

    // The painted graph, when there is one; else the "find your way" hint.
    match overmap_swatch(ui) {
        Some(swatch) => {
            // The swatch draws its own named nodes (`with_node_labels`), so the
            // place names ride the same projection as the painted dots by
            // construction rather than by a second layer agreeing with the first.
            let canvas: UiChild = if swatch.atlas.is_some() {
                Box::new(graph_atlas_swatch(
                    &swatch,
                    |ui: &mut UiState, event| match event {
                        GraphAtlasEvent::Hover(id) => ui.hover_overmap(id),
                        GraphAtlasEvent::Focus(Some(id)) => ui.overmap_motion_selected = Some(id),
                        GraphAtlasEvent::Focus(None) => {},
                        GraphAtlasEvent::Drag(drag) => {
                            let is_up = matches!(drag.phase, cambium::PointerPhase::Up);
                            let dragged = ui.drag_overmap_node(drag.clone());
                            if is_up && !dragged && !ui.overmap_is_historical() {
                                ui.activate_overmap_node(drag.id);
                            }
                        },
                        GraphAtlasEvent::Activate(id) => {
                            if !ui.overmap_is_historical() {
                                ui.activate_overmap_node(id);
                            }
                        },
                    },
                ))
            } else {
                Box::new(graph_canvas_swatch_with_drag_and_relations(
                    &swatch,
                    // A click asks the host to travel there; the drag path
                    // consumes its own release-click so a pull stays local
                    // curation rather than becoming a trip request.
                    |ui: &mut UiState, id: String| {
                        if !ui.overmap_is_historical() {
                            ui.activate_overmap_node(id);
                        }
                    },
                    // Enter/leave lifts the hovered node on the painted leaf.
                    |ui: &mut UiState, id: Option<String>| ui.hover_overmap(id),
                    // Pulling a site changes only this view's local
                    // placement override. Campaign geography remains the
                    // source of truth until a separate authoring command.
                    |ui: &mut UiState, event| {
                        ui.drag_overmap_node(event);
                    },
                    // Relation cells keep their own stable local identity,
                    // even where two routes share the same endpoints.
                    |ui: &mut UiState, id: String| ui.select_overmap_relation(id),
                    |ui: &mut UiState, id: Option<String>| ui.hover_overmap_relation(id),
                    // Never called: the swatch renders no Expand chip
                    // (`with_expand(false)`).
                    |_ui: &mut UiState| {},
                ))
            };
            body.push(Box::new(el("div", canvas).attr("class", "overmap-graph")));
        },
        None => {
            body.push(Box::new(
                el(
                    "div",
                    text("no map yet — travel or ask around to find your way"),
                )
                .attr("class", "side-hint"),
            ));
            return Some(crate::widgets::overlay_panel(
                "overmap",
                "Overmap".to_owned(),
                actions,
                body,
            ));
        },
    }

    if overmap_atlas(ui).is_some() {
        let selected_motion = ui.overmap_motion_selected.clone();
        let selected_is_pinned = selected_motion
            .as_ref()
            .is_some_and(|id| ui.overmap_motion.mode(id) == scenotime::ReturnMotionMode::Pinned);
        body.push(Box::new(
            el(
                "section",
                (
                    el("div", text("Label return")).attr("class", "source-time-label"),
                    el("label", text("Strength")),
                    lens(
                        |state: &mut Slider| slider(state),
                        |ui: &mut UiState| &mut ui.overmap_return_strength,
                    ),
                    el("label", text("Momentum")),
                    lens(
                        |state: &mut Slider| slider(state),
                        |ui: &mut UiState| &mut ui.overmap_return_damping,
                    ),
                    clickable(
                        el(
                            "button",
                            text(if ui.overmap_reduced_motion {
                                "Reduced motion: on"
                            } else {
                                "Reduced motion: off"
                            }),
                        )
                        .attr("class", "btn btn-mini")
                        .attr("type", "button"),
                        |ui: &mut UiState, _| ui.toggle_overmap_reduced_motion(),
                    ),
                    clickable(
                        el(
                            "button",
                            text(if selected_is_pinned {
                                "Release label"
                            } else {
                                "Pin label"
                            }),
                        )
                        .attr("class", "btn btn-mini")
                        .attr("type", "button")
                        .attr(
                            "aria-disabled",
                            if selected_motion.is_some() {
                                "false"
                            } else {
                                "true"
                            },
                        ),
                        |ui: &mut UiState, _| ui.toggle_overmap_motion_pin(),
                    ),
                ),
            )
            .attr("class", "overmap-motion"),
        ));
    }

    // The routes, listed with weights: painted edges show connectivity, but the
    // costs that drive travel time (weight x pace) still need to be legible.
    let name_of = |id: &str| {
        overmap
            .node(id)
            .map(|n| n.name.clone())
            .unwrap_or_else(|| id.to_owned())
    };
    for (index, edge) in overmap.edges.iter().enumerate() {
        let relation = overmap_relation_id(index, edge);
        let selected = ui.overmap_selected_relation.as_deref() == Some(relation.as_str());
        let hidden = ui.overmap_hidden_relations.contains(&relation);
        let mut class = String::from("side-line overmap-route");
        if selected {
            class.push_str(" selected");
        }
        if hidden {
            class.push_str(" hidden");
        }
        let row_relation = relation.clone();
        body.push(Box::new(clickable(
            el(
                "button",
                text(format!(
                    "{} to {} ({}){}",
                    name_of(&edge.from),
                    name_of(&edge.to),
                    edge.weight,
                    if hidden { " [hidden]" } else { "" },
                )),
            )
            .attr("class", class)
            .attr("type", "button")
            .attr("aria-current", if selected { "true" } else { "false" }),
            move |ui: &mut UiState, _| ui.select_overmap_relation(row_relation.clone()),
        )));
    }

    if let Some((index, edge)) = overmap.edges.iter().enumerate().find(|(index, edge)| {
        let relation = overmap_relation_id(*index, edge);
        ui.overmap_selected_relation.as_deref() == Some(relation.as_str())
    }) {
        let relation = overmap_relation_id(index, edge);
        let hidden = ui.overmap_hidden_relations.contains(&relation);
        let toggle_relation = relation.clone();
        body.push(Box::new(
            el(
                "section",
                (
                    el("strong", text("Route")),
                    el(
                        "div",
                        text(format!(
                            "{} to {} · travel weight {}{}",
                            name_of(&edge.from),
                            name_of(&edge.to),
                            edge.weight,
                            if edge.directed { " · one way" } else { "" },
                        )),
                    ),
                    clickable(
                        el(
                            "button",
                            text(if hidden { "Show route" } else { "Hide route" }),
                        )
                        .attr("class", "btn btn-mini")
                        .attr("type", "button"),
                        move |ui: &mut UiState, _| {
                            ui.toggle_overmap_relation_visibility(toggle_relation.clone())
                        },
                    ),
                ),
            )
            .attr("class", "overmap-link-card"),
        ));
    }

    // Pace (E1) and stance (E3): the party's marching orders. The chosen pace is
    // marked; a stance is set on the lead token by the host.
    // Pace and stance are catalog `segmented_control` rows: one-of-N choice, which
    // is exactly the component's shape. The row writes only its own
    // `SelectionState`; `pump_selection_rows` notices the divergence from the
    // world and dispatches the real request, so a click still travels the
    // ordinary host-adjudicated path.
    if !historical {
        let pace_items = crate::state::pace_items();
        body.push(Box::new(
            el::<_, UiState, ()>(
                "div",
                lens(
                    move |sel: &mut cambium::SelectionState| segmented_control(sel, &pace_items),
                    |ui: &mut UiState| &mut ui.pace_selection,
                ),
            )
            .attr("class", "overmap-controls"),
        ));

        let stance_items = crate::state::stance_items();
        body.push(Box::new(
            el::<_, UiState, ()>(
                "div",
                lens(
                    move |sel: &mut cambium::SelectionState| segmented_control(sel, &stance_items),
                    |ui: &mut UiState| &mut ui.stance_selection,
                ),
            )
            .attr("class", "overmap-controls"),
        ));
    }

    let hint = if historical {
        "historical projection: travel and exploration orders are disabled".to_owned()
    } else {
        match &here {
            Some(node) => format!("here: {} — click a place to travel", name_of(node)),
            None => "the party is not on the overmap yet".to_owned(),
        }
    };
    body.push(Box::new(el("div", text(hint)).attr("class", "side-hint")));

    Some(crate::widgets::overlay_panel(
        "overmap",
        "Overmap".to_owned(),
        actions,
        body,
    ))
}
