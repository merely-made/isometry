//! Local return motion for overmap labels and callouts.
//!
//! The campaign's overmap coordinates and geographic footprints are source
//! authority. This state only remembers a normalized *visual offset* keyed by
//! the projected item's stable id. The renderer therefore obtains a placement
//! by adding [`OvermapMotionState::project`] to its already-derived home point;
//! it never writes the campaign position back through this module.

use std::collections::BTreeMap;

use scenotime::{ReturnMotion, ReturnMotionMode, ReturnMotionSettings};

/// Result of advancing all live overmap label offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OvermapMotionTick {
    /// At least one local visual offset changed, so a deferred leaf refresh is useful.
    pub changed: bool,
    /// At least one gesture or return still needs another host frame.
    pub active: bool,
}

/// Product adapter around shared, kernel-neutral return motion.
///
/// Coordinates passed to [`Self::project`] remain supplied by the campaign's
/// projection. Only the returned value differs while a label is free or pinned.
#[derive(Debug, Default)]
pub struct OvermapMotionState {
    settings: ReturnMotionSettings,
    items: BTreeMap<String, ReturnMotion>,
}

impl OvermapMotionState {
    pub fn new(settings: ReturnMotionSettings) -> Self {
        Self {
            settings,
            items: BTreeMap::new(),
        }
    }

    pub const fn settings(&self) -> ReturnMotionSettings {
        self.settings
    }

    /// Update the user-controlled return tuning. Existing local offsets retain
    /// their position, so changing a slider never moves campaign geography.
    pub fn set_settings(&mut self, settings: ReturnMotionSettings) {
        self.settings = settings;
        for motion in self.items.values_mut() {
            motion.set_settings(settings);
        }
    }

    pub fn begin(&mut self, id: impl Into<String>) -> bool {
        self.motion_mut(id.into()).begin()
    }

    /// Set a normalized offset from the item's immutable home placement.
    pub fn move_to(&mut self, id: impl AsRef<str>, offset: (f32, f32)) -> bool {
        self.items
            .get_mut(id.as_ref())
            .is_some_and(|motion| motion.move_to(offset))
    }

    pub fn end(&mut self, id: impl AsRef<str>) -> bool {
        self.items
            .get_mut(id.as_ref())
            .is_some_and(ReturnMotion::end)
    }

    pub fn pin(&mut self, id: impl Into<String>) -> bool {
        self.motion_mut(id.into()).pin()
    }

    pub fn unpin(&mut self, id: impl AsRef<str>) -> bool {
        self.items
            .get_mut(id.as_ref())
            .is_some_and(ReturnMotion::unpin)
    }

    pub fn mode(&self, id: impl AsRef<str>) -> ReturnMotionMode {
        self.items
            .get(id.as_ref())
            .map_or(ReturnMotionMode::Home, ReturnMotion::mode)
    }

    pub fn offset(&self, id: impl AsRef<str>) -> (f32, f32) {
        self.items
            .get(id.as_ref())
            .map_or((0.0, 0.0), ReturnMotion::offset)
    }

    /// Add only the local visual offset to a source-derived home placement.
    pub fn project(&self, id: impl AsRef<str>, home: (f32, f32)) -> (f32, f32) {
        let offset = self.offset(id);
        (home.0 + offset.0, home.1 + offset.1)
    }

    /// Advance all items with host-supplied elapsed seconds. `active` drives a
    /// deferred leaf refresh and must not trigger a full overmap projection rebuild.
    pub fn tick(&mut self, dt: f32, reduced_motion: bool) -> OvermapMotionTick {
        self.items
            .values_mut()
            .fold(OvermapMotionTick::default(), |mut result, motion| {
                let tick = motion.tick(dt, reduced_motion);
                result.changed |= tick.changed;
                result.active |= tick.active;
                result
            })
    }

    /// Drop identities absent from the current projection after a full source
    /// refresh. Pinned placement is UI-local, never campaign persistence.
    pub fn retain(&mut self, mut present: impl FnMut(&str) -> bool) {
        self.items.retain(|id, _| present(id));
    }

    fn motion_mut(&mut self, id: String) -> &mut ReturnMotion {
        self.items
            .entry(id)
            .or_insert_with(|| ReturnMotion::new(self.settings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drag_changes_only_the_returned_visual_placement() {
        let mut state = OvermapMotionState::default();
        let home = (0.25, 0.75);
        assert!(state.begin("watchtower"));
        assert!(state.move_to("watchtower", (0.1, -0.2)));
        assert_eq!(home, (0.25, 0.75), "home remains source-derived input");
        assert_eq!(state.project("watchtower", home), (0.35, 0.55));
    }

    #[test]
    fn pinned_label_holds_and_unpinned_label_returns() {
        let mut state = OvermapMotionState::default();
        state.begin("watchtower");
        state.move_to("watchtower", (0.4, 0.0));
        state.end("watchtower");
        assert!(state.pin("watchtower"));
        assert!(!state.tick(1.0, false).active);
        assert_eq!(state.mode("watchtower"), ReturnMotionMode::Pinned);
        assert!(state.unpin("watchtower"));
        assert!(state.tick(1.0 / 60.0, false).active);
    }

    #[test]
    fn reduced_motion_settles_all_items_and_stops_frames() {
        let mut state = OvermapMotionState::default();
        state.begin("watchtower");
        state.move_to("watchtower", (0.4, 0.0));
        state.end("watchtower");
        assert_eq!(
            state.tick(1.0 / 60.0, true),
            OvermapMotionTick {
                changed: true,
                active: false
            }
        );
        assert_eq!(state.mode("watchtower"), ReturnMotionMode::Home);
        assert_eq!(state.offset("watchtower"), (0.0, 0.0));
    }
}
