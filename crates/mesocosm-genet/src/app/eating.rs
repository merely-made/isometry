// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Addressed tissue intake through the ordinary play menu.

use super::Host;
use mesocosm_core::{Intent, Outcome};
use mesocosm_views::{BodyMenu, BodyMenuRow, MAX_BODY_MENU_ROWS};

#[cfg(test)]
mod tests;

impl Host {
    pub(super) fn refresh_eating(&mut self, notice: &str) {
        let world = self.runtime.world();
        let state = &mut self.grafting;
        state.hash = self.runtime.state_hash();
        // Reach and dietary admission are checked again by the real Consume.
        state.sources = world
            .organisms
            .iter()
            .filter(|o| Some(o.id) != world.controlled_id() && world.in_reach(o.position))
            .flat_map(|o| o.body().living().map(move |part| (o.id, part.id)))
            .collect();
        state.selected = state.selected.min(state.sources.len().saturating_sub(1));
        state.preview = None;
        state.root = None;
        state.admissible = false;
        let page = state.selected / MAX_BODY_MENU_ROWS * MAX_BODY_MENU_ROWS;
        let mut rows: Vec<_> = state
            .sources
            .iter()
            .skip(page)
            .take(MAX_BODY_MENU_ROWS)
            .map(|&(id, part)| {
                let donor = world
                    .organisms
                    .iter()
                    .find(|o| o.id == id)
                    .expect("reachable donor");
                let tissue = donor.body().part(part).expect("addressed part");
                BodyMenuRow {
                    source: format!(
                        "{} {} (line {})",
                        if donor.is_alive() {
                            "critter"
                        } else {
                            "carcass"
                        },
                        id.0,
                        donor.species.0
                    ),
                    offer: format!("part {} tissue", part.0),
                    mass: format!("{} mg", tissue.mass_mg),
                    ..Default::default()
                }
            })
            .collect();
        let mut detail = "Approach a critter or carcass to inspect its tissue.".to_string();
        if let Some(&(organism, part)) = state.sources.get(state.selected) {
            let mut projected = world.clone();
            match projected.apply(Intent::Consume { organism, part }) {
                Outcome::Consumed {
                    mass_mg,
                    part: grown,
                    ..
                } => {
                    rows[state.selected - page].mass = format!("{} mg intake", mass_mg);
                    rows[state.selected - page].cost = "0 mg action charge".into();
                    rows[state.selected - page].attachment = "into your body".into();
                    detail = format!(
                        "Preview after one ordinary tick: {} mg from this part becomes your tissue. Other donor parts keep their matter. This does not revise your lineage.",
                        mass_mg
                    );
                    state.root = Some(grown);
                    state.preview = Some(Box::new(projected));
                    state.admissible = true;
                },
                Outcome::Rejected(reason) => {
                    let words = mesocosm_views::refusal_words(&reason);
                    rows[state.selected - page].refusal = Some(words.into());
                    detail = format!("Cannot consume this tissue: {words}.");
                },
                _ => {},
            }
        }
        state.reading = BodyMenu::new(
            vec![(
                "available".into(),
                format!("{} addressed parts", state.sources.len()),
            )],
            rows,
            state.selected - page,
            detail,
            "Enter consumes selected tissue",
            if notice.is_empty() {
                "World paused. Preview is disposable; Esc cancels."
            } else {
                notice
            },
        );
        state.reading.headline = "Inspect and consume tissue".into();
        state.reading.facts_title = "reachable tissue".into();
        state.reading.choice_label = "action".into();
        state.reading.keys =
            "Y open / arrows, J/L choose / Enter consume / Esc return / Z,V turn".into();
    }

    pub(super) fn confirm_eating(&mut self) {
        if self.runtime.state_hash() != self.grafting.hash || self.runtime.queued_len() != 0 {
            self.refresh_eating(
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
        self.runtime.queue(Intent::Consume { organism, part });
        self.steps += self.runtime.step(1);
        self.note_outcomes();
        let message = match self.runtime.last_outcomes().first() {
            Some(Outcome::Consumed { mass_mg, .. }) => {
                format!("Consumed {mass_mg} mg. Esc returns to the scene; H offers grafting.")
            },
            _ => "Intake refused by the world. Review the current tissue.".into(),
        };
        self.refresh_eating(&message);
        self.grafting.admissible = false;
        self.grafting.preview = None;
        self.grafting.root = None;
        self.last = None;
    }
}
