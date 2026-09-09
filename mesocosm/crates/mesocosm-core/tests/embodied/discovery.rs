// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PE2: discovery becomes an embodied option.
//!
//! One test per named claim in the gate:
//!
//! 1. **one condition unrelated to food unlocks a candidate** — coming through
//!    the starvation horizon grants the gland, and no meal is anywhere in it;
//! 2. **one meal supplies evidence without unlocking an incompatible
//!    candidate** — the evidence is recorded, the condition that never declared
//!    the meal lane cannot be reached by it, and the donor's other recipe words
//!    are not taught;
//! 3. **one consumed part settles only its own matter and provenance** — the
//!    organ's exact milligrams move, its children keep theirs, and
//!    `from_part` finally names the part it came off;
//! 4. **the PD2 process, re-receipted through the discovery route** — located,
//!    paid for, useful, dormant, and lost with its branch, reached from a
//!    condition rather than from PD2's temporary fixture;
//! 5. **direct and automatic fixtures use the same validator** — one candidate,
//!    two proposal sources, one instruction.

pub(crate) use mesocosm_core::discovery::Miss;

use mesocosm_core::discovery::{self, Condition, HUNGER_TICKS, Input};
use mesocosm_core::{
    Arrangement, Attachment, Intent, Kingdom, Organism, OrganismId, Outcome, PartId, Process,
    ProcessRef, Provenance, Registry, Role, Stage, VolumeRef, World, Yaw, classify,
};

use super::bulk_world;

/// The same twelve-cell plate PD2's fixtures grow, for the same reason: a
/// gland on five of its cells out-holds a fresh soil column, so the dormant
/// state is reachable rather than theoretical.
const FROND: [i32; 3] = [6, 4, 1];

pub(crate) fn condition(name: &str) -> Condition {
    discovery::conditions()
        .into_iter()
        .find(|found| found.name == name)
        .expect("the table holds it")
}

pub(crate) fn hunger_condition() -> Condition {
    condition("mesocosm:endured-hunger")
}

pub(crate) fn plate_eaten() -> Condition {
    condition("mesocosm:plate-eaten")
}

/// The condition PD2's gland arrives through, addressed the way an intent
/// addresses it.
pub(crate) fn hunger() -> mesocosm_core::ConditionId {
    hunger_condition().id()
}

fn gland_ref() -> ProcessRef {
    Registry::native().of_native(Process::Secrete).reference()
}

/// Holds the played body under the starved line, with a hand on it, for
/// `ticks` ticks.
///
/// The budget is topped back up to just short of the line each tick rather
/// than left at zero, because a body at zero eats itself and the claim under
/// test is about **surviving** the stress. `Intent::Resume` is the free verb
/// that keeps the hand on: it moves nothing and resets the idle run, which is
/// exactly what it was built for.
pub(crate) fn endure(world: &mut World, ticks: u64) {
    for _ in 0..ticks {
        let Some(me) = world.controlled_id() else {
            return;
        };
        let upkeep = world.controlled().expect("alive").upkeep_mg();
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == me)
            .expect("in the roster")
            .energy_mg = upkeep * (mesocosm_core::STARVED_UPKEEP_TICKS - 1);
        world.apply(Intent::Resume);
    }
}

