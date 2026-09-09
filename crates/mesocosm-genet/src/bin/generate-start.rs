// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Generate inspectable candidates and selection files for `mesocosm-genet --start`.
use mesocosm_core::{
    Founding, Kingdom,
    world::generation::{BodyPlan, Request, Selection, SoilPattern, VERSION},
};
use std::path::PathBuf;

fn run() -> Result<(), String> {
    let mut request = Request::default();
    let mut observe = 0u32;
    let mut output = PathBuf::from("generated-start");
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        if flag == "--help" {
            println!(
                "generate-start [--request request.json] [--seed N] [--variation N] [--body-plan axial|branched] [--role producer|consumer|decomposer|any] [--movement-organs yes|no|any] [--place 0..8] [--mass MG] [--max-parts N] [--candidates N] [--population N] [--soil-min MG] [--soil-max MG] [--soil-pattern patches|uniform|contrasting] [--min-open-steps 0..4] [--observe 0..128] [--output DIR]\nWrites report.json and start-N.json. Enter with mesocosm-genet --start DIR/start-N.json. Requested criteria are enforced; unsatisfied requests remain visible."
            );
            return Ok(());
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--body-plan" => {
                request.version = VERSION;
                request.criteria.body_plan = match value.as_str() {
                    "axial" => BodyPlan::Axial,
                    "branched" => BodyPlan::Branched,
                    _ => return Err("body plan must be axial or branched".into()),
                };
            },
            "--request" => {
                request = serde_json::from_slice(&std::fs::read(&value).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?
            },
            "--observe" => {
                observe = value.parse().map_err(|_| "invalid trial ticks")?;
                if observe > 128 {
                    return Err("trial limit is 128 ticks".into());
                }
            },
            "--soil-min" => {
                request.soil_min_mg = value.parse().map_err(|_| "invalid soil minimum")?
            },
            "--soil-max" => {
                request.soil_max_mg = value.parse().map_err(|_| "invalid soil maximum")?
            },
            "--population" => {
                request.organisms = value.parse().map_err(|_| "invalid population")?
            },
            "--min-open-steps" => {
                request.version = VERSION;
                request.min_open_steps = value.parse().map_err(|_| "invalid access bound")?;
            },
            "--soil-pattern" => {
                request.version = VERSION;
                request.soil_pattern = match value.as_str() {
                    "patches" => SoilPattern::Patches,
                    "uniform" => SoilPattern::Uniform,
                    "contrasting" => SoilPattern::Contrasting,
                    _ => return Err("unknown soil pattern".into()),
                };
            },
            "--seed" => request.seed = value.parse().map_err(|_| "invalid seed")?,
            "--variation" => request.variation = value.parse().map_err(|_| "invalid variation")?,
            "--place" => request.place = value.parse().map_err(|_| "invalid place")?,
            "--mass" => request.criteria.mass_mg = value.parse().map_err(|_| "invalid mass")?,
            "--max-parts" => {
                request.criteria.max_parts = value.parse().map_err(|_| "invalid part bound")?
            },
            "--candidates" => {
                request.candidates = value.parse().map_err(|_| "invalid candidate count")?
            },
            "--role" => {
                request.criteria.role = match value.as_str() {
                    "producer" => Some(Kingdom::Producer),
                    "consumer" => Some(Kingdom::Consumer),
                    "decomposer" => Some(Kingdom::Decomposer),
                    "any" => None,
                    _ => return Err("unknown role".into()),
                }
            },
            "--movement-organs" => {
                request.criteria.movement_organs = match value.as_str() {
                    "yes" => Some(true),
                    "no" => Some(false),
                    "any" => None,
                    _ => return Err("unknown movement criterion".into()),
                }
            },
            "--output" => output = PathBuf::from(value),
            _ => return Err(format!("unknown argument {flag}")),
        }
    }
    // Same generated content admission as the native host, including its refs.
    let pack = mesocosm_mesh::ContentPack::generate(Founding::Drawn.palette())
        .map_err(|e| format!("{e:?}"))?;
    let began = std::time::Instant::now();
    let prepared = request
        .prepare(pack.palette)
        .map_err(|e| format!("{e:?}"))?;
    let draft = prepared.draft();
    println!(
        "Seed {} / variation {}: {} candidates from {} attempts in {:.3}s",
        request.seed,
        request.variation,
        draft.candidates.len(),
        draft.attempted,
        began.elapsed().as_secs_f64()
    );
    for habitat in &draft.habitat {
        println!(
            "Place {} at {:?}: {} mg soil per column{}",
            habitat.place,
            habitat.centre,
            habitat.soil_mg_per_column,
            if habitat.place == request.place {
                " [selected]"
            } else {
                ""
            }
        );
    }
    // Refuse reuse, so a failed or smaller generation never leaves stale choices.
    std::fs::create_dir(&output)
        .map_err(|e| format!("choose a fresh output directory {}: {e}", output.display()))?;
    let write = |name: &str, bytes: Vec<u8>| {
        std::fs::write(output.join(name), bytes).map_err(|e| e.to_string())
    };
    write(
        "report.json",
        serde_json::to_vec_pretty(&draft).map_err(|e| e.to_string())?,
    )?;
    for (index, candidate) in draft.candidates.iter().enumerate() {
        println!(
            "{index}: {:?}/{:?}, {} segments, {} parts, actuator span {}, at {:?}, local founding supply {} mg",
            candidate.role,
            candidate.symmetry,
            candidate.segments,
            candidate.parts,
            candidate.actuator_span,
            candidate.position,
            candidate.local_soil_mg
        );
        if observe > 0 {
            let observation = prepared
                .observe(index, observe)
                .map_err(|e| format!("{e:?}"))?;
            write(
                &format!("observation-{index}.json"),
                serde_json::to_vec_pretty(&observation).map_err(|e| e.to_string())?,
            )?;
        }
        let selection = Selection {
            request: request.clone(),
            candidate: index,
        };
        write(
            &format!("start-{index}.json"),
            serde_json::to_vec_pretty(&selection).map_err(|e| e.to_string())?,
        )?;
    }
    println!(
        "Refusals: {:?}\nAppearance and admission are previews; ecological persistence is unproven.",
        draft.rejected
    );
    if draft.candidates.is_empty() {
        return Err("no candidates satisfy the request; see report.json".into());
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("generation refused: {error}");
        std::process::exit(1);
    }
}
