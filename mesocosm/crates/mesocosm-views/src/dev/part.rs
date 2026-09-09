// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The selected-part reading for the host dev inspector.

use mesocosm_core::history::{Event, History};
use mesocosm_core::{FeedingMode, IntakePort, NisKind};
use mesocosm_core::{OrganismId, Origin, PartId, Role, World, classify};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartReading {
    pub organism: String,
    pub id: String,
    pub role: String,
    pub process: String,
    pub intake: String,
    pub feeding: String,
    pub condition: String,
    pub discovery_condition: String,
    pub history_event: String,
    pub lineage: String,
    pub donor: String,
}

/// A selection can be unavailable without inventing a body reading.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartInspection {
    pub reading: Option<PartReading>,
    pub notice: Option<String>,
}

/// Reads one part and its exact addressed history without changing state.
pub fn part_of(
    world: &World,
    organism: OrganismId,
    part: PartId,
    history: &History,
) -> PartInspection {
    let Some(organism) = world.organisms.iter().find(|found| found.id == organism) else {
        return unavailable("the selected critter is unavailable");
    };
    let Some(found) = organism.body().part(part) else {
        return unavailable("the selected part is unavailable");
    };
    let role = classify(found.half_extent);
    let Some(explanation) = organism.phenotype.explain(part) else {
        return unavailable("the selected part has no reading");
    };
    let actual: Vec<String> = explanation
        .sites
        .iter()
        .map(|site| {
            let name = site
                .named
                .as_ref()
                .map(|id| id.name.clone())
                .unwrap_or_else(|| "unknown process".into());
            bounded(&format!("{name} ({})", cause_words(site.cause)))
        })
        .collect();
    let process = format!(
        "{}: {}; dormant: unknown",
        if explanation.living && organism.is_alive() {
            "actual"
        } else {
            "historical"
        },
        if actual.is_empty() {
            "none".into()
        } else {
            bounded(&actual.join(", "))
        }
    );
    let lineage = world
        .lineages()
        .get(organism.species)
        .and_then(|line| line.name.clone())
        .map(|name| bounded(&format!("{} — {name}", organism.species.0)))
        .unwrap_or_else(|| organism.species.0.to_string());
    let donor = match &found.provenance.origin {
        Origin::Founding => "founding tissue".into(),
        Origin::Incorporated {
            from_species,
            from_part,
        } => format!("line {} part {}", from_species.0, from_part.0),
    };
    PartInspection {
        reading: Some(PartReading {
            organism: organism.id.0.to_string(),
            id: part.0.to_string(),
            role: role_word(role).into(),
            process,
            intake: intake_words(organism, part),
            feeding: feeding_words(organism),
            condition: format!(
                "{}; {} mg",
                if found.severed {
                    "severed"
                } else if organism.is_alive() {
                    "living"
                } else {
                    "carcass"
                },
                found.mass_mg
            ),
            discovery_condition: "unknown".into(),
            history_event: bounded(&history_event(history, organism.id, part)),
            lineage: bounded(&lineage),
            donor: bounded(&donor),
        }),
        notice: None,
    }
}

/// Display labels only; the core owns the reading and admission relation.
pub fn feeding_word(mode: FeedingMode) -> &'static str {
    match mode {
        FeedingMode::Producer => "producer",
        FeedingMode::Grazer => "grazer",
        FeedingMode::Predator => "predator",
        FeedingMode::Omnivore => "omnivore",
        FeedingMode::Scavenger => "scavenger",
    }
}

fn feeding_words(organism: &mesocosm_core::Organism) -> String {
    if !organism.is_alive() {
        return "carcass; inactive".into();
    }
    let mode = organism.feeding_mode();
    if mode == FeedingMode::Producer {
        return if organism.phenotype.canopy() {
            "producer; active fixation".into()
        } else {
            "no active intake or fixation".into()
        };
    }
    format!(
        "{}; {}",
        feeding_word(mode),
        port_words(organism.phenotype.intake_ports())
    )
}

fn port_words(port: IntakePort) -> String {
    let mut kinds = Vec::new();
    for (kind, label) in [
        (NisKind::Producer, "living producers"),
        (NisKind::Consumer, "living consumers"),
        (NisKind::Decomposer, "living decomposers"),
    ] {
        if port.admits_live(kind) {
            kinds.push(label);
        }
    }
    if port.admits_deadstock() {
        kinds.push("dead stock");
    }
    if kinds.is_empty() {
        "none".into()
    } else {
        kinds.join(", ")
    }
}

fn intake_words(organism: &mesocosm_core::Organism, part: PartId) -> String {
    let declared = organism
        .phenotype
        .mosaic(part)
        .map(|m| m.port())
        .unwrap_or_default();
    if !declared.has_live() && !declared.admits_deadstock() {
        return "none declared".into();
    }
    let active = organism.is_alive() && organism.phenotype.part_port(part).is_some();
    format!(
        "{}: {}",
        if active { "active" } else { "inactive" },
        port_words(declared)
    )
}

fn cause_words(cause: mesocosm_core::phenotype::Expressed) -> String {
    match cause {
        mesocosm_core::phenotype::Expressed::Geometry => "geometry".into(),
        mesocosm_core::phenotype::Expressed::Arranged { revision } => {
            format!("arranged revision {revision}")
        },
    }
}

fn history_event(history: &History, organism: OrganismId, part: PartId) -> String {
    for recorded in history.log().entries().iter().rev() {
        let matches = match recorded.record {
            Event::Grew {
                organism: who,
                part: found,
            }
            | Event::Expressed {
                organism: who,
                part: found,
                ..
            }
            | Event::Inherited {
                organism: who,
                part: found,
                ..
            }
            | Event::Grafted {
                organism: who,
                part: found,
                ..
            }
            | Event::Severed {
                organism: who,
                part: found,
            } => who == organism && found == part,
            _ => false,
        };
        if matches {
            return match recorded.record {
                Event::Grew { .. } => format!("grew tick {}", recorded.tick),
                Event::Expressed { .. } => format!("expressed tick {}", recorded.tick),
                Event::Inherited { .. } => format!("inherited tick {}", recorded.tick),
                Event::Grafted {
                    from, from_part, ..
                } => format!(
                    "grafted from critter {} part {} tick {}",
                    from.0, from_part.0, recorded.tick
                ),
                Event::Severed { .. } => format!("severed tick {}", recorded.tick),
                _ => unreachable!("the match above only accepts part events"),
            };
        }
    }
    "unknown".into()
}

fn unavailable(notice: &str) -> PartInspection {
    PartInspection {
        reading: None,
        notice: Some(notice.into()),
    }
}

fn bounded(value: &str) -> String {
    const LIMIT: usize = 96;
    let mut chars = value.chars();
    let mut result: String = chars.by_ref().take(LIMIT).collect();
    if chars.next().is_some() {
        result.push('…');
    }
    result
}

fn role_word(role: Role) -> &'static str {
    match role {
        Role::Mass => "mass",
        Role::Limb => "limb",
        Role::Plate => "plate",
        Role::Sensor => "sensor",
    }
}

#[cfg(test)]
mod tests;
