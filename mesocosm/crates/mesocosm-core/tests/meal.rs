// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P0: where a meal goes, and TD4: who decides.
//!
//! Before P0, eating granted a part **and** half the mass as energy, so the
//! game's central verb asked the player nothing. P0 split the destination:
//! live now, or grow later. These tests still pin that tradeoff — mutually
//! exclusive receipts, consistent venom, identical replay.
//!
//! What changed on 2026-08-29 is who answers. Mark rejected the hotkey pair as
//! an interface, and TD4 ruled the answer diegetic: **the body routes the
//! meal.** A critter inside `STARVED_UPKEEP_TICKS` of an empty budget burns;
//! one with room to spare builds. So these tests no longer choose a route,
//! they choose a *state*, which is the same choice moved to where it is
//! played from.

use mesocosm_core::{
    Intent, Outcome, Placement, Rejection, STARVED_UPKEEP_TICKS, World, snapshot, state_hash,
};

/// Sets the played critter's budget. Energy lives on the organism now, so a
/// test that wants a starving critter has to say which one.
fn set_energy(world: &mut World, energy_mg: u64) {
    let id = world.controlled_id().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == id)
        .unwrap()
        .energy_mg = energy_mg;
}

/// Sets what one organism's flesh costs to swallow.
fn venom_of(world: &mut World, organism: mesocosm_core::OrganismId, venom_mg: u64) {
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == organism)
        .expect("the organism is in the roster")
        .venom_mg = venom_mg;
}

/// The budget at which the body stops building and starts burning.
fn starving_below(world: &World) -> u64 {
    world.controlled().unwrap().upkeep_mg() * STARVED_UPKEEP_TICKS
}

/// Eating, said the only way it can be said now: what becomes of it is not
/// part of the sentence.
fn eat(organism: mesocosm_core::OrganismId) -> Intent {
    Intent::Metabolize {
        organism,
        placement: Placement::Planned,
    }
}

/// A world with something in reach, and the id of what to eat. The meal tests
/// exercise the two metabolize receipts, so the fixture places a live target
/// directly in reach instead of spending hundreds of ecology ticks chasing a
/// moving organism.
fn fed() -> (World, mesocosm_core::OrganismId) {
    let mut world = World::new(4_242, 24);
    let here = world.position().unwrap();
    let prey = world
        .organisms
        .iter()
        .filter(|o| o.biomass_mg() > 0 && Some(o.id) != world.controlled_id())
        .min_by_key(|o| {
            (0..3)
                .map(|a| (o.position[a] - here[a]).abs())
                .max()
                .unwrap_or(0)
        })
        .map(|o| o.id)
        .expect("the fixture scatters organisms");
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == prey)
        .expect("the selected organism remains in the roster")
        .position = here;
    (world, prey)
}

/// Applies an intent and reports what the **ecology** fed the played critter
/// on the same tick.
///
/// The act resolves first and the enclosure steps after it, so a played meal's
/// receipt has a second meal sitting on top of it. That was invisible while a
/// grazer rarely found anything within reach; TD7's pyramid founds forty
/// producers, so it gets a bite most ticks. These tests are about the *played*
/// meal, so they subtract the ecology's.
fn apply_counting_the_ecologys_own_meal(world: &mut World, intent: Intent) -> (Outcome, u64) {
    let me = world.controlled_id();
    let outcome = world.apply(intent);
    let fed = world
        .drain_events()
        .into_iter()
        .filter_map(|event| match event.record {
            mesocosm_core::Event::Fed { eater, mass_mg, .. } if Some(eater) == me => Some(mass_mg),
            _ => None,
        })
        .sum();
    (outcome, fed)
}

/// The same fixture, emptied out. A starved critter is a state you arrive at
/// by playing badly; a test arrives at it by saying so.
fn starving() -> (World, mesocosm_core::OrganismId) {
    let (mut world, prey) = fed();
    set_energy(&mut world, 0);
    (world, prey)
}

