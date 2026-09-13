// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How a world begins.
//!
//! Split out of `world.rs` at the 600-line ceiling. Worldgen is a different
//! concern from running a world, and it is the one place where every seeded
//! decision about an enclosure is made in a fixed order.

use std::collections::BTreeMap;

use crate::body::SpeciesId;
use crate::development::{DevelopmentError, PartPalette};
use crate::matter::Material;
use crate::organism::ecology;
use crate::organism::{Kingdom, Organism, OrganismId, Signal, Stage};
use crate::plan::Role;
use crate::process::{IntakePort, NisKind, Process, Registry};
use crate::rng::Rng;
use crate::species::{InitialTissueRecipe, Lineages};

use super::{DEVELOPMENT_SALT, ENCLOSURE, GRAFT_SALT, PLACE_SALT, PLACE_SIDE, RECIPE_SALT, World};

/// Matter in one voxel column when the enclosure is founded.
///
/// **The world's entire matter budget is this times the column count**, and
/// nothing ever adds to it: light is the open input, matter is not. (TD6)
///
/// A hundred milligrams is the ecology's own reference body mass, so the rule
/// reads plainly: *the enclosure opens holding one reference body's worth of
/// substance under every voxel column it has*. Sized from the constant, so
/// widening the enclosure widens the budget with it — which S1 verified rather
/// than re-derived: at `ENCLOSURE = 64` this is a 129x129 grid and 1,664,100 mg
/// against the 33x33 grid's 108,900, the same 15.3x the area grew by, and the
/// same ~3x what the founding cohort carries because the cohort scaled with the
/// area too.
const SOIL_SEED_MG_PER_COLUMN: u64 = 100;

mod founding;
pub use founding::Founding;

struct Founder {
    id: OrganismId,
    species: SpeciesId,
    kingdom: Kingdom,
    mass_mg: u64,
    position: [i32; 3],
    stage: Stage,
    age: u32,
    since_offspring: u32,
    signal: Signal,
    venom_mg: u64,
    guise: Kingdom,
    development_seed: u64,
}

impl World {
    /// Builds the standard fixture: one critter and a deterministic scatter of
    /// organisms drawn from the seeded stream.
    pub fn new(seed: u64, organism_count: u32) -> Self {
        Self::founded(seed, organism_count, Founding::default())
            .expect("the shipping founding's palette is valid")
    }

    /// Builds a world whose tiers found from [`Founding`]'s bodies, under the
    /// palette that founding admits.
    pub fn founded(
        seed: u64,
        organism_count: u32,
        founding: Founding,
    ) -> Result<Self, DevelopmentError> {
        Self::found(
            seed,
            organism_count,
            founding.palette(),
            founding,
            crate::world::native_ruleset(),
        )
    }

    /// Builds a world under an explicitly admitted ruleset. (PD4)
    ///
    /// **What a pack door founds through.** The definitions become the world's
    /// own, its [`WorldRules`](crate::rules::WorldRules) become their digest,
    /// and every development on every body in it is validated against exactly
    /// this set. A definition it does not hold is refused rather than resolved
    /// somewhere else, which is the whole reason the set travels beside the
    /// digest.
    ///
    /// Immutable for the world's life, per plan §5: there is no setter. Editing
    /// a pack means founding again, or restoring through
    /// [`restore_under`](crate::snapshot::restore_under).
    pub fn founded_on(
        seed: u64,
        organism_count: u32,
        founding: Founding,
        ruleset: std::sync::Arc<crate::process::Registry>,
    ) -> Result<Self, DevelopmentError> {
        Self::found(seed, organism_count, founding.palette(), founding, ruleset)
    }

    /// Builds a world under an explicitly admitted developmental palette.
    ///
    /// The palette is snapshotted with the world. A host can therefore replace
    /// the baseline fixture references without smuggling asset choices into a
    /// lineage recipe or making replay depend on ambient configuration.
    ///
    /// **Founds [`Founding::Drawn`]**: a caller supplying its own vocabulary is
    /// the soup world, and the roster's bodies name shapes that vocabulary may
    /// not admit. Use [`World::founded`] to pick both together.
    pub fn with_development_palette(
        seed: u64,
        organism_count: u32,
        development_palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        Self::founded_with_palette(seed, organism_count, Founding::Drawn, development_palette)
    }

