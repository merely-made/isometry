// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! **The flow stream accounts for what the compartments did.** PE0's
//! load-bearing invariant, and the executable form of the plan's sentence: *a
//! resource mutation cannot be visible to the state while absent from the flow
//! record.*
//!
//! `matter.rs` next door proves the other half — that the enclosure's total is
//! constant — and would still pass if every milligram moved in silence. This
//! one reads the two records against each other: for every tick, and every
//! account of every body, the recorded transfers must sum to exactly the change
//! the world made. A seam that mutates without emitting fails here even though
//! conservation holds.
//!
//! # The instrument is proved, not assumed
//!
//! [`reconcile`] is handed a doctored tick as well as honest ones, and must
//! report it. An absence is evidence only beside a positive control.

use std::collections::BTreeMap;

use mesocosm_core::flow::{Account, Process, RecordedFlow};
use mesocosm_core::{Intent, OrganismId, Placement, World, state_hash};

// An integration test's crate root resolves `mod` against `tests/`, and a bare
// `tests/transfers.rs` would become a second test binary. The explicit path
// keeps the split files beside the suite they belong to.
#[path = "flows/boundary.rs"]
mod boundary;
#[path = "flows/dev.rs"]
mod dev;
#[path = "flows/refusals.rs"]
mod refusals;
#[path = "flows/transfers.rs"]
mod transfers;

/// Every compartment TD6's conserved sum is made of.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Books {
    soil_mg: u64,
    /// Substance and reserve, per body. Absent means zero: a body that has not
    /// been born yet and one that has been eaten are the same to a ledger.
    bodies: BTreeMap<OrganismId, (u64, u64)>,
}

fn books(world: &World) -> Books {
    Books {
        soil_mg: world.soil().total_mg(),
        bodies: world
            .organisms
            .iter()
            .map(|o| (o.id, (o.biomass_mg(), o.energy_mg)))
            .collect(),
    }
}

/// What the stream says each account did, as a signed milligram delta.
fn claimed(flows: &[RecordedFlow]) -> (i128, BTreeMap<OrganismId, (i128, i128)>) {
    let mut soil = 0i128;
    let mut bodies: BTreeMap<OrganismId, (i128, i128)> = BTreeMap::new();
    for flow in flows {
        let record = &flow.record;
        soil += record.net_on(Account::Soil);
        let amount = i128::from(record.amount_mg);
        // `is_body` rather than a soil comparison since DT3: the dev source
        // is the second account that belongs to nobody.
        if let Some(from) = record.from
            && record.source.is_body()
        {
            let entry = bodies.entry(from.organism).or_default();
            match record.source {
                Account::Substance => entry.0 -= amount,
                Account::Reserve => entry.1 -= amount,
                Account::Soil | Account::Dev => unreachable!("guarded above"),
            }
        }
        if let Some(to) = record.to
            && record.destination.is_body()
        {
            let entry = bodies.entry(to.organism).or_default();
            match record.destination {
                Account::Substance => entry.0 += amount,
                Account::Reserve => entry.1 += amount,
                Account::Soil | Account::Dev => unreachable!("guarded above"),
            }
        }
    }
    (soil, bodies)
}

/// The check itself. `Ok` is silence; `Err` names the account that disagreed.
///
/// One function, used by the runs that must pass **and** by the control that
/// must fail: a check only shown honest ticks has not been shown to detect.
fn reconcile(
    before: &Books,
    after: &Books,
    flows: &[RecordedFlow],
    at: &str,
) -> Result<(), String> {
    let (soil_claim, body_claims) = claimed(flows);
    let soil_moved = i128::from(after.soil_mg) - i128::from(before.soil_mg);
    if soil_moved != soil_claim {
        return Err(format!(
            "soil moved {soil_moved} mg {at}, the stream accounts for {soil_claim} mg"
        ));
    }

    let subjects: std::collections::BTreeSet<OrganismId> = before
        .bodies
        .keys()
        .chain(after.bodies.keys())
        .chain(body_claims.keys())
        .copied()
        .collect();
    for id in subjects {
        let (was_body, was_reserve) = before.bodies.get(&id).copied().unwrap_or_default();
        let (is_body, is_reserve) = after.bodies.get(&id).copied().unwrap_or_default();
        let (body_claim, reserve_claim) = body_claims.get(&id).copied().unwrap_or_default();
        let body_moved = i128::from(is_body) - i128::from(was_body);
        let reserve_moved = i128::from(is_reserve) - i128::from(was_reserve);
        if body_moved != body_claim {
            return Err(format!(
                "{id:?} substance moved {body_moved} mg {at}, the stream accounts for {body_claim} mg"
            ));
        }
        if reserve_moved != reserve_claim {
            return Err(format!(
                "{id:?} reserve moved {reserve_moved} mg {at}, the stream accounts for {reserve_claim} mg"
            ));
        }
    }
    Ok(())
}

