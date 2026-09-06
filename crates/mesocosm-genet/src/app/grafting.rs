// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Host-owned menu focus and a disposable projection of core's graft candidate.

use mesocosm_core::{Crossing, Intent, OrganismId, Outcome, PartId, World};
use mesocosm_views::{GraftMenu, GraftRow, MAX_GRAFT_ROWS};
use winit::keyboard::{Key, NamedKey};

use super::Host;

pub(super) struct Grafting {
    pub open: bool,
    pub selected: usize,
    pub sources: Vec<(OrganismId, PartId)>,
    pub crossing: Crossing,
    pub reading: GraftMenu,
    pub preview: Option<Box<World>>,
    pub root: Option<PartId>,
    pub admissible: bool,
    hash: u64,
}

impl Default for Grafting {
    fn default() -> Self {
        Self {
            open: false,
            selected: 0,
            sources: Vec::new(),
            crossing: Crossing::Carry,
            reading: GraftMenu::default(),
            preview: None,
            root: None,
            admissible: false,
            hash: 0,
        }
    }
}

impl Host {
    pub(super) fn try_graft_key(&mut self, key: &Key) -> bool {
        let letter = match key {
            Key::Character(c) => c.to_lowercase(),
            _ => String::new(),
        };
        if !self.grafting.open {
            if letter != "h" || self.inspection.open {
                return false;
            }
            if self.config.replay.is_some()
                || self.runtime.checkpoint().is_some()
                || self.runtime.queued_len() != 0
                || self.pump.is_some()
            {
                return true;
            }
            self.grafting.open = true;
            self.grafting.selected = 0;
            self.grafting.crossing = Crossing::Carry;
            self.last = None;
            self.refresh_graft("");
            return true;
        }
        match key {
            Key::Named(NamedKey::Escape) => {
                self.grafting = Grafting::default();
                self.last = None;
            },
            Key::Named(NamedKey::ArrowUp | NamedKey::ArrowLeft) => self.move_graft(true),
            Key::Named(NamedKey::ArrowDown | NamedKey::ArrowRight) => self.move_graft(false),
            Key::Named(NamedKey::Tab) => {
                self.grafting.crossing = match self.grafting.crossing {
                    Crossing::Carry => Crossing::Regrow,
                    Crossing::Regrow => Crossing::Carry,
                };
                self.refresh_graft("");
            },
            Key::Named(NamedKey::Enter) => self.confirm_graft(),
            _ if letter == "j" => self.move_graft(true),
            _ if letter == "l" => self.move_graft(false),
            // Rotating the observation remains a view-only operation.
            _ if letter == "z" || letter == "v" => return false,
            _ => {},
        }
        true
    }

    fn move_graft(&mut self, backwards: bool) {
        let count = self.grafting.sources.len();
        if count > 0 {
            self.grafting.selected =
                (self.grafting.selected + if backwards { count - 1 } else { 1 }) % count;
        }
        self.refresh_graft("");
    }