    /// Builds a world with an explicit founding and developmental palette.
    ///
    /// The palette is world state, while the founding selects the heritable
    /// recipes. Keeping both arguments here lets a host admit content-pack
    /// volume references without silently falling back to the primitive
    /// vocabulary or changing which roster is founded.
    pub fn founded_with_palette(
        seed: u64,
        organism_count: u32,
        founding: Founding,
        development_palette: PartPalette,
    ) -> Result<Self, DevelopmentError> {
        Self::found(
            seed,
            organism_count,
            development_palette,
            founding,
            crate::world::native_ruleset(),
        )
    }

    fn found(
        seed: u64,
        organism_count: u32,
        development_palette: PartPalette,
        founding: Founding,
        ruleset: std::sync::Arc<crate::process::Registry>,
    ) -> Result<Self, DevelopmentError> {
        let mut rng = Rng::from_seed(seed);

        // Draft identities and ecology first. Bodies cannot be developed until
        // every founding lineage has its recipe, and constructing root-only
        // placeholder organisms here would preserve the split authority this
        // migration is removing.
        let mut founders = Vec::with_capacity(organism_count as usize + 1);
        // Kingdom floor: guarantee the non-played species cover all three
        // kingdoms before any founder draws a role, so a seed can no longer
        // found with a missing rung (2 of 10 seeds drew zero producers under
        // the old free draw -- guaranteed collapse). The 3 non-played species
        // ids are fixed by the `rng.below(3)` draw below; a Fisher-Yates
        // shuffle of the 3 kingdoms assigns one each, deterministic from the
        // seeded stream. A species beyond the floor (none exist today) draws
        // freely in the `or_insert_with` below, so variety survives if the
        // roster ever grows past 3. (2026-08-29 TD2b)
        let mut floor_kingdoms = [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer];
        for i in (1..floor_kingdoms.len()).rev() {
            floor_kingdoms.swap(i, rng.below(i as u64 + 1) as usize);
        }
        //
        // **A tier is a block of species ids now, not one** (DC4). The roster
        // founds one lineage per archetype, so the shuffled order above lays
        // out consecutive blocks rather than single ids, and every archetype
        // gets a species before any founder picks one.
        let mut species_of: BTreeMap<Kingdom, Vec<SpeciesId>> = BTreeMap::new();
        let mut next_species = 2u32;
        for kingdom in floor_kingdoms {
            let block = (0..founding.lineages(kingdom))
                .map(|_| {
                    let id = SpeciesId(next_species);
                    next_species += 1;
                    id
                })
                .collect();
            species_of.insert(kingdom, block);
        }
        // **Counts make the pyramid** (2026-08-29 TD7). A uniform species draw
        // founded equal thirds, which is an ecology standing on its point: the
        // 20-odd consumers it put on 20-odd producers over-grazed the stand
        // within 200 ticks in every seed of TD6's receipt. The tiers are
        // therefore drawn as a composition rather than per founder — exactly
        // `PRODUCER_SHARE` producers and `CONSUMER_SHARE` consumers of the
        // non-played founders, the rest decomposers — and shuffled into
        // arrival order from the same seeded stream, so the pyramid is the
        // world's shape rather than a distribution it usually lands near. At
        // the shipping 60 that is 40 / 15 / 5. Individual sizes stay what the
        // bodies honestly say.
        let mut kingdoms = pyramid(organism_count as usize);
        for i in (1..kingdoms.len()).rev() {
            kingdoms.swap(i, rng.below(i as u64 + 1) as usize);
        }
        founders.push(Founder {
            id: OrganismId(0),
            species: SpeciesId(1),
            kingdom: Kingdom::Consumer,
            mass_mg: 1_000,
            position: [0, 0, 0],
            stage: Stage::Juvenile,
            // Kept newborn, unlike the mid-life stagger below: the player's
            // life should start near its beginning, not drawn from the same
            // whole-life distribution as the ecology around it. (TD5b)
            age: 0,
            since_offspring: 0,
            signal: Signal::Plain,
            venom_mg: 0,
            guise: Kingdom::Consumer,
            development_seed: founder_seed(seed, OrganismId(0)),
        });

        let mut seated: BTreeMap<Kingdom, usize> = BTreeMap::new();
        for index in 1..=organism_count {
            // Draws happen in a fixed order, so the scatter is reproducible.
            let x = rng.range_i32(-ENCLOSURE, ENCLOSURE);
            let y = rng.range_i32(-2, 2);
            let z = rng.range_i32(-ENCLOSURE, ENCLOSURE);
            let mass = 100 + rng.below(400);
            // A species has one inherited silhouette, so the tier this founder
            // drew names the species rather than the other way round. Within a
            // tier the archetypes take turns, so every one of them founds in
            // every seed and the tier splits evenly between them without
            // spending a draw on it.
            let kingdom = kingdoms[(index - 1) as usize];
            let block = &species_of[&kingdom];
            let seat = seated.entry(kingdom).or_insert(0);
            let species = block[*seat % block.len()];
            *seat += 1;
            // Staggered ages, so the enclosure is mid-life rather than all
            // hatching on the same tick. Proportional to the founder's own
            // lifespan_for_mass rather than a flat 200: a flat stagger left
            // every founder a newborn against a 2,000-3,000-tick life, so
            // nothing died of old age until ~1,800 and the enclosure held no
            // real carrion until then (TD5 finding). Uniform over the whole
            // life puts the founding cohort's mean age at its own midpoint —
            // mid-life — with a near-death tail that seeds carrion from the
            // first ticks, the shape a throwaway rng.below(2000) diagnostic
            // proved: 42 decomposers alive at the 10,000-tick horizon in seed
            // 7. (2026-08-29 TD5b)
            let age = rng.below(u64::from(ecology::lifespan_for_mass(mass).max(1))) as u32;
            // Staggered the same way: an un-staggered founder pool all reads
            // since_offspring 0, gating the world's whole first brood behind
            // one full gestation. (2026-08-29 TD2b)
            let since_offspring =
                rng.below(u64::from(ecology::gestation_for_mass(mass).max(1))) as u32;

            // Most things are honest. A minority lie, in both directions: a
            // harmless thing wearing a warning, and a dangerous thing wearing
            // none. Both are rare, because a world of liars teaches nothing.
            let (signal, venom_mg, guise) = match rng.below(10) {
                0 => (Signal::Warning, 0, kingdom),
                1 => (Signal::Plain, 90 + rng.below(60), Kingdom::Producer),
                2..=3 => (Signal::Warning, 60 + rng.below(60), kingdom),
                _ => (Signal::Plain, 0, kingdom),
            };
            founders.push(Founder {
                id: OrganismId(index),
                species,
                kingdom,
                mass_mg: mass,
                position: [x, y, z],
                stage: Stage::Mature,
                age,
                since_offspring,
                signal,
                venom_mg,
                guise,
                development_seed: founder_seed(seed, OrganismId(index)),
            });
        }

        // The world's graft affinity, before any lineage is assigned a domain:
        // the table is what gives a domain number meaning, so it exists first.
        let affinity = crate::graft::Affinity::native();
        // Everything the world began with is a founding lineage: no parent,
        // no name, because nobody was there to give it one.
        let mut lineages = Lineages::new();
        for founder in &founders {
            lineages.found(founder.species);
        }
        // Each line draws its recipe from its own stream, so body generation
        // never advances the ecology stream.
        //
        // **The pyramid now picks a body, not a field** (DC1.5). It used to
        // author a `Kingdom` onto the founder and write the matching symmetry,
        // and `Organism::kingdom()` read that symmetry straight back — the tier
        // was a decree wearing a body. A kingdom is read off feeding anatomy
        // now, so the tier's only job is to say which anatomy this line draws,
        // and what the world reads afterwards is whatever the body says. The
        // symmetry below is the silhouette that tier opens with and nothing
        // reads it back.
        //
        // One recipe per founding *lineage*, drawn or authored, and the played
        // critter's line beside them: it takes the first body of the consumer
        // tier, which is §5's open question answered provisionally rather than
        // ruled.
        let assignments = std::iter::once((SpeciesId(1), Kingdom::Consumer, 0)).chain(
            species_of.iter().flat_map(|(kingdom, block)| {
                block
                    .iter()
                    .enumerate()
                    .map(move |(slot, species)| (*species, *kingdom, slot))
            }),
        );
        for (species, kingdom, slot) in assignments {
            // An authored tier does not draw at all. Each line has its own
            // salted stream, so a stream left unspent moves nothing else: the
            // tiers that still draw develop bodies identical to another arm's.
            let recipe = match founding.tier(kingdom).get(slot) {
                Some(authored) => authored(),
                None => {
                    let mut stream = Rng::from_seed(seed ^ RECIPE_SALT ^ u64::from(species.0));
                    crate::axis::seed(&mut stream, kingdom)
                },
            };
            lineages.set_recipe(species, recipe);
            lineages.set_symmetry(species, kingdom.symmetry());
            lineages.set_initial_tissue(
                species,
                InitialTissueRecipe::single(match kingdom {
                    Kingdom::Producer => Material::Producer,
                    Kingdom::Consumer => Material::Consumer,
                    Kingdom::Decomposer => Material::Decomposer,
                }),
            );
            // What this line's flesh is, as this world numbers domains. Its own
            // salted stream, so the assignment moves nothing else. (P3)
            let mut stream = Rng::from_seed(seed ^ GRAFT_SALT ^ u64::from(species.0));
            lineages.set_domain(species, affinity.draw(&mut stream));
        }

        // A founder's selected mass is a lower bound. Genesis has no parent
        // ledger to debit, so when a rare recipe needs more than the draw to
        // keep every part positive-mass, the world starts it at that exact
        // structural floor. Births below enforce the stricter filial rule and
        // wait for provisioning instead.
        // The browser lineage carries a crop and a separate jaw port. This is
        // an authored roster declaration, not a geometry inference: removing
        // either port changes the reading while leaving the silhouette intact.
        let omnivore_species = species_of[&Kingdom::Consumer][0];
        let authored_consumers = !founding.tier(Kingdom::Consumer).is_empty();
        let mut organisms = Vec::with_capacity(founders.len());
        for founder in founders {
            let lineage = lineages
                .get(founder.species)
                .expect("every founder registered a lineage");
            let mut body = match lineage.realize(
                founder.development_seed,
                founder.mass_mg,
                development_palette,
            ) {
                Ok(body) => body,
                Err(DevelopmentError::InsufficientMass { parts, .. }) => lineage.realize(
                    founder.development_seed,
                    u64::from(parts),
                    development_palette,
                )?,
                Err(error) => return Err(error),
            };
            let mass_mg = body.total_mass_mg();
            body.plan.symmetry = founder.kingdom.symmetry();
            let mut phenotype = crate::phenotype::BodyPhenotype::seed(body);
            let stock = lineage
                .initial_tissue
                .stock_for(mass_mg)
                .expect("live founders always declare tissue");
            phenotype
                .distribute_stock(stock)
                .expect("founder tissue is scaled to the realized body");
            lineage.apply_intake_ports(&mut phenotype);
            if authored_consumers
                && (founder.species == omnivore_species || founder.species == SpeciesId(1))
            {
                let jaw = phenotype
                    .body()
                    .living()
                    .find(|part| crate::plan::classify(part.half_extent) == Role::Limb)
                    .map(|part| part.id);
                if let Some(jaw) = jaw {
                    let port = IntakePort::live(NisKind::Consumer)
                        .supported_by(Registry::native().of_native(Process::Contract).reference());
                    phenotype.declare_port(jaw, port);
                    lineages.declare_intake_port(founder.species, jaw, port);
                }
            }
            organisms.push(Organism {
                id: founder.id,
                species: founder.species,
                phenotype,
                development_seed: founder.development_seed,
                life_history_mass_mg: mass_mg,
                position: founder.position,
                tier: crate::places::Tier::Near,
                last_seen: None,
                fauna_policy: crate::organism::FaunaPolicy::default(),
                last_fauna_decision: None,
                energy_mg: mass_mg,
                stage: founder.stage,
                age: founder.age,
                since_offspring: founder.since_offspring,
                signal: founder.signal,
                venom_mg: founder.venom_mg,
                guise: founder.guise,
            });
        }

        let grown = crate::places::Places::grown(seed ^ PLACE_SALT, PLACE_SIDE, ENCLOSURE);
        let ground = crate::places::Ground::grow(&grown, ENCLOSURE);

        // The old enclosure was an abstract field, so founders carried an
        // arbitrary y draw. Brick truth makes that invalid: an embodied
        // creature must begin on footing, with enough headroom for the
        // near-tier walker. Keep the draw above in the seeded sequence so the
        // landscape transition does not rearrange every later founder choice.
        for organism in &mut organisms {
            let shape = organism.walker_shape();
            organism.position =
                crate::places::surface_stance_for(&ground, shape, organism.position)
                    .expect("the grown enclosure covers every founding body");
            debug_assert!(shape.stands(&ground, organism.position));
        }

        // The founding population enters the record. Without this a seeded
        // creature's first event is whatever happened *to* it, so its origin
        // is invisible and its causal line begins in the middle. Stamped after
        // the enclosure exists, so a founding birth names the region it
        // actually happened in.
        let pending = organisms
            .iter()
            .map(|o| {
                crate::flow::Envelope::new(
                    0,
                    grown.places.at(o.position),
                    crate::history::Event::Born {
                        organism: o.id,
                        species: o.species,
                        parent: None,
                    },
                )
            })
            .collect();

        let mut world = Self {
            tick: 0,
            epoch: 0,
            // What biology this world realized (PD3), and the definitions it
            // is (PD4). The digest is saved; the set is runtime carriage the
            // one validator resolves against, so the two cannot disagree
            // about what this world admitted.
            rules: crate::rules::WorldRules::of(&ruleset),
            ruleset,
            rng,
            controlled: Some(OrganismId(0)),
            control_lost: None,
            // A world opens under the hand. Nobody has idled yet, so the
            // first tick's instincts leave the played critter alone.
            idle_run: 0,
            // A line starts having come to nothing, and having offered no
            // evidence to anything. (PE2)
            discoveries: Vec::new(),
            last_observation: None,
            hunger_run: 0,
            // The table a default world holds, and no branch taken yet. (P3)
            affinity,
            last_graft: None,
            unlocked: std::collections::BTreeSet::from([SpeciesId(1)]),
            // The starting body already counts: the player is holding it, so
            // the frontier begins where they begin rather than at nothing.
            // Filled after the registry exists, since intricacy reads it.
            frontier: 0,
            lineages,
            development_palette,
            // Places take their own stream, so dividing an enclosure does
            // not rearrange the creatures scattered across it. Grown, not
            // scattered (G1 adoption 2026-08-08): same site draws as the old
            // lattice, so the partition is bit-identical, but links derive
            // from the landscape and the ground below is real.
            places: grown.places.clone(),
            ground,
            soil: crate::places::Soil::seeded(ENCLOSURE, SOIL_SEED_MG_PER_COLUMN),
            ranges: std::collections::BTreeMap::new(),
            record: crate::record::WorldRecord::new(),
            organisms,
            next_organism: organism_count + 1,
            last_tally: crate::organism::Tally::default(),
            pending,
            flows: crate::flow::Ledger::default(),
            // The first epoch starts where the world does, nothing has been
            // weighed, and no line is standing at a checkpoint. (PE3)
            epoch_began: 0,
            at_boundary: false,
            last_round: crate::world::Round::default(),
            forced_birth: None,
            scoring: false,
        };

        // The starting body already counts, and intricacy needs the registry,
        // so the high-water mark is set once the world exists rather than in
        // the initialiser.
        world.frontier = world
            .organisms
            .first()
            .map(|o| world.intricacy(o))
            .unwrap_or(0);
        Ok(world)
    }
}

