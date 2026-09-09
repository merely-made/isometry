// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Keyboard inspection of addressed parts from the last rendered body list.
//! Selection and focus never enter the runtime queue or saved world.

use winit::keyboard::Key;

use super::Host;
use crate::section::BodySelection;

#[derive(Default)]
pub(super) struct Inspection {
    pub open: bool,
    pub selected: Option<BodySelection>,
    pub notice: &'static str,
}

impl Host {
    pub(super) fn try_inspection_key(&mut self, key: &Key) -> bool {
        if !self.config.dev {
            return false;
        }
        let letter = match key {
            Key::Character(value) => value.to_lowercase(),
            _ => String::new(),
        };
        if letter == "i" {
            self.inspection.open = !self.inspection.open;
            self.inspection.selected = None;
            if self.inspection.open {
                self.select_part(false);
            }
            return true;
        }
        if !self.inspection.open {
            return false;
        }
        match letter.as_str() {
            "j" => self.select_part(true),
            "l" => self.select_part(false),
            "u" => {
                self.inspection.selected = None;
                self.inspection.notice = "Selection cleared. J/L selects a drawn part.";
            },
            // Time and follow are deliberate inspector commands. All other
            // keys are consumed while the panel has focus, including play,
            // checkpoint answers and the world-changing dev commands.
            "p" | "." | "," | "[" | "]" | "n" | "b" | "m" => return false,
            _ => {},
        }
        true
    }

    fn select_part(&mut self, backwards: bool) {
        self.update_inspection();
        let selected = self.followed().and_then(|id| {
            let gpu = self.gpu.as_mut()?;
            let selection = gpu
                .section
                .select_part(id, self.inspection.selected, backwards)?;
            gpu.section
                .validate_selection(selection, self.runtime.world(), &self.volumes)
                .then_some(selection)
        });
        self.inspection.selected = selected;
        self.inspection.notice = if selected.is_some() {
            ""
        } else {
            "No exact drawn part available. J/L retries after a frame."
        };
    }

    pub(super) fn update_inspection(&mut self) {
        let Some(selected) = self.inspection.selected else {
            return;
        };
        let valid = self.followed() == Some(selected.organism)
            && self.gpu.as_mut().is_some_and(|gpu| {
                gpu.section
                    .validate_selection(selected, self.runtime.world(), &self.volumes)
            });
        if !valid {
            self.inspection.selected = None;
            self.inspection.notice = "Selection expired: body or view changed. J/L selects again.";
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostConfig;

    #[test]
    fn inspection_focus_consumes_world_commands_without_changing_authority() {
        let mut host = Host::new(HostConfig {
            organisms: 8,
            dev: true,
            ..HostConfig::default()
        });
        let hash = host.runtime.state_hash();
        assert!(host.run_action("i"));
        assert!(host.inspection.open);
        assert!(
            host.inspection.selected.is_none(),
            "headless has no drawn parts"
        );
        for key in [
            "w", "e", "c", "q", "x", "f", "k", "g", "enter", "r", "j", "l", "u",
        ] {
            assert!(host.run_action(key));
        }
        assert_eq!(host.runtime.queued_len(), 0);
        assert_eq!(host.runtime.state_hash(), hash);
        assert!(host.runtime.trace().is_empty());
        assert!(host.run_action("i"));
        assert!(!host.inspection.open);
        assert!(host.run_action("w"));
        assert_eq!(host.runtime.queued_len(), 1);
    }

    #[test]
    fn inspection_is_dev_only_and_follow_keeps_control() {
        let mut host = Host::new(HostConfig {
            organisms: 8,
            ..HostConfig::default()
        });
        assert!(host.run_action("i"));
        assert!(!host.inspection.open);
        host.config.dev = true;
        let controlled = host.runtime.world().controlled_id();
        assert!(host.run_action("i"));
        assert!(host.run_action("n"));
        assert_ne!(host.followed(), controlled);
        assert_eq!(host.runtime.world().controlled_id(), controlled);
        assert_eq!(host.runtime.queued_len(), 0);
    }
}
