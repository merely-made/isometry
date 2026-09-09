// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeSet;

use paredros_identity::SubjectId;
use paredros_world::{
    Migration, PopulationConfig, PopulationOrigin, ProjectError, ProjectIntent, Pursuit,
    Simulation, SimulationError, SiteKind, World, WorldConfig,
};

const SEED: u64 = 4_242;

fn world() -> World {
    World::generate(SEED, WorldConfig::default()).unwrap()
}

fn config(world: &World) -> PopulationConfig {
    let settlement = world
        .map()
        .slots_of_kind(SiteKind::Settlement)
        .next()
        .unwrap();
    let ruin = world.map().slots_of_kind(SiteKind::Ruin).next().unwrap();
    PopulationConfig {
        seed: 91,
        migrations: vec![Migration {
            from: settlement,
            toward: ruin,
            count: 1,
        }],
        ..PopulationConfig::default()
    }
}

fn simulation() -> Simulation {
    let world = world();
    let config = config(&world);
    Simulation::generate(world, config).unwrap()
}

#[test]
fn population_genesis_records_site_and_migration_origins() {
    let first = simulation();
    let second = simulation();
    assert_eq!(first, second);

    let site_kinds = first
        .population()
        .all()
        .filter_map(|life| match life.origin {
            PopulationOrigin::Site { kind, .. } => Some(kind),
            PopulationOrigin::Migration { .. } => None,
        })
        .collect::<Vec<_>>();
    for kind in [
        SiteKind::Settlement,
        SiteKind::Ruin,
        SiteKind::Encounter,
        SiteKind::Dungeon,
    ] {
        assert!(site_kinds.contains(&kind));
    }
    assert!(
        first
            .population()
            .all()
            .any(|life| matches!(life.origin, PopulationOrigin::Migration { .. }))
    );

    let mut names = BTreeSet::new();
    for life in first.population().all() {
        let body = first.game().bodies().get(life.subject).unwrap();
        assert!(body.alive());
        assert_eq!(body.name.as_ref(), Some(&life.name));
        assert_eq!(
            first.game().movement().position(life.subject),
            Some(life.origin_at)
        );
        assert!(names.insert(life.name.as_str()));
    }
}

#[test]
fn unattended_rounds_advance_every_life_and_leave_legible_changes() {
    let mut simulation = simulation();
    let subjects = simulation
        .population()
        .all()
        .map(|life| life.subject)
        .collect::<Vec<_>>();
    let before = subjects
        .iter()
        .map(|subject| simulation.report(*subject).unwrap())
        .collect::<Vec<_>>();

    let decisions = simulation.advance(60).unwrap();
    assert_eq!(simulation.round(), 60);
    assert!(!decisions.is_empty());

    let pursuits = decisions
        .iter()
        .map(|decision| &decision.pursuit)
        .collect::<Vec<_>>();
    assert!(
        pursuits
            .iter()
            .any(|pursuit| matches!(pursuit, Pursuit::Hunger))
    );
    assert!(
        pursuits
            .iter()
            .any(|pursuit| matches!(pursuit, Pursuit::Safety))
    );
    assert!(
        pursuits
            .iter()
            .any(|pursuit| matches!(pursuit, Pursuit::Work(_)))
    );
    assert!(
        pursuits
            .iter()
            .any(|pursuit| matches!(pursuit, Pursuit::Curiosity(_)))
    );
    assert!(
        pursuits
            .iter()
            .any(|pursuit| matches!(pursuit, Pursuit::Travel(_)))
    );
    assert!(
        pursuits
            .iter()
            .any(|pursuit| matches!(pursuit, Pursuit::Project(_)))
    );

    let after = subjects
        .iter()
        .map(|subject| simulation.report(*subject).unwrap())
        .collect::<Vec<_>>();
    assert!(after.iter().all(|report| report.autonomous_actions > 0));
    assert!(after.iter().any(|report| report.completed_projects > 0));
    assert!(!simulation.projects().events().is_empty());
    assert!(
        before
            .iter()
            .zip(&after)
            .filter(|(before, after)| *before != *after)
            .count()
            >= 2
    );
    assert!(after.iter().any(|report| !report.carried.is_empty()));
    assert!(after.iter().any(|report| {
        report.position
            != before
                .iter()
                .find(|earlier| earlier.subject == report.subject)
                .unwrap()
                .position
    }));
}

#[test]
fn simulation_restore_continues_the_same_unobserved_lives() {
    let mut straight = simulation();
    straight.advance(17).unwrap();
    let mut restored = Simulation::restore(&straight.save().unwrap()).unwrap();
    assert_eq!(restored, straight);

    let first = straight.advance(23).unwrap();
    let second = restored.advance(23).unwrap();
    assert_eq!(first, second);
    assert_eq!(restored, straight);
    assert_eq!(
        restored.state_hash().unwrap(),
        straight.state_hash().unwrap()
    );

    let mut save = straight.save_record().unwrap();
    save.decisions[0].intent_index = 0;
    assert!(matches!(
        Simulation::restore_record(save),
        Err(SimulationError::DecisionDiverged(0))
    ));

    let mut save = straight.save_record().unwrap();
    let ProjectIntent::Complete {
        round,
        project,
        subject,
    } = save.projects.intents[0];
    let cause = save
        .decisions
        .iter()
        .position(|decision| {
            decision.round == round
                && decision.subject == subject
                && decision.pursuit == Pursuit::Project(project)
        })
        .unwrap();
    save.decisions.remove(cause);
    assert!(matches!(
        Simulation::restore_record(save),
        Err(SimulationError::ProjectDecisionDiverged(id)) if id == project
    ));

    let mut save = straight.save_record().unwrap();
    assert!(save.projects.intents.len() > 1);
    let ProjectIntent::Complete { round, .. } = &mut save.projects.intents[0];
    *round = u64::MAX;
    assert!(matches!(
        Simulation::restore_record(save),
        Err(SimulationError::Project(ProjectError::OutOfOrder { .. }))
    ));
}

#[test]
fn scheduling_has_no_selected_subject() {
    let mut simulation = simulation();
    let subjects = simulation
        .population()
        .all()
        .map(|life| life.subject)
        .collect::<BTreeSet<SubjectId>>();
    let decisions = simulation.advance(3).unwrap();
    let acted = decisions
        .iter()
        .map(|decision| decision.subject)
        .collect::<BTreeSet<_>>();
    assert_eq!(acted, subjects);
}
