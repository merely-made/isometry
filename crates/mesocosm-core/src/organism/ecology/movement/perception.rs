// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ephemeral target views for one ecology tick.
//!
//! Positions and live body shapes are copied before any organism changes, so
//! every decision sees the same enclosure. None of this is stored world state.

use std::collections::BTreeMap;

use crate::body::SpeciesId;
use crate::organism::ecology::kinship::Kin;
use crate::organism::ecology::sight_for_body;
use crate::organism::{Kingdom, Organism, OrganismId, Signal};
use crate::places::{Ground, Tier, WalkerShape, spot_for};
use crate::process::{FeedingMode, NisKind};

/// An embodied mind's local visual horizon, for a body with no sense organ at
/// all. **The reference and the floor** since TD11, not the flat cap it was:
/// [`sight_range`] scales it by the body's own sensory build, so a blind plan
/// reads exactly this and every other body reads more. Terrain sight is still
/// local, and [`can_perceive`] still decides what is actually visible.
pub(super) const NEAR_SIGHT_RANGE: i32 = 8;
/// Target-query cells are finer than crowding cells: they bound perception
/// work, while crowding deliberately groups a wider ecological neighbourhood.
const SENSORY_CELL: i32 = 4;

#[derive(Clone, Copy, Debug)]
pub(in crate::organism::ecology) struct LivingTarget {
    pub(in crate::organism::ecology) id: OrganismId,
    pub(in crate::organism::ecology) position: [i32; 3],
    pub(in crate::organism::ecology) organism_index: usize,
    /// What this body is edible *as*, read off its own feeding anatomy since
    /// DC1.5 rather than off its silhouette. A grazer takes what fixes, so the
    /// question "is there a leaf on it" is now the same question in both
    /// directions: what a body earns by and what it can be eaten as are one
    /// reading of one anatomy. The rulings register's mixotroph case (§13)
    /// stays deferred — no body can honestly do both yet, so nothing is hidden
    /// from a grazer that ought to be visible to it.
    pub(in crate::organism::ecology) kingdom: Kingdom,
    /// Which line it belongs to, so an eater can tell kin from a stranger.
    /// (TD10)
    pub(in crate::organism::ecology) species: SpeciesId,
    pub(in crate::organism::ecology) mass_mg: u64,
    pub(in crate::organism::ecology) signal: Signal,
    pub(in crate::organism::ecology) shape: WalkerShape,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::organism::ecology) struct CarrionTarget {
    pub(in crate::organism::ecology) position: [i32; 3],
    pub(in crate::organism::ecology) organism_index: usize,
    pub(in crate::organism::ecology) shape: WalkerShape,
}

pub(super) type Cells = BTreeMap<(i32, i32), Vec<usize>>;

fn sensory_cell(position: [i32; 3]) -> (i32, i32) {
    (
        position[0].div_euclid(SENSORY_CELL),
        position[2].div_euclid(SENSORY_CELL),
    )
}

pub(in crate::organism::ecology) fn living_cells(living: &[LivingTarget]) -> Cells {
    cells(living.iter().map(|target| target.position))
}

pub(in crate::organism::ecology) fn carrion_cells(carrion: &[CarrionTarget]) -> Cells {
    cells(carrion.iter().map(|target| target.position))
}

fn cells(positions: impl Iterator<Item = [i32; 3]>) -> Cells {
    let mut cells = Cells::new();
    for (index, position) in positions.enumerate() {
        cells.entry(sensory_cell(position)).or_default().push(index);
    }
    cells
}

/// Candidate indexes in the horizontal cells intersecting a local range.
/// Buckets are BTree-ordered; caller tie-breaks retain original vector order.
pub(super) fn nearby_indexes(
    cells: &Cells,
    position: [i32; 3],
    range: i32,
) -> impl Iterator<Item = usize> + '_ {
    let (min_x, min_z) = sensory_cell([position[0] - range, position[1], position[2] - range]);
    let (max_x, max_z) = sensory_cell([position[0] + range, position[1], position[2] + range]);
    cells
        .range((min_x, i32::MIN)..=(max_x, i32::MAX))
        .filter(move |((_, z), _)| *z >= min_z && *z <= max_z)
        .flat_map(|(_, indexes)| indexes.iter().copied())
}

