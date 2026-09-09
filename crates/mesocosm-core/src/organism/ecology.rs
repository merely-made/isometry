// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The enclosure living on its own schedule.
//!
//! Split out of `organism.rs` on 2026-08-01, because that file had grown past
//! this repo's six-hundred-line ceiling and the rule is to split before adding
//! rather than after. The types live next door; this is what happens to them
//! every tick.
//!
//! Nothing here knows which organism is being played. That is not an oversight;
//! it is the wing's third law holding at the level of the simulation.

use crate::cohort;
use crate::development::PartPalette;
use crate::flow::{Account, FlowEvent, Process, Records, Subject};
use crate::places::{FORAGE_RADIUS, Ground, Places, Soil, Tier, TierLine};
use crate::process::FeedingMode;
use crate::rng::Rng;
use crate::species::Lineages;

use crate::history::{Event, MealKind};
// The routing threshold lives with the played meal's rule in `world`, which is
// the point: TD5 makes it one rule rather than two that agree.
use crate::world::STARVED_UPKEEP_TICKS;

use super::{Kingdom, Organism, OrganismId, Stage, Tally};

mod breeding;
mod flows;
mod kinship;
mod movement;
mod rates;

use flows::{earn, record_intake, release_reserve};
use movement::{
    CarrionTarget, LivingTarget, carrion_cells, choose_carrion_target, choose_living_target,
    disperse, living_cells,
};

pub(crate) use breeding::filial_seed;
/// The two transactions DT3's dev intents reach, so a forced birth is the
/// ordinary birth and a dev-caused death is the ordinary death -- plus the
/// dispersal radius a birth scatters by, which `World::prospect` reads to
/// quote a founder preview against the poorest ground in reach.
pub use breeding::{BIRTH_SCATTER, bear};
pub use flows::perish;
pub(crate) use rates::*;
use rates::{CROWD_CELL, CROWD_COMFORT, UPKEEP_BASE_MG};

/// Which crowding cell a position falls in.
fn cell_of(position: [i32; 3]) -> (i32, i32) {
    (
        position[0].div_euclid(CROWD_CELL),
        position[2].div_euclid(CROWD_CELL),
    )
}

/// One bite: who reached for it, out of what, and how much they reached for.
///
/// Both sides are recorded by index and resolved in a pass of their own,
/// because the borrow that reads the pasture cannot also hold the eaten.
/// `mass_mg` is the mouthful attempted; what the prey could actually pay is
/// decided when the meal is settled.
struct Meal {
    eater: usize,
    prey: usize,
    mass_mg: u64,
    kind: MealKind,
}

/// Advances every organism one tick.
///
/// Deterministic: organisms are visited in id order, offspring are appended in
/// the order their parents produced them, and every random value comes from
/// the seeded stream.
/// One tick of the enclosure.
///
/// `records` collects what happened to *individuals* and what matter did.
/// [`Tally`] counts, which is what a host shows; the causal events are what a
/// history records, because significance needs to know who rather than how many;
/// and the flows are what the bounded ecology readings reduce.
#[allow(clippy::too_many_arguments)]
pub fn step(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    records: &mut Records<'_>,
    lineages: &Lineages,
    palette: PartPalette,
    soil: &mut Soil,
) -> Tally {
    step_inner(
        organisms, next_id, rng, records, lineages, palette, soil, None, None, None, None,
    )
}

/// Advances the enclosure with place-graph ownership enabled. The plain
/// [`step`] entry point remains useful for isolated ecology fixtures; worlds
/// use this path so dispersal and tier hysteresis are part of replayed state.
#[allow(clippy::too_many_arguments)]
pub fn step_with_places(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    records: &mut Records<'_>,
    lineages: &Lineages,
    palette: PartPalette,
    soil: &mut Soil,
    places: &Places,
    focus: Option<[i32; 3]>,
) -> Tally {
    step_inner(
        organisms,
        next_id,
        rng,
        records,
        lineages,
        palette,
        soil,
        Some(places),
        None,
        focus,
        None,
    )
}

