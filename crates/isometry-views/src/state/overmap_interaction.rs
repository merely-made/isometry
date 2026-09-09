//! Atlas-specific local label motion and its interaction receipts.

use super::*;

impl UiState {
    /// Apply normalized controls to the shared, renderer-neutral motion state.
    pub fn sync_overmap_motion_settings(&mut self) {
        let mut settings = self.overmap_motion.settings();
        settings.strength = (self.overmap_return_strength.value * 36.0).clamp(0.0, 36.0);
        // This is velocity retention, so label it as momentum rather than
        // inviting the opposite intuition. Cap it below one so every finite
        // strength return has a bounded settling path.
        settings.damping = self.overmap_return_damping.value.clamp(0.0, 0.995);
        self.overmap_motion.set_settings(settings);
    }

    pub fn toggle_overmap_reduced_motion(&mut self) {
        self.overmap_reduced_motion = !self.overmap_reduced_motion;
    }

    /// Pin or release the currently selected local label placement.
    pub fn toggle_overmap_motion_pin(&mut self) {
        let Some(id) = self.overmap_motion_selected.clone() else {
            return;
        };
        if self.overmap_motion.mode(&id) == scenotime::ReturnMotionMode::Pinned {
            self.overmap_motion.unpin(&id);
        } else {
            self.overmap_motion.pin(id);
        }
    }

