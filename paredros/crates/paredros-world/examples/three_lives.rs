// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Authored/caller-supplied projections, not a GameState or graphical sheet.

use paredros_world::fixtures::three_lives as fixture;
use paredros_world::{SubjectBody, arrest_fall};

fn main() {
    println!(
        "Authored wetland lives at year {}. Shared problem: repair a flooded crossing; danger: a fall.",
        fixture::NOW
    );
    println!("Ordinary contributions below are narrative; arrest-fall availability is evaluated.");
    println!("Load is a supplied 60000 mg equivalent demand, not simulated fall dynamics.");
    for life in fixture::three_lives() {
        assert!(fixture::source_order_is_valid(&life) && fixture::facts_have_sources(&life));
        println!(
            "\n{} ({:?}, revision {:?})",
            life.name, life.subject, life.revision
        );
        println!("  contribution: {}", life.ordinary_task);
        println!("  danger choices: {}", life.danger_choices.join("; "));
        println!(
            "  anatomy: {} living parts, mantle {:?}, gland {:?}",
            life.body.living().count(),
            fixture::MANTLE,
            fixture::FIBRE_GLAND
        );
        for fact in life.facts {
            println!(
                "  history: {} <- {} at year {} ({})",
                fact.description, fact.source_id, fact.acquired_at, fact.acquired_place
            );
        }
        let query = arrest_fall(
            &fixture::knowledge(&life),
            SubjectBody {
                subject: life.subject,
                revision: life.revision,
                body: &life.body,
            },
            life.revision,
            &fixture::inputs(&life),
        );
        println!("  learned-action blockers: {:?}", query.blockers);
        for binding in query.bindings {
            println!(
                "  {:?}: {}; sources {:?}; costs {:?}; blockers {:?}",
                binding.kind,
                if binding.available() {
                    "available"
                } else {
                    "blocked"
                },
                binding.sources,
                binding.costs,
                binding.blockers
            );
        }
    }
}