/// A tick, reconciled. Returns the tick's flows so a caller can assert on them.
fn stepped(world: &mut World, intent: Intent, at: &str) -> Vec<RecordedFlow> {
    let before = books(world);
    world.apply(intent);
    let flows = world.drain_flows();
    let after = books(world);
    if let Err(why) = reconcile(&before, &after, &flows, at) {
        panic!("{why}");
    }
    flows
}

#[test]
fn the_stream_accounts_for_every_compartment_across_a_run() {
    // Long enough that every seam in the cycle fires: soil draw and return,
    // grazing, predation, scavenging, rent paid out of a reserve and out of a
    // body, dispersal, birth, death and decay.
    for seed in [1u64, 7, 4_242] {
        let mut world = World::new(seed, 60);
        for tick in 1..=1_200 {
            stepped(
                &mut world,
                Intent::Idle,
                &format!("on tick {tick} of seed {seed}"),
            );
        }
    }
}

#[test]
fn the_stream_accounts_for_the_played_verbs_too() {
    // The tick is not the only thing that moves matter, so the ledger is read
    // against every acting intent as well: a meal (burned or built, the body
    // decides), a deposit, movement paid in substance, and a carve, which must
    // move none at all.
    let mut world = World::new(11, 40);
    let mut trace = vec![
        Intent::Deposit { mass_mg: 60 },
        Intent::Move { delta: [1, 0, 0] },
        Intent::Move { delta: [-1, 0, 1] },
        Intent::Carve {
            at: world.position().expect("a played critter"),
            radius: 1,
        },
    ];
    let me = world.controlled_id().expect("a played critter");
    for organism in world.living().map(|o| o.id).collect::<Vec<_>>() {
        if organism != me {
            trace.push(Intent::Metabolize {
                organism,
                placement: Placement::Planned,
            });
        }
    }

    for (step, intent) in trace.into_iter().enumerate() {
        stepped(&mut world, intent, &format!("after played step {step}"));
    }
}

#[test]
fn every_flow_is_stamped_with_the_tick_it_happened_on() {
    let mut world = World::new(7, 40);
    for _ in 0..30 {
        let expected = world.tick;
        let flows = stepped(&mut world, Intent::Idle, "while stamping");
        assert!(!flows.is_empty(), "a living enclosure moves matter");
        assert!(
            flows.iter().all(|f| f.tick == expected),
            "one tick's buffer holds one tick"
        );
    }
}

#[test]
fn draining_the_readings_is_not_a_world_change() {
    // The `drain_ground_dirty` precedent, asserted: the ledger is outside the
    // snapshot and outside equality, so a run that reduces readings every tick
    // and one that never looks are the same world.
    let mut drained = World::new(4_242, 60);
    let mut held = World::new(4_242, 60);
    for _ in 0..80 {
        drained.apply(Intent::Idle);
        held.apply(Intent::Idle);
        assert!(!drained.flows().is_empty(), "there was something to drain");
        drained.drain_flows();
    }

    assert!(!held.flows().is_empty(), "and the other one still holds it");
    assert_eq!(
        drained, held,
        "the buffer is not part of a world's identity"
    );
    assert_eq!(state_hash(&drained), state_hash(&held));

    // And the drain itself moves nothing, taken twice on the same world.
    let before = state_hash(&held);
    held.drain_flows();
    assert_eq!(state_hash(&held), before);
}

#[test]
fn a_worlds_snapshot_does_not_carry_the_flow_stream() {
    // The stop rule, structurally: dense per-tick flow must not enter a
    // snapshot to serve presentation. A world holding a tick of it encodes to
    // the same bytes as one that drained.
    let mut world = World::new(4_242, 60);
    world.apply(Intent::Idle);
    let holding = mesocosm_core::snapshot::snapshot(&world).expect("encodable");
    let flows = world.flows().len();
    world.drain_flows();
    let empty = mesocosm_core::snapshot::snapshot(&world).expect("encodable");

    assert!(flows > 0, "the tick had flows to leave out");
    assert_eq!(holding, empty);
}

