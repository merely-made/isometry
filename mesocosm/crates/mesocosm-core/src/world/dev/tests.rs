// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The four dev intents: what each one does, and how each one refuses. (DT3)
//!
//! Conservation and reconciliation through them are `tests/matter.rs` and
//! `tests/flows.rs`, where every other verb is proved; what is here is the
//! transaction and the refusal.

use super::*;
use crate::flow::{Account, Process};
use crate::history::Event;
use crate::organism::Stage;
use crate::rules::{EpochRule, WorldRules};
use crate::world::Intent;

/// A world whose epoch budget is long enough that nothing reaches a boundary
/// by itself inside a test — so a boundary that happens is one somebody asked
/// for.
fn unhurried(seed: u64, founders: u32, epoch: EpochRule) -> World {
    World::new(seed, founders).with_rules(WorldRules::native().ending(epoch).scoring_over(2))
}

/// The four are named as dev intents and nothing else is.
#[test]
fn the_four_dev_intents_are_named_as_such() {
    for intent in [
        Intent::EndEpoch,
        Intent::ForceBirth {
            organism: OrganismId(1),
        },
        Intent::Kill {
            organism: OrganismId(1),
        },
        Intent::PlaceMatter {
            at: [0, 0, 0],
            mass_mg: 10,
        },
    ] {
        assert!(intent.is_dev(), "{intent:?}");
    }
    for intent in [
        Intent::Idle,
        Intent::Resume,
        Intent::Move { delta: [1, 0, 0] },
        Intent::Deposit { mass_mg: 60 },
    ] {
        assert!(!intent.is_dev(), "{intent:?}");
    }
}

// ---------------------------------------------------------------- EndEpoch

/// Under `Timed`, the demand is an early end: the same boundary the budget
/// would have run, and the budget restarts from this tick.
#[test]
fn ending_the_epoch_on_demand_runs_the_timed_boundary_early() {
    let mut world = unhurried(11, 40, EpochRule::Timed { ticks: 1_000 });
    world.apply(Intent::Idle);
    assert_eq!(world.epoch, 0, "the budget is nowhere near spent");
    assert!(!world.at_boundary());

    let outcome = world.apply(Intent::EndEpoch);
    assert_eq!(
        outcome,
        Outcome::EpochEnded { epoch: 0 },
        "the one that closed"
    );
    assert_eq!(world.epoch, 1, "and the world is in the next one");
    assert!(world.at_boundary(), "standing at the lineage checkpoint");
    assert_eq!(
        world.epoch_began(),
        world.tick,
        "the budget restarts from here rather than shortening only the next epoch"
    );
    assert_eq!(world.last_round().epoch, 1, "the round ran");

    // And the next tick closes the boundary again, exactly as a spent budget's
    // does: `at_boundary` is a one-tick fact that a hold makes last.
    world.apply(Intent::Idle);
    assert!(!world.at_boundary());
    assert_eq!(world.epoch, 1, "one demand, one boundary");
}

/// Under `PlayerTriggered` the epoch ends on the demand and on nothing else.
#[test]
fn a_player_triggered_epoch_ends_only_on_the_demand() {
    let mut world = unhurried(11, 40, EpochRule::PlayerTriggered);
    assert!(world.epoch_rule().built(), "DT3 built it");
    for _ in 0..400 {
        world.apply(Intent::Idle);
    }
    assert_eq!(world.epoch, 0, "no budget ever spends itself here");
    assert!(!world.at_boundary());

    assert_eq!(
        world.apply(Intent::EndEpoch),
        Outcome::EpochEnded { epoch: 0 }
    );
    assert_eq!(world.epoch, 1);
    assert!(world.at_boundary());
    assert_eq!(world.epoch_began(), world.tick);
}

/// A `Gated` world refuses the demand, and names the rule that refused.
#[test]
fn a_gated_world_refuses_the_demand_by_name() {
    let mut world = unhurried(11, 40, EpochRule::Gated);
    let outcome = world.apply(Intent::EndEpoch);
    assert_eq!(
        outcome,
        Outcome::Rejected(Rejection::EpochNotOnDemand(EpochRule::Gated))
    );
    assert_eq!(world.epoch, 0, "and nothing happened");
    assert!(!world.at_boundary());
    assert_eq!(world.last_round().turns.len(), 0, "no round ran");
}

