// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Stable text rows for the inspection viewport.

use mesocosm_core::Origin;

use super::{Focus, LifeSheet, SheetView};

pub(super) fn detail_lines(life: &LifeSheet, view: &SheetView) -> Vec<String> {
    let mut lines = match view.focus {
        Focus::Part => part_lines(life, view),
        Focus::Action => action_lines(life, view),
    };
    for blocker in &life.sheet.global_blockers {
        lines.extend(wrap(&format!("Action blocker: {blocker}")));
    }
    lines.push(format!(
        "Learned: {}",
        if life.sheet.learned.is_empty() {
            "none"
        } else {
            "arrest fall"
        }
    ));
    lines.extend(wrap(&format!(
        "Resources: {}",
        life.sheet
            .resources
            .iter()
            .map(|row| format!("{:?} {} units", row.kind, row.available_units))
            .collect::<Vec<_>>()
            .join(", ")
    )));
    lines
}

fn part_lines(life: &LifeSheet, view: &SheetView) -> Vec<String> {
    let Some(part) = life.sheet.parts.get(view.part) else {
        return vec!["No selected part.".into()];
    };
    let mut lines = vec![format!("{} / part {}", part.name, part.id.0)];
    lines.extend(wrap(if part.severed {
        "State: severed. Address remains inspectable; it cannot supply a binding."
    } else {
        "State: living."
    }));
    match &part.provenance.origin {
        Origin::Founding => lines.push("Provenance: founding body.".into()),
        Origin::Incorporated {
            from_species,
            from_part,
        } => lines.extend(wrap(&format!(
            "Provenance: incorporated from species {} part {} at epoch {}.",
            from_species.0, from_part.0, part.provenance.epoch
        ))),
    }
    lines.push(format!(
        "Attachment: {}",
        part.parent
            .map(|id| format!("part {}", id.0))
            .unwrap_or_else(|| "root".into())
    ));
    if part.capabilities.is_empty() {
        lines.push("No action capability projection for this part.".into());
    }
    for cap in &part.capabilities {
        lines.extend(wrap(&format!(
            "{:?}: reach {} body-scale voxels; load {} mg",
            cap.function, cap.reach_voxels, cap.load_capacity_mg
        )));
    }
    lines
}

fn action_lines(life: &LifeSheet, view: &SheetView) -> Vec<String> {
    let Some(action) = life.sheet.actions.get(view.action) else {
        return vec!["No selected action.".into()];
    };
    let mut lines = vec![format!(
        "{} / {}",
        binding_name(action.kind),
        if action.available {
            "available"
        } else {
            "blocked"
        }
    )];
    lines.extend(wrap(&format!("Sources: {}", source_names(&action.sources))));
    lines.extend(wrap(&format!("Costs: {}", cost_names(&action.costs))));
    for blocker in &action.blockers {
        lines.extend(wrap(&format!("Binding blocker: {blocker}")));
    }
    for item in &action.equipment {
        lines.extend(wrap(&format!(
            "{:?} {}: reach {} body-scale voxels; capacity {} mg; attached {}",
            item.function,
            item.item.0,
            item.reach_voxels,
            item.load_capacity_mg,
            item.attached_to
                .map(|part| format!("to part {}", part.0))
                .unwrap_or_else(|| "nowhere".into())
        )));
        lines.extend(wrap(&format!(
            "Carried by: {}",
            item.carried_by
                .map(|subject| format!("subject {}", subject.0))
                .unwrap_or_else(|| "nobody".into())
        )));
    }
    lines
}

fn wrap(text: &str) -> Vec<String> {
    let words: Vec<_> = text.split_whitespace().collect();
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in words {
        if !line.is_empty() && line.len() + word.len() + 1 > 52 {
            lines.push(line);
            line = String::new();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}
fn binding_name(kind: paredros_world::BindingKind) -> &'static str {
    match kind {
        paredros_world::BindingKind::GripLine => "grip + line",
        paredros_world::BindingKind::Adhesion => "adhesion",
        paredros_world::BindingKind::HarnessLine => "harness + line",
    }
}
fn source_names(sources: &[paredros_world::SourceQuery]) -> String {
    sources
        .iter()
        .map(|source| match source {
            paredros_world::SourceQuery::Part(id) => format!("part {}", id.0),
            paredros_world::SourceQuery::Equipment(id) => format!("equipment {}", id.0),
        })
        .collect::<Vec<_>>()
        .join(", ")
}
fn cost_names(costs: &[paredros_world::ResourceCost]) -> String {
    if costs.is_empty() {
        "none".into()
    } else {
        costs
            .iter()
            .map(|cost| format!("{:?} {} units", cost.kind, cost.units))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wraps_long_tail() {
        let lines = wrap("one two three four five six seven eight nine ten eleven twelve");
        assert!(lines.len() > 1);
        assert!(lines.last().unwrap().contains("twelve"));
    }
}
