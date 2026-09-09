// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The tick's birth pass.
//!
//! Split out of `ecology.rs` on 2026-08-29 when TD6's matter cycle pushed that
//! file to the six-hundred-line ceiling — the same split-before-adding move
//! that put `ecology::tests`, `ecology::movement` and `ecology::rates` in files
//! of their own. What stayed next door is rent, feeding, dispersal and death;
//! what moved here is the one pass that makes a new body, which has its own
//! concerns: the filial stream, recipe realization, and the scatter.

use crate::development::PartPalette;
use crate::flow::{Account, FlowEvent, Process, Records, Subject};
use crate::history::Event;
use crate::places::{Ground, Tier};
use crate::rng::Rng;
use crate::species::Lineages;

use super::rates::OFFSPRING_COST;
use super::{Organism, OrganismId, Stage, Tally, movement::surface_stance};

/// How far a birth may land from its parent, on either horizontal axis.
///
/// The **dispersal neighbourhood**: a birth lands somewhere in the square of
/// `(2 * BIRTH_SCATTER + 1)` columns a side centred on the parent, clamped at
/// the enclosure wall. Twelve because a stand has to be able to escape its own
/// shade; a short throw traps every offspring in the competition its parent is
/// already losing.
///
/// Named rather than written twice, because since 2026-09-04 a second place
/// reads it: [`World::prospect`](crate::World) quotes a founder preview
/// against the **poorest** ground in this square rather than the column under
/// the parent, so the quote never exceeds what a birth in reach can afford
/// (playable ecology plan PE3, ruled by Mark 2026-09-02). Two constants that
/// could drift apart would make that a coincidence instead of a guarantee.
pub const BIRTH_SCATTER: i32 = 12;

/// Reproduction, after everything has been fed, so a tick's births do not
/// depend on where in the list a parent happened to sit.
///
/// Newborns are appended by the caller, after this returns, so a child cannot
/// be born into the same pass that made it.
#[allow(clippy::too_many_arguments)]
pub(super) fn breed(
    organisms: &mut [Organism],
    newborns: &mut Vec<Organism>,
    next_id: &mut u32,
    rng: &mut Rng,
    records: &mut Records<'_>,
    lineages: &Lineages,
    palette: PartPalette,
    ground: Option<&Ground>,
    tally: &mut Tally,
) {
    let ready: Vec<usize> = organisms
        .iter()
        .enumerate()
        .filter(|(_, o)| o.can_reproduce())
        .map(|(index, _)| index)
        .collect();
    for index in ready {
        if let Some(child) = bear(
            organisms, index, next_id, rng, records, lineages, palette, ground,
        ) {
            newborns.push(child);
            tally.born += 1;
        }
    }
}