/// A carcass the played critter can reach, built as a producer so it carries a
/// plate, with one further part hanging off that plate.
///
/// The grandchild is the whole point of the third claim: taking an organ must
/// not take what is under it.
pub(crate) fn carcass(world: &mut World) -> (OrganismId, PartId, PartId) {
    let here = world.position().expect("embodied");
    let at = [here[0] + 1, here[1], here[2]];
    let id = OrganismId(9_400);
    let mut corpse = Organism {
        stage: Stage::Carrion,
        ..Organism::founding(
            id,
            mesocosm_core::SpeciesId(5),
            Kingdom::Producer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            at,
            1_200,
        )
    };
    let root = corpse.body().root;
    let plate = corpse
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            400,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a plate attaches to the root");
    let under = corpse
        .phenotype
        .attach(
            VolumeRef::from_tag(9),
            150,
            [7, 1, 1],
            Attachment {
                parent: plate,
                offset: [9, 0, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("and a limb hangs off the plate");
    assert_eq!(classify(FROND), Role::Plate, "the fixture's premise");
    world.organisms.push(corpse);
    (id, plate, under)
}

/// Grows a frond on the played critter directly — the same native
/// developmental fixture PD2's receipts use, because no ordinary meal in this
/// enclosure grows a consumer a plate.
fn frond_on(world: &mut World) -> PartId {
    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let root = organism.body().root;
    organism
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            300,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches above the root")
}

// ---------------------------------------------------------------------------
// 1. One condition unrelated to food unlocks a candidate
// ---------------------------------------------------------------------------

#[test]
fn coming_through_a_stress_unlocks_a_candidate_with_no_meal_in_it() {
    let mut world = bulk_world(4_242, 24);
    assert!(world.discoveries().is_empty(), "a line starts with nothing");

    endure(&mut world, HUNGER_TICKS + 2);
    assert!(world.controlled().is_some(), "and it came through alive");

    let discovery = world
        .discoveries()
        .iter()
        .find(|found| found.condition == hunger())
        .expect("a hundred ticks under the line is the condition");

    // The route is the endurance lane and the evidence is the stress with its
    // magnitude on it. Nothing here is a food category.
    assert_eq!(discovery.route, Input::Endurance);
    assert_eq!(
        discovery.evidence,
        mesocosm_core::Evidence::Endured {
            stress: discovery::Stress::Hunger,
            ticks: HUNGER_TICKS,
        }
    );
    assert_eq!(discovery.source, mesocosm_core::Source::Endured);
    // The realized candidate reference, its parameters, and a digest over all
    // of it: what the execution boundary asks be recorded.
    assert_eq!(discovery.candidate.process, gland_ref());
    assert_eq!(discovery.candidate.site, Role::Plate);
    assert!(discovery.candidate.cells > 0);
    assert_ne!(discovery.digest, 0);

    // And it is a candidate, not an applied change: the body has grown no
    // gland and expresses nothing new.
    assert_eq!(world.gland(), None, "unlocking is not expressing");
}

#[test]
fn the_accumulator_is_the_worlds_and_the_crossing_happens_once() {
    let mut world = bulk_world(4_242, 24);
    endure(&mut world, HUNGER_TICKS - 1);
    assert_eq!(
        u64::from(world.hunger_run()),
        HUNGER_TICKS - 1,
        "the run is world state and counts what it says it counts"
    );
    assert!(world.discoveries().is_empty(), "one tick short is short");

    endure(&mut world, 40);
    assert_eq!(world.discoveries().len(), 1);
    // Past the horizon the run keeps counting and the condition does not fire
    // again: it is a crossing, not a poll.
    assert!(u64::from(world.hunger_run()) > HUNGER_TICKS);
    assert_eq!(world.discoveries().len(), 1);
}

#[test]
fn feeding_the_body_ends_the_stress() {
    // A quantified stress has to be *come through*, so the run resets the
    // moment the body is off the wrong side of the line.
    let mut world = bulk_world(4_242, 24);
    endure(&mut world, 30);
    assert!(world.hunger_run() > 0);

    let me = world.controlled_id().unwrap();
    let upkeep = world.controlled().unwrap().upkeep_mg();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .energy_mg = upkeep * mesocosm_core::STARVED_UPKEEP_TICKS * 4;
    world.apply(Intent::Resume);
    assert_eq!(world.hunger_run(), 0);
}

#[test]
fn an_idle_terrarium_discovers_nothing() {
    // **Played-only, structurally.** `held()` gates the accumulator exactly as
    // it gates PE1's checkpoint, so an enclosure nobody is touching is never
    // asked anything — which is why the population instrument cannot observe
    // this phase.
    let mut world = World::new(4_242, 24);
    for _ in 0..1_200 {
        world.apply(Intent::Idle);
    }
    assert_eq!(world.hunger_run(), 0);
    assert!(world.discoveries().is_empty());
    assert!(world.last_observation().is_none());
}

// ---------------------------------------------------------------------------
// 4. PD2's process, re-receipted through the discovery route
// ---------------------------------------------------------------------------

#[test]
fn the_discovered_candidate_is_located_paid_for_useful_dormant_and_lost_with_its_branch() {
    let mut world = bulk_world(4_242, 24);
    endure(&mut world, HUNGER_TICKS + 1);
    assert!(world.discovered(hunger()), "the condition landed");

    // **Availability is not expression.** A bulk consumer has nowhere to put a
    // gland, so the candidate proposes nothing until the body has the shape.
    assert!(
        world.candidate_intent(hunger()).is_none(),
        "nowhere to put it yet"
    );
    let part = frond_on(&mut world);

    // Located, and paid for: the proposal is the candidate's, and it goes
    // through PD2's ordinary door — priced, charged, recorded.
    let intent = world
        .candidate_intent(hunger())
        .expect("the frond is somewhere to put it");
    let cell_mg = world.phenotype().unwrap().cell_mg(part);
    let matter_before = world.total_matter_mg();
    let outcome = world.apply(intent);
    let Outcome::Expressed {
        part: on, cost_mg, ..
    } = outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!(on, part);
    assert_eq!(
        cost_mg,
        u64::from(hunger_condition().grants.cells) * cell_mg
    );
    assert_eq!(world.total_matter_mg(), matter_before);

    // Useful: the tissue is where the reading says, and it stings.
    let reading = world.gland().expect("it has one now");
    assert_eq!(reading.sites, vec![(part, hunger_condition().grants.cells)]);
    assert!(reading.charged, "charged by its own spoil, as PD2 found");
    assert!(reading.rent_mg > 0, "and it costs rent from here on");

    // Dormant: one column over, where the ground cannot supply what it holds.
    // Two columns, because the first one is still holding this body's own
    // spoil: the development paid its price into the ground it was standing
    // on, so a fresh gland is always charged where it was made. Dormancy is
    // something a body walks into, which is the right shape for it — carrying
    // a big gland means keeping to rich ground.
    let mut dry_world = world.clone();
    dry_world.apply(Intent::Move { delta: [2, 0, 0] });
    dry_world.apply(Intent::Move { delta: [2, 0, 0] });
    let dry = dry_world.gland().expect("still has one");
    assert!(!dry.charged, "{} against {}", dry.ground_mg, dry.potency_mg);
    assert_eq!(dry.cells, reading.cells, "and lost no tissue for it");
    assert_eq!(dry.rent_mg, reading.rent_mg, "nor a milligram of rent");

    // Lost with its branch, and the branch can still say what it did.
    let me = world.controlled_id().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .phenotype
        .sever(part);
    let gone = world.gland().expect("the loss is still readable");
    assert!(gone.sites.is_empty());
    assert_eq!(gone.rent_mg, 0);
    assert_eq!(gone.lost, vec![part]);
    // And the discovery outlives the branch: what a line came to is not undone
    // by what happened to one body.
    assert!(world.discovered(hunger()));
}

// ---------------------------------------------------------------------------
// 5. One candidate, two proposal sources, one validator
// ---------------------------------------------------------------------------

#[test]
fn direct_and_automatic_fixtures_lower_the_same_candidate_the_same_way() {
    // PD1b made this a property of the type rather than a test that has to
    // keep being re-passed — `Arrangement` is diagnostic metadata the
    // validator never reads — and PE2's candidates are proposal sources like
    // any other, so they inherit it. NPC acquisition is still an open ruling
    // (plan §6, ruling 5); this is the receipt that when it is made, there is
    // one validator waiting rather than two.
    let mut world = bulk_world(4_242, 24);
    endure(&mut world, HUNGER_TICKS + 1);
    frond_on(&mut world);

    let direct = world
        .candidate_proposal(hunger(), Arrangement::Direct)
        .expect("a proposal");
    let automatic = world
        .candidate_proposal(hunger(), Arrangement::Automatic)
        .expect("the same proposal, differently authored");
    assert_eq!(direct.parts, automatic.parts);
    assert_eq!(direct.sites, automatic.sites);
    assert_eq!(direct.expect, automatic.expect);

    let mut by_hand = world.phenotype().unwrap().clone();
    let mut by_game = by_hand.clone();
    let hand = by_hand
        .develop(mesocosm_core::Registry::native(), &direct)
        .expect("it validates");
    let game = by_game
        .develop(mesocosm_core::Registry::native(), &automatic)
        .expect("and so does it");
    assert_eq!(
        hand.instruction, game.instruction,
        "one candidate, one instruction, whoever proposed it"
    );
    assert_eq!(hand.source, Arrangement::Direct);
    assert_eq!(game.source, Arrangement::Automatic);
    assert_eq!(by_hand, by_game, "and one body afterwards");
}

// ---------------------------------------------------------------------------
// The record survives the round trip
// ---------------------------------------------------------------------------

#[test]
fn a_discovery_survives_a_snapshot_and_replays_to_the_same_hash() {
    let mut world = bulk_world(4_242, 24);
    endure(&mut world, HUNGER_TICKS + 1);
    assert_eq!(world.discoveries().len(), 1);

    let restored =
        mesocosm_core::restore(&mesocosm_core::snapshot(&world).unwrap()).expect("decodes");
    assert_eq!(restored.discoveries(), world.discoveries());
    assert_eq!(restored.hunger_run(), world.hunger_run());
    assert_eq!(restored.last_observation(), world.last_observation());
    assert_eq!(
        mesocosm_core::state_hash(&restored),
        mesocosm_core::state_hash(&world),
        "a discovery is world state, so it is inside the hash"
    );
}

#[test]
fn the_causal_record_names_the_condition_a_line_came_through() {
    let mut world = bulk_world(4_242, 24);
    let mut past = mesocosm_core::History::new();
    for _ in 0..(HUNGER_TICKS + 1) {
        let Some(me) = world.controlled_id() else {
            break;
        };
        let upkeep = world.controlled().unwrap().upkeep_mg();
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == me)
            .unwrap()
            .energy_mg = upkeep * (mesocosm_core::STARVED_UPKEEP_TICKS - 1);
        world.apply(Intent::Resume);
        past.record_all(world.drain_events());
    }

    let discovered = past
        .log()
        .entries()
        .iter()
        .filter_map(|recorded| match recorded.record {
            mesocosm_core::Event::Discovered { condition, .. } => Some(condition),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(discovered, vec![hunger()]);
    assert_eq!(
        discovery::name_of(discovered[0]),
        Some("mesocosm:endured-hunger"),
        "and the digest resolves to something a panel can say"
    );
}