    /// Handle captured Swatch gestures without changing geographic source data.
    /// Returns whether the gesture crossed drag slop, so the atlas route can
    /// translate a quiet release into its normal travel activation.
    pub fn drag_overmap_node(&mut self, event: cambium::GraphCanvasNodeDrag<String>) -> bool {
        const SLOP: f32 = 0.015;
        let atlas = crate::overmap::overmap_atlas(self).is_some();
        match event.phase {
            cambium::PointerPhase::Down => {
                self.overmap_drag_start = Some((event.id.clone(), event.position));
                self.overmap_drag_grab_offset = None;
                self.overmap_dragged_node = None;
                if atlas {
                    self.overmap_motion_selected = Some(event.id.clone());
                    self.sync_overmap_motion_settings();
                    let home = crate::overmap::overmap_home_positions(self)
                        .get(&event.id)
                        .copied()
                        .unwrap_or((0.5, 0.5));
                    let visual = self.overmap_motion.project(&event.id, home);
                    self.overmap_drag_grab_offset =
                        Some((visual.0 - event.position.0, visual.1 - event.position.1));
                    self.overmap_motion.begin(event.id);
                }
                false
            },
            cambium::PointerPhase::Move | cambium::PointerPhase::Up => {
                let Some((started_id, start)) = self.overmap_drag_start.clone() else {
                    return false;
                };
                if started_id != event.id {
                    return false;
                }
                let dx = event.position.0 - start.0;
                let dy = event.position.1 - start.1;
                let dragged = self.overmap_dragged_node.is_some() || dx.hypot(dy) >= SLOP;
                if dragged {
                    self.overmap_dragged_node = Some(event.id.clone());
                    self.overmap_motion_selected = Some(event.id.clone());
                    if atlas {
                        let home = crate::overmap::overmap_home_positions(self)
                            .get(&event.id)
                            .copied()
                            .unwrap_or((0.5, 0.5));
                        let grab = self.overmap_drag_grab_offset.unwrap_or((0.0, 0.0));
                        self.overmap_motion.move_to(
                            &event.id,
                            (
                                event.position.0 + grab.0 - home.0,
                                event.position.1 + grab.1 - home.1,
                            ),
                        );
                    } else {
                        self.overmap_position_overrides
                            .insert(event.id.clone(), event.position);
                    }
                }
                if matches!(event.phase, cambium::PointerPhase::Up) {
                    if atlas {
                        self.overmap_motion.end(&event.id);
                    }
                    self.overmap_drag_start = None;
                    self.overmap_drag_grab_offset = None;
                    if atlas {
                        self.overmap_dragged_node = None;
                    }
                }
                dragged
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_campaign::{CampaignMap, MapScale, WorldPlace};

    fn atlas_ui() -> UiState {
        let mut ui = UiState::new(isometry_core::MapDocument::new("region", 2, 2));
        ui.campaign_maps.insert(
            "region".to_owned(),
            CampaignMap {
                id: "region".to_owned(),
                scale: MapScale::Region,
                document: isometry_core::MapDocument::new("region", 2, 2),
                spawn_zones: Vec::new(),
                transitions: Vec::new(),
                encounter_anchors: Vec::new(),
            },
        );
        ui.world.places.insert(
            "watchtower".to_owned(),
            WorldPlace {
                id: "watchtower".to_owned(),
                name: "Watchtower".to_owned(),
                tags: Vec::new(),
                map: Some("region".to_owned()),
                position: Some((0, 0)),
            },
        );
        ui.world.reveal("dm", "watchtower");
        ui.world
            .apply(&isometry_campaign::WorldEvent::PartyMoved {
                party: "dm".to_owned(),
                node: "watchtower".to_owned(),
            })
            .unwrap();
        ui
    }

    fn drag(
        id: &str,
        phase: cambium::PointerPhase,
        position: (f32, f32),
    ) -> cambium::GraphCanvasNodeDrag<String> {
        cambium::GraphCanvasNodeDrag {
            id: id.to_owned(),
            phase,
            position,
        }
    }

    #[test]
    fn atlas_drag_keeps_geography_home_and_preserves_grab_delta() {
        let mut ui = atlas_ui();
        let home = crate::overmap::overmap_home_positions(&ui)["watchtower"];
        let source = ui.world.places["watchtower"].position;
        ui.overmap_motion.begin("watchtower");
        ui.overmap_motion.move_to("watchtower", (0.1, 0.0));
        ui.overmap_motion.end("watchtower");
        let down = (home.0 + 0.05, home.1);
        ui.drag_overmap_node(drag("watchtower", cambium::PointerPhase::Down, down));
        ui.drag_overmap_node(drag(
            "watchtower",
            cambium::PointerPhase::Move,
            (down.0 + 0.08, down.1),
        ));
        ui.drag_overmap_node(drag(
            "watchtower",
            cambium::PointerPhase::Up,
            (down.0 + 0.08, down.1),
        ));
        assert_eq!(ui.world.places["watchtower"].position, source);
        assert!((ui.overmap_motion.offset("watchtower").0 - 0.18).abs() < 1e-5);
    }

    #[test]
    fn atlas_quiet_release_is_available_for_travel_activation() {
        let mut ui = atlas_ui();
        let home = crate::overmap::overmap_home_positions(&ui)["watchtower"];
        assert!(!ui.drag_overmap_node(drag("watchtower", cambium::PointerPhase::Down, home)));
        assert!(!ui.drag_overmap_node(drag("watchtower", cambium::PointerPhase::Up, home)));
        ui.activate_overmap_node("watchtower".to_owned());
        assert_eq!(ui.overmap_travel_request.as_deref(), Some("watchtower"));
    }

    #[test]
    fn selected_label_can_pin_and_uses_the_configured_return_settings() {
        let mut ui = atlas_ui();
        ui.overmap_return_strength.value = 0.25;
        ui.overmap_return_damping.value = 0.6;
        ui.sync_overmap_motion_settings();
        assert_eq!(ui.overmap_motion.settings().strength, 9.0);
        assert_eq!(ui.overmap_motion.settings().damping, 0.6);
        ui.overmap_motion_selected = Some("watchtower".to_owned());
        ui.overmap_motion.begin("watchtower");
        ui.overmap_motion.move_to("watchtower", (0.2, 0.0));
        ui.overmap_motion.end("watchtower");
        ui.toggle_overmap_motion_pin();
        assert_eq!(
            ui.overmap_motion.mode("watchtower"),
            scenotime::ReturnMotionMode::Pinned
        );
        ui.toggle_overmap_motion_pin();
        assert_eq!(
            ui.overmap_motion.mode("watchtower"),
            scenotime::ReturnMotionMode::Free
        );
    }
}