/// Advances a world-owned ecology against its voxel truth. Graph-only
/// fixtures keep using [`step_with_places`]; a replayed world calls this path
/// so embodied bodies obey the same footing and sight rules as the player.
///
/// `held` names the one organism a hand is currently on. It is the **only**
/// thing here that knows a player exists, and it is deliberately narrow: a
/// held body still ages, still pays rent, and still dies. All it is spared is
/// being walked somewhere it did not choose to go, so the keys and the
/// instincts never fight over the same body on the same tick. `None` — nobody
/// embodied, or a hand that has been still long enough to have let go — leaves
/// every organism on its own drives, which is the terrarium's resting state.
#[allow(clippy::too_many_arguments)]
pub fn step_with_ground(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    records: &mut Records<'_>,
    lineages: &Lineages,
    palette: PartPalette,
    soil: &mut Soil,
    places: &Places,
    ground: &Ground,
    focus: Option<[i32; 3]>,
    held: Option<OrganismId>,
) -> Tally {
    step_inner(
        organisms,
        next_id,
        rng,
        records,
        lineages,
        palette,
        soil,
        Some(places),
        Some(ground),
        focus,
        held,
    )
}

#[allow(clippy::too_many_arguments)]
fn step_inner(
    organisms: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    records: &mut Records<'_>,
    lineages: &Lineages,
    palette: PartPalette,
    soil: &mut Soil,
    places: Option<&Places>,
    ground: Option<&Ground>,
    focus: Option<[i32; 3]>,
    held: Option<OrganismId>,
) -> Tally {
    let mut tally = Tally::default();

    // Tier ownership is hysteretic state. Forming the far summary here makes
    // cohort conservation part of the tick's receipt rather than an
    // unaccounted host cache.
    if let (Some(places), Some(focus)) = (places, focus) {
        for organism in organisms.iter_mut().filter(|o| o.is_alive()) {
            let previous = organism.tier;
            organism.tier = TierLine::default().tick(places, previous, organism.position, focus);
            match (previous, organism.tier) {
                (Tier::Far, Tier::Near) => tally.promoted += 1,
                (Tier::Near, Tier::Far) => tally.demoted += 1,
                _ => {},
            }
        }
        let far = cohort::from_organisms(organisms, places);
        let (members, biomass, _) = cohort::conserved_totals(&far);
        tally.far_cohorts = far.len() as u32;
        tally.far_members = members as u32;
        tally.far_biomass_mg = biomass;
    }

    // Crowding, counted once per tick on a coarse grid. **TD6 asked whether the
    // soil made this redundant and measured the answer: no.** The soil bounds a
    // stand's *mass* — that is the fixed point the round was for — but a closed
    // matter budget says nothing about how many bodies that mass is divided
    // into, and a stand with crowding removed answered a finite enclosure by
    // subdividing: 620 producers and still climbing at the horizon, whatever
    // the soil was seeded with. Density is the job crowding still does; it just
    // is no longer the only regulator, and it no longer needs a rent floor
    // under it, because a shaded producer now simply draws less out of a
    // column instead of being handed a minimum from nowhere.
    let mut density: std::collections::BTreeMap<(i32, i32), u32> =
        std::collections::BTreeMap::new();
    for organism in organisms.iter().filter(|o| o.is_alive()) {
        *density.entry(cell_of(organism.position)).or_default() += 1;
    }

    // Carrion positions are read before anything changes, so decomposers all
    // see the same world within a tick rather than racing each other.
    let carrion: Vec<CarrionTarget> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.stage == Stage::Carrion)
        .map(|(index, o)| CarrionTarget {
            position: o.position,
            organism_index: index,
            shape: o.walker_shape(),
        })
        .collect();
    let carrion_cells = carrion_cells(&carrion);

    // Living bodies, read before anything changes, so feeding decisions all
    // see the same enclosure within a tick rather than racing each other.
    let living: Vec<LivingTarget> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.is_alive())
        .map(|(index, o)| LivingTarget {
            id: o.id,
            position: o.position,
            organism_index: index,
            kingdom: o.kingdom(),
            species: o.species,
            mass_mg: o.biomass_mg(),
            signal: o.signal,
            shape: o.walker_shape(),
        })
        .collect();
    let living_cells = living_cells(&living);

    // **Kinship tempers the appetite** (TD10). Built once per tick and read by
    // both the bite and the walk toward one; see `kinship.rs` for the rule.
    let kin = kinship::Kin::new(lineages);

    let mut meals: Vec<Meal> = Vec::new();
    let mut newborns: Vec<Organism> = Vec::new();

    // **Rent, then the bite each body reaches for.** Nothing is eaten in this
    // pass: resolving a meal moves mass out of one body and into another, and
    // this borrow can only hold one of them. Producers are the exception and
    // settle here, because their counterparty is the ground rather than a
    // second body.
    for (index, organism) in organisms.iter_mut().enumerate() {
        organism.age = organism.age.saturating_add(1);
        if !organism.is_alive() {
            continue;
        }
        let column = soil.column_at(organism.position);
        // One anatomy read a tick, reused by every record this body writes.
        let subject = Subject::of(organism);
        let at = organism.position;

        // Everything alive pays rent, every tick, **and the rent scales with
        // the body**. Budget first, then the body itself: a creature with
        // nothing left to spend eats itself. Either way the milligrams go back
        // to the ground it is standing on — a body that burns itself has to
        // put that mass somewhere, and that somewhere is the cycle. (TD6)
        let owed = organism.upkeep_mg();
        let rent = organism.pay_upkeep();
        soil.deposit(column, owed - rent.unpaid_mg);
        for (out_of, mg) in [
            (Account::Reserve, rent.reserve_mg),
            (Account::Substance, rent.substance_mg),
        ] {
            records.flow(
                at,
                FlowEvent::returned(Process::Upkeep, subject, out_of, mg),
            );
        }

        // Nothing draws more than it can hold: past the body plan's adult mass
        // and a full reserve, more matter would only be handed straight back.
        // (TD6 determinate growth)
        let room = organism.intake_room_mg();

        match organism.feeding_mode() {
            // Producers make biomass out of the ground, drawn rather than
            // minted, which is what makes runaway growth impossible rather
            // than merely discouraged (TD6). **Roots forage** (TD7): the read
            // is a whole neighbourhood, the draw is one tick's income out of
            // the richest column in it — wide reach at the speed of growth,
            // never the radius' worth of columns at once.
            FeedingMode::Producer => {
                // An inactive or severed intake declaration also reads as
                // `Producer`; only a living Fix allocation may draw soil.
                if organism.phenotype.canopy() {
                    let crowd = density
                        .get(&cell_of(organism.position))
                        .copied()
                        .unwrap_or(1)
                        .max(1);
                    let income = producer_income_for_mass(organism.biomass_mg());
                    // Floored at rent, as it has been since TD2. A shaded-out
                    // producer stagnates rather than starving, because otherwise
                    // a whole stand of identical plants crosses the starvation
                    // line on the same tick and the patch goes extinct instead of
                    // thinning. The floor is not a hand-out any more: it is a
                    // *request*, and the column answers it or does not. (TD6)
                    let want = (income * u64::from(CROWD_COMFORT) / u64::from(crowd))
                        .clamp(UPKEEP_BASE_MG, income)
                        .min(room);
                    let drawn = soil.draw_richest_within(column, FORAGE_RADIUS, want);
                    let landed = earn(organism, drawn);
                    soil.deposit(column, landed.spilled_mg);
                    record_intake(records, at, None, subject, &landed);
                }
            },
            // **The bite scales with build** (TD9): the mouthful reads the same
            // three body-plan numbers the rent above reads, so the body that
            // pays for its machinery is the body that gets to use it.
            FeedingMode::Grazer | FeedingMode::Predator | FeedingMode::Omnivore => {
                let amount = feeding_rate_for_body(
                    organism.biomass_mg(),
                    organism.actuator_span(),
                    organism.mass_ceiling_mg(),
                )
                .min(room);
                if amount > 0
                    && let Some(prey) =
                        choose_living_target(organism, &living, &living_cells, ground, &kin)
                {
                    let kind = if living
                        .iter()
                        .find(|target| target.organism_index == prey)
                        .is_some_and(|target| target.kingdom == Kingdom::Producer)
                    {
                        MealKind::Grazing
                    } else {
                        MealKind::Predation
                    };
                    meals.push(Meal {
                        eater: index,
                        prey,
                        mass_mg: amount,
                        kind,
                    });
                } else if amount > 0
                    && let Some(source) =
                        choose_carrion_target(organism, &carrion, &carrion_cells, ground)
                {
                    meals.push(Meal {
                        eater: index,
                        prey: source,
                        mass_mg: decay_rate_for_body(
                            organism.biomass_mg(),
                            organism.actuator_span(),
                            organism.mass_ceiling_mg(),
                        )
                        .min(room),
                        kind: MealKind::Scavenging,
                    });
                }
            },
            // Decomposers only earn where something has died.
            FeedingMode::Scavenger => {
                let amount = decay_rate_for_body(
                    organism.biomass_mg(),
                    organism.actuator_span(),
                    organism.mass_ceiling_mg(),
                )
                .min(room);
                if amount > 0
                    && let Some(source) =
                        choose_carrion_target(organism, &carrion, &carrion_cells, ground)
                {
                    meals.push(Meal {
                        eater: index,
                        prey: source,
                        mass_mg: amount,
                        kind: MealKind::Scavenging,
                    });
                }
            },
        }
    }

    // **The meals, one at a time, both bodies in hand.** What the eater is
    // credited is exactly what came out of the prey — no more, because two
    // grazers can reach the same small producer and the second one finds less
    // than it bit for. Crediting first and reconciling afterwards is what
    // conjured matter here before, and there is no reconciliation that is
    // always payable.
    for meal in &meals {
        let prey_mass_mg = organisms[meal.prey].biomass_mg();
        let taken = meal.mass_mg - organisms[meal.prey].spend_mass(meal.mass_mg);
        if taken == 0 {
            continue;
        }
        let from = organisms[meal.prey].id;
        let prey_at = organisms[meal.prey].position;
        let prey_column = soil.column_at(prey_at);
        // PD2: the inherited toxin plus whatever the prey's glands are making
        // on the ground it is standing on. Read before the deposits below,
        // because charging the gland is a question about the column as it was
        // when the bite landed.
        let venom_mg = organisms[meal.prey].bite_mg(soil.matter_mg(prey_column));
        let prey = Subject::of(&organisms[meal.prey]);
        let column = soil.column_at(organisms[meal.eater].position);
        let eater = &mut organisms[meal.eater];
        let eater_id = eater.id;
        let eater_subject = Subject::of(eater);
        let at = eater.position;
        let landed = earn(eater, taken);
        soil.deposit(column, landed.spilled_mg);
        record_intake(records, at, Some(prey), eater_subject, &landed);
        // What the mouth could not hold left the prey all the same, so it is
        // the prey's substance the ground got.
        records.flow(
            at,
            FlowEvent::returned(Process::Spill, prey, Account::Substance, landed.spilled_mg),
        );
        // Gains before costs, same order act.rs's played meal settled on: a
        // nearly starved eater must not lose part of the toxin to the zero
        // floor and then still bank the whole bite. A bite doses by the
        // fraction of the flesh it took, not a flat per-meal tax, so an NPC
        // that nibbles a venomous body across many ticks pays what the whole
        // body would have cost, split by mouthful. Energy is unsigned, so a
        // dose beyond what the eater has is forgiven, not owed — matching
        // act.rs's floor. What it actually paid returns to the column under
        // the prey, not the eater: nothing evaporates. (closes the live
        // inconsistency: only the played meal charged venom before this)
        let dose = venom_mg.saturating_mul(taken) / prey_mass_mg.max(1);
        let before_venom = eater.energy_mg;
        eater.energy_mg = before_venom.saturating_sub(dose);
        let paid = before_venom - eater.energy_mg;
        soil.deposit(prey_column, paid);
        records.flow(
            prey_at,
            FlowEvent::returned(Process::Spill, eater_subject, Account::Reserve, paid),
        );
        records.event(
            at,
            Event::Fed {
                eater: eater_id,
                from,
                mass_mg: taken,
                kind: meal.kind,
            },
        );
    }

    // **What the tick did to each body**, once every ledger has settled.
    for organism in organisms.iter_mut() {
        let column = soil.column_at(organism.position);
        match organism.stage {
            Stage::Juvenile | Stage::Mature => {
                // The hand's body is not driven. Everything above still
                // happened to it — rent, income, the meal it could reach —
                // and everything below still will; only locomotion is the
                // player's while the player is there. (TD4)
                if let Some(places) = places
                    && held != Some(organism.id)
                    && disperse(
                        organism,
                        places,
                        ground,
                        rng,
                        soil,
                        &living,
                        &living_cells,
                        &carrion,
                        &carrion_cells,
                        records,
                        &kin,
                    )
                {
                    tally.moved += 1;
                }

                if organism.stage == Stage::Juvenile
                    && organism.age >= maturity_for_mass(organism.life_history_mass_mg())
                {
                    organism.stage = Stage::Mature;
                    records.event(
                        organism.position,
                        Event::Matured {
                            organism: organism.id,
                        },
                    );
                    tally.matured += 1;
                }

                organism.since_offspring = organism.since_offspring.saturating_add(1);

                let starved = organism.biomass_mg() <= STARVATION_MG;
                let aged = organism.age >= lifespan_for_mass(organism.life_history_mass_mg());
                if organism.biomass_mg() == 0 {
                    // Eaten to nothing. There is no corpse to leave, so this
                    // skips carrion entirely and returns straight to the world.
                    organism.stage = Stage::Spent;
                    release_reserve(organism, soil, records);
                    records.event(
                        organism.position,
                        Event::Returned {
                            organism: organism.id,
                        },
                    );
                    tally.returned += 1;
                } else if starved || aged {
                    // Through the one death, which `Intent::Kill` also reaches
                    // (DT3): the corpse and the record it leaves cannot tell
                    // which door asked.
                    flows::perish(organism, soil, records);
                    tally.died += 1;
                }
            },

            Stage::Carrion => {
                // The dead return whether or not a decomposer is present, just
                // far more slowly. Locked matter is a real failure mode — and
                // since TD6 "return" is literal: the milligram goes into the
                // column the body is lying on.
                //
                // **TD8 slows this arm, and only this arm.** A corpse was
                // returning a milligram every tick, so it was an event a
                // decomposer had to be standing next to; one milligram every
                // `CARRION_DECAY_TICKS` makes it a standing resource. `age`
                // keeps counting after death, so each corpse carries its own
                // phase and the enclosure does not decay in lockstep. The
                // scavenger's own draw (`decay_rate_for_body`) is untouched:
                // the yield lever was measured and ruled out, this is duration.
                let returning = u64::from(organism.age.is_multiple_of(CARRION_DECAY_TICKS));
                let unreturned = organism.spend_mass(returning);
                soil.deposit(column, returning - unreturned);
                records.flow(
                    organism.position,
                    FlowEvent::returned(
                        Process::Decay,
                        Subject::of(organism),
                        Account::Substance,
                        returning - unreturned,
                    ),
                );
                if organism.biomass_mg() == 0 {
                    organism.stage = Stage::Spent;
                    records.event(
                        organism.position,
                        Event::Returned {
                            organism: organism.id,
                        },
                    );
                    tally.returned += 1;
                }
            },

            Stage::Spent => {},
        }
    }

    breeding::breed(
        organisms,
        &mut newborns,
        next_id,
        rng,
        records,
        lineages,
        palette,
        ground,
        &mut tally,
    );

    organisms.extend(newborns);
    organisms.retain(|o| o.stage != Stage::Spent);

    // The ground settles, after everything the tick put into it. Pure
    // transport between columns: no matter is made or lost here.
    soil.percolate()
        .expect("conserved soil transport fits the finite world");
    tally
}

#[cfg(test)]
mod tests;