/// Share of the non-played founders that are producers. Two thirds: the base
/// of the chain has to out-number what grazes it, and TD6 measured what
/// happens when it does not.
const PRODUCER_SHARE: (usize, usize) = (2, 3);
/// Share that are consumers. A quarter — fewer mouths than plants, and still
/// enough of them to be an ecology rather than a stand with visitors.
const CONSUMER_SHARE: (usize, usize) = (1, 4);

/// The founding composition: many producers, fewer consumers, few
/// decomposers, in that order.
///
/// Exact rather than drawn, so the pyramid is a guarantee. **Every kingdom is
/// still founded** — the TD2b floor, kept: a tier that rounds to nothing takes
/// one founder from the widest rather than leaving a rung out of the chain.
fn pyramid(count: usize) -> Vec<Kingdom> {
    let producers = count * PRODUCER_SHARE.0 / PRODUCER_SHARE.1;
    let consumers = count * CONSUMER_SHARE.0 / CONSUMER_SHARE.1;
    let mut tiers = [
        producers,
        consumers,
        count.saturating_sub(producers + consumers),
    ];
    for tier in 0..tiers.len() {
        if tiers[tier] > 0 {
            continue;
        }
        let widest = (0..tiers.len())
            .max_by_key(|&other| tiers[other])
            .expect("three tiers");
        if tiers[widest] > 1 {
            tiers[widest] -= 1;
            tiers[tier] += 1;
        }
    }
    [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer]
        .into_iter()
        .zip(tiers)
        .flat_map(|(kingdom, many)| std::iter::repeat_n(kingdom, many))
        .collect()
}

fn founder_seed(world_seed: u64, organism: OrganismId) -> u64 {
    let mut stream = Rng::from_seed(world_seed ^ DEVELOPMENT_SALT ^ u64::from(organism.0));
    stream.next_u64()
}

#[cfg(test)]
mod tests;
