// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Paired reserve-assistance evidence for the VB4a family clearing.

use mesocosm_core::{Event, Founding, Intent, MealKind, Outcome, World, state_hash};
use serde::{Deserialize, Serialize};

const SEED: u64 = 7;
const TICKS: usize = 2_048;

#[derive(Clone, Debug, Serialize)]
struct Step {
    index: usize,
    intent: String,
    outcome: String,
    accepted: bool,
    refused: bool,
    reserve_mg: Option<u64>,
    body_mass_mg: Option<u64>,
    living: usize,
    offspring: usize,
    total_matter_mg: u64,
    state_hash: u64,
    events: usize,
    feeding_mg: u64,
    uptake_mg: u64,
    predation_mg: u64,
    cause_events: Vec<String>,
    refusal_cause: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct Run {
    reserve_assisted: bool,
    initial_parent: Option<String>,
    origin_parent_reserve_mg: Option<u64>,
    first_natural_birth_step: Option<usize>,
    origin_error: Option<String>,
    replay_hash_matches: bool,
    final_hash: Option<u64>,
    assistance_fact: &'static str,
    origin_events: Vec<String>,
    timeline: Vec<Step>,
    recorded_state_hash: Option<u64>,
    hash_matches_recorded: Option<bool>,
}

#[derive(Deserialize)]
struct RecordedTrace {
    seed: u64,
    body_layout: String,
    scene: String,
    trophic_grammar: u32,
    intents: Vec<Intent>,
    state_hash: u64,
    content: Option<RecordedContent>,
}

#[derive(Deserialize)]
struct RecordedContent {
    palette: mesocosm_core::PartPalette,
}

fn main() {
    let output = std::env::args().nth(1);
    let recorded = match std::env::args().nth(2) {
        Some(path) => Some(match read_trace(&path) {
            Ok(trace) => trace,
            Err(error) => {
                eprintln!("family clearing trace: {error}");
                std::process::exit(1);
            },
        }),
        None => None,
    };
    let (seed, palette, expected_hash, recorded_intents) = recorded.as_ref().map_or(
        (SEED, Founding::SpacedRoster.palette(), None, None),
        |trace| {
            (
                trace.seed,
                trace
                    .content
                    .as_ref()
                    .map_or(Founding::SpacedRoster.palette(), |content| content.palette),
                Some(trace.state_hash),
                Some(trace.intents.clone()),
            )
        },
    );
    let assisted = World::family_clearing(seed, Founding::SpacedRoster, palette, true);
    let lean = World::family_clearing(seed, Founding::SpacedRoster, palette, false);
    let (trace, assisted_start, lean_start, trace_error) =
        match (&assisted, &lean, recorded_intents) {
            (Ok(world), Ok(lean_world), Some(intents)) => {
                (intents, snapshot(world), snapshot(lean_world), None)
            },
            (Ok(world), Ok(lean_world), None) => match prepare_trace(world) {
                Ok(trace) => (trace, snapshot(world), snapshot(lean_world), None),
                Err(error) => (
                    Vec::new(),
                    snapshot(world),
                    snapshot(lean_world),
                    Some(error),
                ),
            },
            _ => (Vec::new(), None, None, Some("origin setup refused".into())),
        };
    let mut a = run(
        assisted,
        true,
        &trace,
        assisted_start.as_deref(),
        expected_hash,
    );
    let mut l = run(lean, false, &trace, lean_start.as_deref(), expected_hash);
    if let Some(error) = trace_error.as_ref() {
        a.origin_error = Some(error.clone());
        l.origin_error = Some(error.clone());
    }
    let receipt = serde_json::json!({ "seed": seed, "scene": "family_clearing",
        "intent_stream": if recorded.is_some() { "same recorded native playthrough for both origins" } else { "same opening and bounded birth wait for both origins" },
        "reserve_assistance_difference": "only parent excess reserve differs; authored births, carrion donor, intake declaration and prior discovery remain origin facts", "runs": [a, l] });
    let out = serde_json::to_string_pretty(&receipt).expect("receipt JSON");
    if let Some(path) = output {
        std::fs::write(path, out).expect("write family clearing receipt");
    } else {
        println!("{out}");
    }
    if trace_error.is_some()
        || recorded.is_some() && receipt["runs"][0]["hash_matches_recorded"] != true
        || !receipt["runs"].as_array().is_some_and(|runs| {
            runs.iter()
                .all(|r| r["origin_error"].is_null() && r["replay_hash_matches"] == true)
        })
    {
        std::process::exit(1);
    }
}

fn read_trace(path: &str) -> Result<RecordedTrace, String> {
    let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
    let trace: RecordedTrace = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if trace.body_layout != "spaced" {
        return Err(format!(
            "body_layout must be spaced, got {}",
            trace.body_layout
        ));
    }
    if trace.scene != "family_clearing" {
        return Err(format!(
            "scene must be family_clearing, got {}",
            trace.scene
        ));
    }
    if trace.trophic_grammar != mesocosm_core::TROPHIC_GRAMMAR_REVISION {
        return Err(format!(
            "trophic grammar {} is not current",
            trace.trophic_grammar
        ));
    }
    Ok(trace)
}

fn snapshot(world: &World) -> Option<Vec<u8>> {
    mesocosm_core::snapshot(world).ok()
}

fn prepare_trace(origin: &World) -> Result<Vec<Intent>, String> {
    let mut world = origin.clone();
    let mut trace = world
        .family_practice_opening()
        .ok_or("family opening unavailable")?;
    // The clearing origin contains authored births and the donor's authored
    // death. They are origin facts, not outcomes of the paired intent stream.
    world.drain_events();
    world.drain_flows();
    for intent in &trace {
        let outcome = world.apply(intent.clone());
        world.drain_events();
        world.drain_flows();
        if matches!(outcome, Outcome::Rejected(_)) {
            return Err(format!("opening intent refused: {outcome:?}"));
        }
    }
    trace.extend(world.family_practice_birth_run().take(TICKS));
    Ok(trace)
}

fn run(
    origin: Result<World, mesocosm_core::development::DevelopmentError>,
    assisted: bool,
    trace: &[Intent],
    start: Option<&[u8]>,
    expected_hash: Option<u64>,
) -> Run {
    let assistance_fact = if assisted {
        "parent reserve assistance retained"
    } else {
        "parent excess reserve returned to local soil after setup"
    };
    let mut world = match origin {
        Ok(world) => world,
        Err(error) => {
            return Run {
                reserve_assisted: assisted,
                initial_parent: None,
                origin_parent_reserve_mg: None,
                first_natural_birth_step: None,
                origin_error: Some(format!("{error:?}")),
                replay_hash_matches: false,
                final_hash: None,
                assistance_fact,
                origin_events: Vec::new(),
                timeline: Vec::new(),
                recorded_state_hash: expected_hash,
                hash_matches_recorded: None,
            };
        },
    };
    let parent_id = world.controlled_id();
    let initial_parent = parent_id.map(|id| format!("{id:?}"));
    let origin_parent_reserve_mg = world
        .organisms
        .iter()
        .find(|o| Some(o.id) == parent_id)
        .map(|o| o.energy_mg);
    let origin_events = world
        .events()
        .iter()
        .map(|event| format!("{:?}", event.record))
        .collect();
    let mut history = mesocosm_core::History::new();
    history.record_all(world.drain_events());
    world.drain_flows();
    let mut epoch_seen = world.epoch;
    let mut offspring = 0;
    let mut first_natural_birth_step = None;
    let mut timeline = Vec::with_capacity(trace.len());
    for (index, intent) in trace.iter().cloned().enumerate() {
        let outcome = world.apply(intent.clone());
        let events = world.drain_events();
        let flows = world.drain_flows();
        history.record_all(events.iter().copied());
        // Native epoch review records this core assessment. Reconstruct it
        // from the same origin history, including for refused lean choices.
        if world.epoch != epoch_seen {
            epoch_seen = world.epoch;
            world.reckon(&history);
        }
        let births = events.iter().filter(|e| matches!(e.record, Event::Born { parent: Some(parent), .. } if Some(parent) == parent_id)).count();
        offspring += births;
        if births > 0 {
            first_natural_birth_step.get_or_insert(index);
        }
        let feeding_mg = flow_sum(&flows, mesocosm_core::flow::Process::Feeding, parent_id);
        let uptake_mg = flow_sum(&flows, mesocosm_core::flow::Process::Uptake, parent_id);
        let predation_mg = events
            .iter()
            .filter_map(|event| match event.record {
                Event::Fed {
                    from,
                    mass_mg,
                    kind: MealKind::Predation,
                    ..
                } if Some(from) == parent_id => Some(mass_mg),
                _ => None,
            })
            .sum();
        let parent = world
            .organisms
            .iter()
            .find(|o| Some(o.id) == parent_id && o.is_alive());
        let cause_events = events
            .iter()
            .filter_map(|event| match event.record {
                Event::Died { .. } => Some(format!("{:?}", event.record)),
                _ => None,
            })
            .collect();
        let refusal_cause =
            matches!(outcome, Outcome::Rejected(_)).then(|| format!("{:?}", outcome));
        timeline.push(Step {
            index,
            intent: format!("{intent:?}"),
            outcome: format!("{outcome:?}"),
            accepted: !matches!(outcome, Outcome::Rejected(_)),
            refused: matches!(outcome, Outcome::Rejected(_)),
            reserve_mg: parent.map(|o| o.energy_mg),
            body_mass_mg: parent.map(|o| o.biomass_mg()),
            living: world.living().count(),
            offspring,
            total_matter_mg: world.total_matter_mg(),
            state_hash: state_hash(&world),
            events: events.len(),
            feeding_mg,
            uptake_mg,
            predation_mg,
            cause_events,
            refusal_cause,
        });
    }
    let final_hash = Some(state_hash(&world));
    let replay_hash_matches = start.is_some_and(|bytes| {
        let Ok(mut replay) = mesocosm_core::restore(bytes) else {
            return false;
        };
        let mut history = mesocosm_core::History::new();
        history.record_all(replay.drain_events());
        let mut epoch_seen = replay.epoch;
        for intent in trace.iter().cloned() {
            replay.apply(intent);
            history.record_all(replay.drain_events());
            replay.drain_flows();
            if replay.epoch != epoch_seen {
                epoch_seen = replay.epoch;
                replay.reckon(&history);
            }
        }
        final_hash == Some(state_hash(&replay))
    });
    Run {
        reserve_assisted: assisted,
        initial_parent,
        origin_parent_reserve_mg,
        first_natural_birth_step,
        origin_error: None,
        replay_hash_matches,
        final_hash,
        assistance_fact,
        origin_events,
        timeline,
        recorded_state_hash: expected_hash,
        hash_matches_recorded: expected_hash.map(|expected| final_hash == Some(expected)),
    }
}

fn flow_sum(
    flows: &[mesocosm_core::flow::RecordedFlow],
    process: mesocosm_core::flow::Process,
    parent: Option<mesocosm_core::OrganismId>,
) -> u64 {
    flows
        .iter()
        .filter(|f| {
            f.record.process == process && f.record.to.is_some_and(|s| Some(s.organism) == parent)
        })
        .map(|f| f.record.amount_mg)
        .sum()
}