/// **One boundary, one door** (DT4), proven by the two doors landing on the
/// same world.
///
/// There used to be a third: `World::end_epoch(history)`, a manual door that
/// bumped the epoch and restarted the budget but never ran the adaptation round
/// and left `at_boundary` *false* — so an epoch closed through it gave the
/// unplayed lines no turn and stood at no lineage checkpoint. It is deleted.
/// What is left is this block in [`World::apply`], reached two ways, and the
/// claim is that the way does not matter.
///
/// Both worlds run the identical intent stream but for the last tick, and both
/// intents are pure: `Resume` and an accepted `EndEpoch` each move nothing,
/// write no event, and reset the idle run. So a difference between the two
/// worlds afterwards could only be the boundary itself.
#[test]
fn the_demand_and_the_spent_budget_leave_the_same_world() {
    const BUDGET: u64 = 60;
    let rule = EpochRule::Timed { ticks: BUDGET };

    // The budget's own door: nobody asks, and the epoch ends when it is spent.
    let mut by_budget = unhurried(11, 40, rule);
    for _ in 0..BUDGET {
        by_budget.apply(Intent::Resume);
    }

    // A hand's door, on the same tick: one intent short of the budget, then the
    // demand instead of the filler.
    let mut by_demand = unhurried(11, 40, rule);
    for _ in 0..BUDGET - 1 {
        by_demand.apply(Intent::Resume);
    }
    assert_eq!(
        by_demand.apply(Intent::EndEpoch),
        Outcome::EpochEnded { epoch: 0 }
    );

    // Both closed one epoch, both stand at the checkpoint, both ran the round.
    assert_eq!(by_budget.epoch, 1);
    assert!(by_budget.at_boundary());
    assert_eq!(by_budget.last_round().epoch, 1);
    assert_eq!(by_demand.epoch, 1);
    assert!(by_demand.at_boundary());
    assert_eq!(by_demand.last_round().epoch, 1);

    // And they are the same world, to the byte the hash reads.
    assert_eq!(
        crate::state_hash(&by_budget),
        crate::state_hash(&by_demand),
        "two doors, one boundary"
    );
    assert_eq!(by_budget, by_demand);
}

// -------------------------------------------------------------- ForceBirth

