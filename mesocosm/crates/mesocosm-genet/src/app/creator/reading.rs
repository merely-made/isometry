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
                        "{} · variation {} · {role} · organs {organs} · {} mg · segments {}..{}",
                        self.request.criteria.body_plan.label(),
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
        if self.request.fixed_body.is_some() {
            self.reading = BodyMenu::new(
                vec![
                    (
                        "habitat".into(),
                        format!(
                            "seed {} / place {} / {soil}",
                            self.request.seed, self.request.place
                        ),
                    ),
                    (
                        "nutrients".into(),
                        format!(
                            "{}: {}..{} mg/column",
                            self.request.soil_pattern.label(),
                            self.request.soil_min_mg,
                            self.request.soil_max_mg
                        ),
                    ),
                    (
                        "population".into(),
                        format!(
                            "{} founders; require {} terrain directions",
                            self.request.organisms, self.request.min_open_steps
                        ),
                    ),
                    (
                        "body".into(),
                        format!(
                            "held seed {}; {} mg",
                            self.request.fixed_body.as_ref().unwrap().seed,
                            self.request.criteria.mass_mg
                        ),
                    ),
                    (
                        "access".into(),
                        draft.and_then(|d| d.candidates.first()).map_or(
                            if self.pending {
                                "pending"
                            } else {
                                "start refused"
                            }
                            .into(),
                            |c| format!("{} successful directional steps", c.open_steps),
                        ),
                    ),
                    (
                        "trial".into(),
                        self.observation
                            .as_ref()
                            .map_or("pending or refused".into(), |o| o.evidence.summary()),
                    ),
                    (
                        "window".into(),
                        format!("{} ticks; O changes it", self.trial_ticks),
                    ),
                    (
                        "food".into(),
                        self.observation
                            .as_ref()
                            .map_or("pending or refused".into(), |o| {
                                format!(
                                    "feeding {}mg / uptake {}mg / incoming {}mg",
                                    o.evidence.subject_feeding_mg,
                                    o.evidence.subject_uptake_mg,
                                    o.evidence.subject_incoming_mg,
                                )
                            }),
                    ),
                    (
                        "reserve".into(),
                        self.observation
                            .as_ref()
                            .map_or("pending or refused".into(), |o| {
                                format!(
                                    "{} -> {} mg; alive {}",
                                    o.evidence.subject_reserve_before_mg,
                                    o.evidence.subject_reserve_after_mg,
                                    o.evidence.subject_alive,
                                )
                            }),
                    ),
                ],
                Vec::new(),
                0,
                "Trial runs on a copy. Diet compatibility does not prove food is reachable.",
                if self.pending {
                    "Waiting for the habitat and trial."
                } else if self.count() > 0 {
                    "Enter starts at tick zero. A short trial does not establish persistence."
                } else {
                    "Start refused. Change the habitat to admit this body."
                },
                self.reading.status.clone(),
            );
            if self.compare_view {
                let rows: Vec<BodyMenuRow> = self
                    .comparison
                    .trials()
                    .iter()
                    .map(|trial| BodyMenuRow {
                        source: format!("{}:{}", trial.key.seed, trial.key.place),
                        offer: format!(
                            "{} · {:?} soil {}..{} · {} founders · access {}",
                            trial.observation.evidence.summary(),
                            trial.key.soil_pattern,
                            trial.key.soil_min_mg,
                            trial.key.soil_max_mg,
                            trial.key.organisms,
                            trial.key.min_open_steps,
                        ),
                        mass: format!("{} ticks", trial.key.ticks),
                        cost: format!(
                            "reserve {}→{} mg; +{} / -{}",
                            trial.observation.evidence.subject_reserve_before_mg,
                            trial.observation.evidence.subject_reserve_after_mg,
                            trial.observation.evidence.births,
                            trial.observation.evidence.deaths,
                        ),
                        attachment: format!(
                            "{} soil, {}..{} mg, {} founders",
                            trial.key.soil_pattern.label(),
                            trial.key.soil_min_mg,
                            trial.key.soil_max_mg,
                            trial.key.organisms
                        ),
                        refusal: None,
                    })
                    .collect();
                self.reading.facts = vec![
                    (
                        "held body".into(),
                        self.request.fixed_body.as_ref().map_or_else(
                            || "none".into(),
                            |body| format!("{} / {:?}", body.seed, body.role),
                        ),
                    ),
                    ("trials".into(), format!("{} / 3 recorded", rows.len())),
                ];
                self.reading.rows = rows;
                let current =
                    super::comparison::HabitatKey::from_request(&self.request, self.trial_ticks);
                self.reading.selected = self
                    .comparison
                    .trials()
                    .iter()
                    .position(|trial| current.as_ref() == Some(&trial.key))
                    .unwrap_or(MAX_BODY_MENU_ROWS);
                self.reading.detail =
                    "Completed trials for this held body. Evidence is from the disposable copy; identical requests replace their row.".into();
                self.reading.choice = if self.pending {
                    "waiting for the current habitat".into()
                } else if self.count() == 0 {
                    "current habitat refused; history is for comparison".into()
                } else {
                    format!(
                        "Enter begins current seed {}, place {} at tick zero",
                        self.request.seed, self.request.place
                    )
                };
            }
        }
        self.reading.headline = "Character creator".into();
        self.reading.facts_title = "Habitat and body".into();
        self.reading.choice_label = "Start".into();
        self.reading.keys = "↑/↓ choose · B body plan · K keep · U clear · R vary · N habitat · P place · C role · M organs · +/- mass · Z/V change view".into();
        self.reading.keys.push_str(" / H compare habitats");
        if self.request.fixed_body.is_some() {
            self.reading.keys = format!(
                "H release body  /  D {}  /  O trial length  /  N seed  /  P place  /  F nutrients  /  [/] amount  /  ,/. population  /  T access  /  Z/V view",
                if self.compare_view {
                    "current trial"
                } else {
                    "comparison"
                }
            );
        }
        if self.draft_path.is_some() {
            self.reading.keys.push_str(" · S save criteria");
        }
        if !self.pending && self.count() > 0 {
            self.reading.keys.push_str(" · Enter begin");
        }
        self.reading.keys.push_str(" · Esc cancel");
    }
}
