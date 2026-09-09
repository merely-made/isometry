// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Local affordance lookup and locomotion for ecology ticks.
//!
//! The spatial buckets are rebuilt from the tick's immutable reading. They
//! are an acceleration structure, never state: their only job is to avoid
//! asking every embodied body about every other body before a sight query.

use super::kinship::Kin;
use super::{dispersal_for, is_hungry, travels};
use crate::flow::Records;
use crate::history::Event;
use crate::organism::{
    FaunaDecisionTrace, FaunaDrive, FaunaSenses, FaunaTraits, LastSeen, Organism, OrganismId,
};
use crate::organism::{Kingdom, Signal};
use crate::places::{
    Ground, Places, Soil, Tier, WalkerShape, route_step_for, step_for as grounded_step,
    surface_stance_for,
};
use crate::process::{FeedingMode, NisKind};
use crate::rng::Rng;
use std::cmp::Reverse;

mod perception;

pub(super) use perception::{CarrionTarget, LivingTarget, carrion_cells, living_cells};
use perception::{
    Cells, can_perceive, can_perceive_position, forage_gradient, nearby_indexes, sight_range,
};

/// How far a consumer reaches for a meal, in voxel units.
pub(super) const GRAZE_RANGE: i32 = 5;
/// How far a decomposer reaches for the dead, in voxel units.
pub(super) const DECOMPOSE_RANGE: i32 = 6;
/// A remembered sight line can take a short detour, but never authorizes a
/// global navigation search in an ecology tick.
const MEMORY_ROUTE_BUDGET: i32 = 8;
/// Direct observation is fresh for this many failed perception ticks.
const MEMORY_TICKS: u8 = 8;

/// Chooses a food source within the body's actual reach. This is local for
/// both tiers, so it never needs a global scan.
pub(super) fn choose_living_target(
    organism: &Organism,
    living: &[LivingTarget],
    cells: &Cells,
    ground: Option<&Ground>,
    kin: &Kin,
) -> Option<usize> {
    let reach = GRAZE_RANGE + organism.body().reach();
    let sight = sight_range(organism, reach, ground);
    let observer_shape = organism.walker_shape();
    let hungry = is_hungry(organism);
    let mut candidates: Vec<(u64, usize, usize)> = Vec::new();
    for order in nearby_indexes(cells, organism.position, sight) {
        let Some(target) = living.get(order) else {
            continue;
        };
        if target.id == organism.id
            || !organism.admits(target.kingdom.nis_kind(), false)
            || chebyshev(organism.position, target.position) > reach
            || (target.signal != Signal::Plain && target.kingdom != Kingdom::Producer)
        {
            continue;
        }
        // **Kinship tempers the appetite** (TD10). The remove is in the same
        // voxels the distance term is, so kin read as further off rather than
        // as forbidden: a body of the eater's own line ranks behind everything
        // else in reach, and is still taken when it is the only thing there.
        let remove = kin.remove(organism.species, target.species, reach, hungry);
        let distance = (chebyshev(organism.position, target.position) + remove) as u64;
        let danger = u64::from(target.signal == Signal::Warning) * 4;
        let score =
            (distance.saturating_mul(16) + danger).saturating_sub(target.mass_mg.min(256) / 64);
        candidates.push((score, order, target.organism_index));
    }
    candidates.sort_unstable();
    candidates.into_iter().find_map(|(_, order, index)| {
        living
            .get(order)
            .is_some_and(|target| can_perceive(organism, observer_shape, target, sight, ground))
            .then_some(index)
    })
}

/// Carrion feeding is local too. Returning the original organism index keeps
/// the drain pass independent from the derived bucket representation.
pub(super) fn choose_carrion_target(
    organism: &Organism,
    carrion: &[CarrionTarget],
    cells: &Cells,
    ground: Option<&Ground>,
) -> Option<usize> {
    if !organism.admits(NisKind::Producer, true) {
        return None;
    }
    let observer_shape = organism.walker_shape();
    nearby_indexes(cells, organism.position, DECOMPOSE_RANGE)
        .filter_map(|order| carrion.get(order).map(|target| (order, target)))
        .filter(|(_, target)| {
            (0..3).all(|axis| {
                (target.position[axis] - organism.position[axis]).abs() <= DECOMPOSE_RANGE
            }) && can_perceive_position(
                organism,
                observer_shape,
                target.position,
                target.shape,
                DECOMPOSE_RANGE,
                ground,
            )
        })
        .min_by_key(|(order, _)| *order)
        .map(|(_, target)| target.organism_index)
}