/// A forced birth is the ordinary birth: the child arrives the way the tick's
/// own birth pass would have delivered it, and pays what a birth pays.
#[test]
fn a_forced_birth_is_the_ordinary_birth_with_the_clock_taken_off_it() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);
    world.drain_events();

    // Somebody the ecology's own timing gate would refuse right now: a body
    // that has just bred, or has not grown up. Nothing about the parent is
    // doctored — the point is that the gate says no and the intent proceeds.
    let parent = world
        .living()
        .find(|o| !o.can_reproduce() && o.biomass_mg() > 400)
        .expect("a founder that is not ready to breed")
        .id;
    let (was_body, was_reserve) = world
        .organisms
        .iter()
        .find(|o| o.id == parent)
        .map(|o| (o.biomass_mg(), o.energy_mg))
        .expect("on the roster");

    let outcome = world.apply(Intent::ForceBirth { organism: parent });
    let Outcome::Bore {
        parent: bore,
        offspring,
    } = outcome
    else {
        panic!("the birth was refused: {outcome:?}");
    };
    assert_eq!(bore, parent);

    // The child is in the enclosure, juvenile, and no older than a newborn.
    let child = world
        .organisms
        .iter()
        .find(|o| o.id == offspring)
        .expect("the child joined the roster");
    assert_eq!(child.stage, Stage::Juvenile);
    assert_eq!(
        child.age, 0,
        "it joined where the tick's own newborns join, so the tick did not age it"
    );
    assert_eq!(
        child.species,
        world
            .organisms
            .iter()
            .find(|o| o.id == parent)
            .expect("alive")
            .species
    );
    let (child_body, child_reserve) = (child.biomass_mg(), child.energy_mg);

    // The ordinary record: an `Event::Born` naming the parent, and nothing
    // that says a hand was involved.
    let born = world
        .drain_events()
        .into_iter()
        .find(|recorded| {
            matches!(recorded.record, Event::Born { organism, .. } if organism == offspring)
        })
        .expect("the birth is in the event record");
    assert!(matches!(
        born.record,
        Event::Born { parent: Some(who), .. } if who == parent
    ));

    // And the ordinary transfer: both halves out of the parent's own accounts
    // and into the matching account of the child.
    let paid: Vec<(Account, u64)> = world
        .flows()
        .iter()
        .map(|flow| flow.record)
        .filter(|record| {
            record.process == Process::Birth && record.to.map(|to| to.organism) == Some(offspring)
        })
        .map(|record| {
            assert_eq!(record.from.map(|from| from.organism), Some(parent));
            assert_eq!(record.source, record.destination);
            (record.destination, record.amount_mg)
        })
        .collect();
    assert_eq!(paid.len(), 2, "one record per account: {paid:?}");
    assert!(paid.contains(&(Account::Substance, child_body)));
    assert!(paid.contains(&(Account::Reserve, child_reserve)));
    assert!(
        child_body <= was_body / 4 && child_reserve <= was_reserve,
        "and it is a quarter of the parent at most, out of what the parent had"
    );
    // That the parent's own accounts land where the stream says is the tick's
    // whole-ledger claim, and `tests/flows.rs` reconciles it there — the parent
    // also ate, paid rent and grew on this tick, so the arithmetic here would
    // be about the tick rather than about the birth.
}

/// The three ways there is no birth to have, each refused by name.
#[test]
fn a_forced_birth_is_refused_when_it_cannot_happen() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);

    let missing = OrganismId(u32::MAX);
    assert_eq!(
        world.apply(Intent::ForceBirth { organism: missing }),
        Outcome::Rejected(Rejection::NoSuchOrganism(missing))
    );

    // A corpse has no offspring to provision.
    let corpse = world.living().next().expect("somebody is alive").id;
    assert!(matches!(
        world.apply(Intent::Kill { organism: corpse }),
        Outcome::Killed { .. }
    ));
    assert_eq!(
        world.apply(Intent::ForceBirth { organism: corpse }),
        Outcome::Rejected(Rejection::NotLiving(corpse))
    );

    // **Provisioning binds.** A body with almost nothing left cannot pay for
    // its line's recipe out of a quarter of itself, which is the condition a
    // natural birth waits on rather than a rule this door invented.
    let poor = world
        .living()
        .find(|o| Some(o.id) != world.controlled_id())
        .expect("somebody else is alive")
        .id;
    let body = world
        .organisms
        .iter_mut()
        .find(|o| o.id == poor)
        .expect("here");
    let almost_all = body.biomass_mg().saturating_sub(2);
    body.spend_mass(almost_all);
    assert_eq!(
        world.apply(Intent::ForceBirth { organism: poor }),
        Outcome::Rejected(Rejection::InsufficientMass)
    );
}

// --------------------------------------------------------------------- Kill