    fn refresh_graft(&mut self, notice: &str) {
        let world = self.runtime.world();
        let state = &mut self.grafting;
        state.hash = self.runtime.state_hash();
        state.sources = world.graft_sources();
        state.selected = state.selected.min(state.sources.len().saturating_sub(1));
        state.preview = None;
        state.root = None;
        state.admissible = false;
        let page = state.selected / MAX_GRAFT_ROWS * MAX_GRAFT_ROWS;
        let mut rows = Vec::new();
        for &(donor, part) in state.sources.iter().skip(page).take(MAX_GRAFT_ROWS) {
            let organism = world
                .organisms
                .iter()
                .find(|o| o.id == donor)
                .expect("source exists");
            let branch = organism
                .phenotype
                .harvest(part)
                .expect("source holds tissue");
            rows.push(GraftRow {
                donor: format!("carcass {} (line {})", donor.0, organism.species.0),
                tissue: format!("part {} + branch ({} parts)", part.0, branch.len()),
                mass: format!("{} mg", branch.mass_mg()),
                ..GraftRow::default()
            });
        }
        let mut detail =
            "No reachable carcass branches. Approach a carcass, then press H.".to_owned();
        let mut status = "World paused while this menu is open.".to_owned();
        if let Some(&(donor, part)) = state.sources.get(state.selected) {
            match world.preview_graft(donor, part, state.crossing) {
                Ok(preview) => {
                    let slot = &mut rows[state.selected - page];
                    slot.cost = format!("{} mg reserve", preview.cost_mg);
                    slot.attachment = format!(
                        "onto your part {} at {:?} body voxels",
                        preview.attachment.parent.0, preview.attachment.offset
                    );
                    let terms = match (state.crossing, preview.verdict) {
                        (Crossing::Carry, mesocosm_core::graft::Verdict::Adapter) => {
                            "Tissue arrives inactive; an adapter is required."
                        },
                        (Crossing::Carry, _) => "Donor arrangement retained.",
                        (Crossing::Regrow, _) => "Your body's rules arrange this tissue.",
                    };
                    detail = format!(
                        "{} parts, {} mg tissue. {terms} Amber marks the new root. Diet after graft: {}.",
                        preview.parts.len(),
                        preview.mass_mg,
                        mesocosm_views::dev::part::feeding_word(mesocosm_core::FeedingMode::of(
                            &preview.phenotype
                        ))
                    );
                    state.root = Some(preview.root);
                    let mut projected = world.clone();
                    projected
                        .organisms
                        .iter_mut()
                        .find(|o| o.id == preview.recipient)
                        .expect("recipient exists")
                        .phenotype = preview.phenotype;
                    state.preview = Some(Box::new(projected));
                    state.admissible = true;
                    status = "Preview: not applied. Enter grafts this branch; Esc leaves both bodies unchanged.".into();
                },
                Err(refusal) => {
                    let reason = mesocosm_views::refusal_words(&refusal);
                    rows[state.selected - page].refusal = Some(reason.into());
                    detail = format!("Cannot graft this branch: {reason}.");
                    status =
                        "Nothing applied. Choose another branch or try regrowing its arrangement."
                            .into();
                },
            }
        }
        if !notice.is_empty() {
            status = notice.into();
        }
        let choice = match state.crossing {
            Crossing::Carry => "Carry donor tissue (compatibility shown above)",
            Crossing::Regrow => "Regrow arrangement under your world's rules",
        };
        state.reading = GraftMenu::new(
            vec![
                (
                    "available".into(),
                    format!(
                        "{} branches; page {}",
                        state.sources.len(),
                        page / MAX_GRAFT_ROWS + 1
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
            choice,
            status,
        );
    }

    fn confirm_graft(&mut self) {
        if self.runtime.state_hash() != self.grafting.hash || self.runtime.queued_len() != 0 {
            self.refresh_graft(
                "The world changed. Review the refreshed preview before confirming.",
            );
            return;
        }
        if !self.grafting.admissible {
            return;
        }
        let Some(&(organism, part)) = self.grafting.sources.get(self.grafting.selected) else {
            return;
        };
        self.runtime.queue(Intent::Graft {
            organism,
            part,
            crossing: self.grafting.crossing,
        });
        self.steps += self.runtime.step(1);
        self.note_outcomes();
        let outcome = self.runtime.last_outcomes().first().cloned();
        let applied_root = match &outcome {
            Some(Outcome::Grafted { root, .. }) => Some(*root),
            _ => None,
        };
        let message = match outcome {
            Some(Outcome::Grafted { root, parts, .. }) => format!(
                "Grafted {parts} parts. New root: part {}. Esc returns to play.",
                root.0
            ),
            Some(Outcome::Rejected(reason)) => format!(
                "Not applied: {}. Review before retrying.",
                mesocosm_views::refusal_words(&reason)
            ),
            _ => "The world is holding at a checkpoint. Esc returns to its question.".into(),
        };
        self.refresh_graft(&message);
        // Confirmation shows the actual result, not the next speculative graft.
        self.grafting.preview = None;
        self.grafting.admissible = false;
        self.grafting.root = applied_root;
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
        self.grafting.reading.detail =
            "Showing your actual body. Selecting a branch starts a new preview.".into();
    }
}

/// Fit the candidate beside the panel using the same camera basis as terrain
/// and body rendering. This temporary framing never changes saved camera settings.
pub(super) fn framing(
    world: &World,
    frame: (u32, u32),
    mode: crate::section::CameraMode,
    pitch: Option<f32>,
    scale: f32,
) -> Option<([f32; 3], f32)> {
    let organism = world.controlled()?;
    let bounds = organism.body().aabb();
    let [right, up, _] = crate::section::camera_basis(mode, pitch);
    let size: [f32; 3] = [0, 1, 2].map(|i| (bounds.max[i] - bounds.min[i]) as f32 * scale);
    let span = |axis: [f32; 3]| (0..3).map(|i| size[i] * axis[i].abs()).sum::<f32>();
    let available = (frame.0 as f32 - mesocosm_views::GRAFT_WIDTH as f32 - 24.0).max(80.0);
    let half = (span(up).max(span(right) * frame.1 as f32 / available) * 0.75).max(2.0);
    let mut centre = [0, 1, 2].map(|i| {
        organism.position[i] as f32 + (bounds.min[i] + bounds.max[i]) as f32 * 0.5 * scale
    });
    if pitch.is_some() {
        centre[1] -= bounds.min[1] as f32 * scale;
    }
    let offset = half * (frame.0 as f32 - available) / frame.1.max(1) as f32;
    for i in 0..3 {
        centre[i] += right[i] * offset;
    }
    Some((centre, half))
}

pub(super) fn selection(
    section: &crate::section::Section,
    world: &World,
    root: Option<PartId>,
) -> Option<crate::section::BodySelection> {
    let organism = world.controlled()?;
    let root = root?;
    let mut selected = None;
    for _ in 0..organism.body().len() {
        selected = section.select_part(organism.id, selected, false);
        if selected.is_some_and(|part| part.part == root) {
            return selected;
        }
    }
    None
}

#[cfg(test)]
mod tests;
