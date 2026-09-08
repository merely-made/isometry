// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::Creator;
use mesocosm_views::{BodyMenu, BodyMenuRow, MAX_BODY_MENU_ROWS};

impl Creator {
    pub(super) fn refresh(&mut self) {
        let page = self.selected / MAX_BODY_MENU_ROWS;
        let draft = self.prepared.as_ref().map(|p| p.draft());
        let rows = draft
            .map(|d| {
                d.candidates
                    .iter()
                    .enumerate()
                    .skip(page * MAX_BODY_MENU_ROWS)
                    .take(MAX_BODY_MENU_ROWS)
                    .map(|(i, c)| BodyMenuRow {
                        source: format!("{}", i + 1),
                        offer: format!("{:?}, {} parts", c.role, c.parts),
                        mass: format!("{} mg", c.body.total_mass_mg()),
                        cost: format!("{} segments", c.segments),
                        attachment: format!(
                            "movement organs: {}",
                            if c.actuator_span > 0 { "yes" } else { "no" }
                        ),
                        refusal: None,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let soil = draft
            .map(|d| {
                format!(
                    "{} mg/column",
                    d.habitat[self.request.place as usize].soil_mg_per_column
                )
            })
            .unwrap_or_else(|| "pending".into());
        let role = self
            .request
            .criteria
            .role
            .map_or("any".into(), |r| format!("{r:?}"));
        let organs = match self.request.criteria.movement_organs {
            None => "any",
            Some(true) => "yes",
            Some(false) => "no",
        };
        let status: String = if self.pending {
            "Generating candidates…".into()
        } else {
            let report = draft.map_or(String::new(), |d| {
                let reasons = d
                    .rejected
                    .iter()
                    .map(|(why, n)| format!("{n} {why}"))
                    .collect::<Vec<_>>()
                    .join("; ");
                format!(
                    "{} candidates / {} attempts. {}",
                    d.candidates.len(),
                    d.attempted,
                    reasons
                )
            });
            format!("{report} {}", self.notice).trim().into()
        };
        self.reading = BodyMenu::new(
            vec![
                (
                    "habitat".into(),
                    format!(
                        "seed {}, place {} · {soil}",
                        self.request.seed, self.request.place
                    ),
                ),
                (
                    "body".into(),
                    format!(
                        "variation {} · {role} · organs {organs} · {} mg · segments {}..{}",
                        self.request.variation,
                        self.request.criteria.mass_mg,
                        self.request.criteria.min_segments,
                        self.request.criteria.max_segments
                    ),
                ),
            ],
            rows,
            self.selected % MAX_BODY_MENU_ROWS,
            if self.count() > 0 {
                "K keeps feeding role, movement organs and segment count. R varies the rest."
            } else {
                "Adjust the criteria or habitat; requested traits stay fixed during generation."
            },
            if self.pending {
                "Waiting for candidates."
            } else if self.count() == 0 {
                "Change the criteria or habitat to find a starting life."
            } else {
                "Enter begins this life; Escape cancels."
            },
            status,
        );
        self.reading.headline = "Character creator".into();
        self.reading.facts_title = "Habitat and body".into();
        self.reading.choice_label = "Start".into();
        self.reading.keys = "↑/↓ choose · K keep traits · U clear traits · R vary · N habitat · P place · C role · M organs · +/- mass · Z/V rotate".into();
        if self.draft_path.is_some() {
            self.reading.keys.push_str(" · S save criteria");
        }
        if !self.pending && self.count() > 0 {
            self.reading.keys.push_str(" · Enter begin");
        }
        self.reading.keys.push_str(" · Esc cancel");
    }
}