/// **A dev-caused death reads as a natural one.** The record, the corpse and
/// the matter it releases are compared against a death the ecology took.
#[test]
fn a_dev_kill_leaves_what_a_natural_death_leaves() {
    let mut world = World::new(7, 60);

    // A death the ecology took, on its own schedule. Founders age out well
    // inside this window.
    let natural = loop {
        world.apply(Intent::Idle);
        let died: Vec<_> = world
            .drain_events()
            .into_iter()
            .filter_map(|recorded| match recorded.record {
                Event::Died { organism, species } => Some((organism, species)),
                _ => None,
            })
            .collect();
        if let Some(found) = died.first().copied() {
            break found;
        }
        assert!(world.tick < 4_000, "nothing died in four thousand ticks");
    };
    let natural_flow = world
        .flows()
        .iter()
        .map(|flow| flow.record)
        .find(|record| {
            record.process == Process::Death
                && record.from.map(|from| from.organism) == Some(natural.0)
        })
        .expect("a natural death releases its reserve into the ground");
    let natural_corpse = world
        .organisms
        .iter()
        .find(|o| o.id == natural.0)
        .map(|corpse| (corpse.stage, corpse.energy_mg))
        .expect("the corpse is on the roster");
    assert_eq!(
        natural_corpse,
        (Stage::Carrion, 0),
        "it let go of what it banked"
    );

    // And now one a hand asked for.
    let target = world
        .living()
        .find(|o| Some(o.id) != world.controlled_id())
        .expect("somebody else is alive")
        .id;
    let (species, reserve_was, body_was) = world
        .organisms
        .iter()
        .find(|o| o.id == target)
        .map(|o| (o.species, o.energy_mg, o.biomass_mg()))
        .expect("here");
    world.drain_events();

    let outcome = world.apply(Intent::Kill { organism: target });
    assert_eq!(
        outcome,
        Outcome::Killed {
            organism: target,
            substance_mg: body_was,
            reserve_mg: reserve_was,
        },
        "the corpse weighs what the body weighed, and the reserve is what it let go"
    );

    // The same event, with the same fields.
    let died = world
        .drain_events()
        .into_iter()
        .find_map(|recorded| match recorded.record {
            Event::Died { organism, species } if organism == target => Some(species),
            _ => None,
        })
        .expect("a dev kill writes the same Died the ecology writes");
    assert_eq!(died, species);

    // The same flow, out of the same account, through the same process.
    let dev_flow = world
        .flows()
        .iter()
        .map(|flow| flow.record)
        .find(|record| {
            record.process == Process::Death
                && record.from.map(|from| from.organism) == Some(target)
        })
        .expect("a dev kill releases its reserve the same way");
    assert_eq!(dev_flow.source, natural_flow.source);
    assert_eq!(dev_flow.destination, natural_flow.destination);
    assert_eq!(dev_flow.amount_mg, reserve_was);

    // And the same corpse: carrion, holding what it weighed, banking nothing.
    let corpse = world
        .organisms
        .iter()
        .find(|o| o.id == target)
        .expect("the corpse stayed");
    assert_eq!((corpse.stage, corpse.energy_mg), natural_corpse);
    assert!(corpse.biomass_mg() > 0, "there is a body to decompose");
}

/// A body that is already dead cannot be killed, and one that was never there
/// cannot either.
#[test]
fn killing_what_is_not_living_is_refused_by_name() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);
    let missing = OrganismId(u32::MAX);
    assert_eq!(
        world.apply(Intent::Kill { organism: missing }),
        Outcome::Rejected(Rejection::NoSuchOrganism(missing))
    );

    let target = world.living().next().expect("somebody is alive").id;
    assert!(matches!(
        world.apply(Intent::Kill { organism: target }),
        Outcome::Killed { .. }
    ));
    assert_eq!(
        world.apply(Intent::Kill { organism: target }),
        Outcome::Rejected(Rejection::NotLiving(target)),
        "a corpse cannot die twice"
    );
}

/// Killing the critter under the hand loses control the way any other death
/// does — no second path, and no exemption.
#[test]
fn killing_the_played_critter_loses_control_like_any_other_death() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);
    let me = world.controlled_id().expect("a played critter");
    assert!(matches!(
        world.apply(Intent::Kill { organism: me }),
        Outcome::Killed { .. }
    ));
    assert_eq!(world.controlled_id(), None);
    assert_eq!(world.control_lost(), Some(me));
}

// -------------------------------------------------------------- PlaceMatter