#[test]
fn a_starved_body_burns_its_meal() {
    let (mut world, prey) = starving();
    let parts = world.body().unwrap().len();

    let outcome = world.apply(eat(prey));

    assert!(matches!(outcome, Outcome::Burned { .. }), "got {outcome:?}");
    assert_eq!(world.body().unwrap().len(), parts, "burning grows nothing");
    assert!(world.energy_mg().unwrap() > 0, "and it pays now");
}

#[test]
fn a_provisioned_body_builds_with_it() {
    // The half of the tradeoff that used to be free. Growth is the slow
    // answer, and a meal cannot be both meals.
    let (mut world, prey) = fed();
    let parts = world.body().unwrap().len();
    let energy = world.energy_mg().unwrap();
    assert!(!world.is_starved(), "the fixture starts with room to spare");

    let outcome = world.apply(eat(prey));

    assert!(
        matches!(
            outcome,
            Outcome::Incorporated { .. } | Outcome::IncorporatedPair { .. }
        ),
        "got {outcome:?}"
    );
    assert!(world.body().unwrap().len() > parts, "the body grew");
    assert!(
        world.energy_mg().unwrap() <= energy,
        "and it paid nothing immediately"
    );
}

#[test]
fn the_budget_decides_and_the_threshold_is_where_it_says_it_is() {
    // The whole of TD4's income ruling, in one reading. Nothing about the
    // intent differs between these two worlds; only the ledger does.
    let (base, prey) = fed();
    let line = starving_below(&base);

    let mut just_under = base.clone();
    set_energy(&mut just_under, line - 1);
    assert!(just_under.is_starved());
    assert!(matches!(
        just_under.apply(eat(prey)),
        Outcome::Burned { .. }
    ));

    let mut just_over = base;
    set_energy(&mut just_over, line);
    assert!(!just_over.is_starved());
    assert!(matches!(
        just_over.apply(eat(prey)),
        Outcome::Incorporated { .. } | Outcome::IncorporatedPair { .. }
    ));
}

#[test]
fn the_same_meal_cannot_be_spent_twice() {
    // Mutually exclusive receipts, which is the done-condition's exact words.
    // Whichever way the body routes it, the organism is consumed and the other
    // route is no longer available.
    for (mut world, prey) in [fed(), starving()] {
        let before = world.organisms.len();

        world.apply(eat(prey));
        assert_eq!(world.organisms.len(), before - 1, "the meal is gone");

        let again = world.apply(eat(prey));
        assert_eq!(again, Outcome::Rejected(Rejection::NoSuchOrganism(prey)));
    }
}

#[test]
fn burning_and_growing_diverge_from_one_world() {
    // The choice has to *matter*, not merely exist. Two worlds identical up to
    // the meal end up in different states, and the difference is exactly the
    // one the design claims: mass or budget. What separates them now is the
    // budget they ate on, which is the point.
    let (grown_world, prey) = fed();
    let mut grown = grown_world;
    let mut burned = grown.clone();
    set_energy(&mut burned, 0);

    burned.apply(eat(prey));
    grown.apply(eat(prey));

    assert_ne!(
        state_hash(&burned),
        state_hash(&grown),
        "the routes are not the same act"
    );
    assert!(
        grown.body().unwrap().len() > burned.body().unwrap().len(),
        "one grows later"
    );
    assert!(
        burned.energy_mg().unwrap() > 0,
        "and the other lives now, from nothing"
    );
}