/// How far this body searches, in voxels.
///
/// **Sight reads the body** (TD11). The near tier's cap is no longer a flat
/// eight for every anatomy: it is [`sight_for_body`], the same build multiple
/// TD7's rent and TD9's bite divide by, handed the body's sensory span. A body
/// with no sense organ still reads `NEAR_SIGHT_RANGE`, so nothing about the
/// blind case moved.
///
/// **The near cap no longer clamps by `reach`,** and that is the second half of
/// the ruling rather than a tidy-up. The old cap was `min(reach, 8)`, so a
/// body could only look as far as it could grab — which is the very conflation
/// TD10's finding named ("forages at eight, bites at fifty") with the sign
/// reversed. Measured before it was removed: seed 2 is the one seed whose
/// consumers carry sense organs, 99% of them, and the clamp threw away every
/// voxel the derivation gave them (sensory span 12, derived 12, reach 8,
/// answer 8). Sight is a sensory reading; clamping it by an actuator span is
/// reading the wrong tissue. The far tier still answers `reach`, unchanged,
/// because out there the search span *is* the whole model.
///
/// Nothing loses horizon: for a blind body the answer is `NEAR_SIGHT_RANGE`
/// exactly, and it was `min(reach, NEAR_SIGHT_RANGE)` before — so only bodies
/// with anatomy to read gain anything.
pub(super) fn sight_range(organism: &Organism, reach: i32, ground: Option<&Ground>) -> i32 {
    if organism.tier == Tier::Near && ground.is_some() {
        sight_for_body(
            NEAR_SIGHT_RANGE,
            organism.sensor_span(),
            organism.mass_ceiling_mg(),
        )
    } else {
        reach
    }
}

/// The heading a hungry body takes when it can resolve nothing at all. (TD11)
///
/// **Hunger follows a gradient.** The old fallback was one random voxel, which
/// is a body with an empty window doing a random walk over an enclosure whose
/// pasture TD10 measured sitting 8 to 17 voxels off. A producer already has the
/// analogous organ — `Soil::draw_richest_within` reads a radius of columns and
/// takes the best one — so a consumer gets the same shape of read over what it
/// has: the sensory buckets, which the tick builds anyway.
///
/// **It is a gradient, not a second sight, and that is what keeps the raycast
/// gate honest.** The answer is a *bucket centre*, never a body: it says life
/// is denser that way and nothing else. Nothing here can be pursued, bitten, or
/// remembered — [`can_perceive`] still decides every one of those, and a body
/// that walks up a gradient into an occluded stand still sees nothing when it
/// arrives. The horizon is the span the body was already searching in with the
/// near cap taken off (bite reach, or the decomposer's), so no new number: a
/// body that bites at fifty also *smells* at fifty, which is the exact
/// asymmetry TD10's sixth finding named.
///
/// **It composes with the kinship discount** (TD10): a bucket's weight counts
/// only what this body would actually eat, and a body of the eater's own line
/// scores zero, so the gradient cannot point a cohort at itself. Distant kin,
/// whose remove has already halved away to nothing, count as ordinary pasture —
/// the same reading the bite makes.
///
/// **One voxel, as the random step was.** The caller walks the heading with a
/// single `grounded_step`, not the pursuit budget: a body with nothing in sight
/// is nearly out of reserve, and the pursuit budget charged it several
/// milligrams a tick to search. Measured on the way in — seed 1's decomposers
/// scavenged 4,220 mg against TD10's 197,347 and were gone by tick 200.
///
/// Returns `None` when the richest bucket is the one the body is standing in
/// (there is nothing to close distance to, so the caller's random step is the
/// honest move) or when nothing in the horizon is edible at all.
pub(super) fn forage_gradient(
    organism: &Organism,
    living: &[LivingTarget],
    living_cells: &Cells,
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
    kin: &Kin,
) -> Option<[i32; 3]> {
    let mode = organism.feeding_mode();
    let at = organism.position;
    match mode {
        FeedingMode::Grazer | FeedingMode::Predator | FeedingMode::Omnivore => {
            let horizon = super::GRAZE_RANGE + organism.body().reach();
            let edible_live = |target: &LivingTarget| edible(organism, target, horizon, kin);
            if living.iter().any(edible_live) {
                densest_cell(at, living_cells, horizon, |index| {
                    living
                        .get(index)
                        .map_or(0, |target| u32::from(edible_live(target)))
                })
            } else {
                carrion_gradient(organism, at, carrion, carrion_cells)
            }
        },
        FeedingMode::Scavenger => carrion_gradient(organism, at, carrion, carrion_cells),
        // A producer has no edible set, so it keeps TD9's random creep.
        FeedingMode::Producer => None,
    }
}