/// **One birth, and the only one there is.**
///
/// Split out of [`breed`]'s loop for DT3, which needs a birth from a named
/// parent *now* and must not have a second one written for it: the dev tools
/// plan's stop rule is that a forced birth is the ordinary birth. `breed` calls
/// this for every parent the timing gate admits; `Intent::ForceBirth` calls it
/// for the one parent a hand named, and skips nothing else.
///
/// Returns the newborn for the caller to append — a child cannot be born into
/// the pass that made it — having already debited the parent and written both
/// records. `None` is the birth not happening: this line is not in the registry,
/// or **provisioning would not cover the recipe**, which is the binding
/// condition a natural birth waits on rather than a rule this door invented.
/// It spends no entropy when it refuses, so a refused birth cannot move a later
/// one.
#[allow(clippy::too_many_arguments)]
pub fn bear(
    organisms: &mut [Organism],
    index: usize,
    next_id: &mut u32,
    rng: &mut Rng,
    records: &mut Records<'_>,
    lineages: &Lineages,
    palette: PartPalette,
    ground: Option<&Ground>,
) -> Option<Organism> {
    let (child, cost, endowment) = {
        let parent = &organisms[index];
        let cost = parent.biomass_mg() / OFFSPRING_COST;
        // A child's opening budget is **provisioned**, out of the parent's own
        // reserve. Until TD6 it was conjured: the parent paid `cost` once, in
        // body mass, and the child was handed a body worth `cost` *and* a
        // budget worth `cost`, so every birth in the enclosure minted matter.
        // A parent with nothing banked still breeds; its child simply starts
        // hungry, which is an honest consequence rather than a free lunch.
        let endowment = parent.energy_mg.min(cost);
        let child_id = OrganismId(*next_id);
        let development_seed = filial_seed(parent.development_seed, child_id);
        let lineage = lineages.get(parent.species)?;
        // Provisioning is binding. A complex recipe may need more positive-
        // mass parts than a quarter of this parent can pay for; in that case
        // the birth waits, spending neither matter nor ecology entropy.
        let Ok(mut body) = lineage.realize(development_seed, cost, palette) else {
            return None;
        };
        body.plan.symmetry = parent.body().plan.symmetry;
        // Wide enough to leave a crowded cell. Dispersal is how a stand
        // escapes its own shade, so a short throw would trap every offspring
        // in the same competition its parent is already losing.
        let scatter = [
            rng.range_i32(-BIRTH_SCATTER, BIRTH_SCATTER),
            0,
            rng.range_i32(-BIRTH_SCATTER, BIRTH_SCATTER),
        ];
        let position = [
            parent.position[0] + scatter[0],
            parent.position[1],
            parent.position[2] + scatter[2],
        ];
        // A birth cannot scatter through the wall either: a parent near the
        // edge threw offspring past Ground's resident bound, the far tier's
        // own escape route since it skips step_for's check entirely. (TD2b)
        let position = if let Some(ground) = ground {
            let bound = ground.extent();
            [
                position[0].clamp(-bound, bound),
                position[1],
                position[2].clamp(-bound, bound),
            ]
        } else {
            position
        };
        let walker_shape = crate::places::WalkerShape::from_aabb(body.aabb());
        let mut phenotype = crate::phenotype::BodyPhenotype::seed(body);
        lineage.apply_intake_ports(&mut phenotype);
        let child = Organism {
            id: child_id,
            species: parent.species,
            // A child starts small but structurally filial: the lineage recipe
            // grew this body under the current world's palette, and the whole
            // graph contains exactly what the parent paid. Its allocation is
            // seeded against those actual parts in the same unpublished
            // candidate, so a newborn is never a body without a phenotype.
            phenotype,
            development_seed,
            life_history_mass_mg: cost,
            energy_mg: endowment,
            // A near-tier child is an embodied body immediately, rather
            // than an abstract point that has to be repaired next tick.
            position: match (ground, parent.tier) {
                (Some(ground), Tier::Near) => surface_stance(ground, walker_shape, position)
                    .or_else(|| surface_stance(ground, walker_shape, parent.position))
                    .unwrap_or(parent.position),
                _ => position,
            },
            tier: parent.tier,
            last_seen: None,
            fauna_policy: parent.fauna_policy.inherited(development_seed),
            last_fauna_decision: None,
            stage: Stage::Juvenile,
            age: 0,
            since_offspring: 0,
            // A lie is heritable. An offspring wears its parent's colours and
            // carries its parent's bite, which is what makes a mimic lineage a
            // thing you can learn rather than a coin flip per organism.
            signal: parent.signal,
            venom_mg: parent.venom_mg,
            guise: parent.guise,
        };
        *next_id += 1;
        records.event(
            child.position,
            Event::Born {
                organism: child.id,
                species: child.species,
                parent: Some(parent.id),
            },
        );
        (child, cost, endowment)
    };

    let born_at = child.position;
    let heir = Subject::of(&child);
    let parent = &mut organisms[index];
    let forebear = Subject::of(parent);
    // `cost` is a quarter of what this body weighs, so the debit is always
    // payable in full whichever door asked for the birth; a shortfall here
    // would be matter out of nothing.
    let short = parent.spend_mass(cost);
    debug_assert_eq!(short, 0, "a birth outran its parent");
    parent.energy_mg -= endowment;
    parent.since_offspring = 0;

    // A birth is a transfer, not a spawn. Both halves come out of the
    // parent's own accounts and land in the matching account of the child,
    // which is what TD6 made true and this is what says so.
    for (account, mg) in [(Account::Substance, cost), (Account::Reserve, endowment)] {
        records.flow(
            born_at,
            FlowEvent::between(Process::Birth, forebear, account, heir, account, mg),
        );
    }
    Some(child)
}

const FILIAL_SALT: u64 = 0x4649_4C49_414C_0001;

/// The development seed a birth realizes a child under.
///
/// Crate-visible since PE3b, because a founder *preview* has to be grown under
/// the seed the birth will actually use or it is a picture of a different body.
pub(crate) fn filial_seed(parent: u64, child: OrganismId) -> u64 {
    let mut stream = Rng::from_seed(parent ^ FILIAL_SALT ^ u64::from(child.0));
    stream.next_u64()
}