#[test]
fn venom_is_charged_whatever_the_meal_becomes() {
    // The defect this replaces: the explicit editor path subtracted venom and
    // the automatic path did not, so the safe-looking verb was the dangerous
    // one. A warning signal is only worth reading if believing it changes what
    // happens, on **every** route — and now the route is not the player's, so
    // there is no route they could pick to dodge it.
    const VENOM: u64 = 40;

    for starved in [false, true] {
        let (mut poisoned, prey) = fed();
        // **Both sides say what the prey is carrying.** The control used to be
        // "the fixture's prey, untouched", which quietly assumed the nearest
        // organism was harmless; genesis gives three founders in ten some
        // venom, and S1's wider enclosure put one of those under the played
        // critter's nose — 74 mg of it, so the "clean" world was the poisoned
        // one and the subtraction underflowed. Setting both ends of the
        // comparison makes the claim independent of the draw. (2026-08-29 S1)
        let mut clean = poisoned.clone();
        venom_of(&mut poisoned, prey, VENOM);
        venom_of(&mut clean, prey, 0);
        if starved {
            // Half the line: inside it, but with enough left that the zero
            // floor cannot forgive part of the toxin.
            let budget = starving_below(&poisoned) / 2;
            set_energy(&mut poisoned, budget);
            set_energy(&mut clean, budget);
        }

        poisoned.apply(eat(prey));
        let outcome = clean.apply(eat(prey));
        assert_eq!(
            matches!(outcome, Outcome::Burned { .. }),
            starved,
            "the budget picked the route: {outcome:?}"
        );
        assert_eq!(
            clean.energy_mg().unwrap() - poisoned.energy_mg().unwrap(),
            VENOM,
            "the toxin cost exactly itself, starved: {starved}"
        );
    }
}

#[test]
fn being_nearly_starved_does_not_make_venom_safer() {
    // The hazard the previous ordering created. Subtracting venom before adding
    // the meal meant a critter at low energy lost part of the toxin to the zero
    // floor and then collected the full mass, so approaching death was a
    // discount on poison. Gains land first, so the toxin is paid in full.
    let (mut world, prey) = fed();

    let (mass, venom) = {
        let o = world.organisms.iter_mut().find(|o| o.id == prey).unwrap();
        o.venom_mg = 500;
        (o.biomass_mg(), o.venom_mg)
    };
    set_energy(&mut world, 10); // nearly starved
    assert!(
        venom > world.energy_mg().unwrap() + mass,
        "the toxin outweighs the meal"
    );

    let (_, fed) = apply_counting_the_ecologys_own_meal(&mut world, eat(prey));

    assert!(
        world.energy_mg().unwrap() <= fed,
        "a meal that cannot cover its venom leaves nothing: {} mg held against \
         the {fed} mg the enclosure fed it on the same tick",
        world.energy_mg().unwrap(),
    );
}

#[test]
fn two_starving_critters_pay_the_same_toxin() {
    // The same claim from the other side: within a route, the cost of a
    // venomous meal is a property of the meal, not of how desperate you were
    // when you ate it. Desperation changes where the meal *goes*; it never
    // discounts what it costs.
    let (base, prey) = fed();
    let line = starving_below(&base);

    let mut nearly = base.clone();
    let mut barely = base;
    for world in [&mut nearly, &mut barely] {
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == prey)
            .unwrap()
            .venom_mg = 60;
    }
    set_energy(&mut nearly, line / 8);
    set_energy(&mut barely, line - 1);

    // Measured on everything the body holds, budget and substance together,
    // less what the enclosure fed it on the same tick. A budget alone no longer
    // says it: the two are starved by different margins, so the ecology's own
    // bite lands in a different account in each, which is a fact about routing
    // rather than about what the venom cost.
    let held = |world: &World| {
        let me = world.controlled().expect("embodied");
        me.energy_mg + me.biomass_mg()
    };
    let (nearly_before, barely_before) = (held(&nearly), held(&barely));
    let (_, nearly_fed) = apply_counting_the_ecologys_own_meal(&mut nearly, eat(prey));
    let (_, barely_fed) = apply_counting_the_ecologys_own_meal(&mut barely, eat(prey));

    assert_eq!(
        held(&nearly) - nearly_before - nearly_fed,
        held(&barely) - barely_before - barely_fed,
        "the same meal nets the same amount at any starving budget"
    );
}