fn chebyshev(from: [i32; 3], to: [i32; 3]) -> i32 {
    (0..3)
        .map(|axis| (from[axis] - to[axis]).abs())
        .max()
        .unwrap_or(0)
}

/// Finds a valid surface stance for a graph position, if it is resident in the
/// grown ground. Callers choose the fallback: a near-tier birth remains beside
/// its grounded parent when its scatter leaves the grown enclosure.
pub(super) fn surface_stance(
    ground: &Ground,
    shape: WalkerShape,
    position: [i32; 3],
) -> Option<[i32; 3]> {
    surface_stance_for(ground, shape, position)
}

fn preferred_living<'a>(
    organism: &Organism,
    candidates: impl Iterator<Item = (usize, &'a LivingTarget)>,
    ground: Option<&Ground>,
    sight: i32,
    kin: &Kin,
) -> Option<(OrganismId, [i32; 3])> {
    let observer_shape = organism.walker_shape();
    let hungry = is_hungry(organism);
    let mut ranked: Vec<(i32, u64, usize, &'a LivingTarget)> = candidates
        .filter(|(_, target)| {
            target.id != organism.id && organism.admits(target.kingdom.nis_kind(), false)
        })
        .map(|(order, target)| {
            // The same remove the bite applies (TD10), against sight rather
            // than reach: a hunter should not walk toward its own line either.
            (
                chebyshev(organism.position, target.position)
                    + kin.remove(organism.species, target.species, sight, hungry),
                target.mass_mg,
                order,
                target,
            )
        })
        .collect();
    ranked.sort_unstable_by_key(|(distance, mass, order, _)| (*distance, Reverse(*mass), *order));
    ranked.into_iter().find_map(|(_, _, _, target)| {
        can_perceive(organism, observer_shape, target, sight, ground)
            .then_some((target.id, target.position))
    })
}

fn policy_living<'a>(
    organism: &mut Organism,
    candidates: impl Iterator<Item = (usize, &'a LivingTarget)>,
    ground: &Ground,
    sight: i32,
    kin: &Kin,
) -> Option<MovementTarget> {
    let traits = FaunaTraits::read(organism);
    let own_mass = organism.biomass_mg();
    let policy = organism.fauna_policy;
    let observer_shape = organism.walker_shape();
    let hungry = is_hungry(organism);
    let candidate = candidates
        .filter(|(_, target)| {
            target.id != organism.id && organism.admits(target.kingdom.nis_kind(), false)
        })
        .filter_map(|(order, target)| {
            // Kin read as further away here too (TD10), and it is the same one
            // remove. The bite alone was measured not to reach: a hunter that
            // still *walked* toward its own line arrived where its own line was
            // the only thing it could see, and then had to eat it. The decision
            // trace below therefore records the discounted distance rather than
            // the geometric one, because that is the reading the body acted on.
            let distance = chebyshev(organism.position, target.position)
                + kin.remove(organism.species, target.species, sight, hungry);
            if !can_perceive(organism, observer_shape, target, sight, Some(ground)) {
                return None;
            }
            let senses = FaunaSenses::read(
                organism,
                traits,
                target.id,
                distance,
                target.mass_mg,
                target.signal,
            );
            let scores = policy.score(senses, own_mass, sight);
            let drive = scores.selected();
            let rank = (
                scores.score(drive),
                Reverse(distance),
                target.mass_mg,
                Reverse(order),
            );
            Some((rank, target, senses, scores, drive))
        })
        .max_by_key(|(rank, ..)| *rank);

    let Some((_, target, senses, scores, drive)) = candidate else {
        organism.last_fauna_decision = None;
        return None;
    };
    organism.fauna_policy.remember(scores);
    organism.last_fauna_decision = Some(FaunaDecisionTrace {
        traits,
        senses,
        selected_drive: drive,
        selected_target: Some(target.id),
        scores,
    });
    Some(match drive {
        FaunaDrive::Pursue => MovementTarget::Seen(target.id, target.position),
        FaunaDrive::Avoid => MovementTarget::Avoid(target.id, target.position),
        FaunaDrive::Hold => MovementTarget::Hold(target.id, target.position),
    })
}

#[derive(Clone, Copy)]
enum MovementTarget {
    Seen(OrganismId, [i32; 3]),
    Avoid(OrganismId, [i32; 3]),
    Hold(OrganismId, [i32; 3]),
    Other([i32; 3]),
}

fn preferred_carrion<'a>(
    organism: &Organism,
    candidates: impl Iterator<Item = (usize, &'a CarrionTarget)>,
    ground: Option<&Ground>,
    sight: i32,
) -> Option<[i32; 3]> {
    if !organism.admits(NisKind::Producer, true) {
        return None;
    }
    let observer_shape = organism.walker_shape();
    let mut ranked: Vec<(i32, usize, &'a CarrionTarget)> = candidates
        .map(|(order, target)| (chebyshev(organism.position, target.position), order, target))
        .collect();
    ranked.sort_unstable_by_key(|(distance, order, _)| (*distance, *order));
    ranked.into_iter().find_map(|(_, _, target)| {
        can_perceive_position(
            organism,
            observer_shape,
            target.position,
            target.shape,
            sight,
            ground,
        )
        .then_some(target.position)
    })
}

#[allow(clippy::too_many_arguments)]
fn preferred_target(
    organism: &mut Organism,
    living: &[LivingTarget],
    living_cells: &Cells,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    ground: Option<&Ground>,
    kin: &Kin,
) -> Option<MovementTarget> {
    match organism.feeding_mode() {
        FeedingMode::Grazer | FeedingMode::Predator | FeedingMode::Omnivore => {
            let reach = GRAZE_RANGE + organism.body().reach();
            let sight = sight_range(organism, reach, ground);
            let living = if let (Tier::Near, Some(ground)) = (organism.tier, ground) {
                policy_living(
                    organism,
                    nearby_indexes(living_cells, organism.position, sight)
                        .filter_map(|order| living.get(order).map(|target| (order, target))),
                    ground,
                    sight,
                    kin,
                )
            } else {
                organism.last_fauna_decision = None;
                preferred_living(organism, living.iter().enumerate(), ground, sight, kin)
                    .map(|(id, at)| MovementTarget::Seen(id, at))
            };
            living.or_else(|| preferred_carrion_target(organism, carrion, carrion_cells, ground))
        },
        FeedingMode::Scavenger => {
            preferred_carrion_target(organism, carrion, carrion_cells, ground)
        },
        FeedingMode::Producer => {
            organism.last_fauna_decision = None;
            None
        },
    }
}

fn preferred_carrion_target(
    organism: &mut Organism,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    ground: Option<&Ground>,
) -> Option<MovementTarget> {
    organism.last_fauna_decision = None;
    let sight = sight_range(organism, DECOMPOSE_RANGE + organism.body().reach(), ground);
    let target = if organism.tier == Tier::Near && ground.is_some() {
        preferred_carrion(
            organism,
            nearby_indexes(carrion_cells, organism.position, sight)
                .filter_map(|order| carrion.get(order).map(|target| (order, target))),
            ground,
            sight,
        )
    } else {
        preferred_carrion(organism, carrion.iter().enumerate(), ground, sight)
    };
    target.map(MovementTarget::Other)
}

fn remembered_target(
    organism: &mut Organism,
    living: &[LivingTarget],
    ground: Option<&Ground>,
) -> Option<[i32; 3]> {
    if organism.tier != Tier::Near || ground.is_none() {
        // A place-tier transition changes the embodied perception model. Do
        // not revive an old local sighting if this body later returns near.
        organism.last_seen = None;
        return None;
    }
    let memory = organism.last_seen?;
    if memory.ticks_left == 0
        || !living.iter().any(|target| {
            target.id == memory.target && organism.admits(target.kingdom.nis_kind(), false)
        })
    {
        organism.last_seen = None;
        return None;
    }
    organism.last_seen = Some(LastSeen {
        ticks_left: memory.ticks_left - 1,
        ..memory
    });
    Some(memory.position)
}

/// Moves an organism toward the affordance it currently needs. Near bodies
/// move by legal integer steps; far bodies traverse one place-graph edge.
#[allow(clippy::too_many_arguments)]
pub(super) fn disperse(
    organism: &mut Organism,
    places: &Places,
    ground: Option<&Ground>,
    rng: &mut Rng,
    soil: &mut Soil,
    living: &[LivingTarget],
    living_cells: &Cells,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    records: &mut Records<'_>,
    kin: &Kin,
) -> bool {
    let target = preferred_target(
        organism,
        living,
        living_cells,
        carrion,
        carrion_cells,
        ground,
        kin,
    );
    let (target, pursuing_memory) = match target {
        Some(MovementTarget::Seen(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (Some(at), false)
        },
        Some(MovementTarget::Avoid(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (
                Some([
                    organism.position[0] + (organism.position[0] - at[0]),
                    organism.position[1],
                    organism.position[2] + (organism.position[2] - at[2]),
                ]),
                false,
            )
        },
        Some(MovementTarget::Hold(id, at)) => {
            organism.last_seen = Some(LastSeen {
                target: id,
                position: at,
                ticks_left: MEMORY_TICKS,
            });
            (None, false)
        },
        Some(MovementTarget::Other(at)) => (Some(at), false),
        None => (remembered_target(organism, living, ground), true),
    };
    let old = organism.position;
    let shape = organism.walker_shape();
    // **No actuator, no travel** (TD8). Zeroing `dispersal_for` is necessary
    // and not sufficient: the hungry wander and the far tier's graph hop never
    // asked for a budget, so a body with nothing that contracts still walked —
    // and it walked at a plant's rent, which is the free lunch the ruling
    // withdraws. It reads its drives and its memory above, exactly as before;
    // what it cannot do is act on them by going somewhere.
    //
    // **Producers creep** (TD9). TD8 made the *stand* sessile as a side-effect,
    // because a producer is unlimbed by construction, and that retired the one
    // way a shaded plant had of leaving its own shade. `travels` restores it as
    // a rule about how a producer lives rather than about bodies without limbs,
    // so the free lunch stays withdrawn: an unlimbed consumer still reads false
    // here and still goes nowhere.
    let next = if !travels(organism) {
        organism.position
    } else if let Some(target) = target {
        if organism.tier == Tier::Far {
            let next = graph_step(places, organism.position, target);
            ground
                .and_then(|ground| surface_stance(ground, shape, next))
                .unwrap_or(next)
        } else if let Some(ground) = ground {
            walk_grounded(organism, ground, shape, target, pursuing_memory)
        } else {
            let mut at = organism.position;
            for _ in 0..dispersal_for(organism) {
                at = integer_step(at, target);
                if chebyshev(at, target) <= organism.body().reach() + GRAZE_RANGE {
                    break;
                }
            }
            at
        }
    } else if is_hungry(organism) {
        // **The size of the creep** (TD9). A producer gets no target from
        // `preferred_target`, so this branch is its entire travel budget: one
        // grounded voxel, only while its reserve is under `HUNGRY_UPKEEP_TICKS`
        // of rent, paid for in substance like every other step. A creeping body
        // never takes the place-graph hop below — a stand spreads out of its
        // own shade at the speed of growth, it does not relocate to the next
        // place — and with no ground under it there is nothing to creep across.
        let creeping = organism.actuator_span() == 0;
        if creeping && (organism.tier == Tier::Far || ground.is_none()) {
            organism.position
        } else if organism.tier == Tier::Far {
            let next = diffuse(places, organism.position, rng);
            ground
                .and_then(|ground| surface_stance(ground, shape, next))
                .unwrap_or(next)
        } else if let Some(ground) = ground {
            // **Hunger follows a gradient** (TD11), and it is exactly the same
            // *one* grounded voxel the random wander took — the direction
            // changes, never the size. See `perception::forage_gradient` for
            // the rule, for why a bucket centre is not a second sight, and for
            // what the pursuit budget cost when it was tried here instead.
            // A heading the ground refuses falls back to the wander: the
            // gradient has no route-finder, so it is the only thing here that
            // can get around an obstruction.
            let uphill =
                forage_gradient(organism, living, living_cells, carrion, carrion_cells, kin)
                    .map(|at| grounded_step(ground, shape, organism.position, at))
                    .filter(|next| *next != organism.position);
            match uphill {
                Some(next) => next,
                None => {
                    const WANDER: [[i32; 2]; 4] = [[1, 0], [-1, 0], [0, 1], [0, -1]];
                    let [dx, dz] = WANDER[rng.below(WANDER.len() as u64) as usize];
                    grounded_step(
                        ground,
                        shape,
                        organism.position,
                        [
                            organism.position[0] + dx,
                            organism.position[1],
                            organism.position[2] + dz,
                        ],
                    )
                },
            }
        } else {
            diffuse(places, organism.position, rng)
        }
    } else {
        organism.position
    };

    if next != old {
        super::flows::pay_travel(organism, soil, records, old, chebyshev(old, next) as u64);
        organism.position = next;
        records.event(
            next,
            Event::Moved {
                organism: organism.id,
                from: old,
                to: next,
            },
        );
        true
    } else {
        false
    }
}

/// One tick's grounded travel toward a place, spending the dispersal budget a
/// step at a time and stopping where the ground refuses or the body arrives.
///
/// Lifted out of `disperse` in TD11 so the hungry gradient walks by exactly the
/// same rule a pursuit does; the arithmetic is unchanged.
fn walk_grounded(
    organism: &Organism,
    ground: &Ground,
    shape: WalkerShape,
    target: [i32; 3],
    pursuing_memory: bool,
) -> [i32; 3] {
    let mut at = organism.position;
    for _ in 0..dispersal_for(organism) {
        let next = if pursuing_memory {
            route_step_for(ground, shape, at, target, MEMORY_ROUTE_BUDGET)
                .unwrap_or_else(|| grounded_step(ground, shape, at, target))
        } else {
            grounded_step(ground, shape, at, target)
        };
        if next == at {
            break;
        }
        at = next;
        if chebyshev(at, target) <= organism.body().reach() + GRAZE_RANGE {
            break;
        }
    }
    at
}

fn integer_step(from: [i32; 3], to: [i32; 3]) -> [i32; 3] {
    [
        from[0] + (to[0] - from[0]).signum(),
        from[1] + (to[1] - from[1]).signum(),
        from[2] + (to[2] - from[2]).signum(),
    ]
}

fn graph_step(places: &Places, position: [i32; 3], target: [i32; 3]) -> [i32; 3] {
    let Some(current) = places.at(position) else {
        return integer_step(position, target);
    };
    let Some(goal) = places.at(target) else {
        return integer_step(position, target);
    };
    let Some(next) = places
        .neighbours(current)
        .iter()
        .filter_map(|id| places.get(*id))
        .min_by_key(|place| places.hops(place.id, goal).unwrap_or(u32::MAX))
    else {
        return integer_step(position, target);
    };
    [next.centre[0], position[1], next.centre[1]]
}

fn diffuse(places: &Places, position: [i32; 3], rng: &mut Rng) -> [i32; 3] {
    let Some(current) = places.at(position) else {
        return position;
    };
    let neighbours = places.neighbours(current);
    if neighbours.is_empty() {
        return position;
    }
    let id = neighbours[rng.below(neighbours.len() as u64) as usize];
    let Some(place) = places.get(id) else {
        return position;
    };
    [place.centre[0], position[1], place.centre[1]]
}

#[cfg(test)]
mod tests;
