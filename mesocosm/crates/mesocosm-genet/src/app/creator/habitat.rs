// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::Creator;
use mesocosm_core::world::generation::{FixedBody, SoilPattern, VERSION};

impl Creator {
    pub(super) fn habitat_key(&mut self, key: &str) -> bool {
        if self.request.fixed_body.is_some()
            && matches!(key, "b" | "k" | "u" | "r" | "c" | "m" | "+" | "=" | "-")
        {
            self.notice = "H releases the body before changing its criteria.".into();
            self.refresh();
            return true;
        }
        match key {
            "h" => {
                if self.request.fixed_body.is_some() {
                    self.request.fixed_body = None;
                    self.habitat_view = false;
                    self.comparison.clear();
                    self.compare_view = false;
                } else {
                    let Some(c) = self
                        .prepared
                        .as_ref()
                        .and_then(|p| p.draft().candidates.get(self.selected))
                    else {
                        return true;
                    };
                    self.request.fixed_body = Some(FixedBody {
                        seed: c.seed,
                        role: c.role,
                    });
                    self.habitat_view = true;
                    self.comparison.clear();
                }
            },
            "f" => {
                self.request.soil_pattern = match self.request.soil_pattern {
                    SoilPattern::Patches => SoilPattern::Uniform,
                    SoilPattern::Uniform => SoilPattern::Contrasting,
                    SoilPattern::Contrasting => SoilPattern::Patches,
                }
            },
            "t" => self.request.min_open_steps = (self.request.min_open_steps + 1) % 5,
            "," => self.request.organisms = self.request.organisms.saturating_sub(3).max(3),
            "." => self.request.organisms = (self.request.organisms + 3).min(120),
            "[" => {
                self.request.soil_min_mg = (self.request.soil_min_mg / 2).max(1);
                self.request.soil_max_mg = (self.request.soil_max_mg / 2).max(1);
            },
            "]" => {
                self.request.soil_min_mg = (self.request.soil_min_mg * 2).min(10_000);
                self.request.soil_max_mg = (self.request.soil_max_mg * 2).min(10_000);
            },
            _ => return false,
        }
        self.request.version = VERSION;
        self.regenerate();
        true
    }
}