#[test]
fn a_refused_placement_costs_neither_the_meal_nor_its_venom() {
    // One transaction. An earlier cut removed the organism and charged venom
    // before the attachment was known to succeed, so a refusal ate the meal and
    // poisoned you for it.
    //
    // The ecology steps whether or not the played verb landed (TD5: nothing
    // exempts the held body from its own instinctive feeding), and that pass
    // now charges venom too, so a rejection is judged against an idle twin of
    // the same tick rather than against rent alone — otherwise the ecology's
    // own bite, not the refusal, would be on trial.
    let (mut world, prey) = fed();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == prey)
        .unwrap()
        .venom_mg = 90;
    let mut control = world.clone();

    let roster = world.organisms.len();
    let parts = world.body().unwrap().len();

    let outcome = world.apply(Intent::Metabolize {
        organism: prey,
        placement: Placement::Explicit {
            parent: mesocosm_core::PartId(9_999),
            offset: [2, 0, 0],
            yaw: mesocosm_core::Yaw::Zero,
        },
    });
    control.apply(Intent::Idle);

    assert_eq!(
        outcome,
        Outcome::Rejected(Rejection::NoSuchParent(mesocosm_core::PartId(9_999)))
    );
    assert_eq!(
        world.energy_mg().unwrap(),
        control.energy_mg().unwrap(),
        "the refusal itself cost nothing beyond what the same tick's own ecology would have"
    );
    assert_eq!(world.body().unwrap().len(), parts, "nothing was grown");
    assert!(
        world.organisms.iter().any(|o| o.id == prey),
        "and the meal is still there to eat"
    );
    assert!(world.organisms.len() >= roster.saturating_sub(1));
}

#[test]
fn a_refused_meal_leaves_the_world_untouched() {
    // Placement is resolved before the organism is consumed, so a body with
    // nowhere to put a part does not lose the meal as well.
    //
    // Judged against an idle twin of the same tick, not against rent alone:
    // the ecology's own instinctive feeding (and, since the NPC-venom fix,
    // its own venom) rides along on every apply() regardless of the played
    // verb, seed 4,242's founders included, so "rent only" is no longer a
    // safe prediction on its own.
    let (mut world, _) = fed();
    let mut control = world.clone();
    let absent = mesocosm_core::OrganismId(9_999);
    let roster = world.organisms.len();
    let parts = world.body().unwrap().len();

    let outcome = world.apply(eat(absent));
    control.apply(Intent::Idle);

    assert_eq!(
        outcome,
        Outcome::Rejected(Rejection::NoSuchOrganism(absent))
    );
    assert_eq!(world.body().unwrap().len(), parts, "nothing was grown");
    assert_eq!(
        world.energy_mg().unwrap(),
        control.energy_mg().unwrap(),
        "the refusal itself cost nothing beyond what the same tick's own ecology would have"
    );
    // The ecology still stepped, so the roster may change by birth or death,
    // but no organism was eaten by this refusal.
    assert!(world.organisms.len() >= roster.saturating_sub(1));
}

#[test]
fn both_routes_replay_identically() {
    // The determinism boundary the wing rests on, and the reason routing could
    // leave the intent at all: the budget that decides is world state, so it
    // snapshots, restores, and replays with everything else. A trace does not
    // have to carry the decision to reproduce it.
    for (world, prey) in [fed(), starving()] {
        let mut straight = world.clone();
        straight.apply(eat(prey));
        straight.apply(Intent::Move { delta: [1, 0, 0] });

        let mut forked = world;
        forked.apply(eat(prey));
        let bytes = snapshot(&forked).unwrap();
        let mut resumed = mesocosm_core::restore(&bytes).unwrap();
        resumed.apply(Intent::Move { delta: [1, 0, 0] });

        assert_eq!(
            state_hash(&straight),
            state_hash(&resumed),
            "the run replays"
        );
    }
}