fn carrion_gradient(
    organism: &Organism,
    at: [i32; 3],
    carrion: &[CarrionTarget],
    carrion_cells: &Cells,
) -> Option<[i32; 3]> {
    let horizon = super::DECOMPOSE_RANGE + organism.body().reach();
    densest_cell(at, carrion_cells, horizon, |index| {
        u32::from(carrion.get(index).is_some() && organism.admits(NisKind::Producer, true))
    })
}

/// Whether a living target counts toward the gradient: the same edibility the
/// bite asks, minus the geometry, plus TD10's kin test as a hard zero.
fn edible(organism: &Organism, target: &LivingTarget, horizon: i32, kin: &Kin) -> bool {
    target.id != organism.id
        && organism.admits(target.kingdom.nis_kind(), false)
        && (target.signal == Signal::Plain || target.kingdom == Kingdom::Producer)
        // A gradient toward your own line is not a fix. `hungry` is true by
        // construction — this only runs inside the tick's hunger horizon — so
        // the remove is already the forgiving one the bite would apply.
        && kin.remove(organism.species, target.species, horizon, true) == 0
}

/// The centre of the richest sensory bucket in the **nearest ring that holds
/// anything**, or `None` if that is the bucket the body already stands in.
///
/// **Nearest ring first, richest inside it** — the same lexicographic shape
/// `preferred_living` ranks by, and for the same reason. A plain "densest
/// anywhere in the horizon" read was written first and measured: it is a
/// long-range attractor, and it emptied seed 1's decomposer kingdom by tick 200
/// (4,220 scavenged milligrams against TD10's 197,347) by marching every body
/// past the corpse beside it toward one distant heap. A bucket is four voxels
/// wide, so a ring is already a coarse read: the choice inside it is the
/// gradient, and the ring ordering is what keeps it local.
///
/// Ties go to the lowest bucket key, which the BTree hands over in ascending
/// order — so a body between two equal stands picks the same one every replay.
fn densest_cell(
    position: [i32; 3],
    cells: &Cells,
    range: i32,
    weight: impl Fn(usize) -> u32,
) -> Option<[i32; 3]> {
    let (min_x, min_z) = sensory_cell([position[0] - range, position[1], position[2] - range]);
    let (max_x, max_z) = sensory_cell([position[0] + range, position[1], position[2] + range]);
    let here = sensory_cell(position);
    let mut best: Option<((i32, i32), i32, u32)> = None;
    for ((x, z), indexes) in cells.range((min_x, i32::MIN)..=(max_x, i32::MAX)) {
        if *z < min_z || *z > max_z {
            continue;
        }
        let mass: u32 = indexes.iter().map(|index| weight(*index)).sum();
        if mass == 0 {
            continue;
        }
        let gap = (x - here.0).abs().max((z - here.1).abs());
        if best.is_none_or(|(_, held_gap, held)| gap < held_gap || (gap == held_gap && mass > held))
        {
            best = Some(((*x, *z), gap, mass));
        }
    }
    let ((x, z), _, _) = best?;
    ((x, z) != here).then_some([
        x * SENSORY_CELL + SENSORY_CELL / 2,
        position[1],
        z * SENSORY_CELL + SENSORY_CELL / 2,
    ])
}

pub(super) fn can_perceive(
    organism: &Organism,
    observer_shape: WalkerShape,
    target: &LivingTarget,
    range: i32,
    ground: Option<&Ground>,
) -> bool {
    can_perceive_position(
        organism,
        observer_shape,
        target.position,
        target.shape,
        range,
        ground,
    )
}

pub(super) fn can_perceive_position(
    organism: &Organism,
    observer_shape: WalkerShape,
    target: [i32; 3],
    target_shape: WalkerShape,
    range: i32,
    ground: Option<&Ground>,
) -> bool {
    match (organism.tier, ground) {
        (Tier::Near, Some(ground)) => spot_for(
            ground,
            observer_shape,
            organism.position,
            target_shape,
            target,
            range,
        ),
        _ => true,
    }
}