/// **A birth reconciles to the milligram** (PE1).
///
/// The tick as a whole already has to, and `stepped` asserts that. What this
/// adds is the birth's own two records read against the body they made: a
/// newborn's entire substance and entire budget came out of its parent's
/// matching accounts and are in the stream as such, so the number the
/// checkpoint puts on the screen is the number the ledger reconciled. A spawn
/// that minted matter — which is what a birth was before TD6 — would fail both
/// the tick and this.
#[test]
fn a_birth_reconciles_to_the_milligram() {
    let mut world = World::new(11, 60);
    world.apply(Intent::Idle);
    world.drain_events();

    // Somebody the ordinary gate will let breed on the next tick. The gate
    // itself is untouched: the mass condition below is the ecology's.
    let roster: Vec<OrganismId> = world.living().map(|o| o.id).collect();
    let parent = roster
        .into_iter()
        .find(|id| {
            let Some(candidate) = world.organisms.iter_mut().find(|o| o.id == *id) else {
                return false;
            };
            let was = (candidate.stage, candidate.since_offspring);
            candidate.stage = mesocosm_core::Stage::Mature;
            candidate.since_offspring = u32::MAX;
            if candidate.can_reproduce() {
                true
            } else {
                (candidate.stage, candidate.since_offspring) = was;
                false
            }
        })
        .expect("some founder is ready to breed");

    for tick in 0..40 {
        let flows = stepped(&mut world, Intent::Idle, &format!("on birth tick {tick}"));
        let born: Vec<OrganismId> = world
            .drain_events()
            .into_iter()
            .filter_map(|recorded| match recorded.record {
                mesocosm_core::history::Event::Born {
                    organism,
                    parent: Some(who),
                    ..
                } if who == parent => Some(organism),
                _ => None,
            })
            .collect();
        let Some(child) = born.first().copied() else {
            continue;
        };

        let paid: Vec<(Account, u64)> = flows
            .iter()
            .map(|flow| &flow.record)
            .filter(|record| {
                record.process == Process::Birth && record.to.map(|to| to.organism) == Some(child)
            })
            .map(|record| {
                assert_eq!(
                    record.from.map(|from| from.organism),
                    Some(parent),
                    "a birth is a transfer out of the parent, not a spawn"
                );
                assert_eq!(
                    record.source, record.destination,
                    "body pays for body and reserve for reserve"
                );
                (record.destination, record.amount_mg)
            })
            .collect();
        assert_eq!(paid.len(), 2, "one record per account: {paid:?}");

        let substance = paid
            .iter()
            .find(|(account, _)| *account == Account::Substance)
            .expect("the body it was given")
            .1;
        let reserve = paid
            .iter()
            .find(|(account, _)| *account == Account::Reserve)
            .expect("the budget it was given")
            .1;
        let newborn = world
            .living()
            .find(|o| o.id == child)
            .expect("the child is in the enclosure");
        assert_eq!(
            newborn.biomass_mg(),
            substance,
            "every milligram the child weighs is in the stream"
        );
        assert_eq!(
            newborn.energy_mg, reserve,
            "and so is every milligram it can spend"
        );
        return;
    }
    panic!("no birth in forty ticks");
}