/// Placed matter enters the ground through the dev source, and the stream says
/// so.
#[test]
fn placed_matter_enters_the_ground_through_the_dev_source() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);
    let at = [3, 0, -4];
    let column = world.soil().column_at(at);
    let held = world.soil().matter_mg(column);

    let outcome = world.apply(Intent::PlaceMatter { at, mass_mg: 900 });
    assert_eq!(outcome, Outcome::Placed { at, mass_mg: 900 });

    let placed = world
        .flows()
        .iter()
        .map(|flow| flow.record)
        .find(|record| record.process == Process::Place)
        .expect("the placement is in the flow record");
    assert_eq!(placed.source, Account::Dev);
    assert_eq!(placed.destination, Account::Soil);
    assert_eq!(placed.amount_mg, 900);
    assert!(
        placed.from.is_none() && placed.to.is_none(),
        "neither end is a body"
    );
    assert_eq!(Account::issued_mg(world.flows()), 900);

    // The ground has it. The column moved on its own this tick too — rent,
    // uptake and percolation all landed in it — so the claim here is that it
    // is richer than it was; that the enclosure's total rose by exactly what
    // the dev source issued and by nothing else is `tests/matter.rs`, where
    // the check subtracts that account with no tolerance at all.
    assert!(world.soil().matter_mg(column) > held);
}

/// Off the grid and over the bound, each refused by name — and a placement of
/// nothing is not a transaction.
#[test]
fn a_placement_off_the_grid_or_over_the_bound_is_refused_by_name() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);
    let extent = world.soil().extent();
    let before = world.soil().total_mg();

    let outside = [extent + 1, 0, 0];
    assert_eq!(
        world.apply(Intent::PlaceMatter {
            at: outside,
            mass_mg: 100
        }),
        Outcome::Rejected(Rejection::OffGrid(outside)),
        "refused rather than clamped onto the wall"
    );
    let far = [0, 0, -extent - 40];
    assert_eq!(
        world.apply(Intent::PlaceMatter {
            at: far,
            mass_mg: 100
        }),
        Outcome::Rejected(Rejection::OffGrid(far))
    );

    let over = PLACE_MATTER_MAX_MG + 1;
    assert_eq!(
        world.apply(Intent::PlaceMatter {
            at: [0, 0, 0],
            mass_mg: over
        }),
        Outcome::Rejected(Rejection::OverBound {
            mass_mg: over,
            max_mg: PLACE_MATTER_MAX_MG,
        })
    );
    assert!(matches!(
        world.apply(Intent::PlaceMatter {
            at: [0, 0, 0],
            mass_mg: PLACE_MATTER_MAX_MG
        }),
        Outcome::Placed { .. }
    ));

    assert_eq!(
        world.apply(Intent::PlaceMatter {
            at: [0, 0, 0],
            mass_mg: 0
        }),
        Outcome::Rejected(Rejection::InsufficientMass),
        "a transfer of nothing is not a transaction"
    );

    // Only the one accepted placement reached the ground; the ticks in between
    // moved matter around inside the enclosure without adding any.
    assert!(world.soil().total_mg() >= before);
    assert!(
        world
            .flows()
            .iter()
            .all(|flow| flow.record.process != Process::Place),
        "the last tick's placement was refused, so it recorded nothing"
    );
}

/// A refused dev intent moves nothing and records nothing — the same commit
/// point every other verb shares.
#[test]
fn a_refused_dev_intent_reaches_neither_record() {
    let mut world = World::new(11, 40);
    world.apply(Intent::Idle);
    let before = crate::state_hash(&world);
    let mut refused = world.clone();
    let outcome = refused.apply(Intent::PlaceMatter {
        at: [0, 0, 0],
        mass_mg: u64::MAX,
    });
    assert!(matches!(outcome, Outcome::Rejected(_)));

    // The control is a refused *play* intent on the same tick, not an idle
    // one: a refusal still resets the world's idle run, because a hand that
    // asked for something impossible is a hand on the critter.
    let mut ordinary = world.clone();
    assert!(matches!(
        ordinary.apply(Intent::Deposit { mass_mg: u64::MAX }),
        Outcome::Rejected(_)
    ));
    assert_eq!(
        crate::state_hash(&refused),
        crate::state_hash(&ordinary),
        "a refused dev intent leaves the world exactly where a refused play intent does"
    );
    assert_ne!(
        before,
        crate::state_hash(&ordinary),
        "and the tick happened"
    );
}
