// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Choosing a discovered development through the shared body-preview panel.

use super::Host;
use mesocosm_core::{Intent, Outcome};
use mesocosm_views::{BodyMenu, BodyMenuRow, MAX_BODY_MENU_ROWS};

impl Host {
    pub(super) fn refresh_expression(&mut self, notice: &str) {
        let world = self.runtime.world();
        let state = &mut self.grafting;
        state.hash = self.runtime.state_hash();
        state.sources.clear();
        state.conditions = world.discoveries().iter().map(|d| d.condition).collect();
        state.selected = state.selected.min(state.conditions.len().saturating_sub(1));
        state.preview = None;
        state.root = None;
        state.admissible = false;
        let page = state.selected / MAX_BODY_MENU_ROWS * MAX_BODY_MENU_ROWS;
        let mut rows: Vec<_> = world
            .discoveries()
            .iter()
            .skip(page)
            .take(MAX_BODY_MENU_ROWS)
            .map(|discovery| {
                let words = mesocosm_views::vitals::discovery_words(discovery);
                BodyMenuRow {
                    source: words.what,
                    offer: words.grants,
                    mass: words.route,
                    ..BodyMenuRow::default()
                }
            })
            .collect();
        let mut detail = "No discoveries yet. Live through the world's conditions to learn what this body could express.".to_owned();
        let mut status = "World paused while this menu is open.".to_owned();
        if let Some(&condition) = state.conditions.get(state.selected) {
            match world.preview_expression(condition) {
                Ok(preview) => {
                    let row = &mut rows[state.selected - page];
                    row.cost = format!("{} mg reserve", preview.cost_mg);
                    row.attachment = format!(
                        "your part {}; revision {}",
                        preview.part.0, preview.revision
                    );
                    let process = world.discoveries()[state.selected].candidate.process;
                    let before = tissue_mg(
                        &world.controlled().unwrap().phenotype,
                        preview.part,
                        process,
                    );
                    let after = tissue_mg(&preview.phenotype, preview.part, process);
                    detail = format!(
                        "Tissue performing this process: {before} -> {after} mg. Surface markings show each process’s tissue share; magenta marks secretion. Amber highlights the part. Your lineage program is unchanged."
                    );
                    state.root = Some(preview.part);
                    let mut projected = world.clone();
                    projected
                        .organisms
                        .iter_mut()
                        .find(|o| o.id == preview.recipient)
                        .expect("checked recipient exists")
                        .phenotype = preview.phenotype;
                    state.preview = Some(Box::new(projected));
                    state.admissible = true;
                    status = "Preview: not applied. Enter expresses this discovery; Esc leaves your body unchanged.".into();
                },
                Err(reason) => {
                    let words = mesocosm_views::refusal_words(&reason);
                    rows[state.selected - page].refusal = Some(words.into());
                    detail = format!(
                        "Cannot express this discovery: {words}. Your lineage program stays unchanged."
                    );
                    status =
                        "Nothing applied. Select a discovery to inspect its requirements.".into();
                },
            }
        }
        if !notice.is_empty() {
            status = notice.into();
        }
        state.reading = BodyMenu::new(
            vec![
                (
                    "available".into(),
                    format!(
                        "{} discoveries; page {}",
                        state.conditions.len(),
                        page / MAX_BODY_MENU_ROWS + 1
                    ),
                ),
                (
                    "your reserve".into(),
                    world
                        .controlled()
                        .map_or("no body".into(), |o| format!("{} mg", o.energy_mg)),
                ),
            ],
            rows,
            state.selected - page,
            detail,
            "current body only",
            status,
        );
        state.reading.headline = "Express discovery".into();
        state.reading.facts_title = "the development".into();
        state.reading.choice_label = "applies to".into();
        state.reading.keys =
            "[O] open  [arrows, J/L] select  [Enter] express  [Esc] cancel  [Z/V] turn".into();
    }

    pub(super) fn confirm_expression(&mut self) {
        if self.runtime.state_hash() != self.grafting.hash || self.runtime.queued_len() != 0 {
            self.refresh_expression(
                "The world changed. Review the refreshed preview before confirming.",
            );
            return;
        }
        if !self.grafting.admissible {
            return;
        }
        let Some(&condition) = self.grafting.conditions.get(self.grafting.selected) else {
            return;
        };
        self.runtime.queue(Intent::Express { condition });
        self.steps += self.runtime.step(1);
        self.note_outcomes();
        let (part, message) = match self.runtime.last_outcomes().first() {
            Some(Outcome::Expressed {
                part,
                cost_mg,
                revision,
            }) => (
                Some(*part),
                format!(
                    "Expressed on part {} for {cost_mg} mg; revision {revision}. Your lineage program is unchanged. Esc returns to play.",
                    part.0
                ),
            ),
            Some(Outcome::Rejected(reason)) => (
                None,
                format!("Not applied: {}.", mesocosm_views::refusal_words(reason)),
            ),
            _ => (
                None,
                "The world is holding at a checkpoint. Esc returns to its question.".into(),
            ),
        };
        self.refresh_expression(&message);
        self.grafting.preview = None;
        self.grafting.admissible = false;
        self.grafting.root = part;
        self.grafting.reading.keys = self
            .grafting
            .reading
            .keys
            .replace("[Esc] cancel", "[Esc] return");
        for row in &mut self.grafting.reading.rows {
            row.cost.clear();
            row.attachment.clear();
            row.refusal = None;
        }
        self.grafting.reading.detail = format!(
            "Showing your actual body: {} mg of tissue performing this process. Select a discovery to inspect another development.",
            self.expression_tissue(false).unwrap_or(0)
        );
    }

    pub(super) fn expression_tissue(&self, candidate: bool) -> Option<u64> {
        if self.grafting.operation != super::grafting::BodyOperation::Express {
            return None;
        }
        let part = self.grafting.root?;
        let condition = self.grafting.conditions.get(self.grafting.selected)?;
        let process = self
            .runtime
            .world()
            .discoveries()
            .iter()
            .find(|d| d.condition == *condition)?
            .candidate
            .process;
        let world = if candidate {
            self.grafting.preview.as_deref()?
        } else {
            self.runtime.world()
        };
        Some(tissue_mg(&world.controlled()?.phenotype, part, process))
    }
}

fn tissue_mg(
    phenotype: &mesocosm_core::BodyPhenotype,
    part: mesocosm_core::PartId,
    process: mesocosm_core::process::ProcessRef,
) -> u64 {
    let Some(mosaic) = phenotype.mosaic(part) else {
        return 0;
    };
    let cells: u64 = mosaic
        .sites()
        .iter()
        .filter(|site| site.process == process)
        .map(|site| {
            site.cells
                .iter()
                .filter(|cell| mosaic.is_living(**cell))
                .count() as u64
        })
        .sum();
    cells * phenotype.cell_mg(part)
}

#[cfg(test)]
mod tests;