#[test]
fn a_filially_expressed_birth_reconciles_to_the_milligram() {
    // **PD5's conservation case.** A descendant born under its line's revision
    // is developed in the tick it arrived in, and that development costs
    // milligrams: what it pays leaves its own reserve and lands in the column
    // under it, because nothing evaporates (TD6). `stepped` reconciles every
    // account against the stream on every tick below, so the claim this test
    // adds on top is the narrower one — the child's expressed sites cost
    // exactly what the flow record says, and exactly what the record of the
    // birth says they cost.
    let mut world = World::new(4_242, 24);
    let me = world.controlled_id().expect("a played critter");
    // A plain bulk consumer, so the horizon below is reached by not eating
    // rather than raced by a canopy's own income.
    let organism = world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .expect("here");
    let (species, position) = (organism.species, organism.position);
    *organism = mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
        ..mesocosm_core::Organism::founding(
            me,
            species,
            mesocosm_core::Kingdom::Consumer,
            mesocosm_core::VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };

    // Come to the gland the way the game does: through the starvation horizon,
    // with a hand on the body. The budget is doctored *before* the reconciled
    // loop rather than inside it — a test that wrote a reserve between two
    // `stepped` calls would hand the check a mutation nobody claimed.
    for _ in 0..=mesocosm_core::discovery::HUNGER_TICKS {
        let upkeep = world.controlled().expect("alive").upkeep_mg();
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == me)
            .expect("in the roster")
            .energy_mg = upkeep * (mesocosm_core::STARVED_UPKEEP_TICKS - 1);
        world.apply(Intent::Resume);
    }
    let condition = world
        .discoveries()
        .first()
        .expect("the line came through the horizon")
        .condition;

    // **The lineage checkpoint** (PE3): a revision is admitted only there, so
    // the fixture shortens the epoch budget rather than idling a thousand
    // ticks. Every tick from here ends one, boundaries and all.
    let mut world = world.with_rules(
        mesocosm_core::WorldRules::native()
            .ending(mesocosm_core::rules::EpochRule::Timed { ticks: 1 })
            .scoring_over(2),
    );
    world.apply(Intent::Idle);
    assert!(world.at_boundary());

    // A line whose descendants grow the shape the declared site needs.
    world.lineages_mut().set_recipe(
        species,
        mesocosm_core::Recipe::of(vec![mesocosm_core::Tagma::new(
            1,
            mesocosm_core::Appendage::Plate,
        )]),
    );
    let revision = match world.apply(Intent::Revise { condition }) {
        mesocosm_core::Outcome::Revised { revision, .. } => revision,
        other => panic!("the commit was refused: {other:?}"),
    };

    let parent = world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .expect("here");
    parent.stage = mesocosm_core::Stage::Mature;
    parent.since_offspring = u32::MAX;
    parent.energy_mg = 100_000;
    assert!(
        parent.can_reproduce(),
        "the ecology's own gate is satisfied"
    );
    world.drain_events();
    world.drain_flows();

    for tick in 0..40 {
        let flows = stepped(&mut world, Intent::Idle, &format!("on birth tick {tick}"));
        let events = world.drain_events();
        let Some(child) = events.iter().find_map(|recorded| match recorded.record {
            mesocosm_core::history::Event::Born {
                organism,
                parent: Some(who),
                ..
            } if who == me => Some(organism),
            _ => None,
        }) else {
            continue;
        };

        let recorded_cost = events
            .iter()
            .find_map(|recorded| match recorded.record {
                mesocosm_core::history::Event::Inherited {
                    organism,
                    revision: under,
                    cost_mg,
                    ..
                } if organism == child && under == revision => Some(cost_mg),
                _ => None,
            })
            .expect("the child was born expressing its line's revision");
        assert!(recorded_cost > 0, "and expressing it cost milligrams");

        let developed: Vec<u64> = flows
            .iter()
            .map(|flow| &flow.record)
            .filter(|record| {
                record.process == Process::Develop
                    && record.from.map(|from| from.organism) == Some(child)
            })
            .map(|record| {
                assert_eq!(record.source, Account::Reserve, "out of the child's budget");
                assert_eq!(record.destination, Account::Soil, "and into the ground");
                record.amount_mg
            })
            .collect();
        assert_eq!(
            developed,
            vec![recorded_cost],
            "one record, for exactly what the birth record says it cost"
        );

        // The other half: the child's reserve is the birth's, less the program.
        let given = flows
            .iter()
            .map(|flow| &flow.record)
            .find(|record| {
                record.process == Process::Birth
                    && record.destination == Account::Reserve
                    && record.to.map(|to| to.organism) == Some(child)
            })
            .expect("a birth provisions a reserve")
            .amount_mg;
        let newborn = world.living().find(|o| o.id == child).expect("alive");
        assert_eq!(newborn.energy_mg, given - recorded_cost);
        assert!(
            newborn.phenotype.secretory_mg() > 0,
            "and it has the organ it paid for"
        );
        return;
    }
    panic!("no birth in forty ticks");
}

#[test]
fn the_check_catches_a_mutation_the_stream_did_not_record() {
    // **The positive control.** A seam that moves matter without emitting is
    // exactly the failure this file exists against, so the check is shown one.
    let mut world = World::new(1, 60);
    world.apply(Intent::Idle);
    let before = books(&world);
    world.apply(Intent::Idle);
    let flows = world.drain_flows();

    let honest = books(&world);
    reconcile(&before, &honest, &flows, "on an honest tick").expect("the tick itself reconciles");

    // The mutation an unrecorded seam would make: a milligram appears in a
    // reserve and no transfer says where it came from.
    let mut doctored = honest.clone();
    let subject = *doctored.bodies.keys().next().expect("a populated world");
    doctored.bodies.get_mut(&subject).expect("just read").1 += 1;

    let complaint = reconcile(&before, &doctored, &flows, "after a silent gain")
        .expect_err("an unrecorded milligram must not pass");
    assert!(
        complaint.contains("reserve moved"),
        "the check must name the account: {complaint}"
    );
}
