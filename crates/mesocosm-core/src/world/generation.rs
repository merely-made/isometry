// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Seeded habitat and constrained starting lives. Preview owns no live world.
//! All candidates use the same terrain and nutrient patches. Entry regenerates
//! and validates the selected candidate before founding the played world.

use serde::{Deserialize, Serialize};

use super::{ENCLOSURE, Founding, World};
use crate::places::{PlaceId, Soil, surface_stance_for};
use crate::{
    BodyDocument, BodyPhenotype, Kingdom, PartPalette, Recipe, Rng, Soma, SpeciesId, Symmetry,
};

/// Bump when seed streams, admission, or founding interpretation change.
pub const VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Criteria {
    /// Feeding role read from realized organs, not biological taxonomy.
    pub role: Option<Kingdom>,
    pub min_segments: u32,
    pub max_segments: u32,
    pub max_parts: u32,
    /// Requires or excludes contractile organs. Producers can also creep,
    /// and player steering is separately ruled, so this is not immobility.
    pub movement_organs: Option<bool>,
    pub mass_mg: u64,
}

impl Default for Criteria {
    fn default() -> Self {
        Self {
            role: None,
            min_segments: 1,
            max_segments: 32,
            max_parts: 128,
            movement_organs: None,
            mass_mg: 800,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub seed: u64,
    /// Varies bodies without changing habitat or background inhabitants.
    pub variation: u64,
    pub place: u16,
    pub candidates: u32,
    pub attempts: u32,
    /// Background founders; the selected played life is added to this count.
    pub organisms: u32,
    pub soil_min_mg: u64,
    pub soil_max_mg: u64,
    pub criteria: Criteria,
}

impl Default for Request {
    fn default() -> Self {
        Self {
            version: VERSION,
            seed: 7,
            variation: 0,
            place: 4,
            candidates: 4,
            attempts: 128,
            organisms: 24,
            soil_min_mg: 60,
            soil_max_mg: 180,
            criteria: Criteria::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub request: Request,
    pub candidate: usize,
}

/// Immutable admitted candidates and founding context for disposable previews.
pub struct Prepared {
    draft: Draft,
    foundation: World,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Habitat {
    pub place: u16,
    pub centre: [i32; 2],
    pub soil_mg_per_column: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Candidate {
    pub seed: u64,
    pub role: Kingdom,
    pub symmetry: Symmetry,
    pub segments: u32,
    pub parts: u32,
    pub actuator_span: u32,
    pub position: [i32; 3],
    pub local_soil_mg: u64,
    pub recipe: Recipe,
    pub body: BodyDocument,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Draft {
    pub request: Request,
    pub habitat: Vec<Habitat>,
    pub candidates: Vec<Candidate>,
    pub attempted: u32,
    pub rejected: std::collections::BTreeMap<String, u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Version(u32),
    Invalid(&'static str),
    Development(String),
    CandidateUnavailable { requested: usize, available: usize },
}

impl Request {
    pub fn validate(&self) -> Result<(), Error> {
        if self.version != VERSION {
            return Err(Error::Version(self.version));
        }
        let c = &self.criteria;
        if !(1..=16).contains(&self.candidates)
            || !(1..=512).contains(&self.attempts)
            || self.attempts < self.candidates
        {
            return Err(Error::Invalid("candidate/search bounds"));
        }
        if !(3..=120).contains(&self.organisms) || self.place >= 9 {
            return Err(Error::Invalid(
                "population must be 3..120; place must be 0..8",
            ));
        }
        if self.soil_min_mg == 0 || self.soil_min_mg > self.soil_max_mg || self.soil_max_mg > 10_000
        {
            return Err(Error::Invalid(
                "soil must be an ordered range in 1..10000 mg",
            ));
        }
        if c.min_segments == 0
            || c.min_segments > c.max_segments
            || c.max_segments > 64
            || !(1..=256).contains(&c.max_parts)
            || !(64..=10_000).contains(&c.mass_mg)
        {
            return Err(Error::Invalid("body criteria outside supported bounds"));
        }
        Ok(())
    }

    pub fn preview(&self, palette: PartPalette) -> Result<Draft, Error> {
        self.draft_world(palette).map(|(draft, _)| draft)
    }

    pub fn prepare(&self, palette: PartPalette) -> Result<Prepared, Error> {
        self.draft_world(palette)
            .map(|(draft, foundation)| Prepared { draft, foundation })
    }

    fn draft_world(&self, palette: PartPalette) -> Result<(Draft, World), Error> {
        self.validate()?;
        let mut world =
            World::founded_with_palette(self.seed, self.organisms, Founding::Drawn, palette)
                .map_err(|e| Error::Development(format!("{e:?}")))?;
        let mut stream = Rng::from_seed(self.seed ^ 0x4841_4249_5441_5401);
        let habitat: Vec<_> = world
            .places
            .all()
            .map(|place| Habitat {
                place: place.id.0,
                centre: place.centre,
                soil_mg_per_column: self.soil_min_mg
                    + stream.below(self.soil_max_mg - self.soil_min_mg + 1),
            })
            .collect();
        world.soil = Soil::seeded(ENCLOSURE, 0);
        for z in -ENCLOSURE..=ENCLOSURE {
            for x in -ENCLOSURE..=ENCLOSURE {
                let position = [x, 0, z];
                let place = world.places.at(position).expect("places cover enclosure");
                world.soil.deposit(
                    world.soil.column_at(position),
                    habitat[place.0 as usize].soil_mg_per_column,
                );
            }
        }
        let centre = habitat[self.place as usize].centre;
        let mut draft = Draft {
            request: self.clone(),
            habitat,
            candidates: Vec::new(),
            attempted: 0,
            rejected: Default::default(),
        };
        let mut stream =
            Rng::from_seed(self.seed ^ self.variation.rotate_left(29) ^ 0x424F_4459_0000_0001);
        while draft.attempted < self.attempts && draft.candidates.len() < self.candidates as usize {
            draft.attempted += 1;
            let candidate_seed = stream.next_u64();
            let role = self.criteria.role.unwrap_or(
                [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer]
                    [((draft.attempted - 1) % 3) as usize],
            );
            match self.candidate(&world, palette, centre, candidate_seed, role) {
                Ok(candidate) => {
                    if draft.candidates.iter().any(|old| {
                        old.recipe == candidate.recipe && old.symmetry == candidate.symmetry
                    }) {
                        *draft.rejected.entry("duplicate recipe".into()).or_default() += 1;
                    } else {
                        draft.candidates.push(candidate);
                    }
                },
                Err(reason) => *draft.rejected.entry(reason.into()).or_default() += 1,
            }
        }
        Ok((draft, world))
    }

    fn candidate(
        &self,
        world: &World,
        palette: PartPalette,
        centre: [i32; 2],
        seed: u64,
        role: Kingdom,
    ) -> Result<Candidate, &'static str> {
        let mut rng = Rng::from_seed(seed);
        let recipe = crate::axis::seed(&mut rng, role);
        let segments = recipe.segments();
        if !(self.criteria.min_segments..=self.criteria.max_segments).contains(&segments) {
            return Err("segment constraint");
        }
        let soma = Soma::develop(&recipe, seed);
        let mut body =
            crate::develop_body(SpeciesId(1), &recipe, &soma, self.criteria.mass_mg, palette)
                .map_err(|_| "development or material constraint")?;
        // Existing axial development owns placement. A symmetry override here
        // would only relabel the plan, so it is not an admitted input yet.
        body.plan.symmetry = role.symmetry();
        if body.parts.len() > self.criteria.max_parts as usize {
            return Err("part constraint");
        }
        let mut organism = world.organisms[0].clone();
        organism.phenotype = BodyPhenotype::seed(body.clone());
        if organism.kingdom() != role {
            return Err("realized trophic role differs");
        }
        let actuator_span = organism.actuator_span();
        if self
            .criteria
            .movement_organs
            .is_some_and(|movement_organs| movement_organs != (actuator_span > 0))
        {
            return Err("movement constraint");
        }
        let shape = organism.walker_shape();
        let position = surface_stance_for(&world.ground, shape, [centre[0], 0, centre[1]])
            .ok_or("no body-sized footing")?;
        if world.places.at(position) != Some(PlaceId(self.place)) {
            return Err("footing outside selected habitat");
        }
        let column = world.soil.column_at(position);
        let local_soil_mg = world
            .soil
            .columns_within(column, 3)
            .map(|c| world.soil.matter_mg(c))
            .sum();
        if local_soil_mg < self.criteria.mass_mg * 2 {
            return Err("insufficient local founding matter for body and reserve");
        }
        Ok(Candidate {
            seed,
            role,
            symmetry: body.plan.symmetry,
            segments,
            parts: body.parts.len() as u32,
            actuator_span,
            position,
            local_soil_mg,
            recipe,
            body,
        })
    }
}

impl Selection {
    /// Found a new life, never replace a subject in an existing played world.
    pub fn enter(&self, palette: PartPalette) -> Result<World, Error> {
        self.request.prepare(palette)?.enter(self.candidate)
    }
}

impl Prepared {
    pub fn draft(&self) -> &Draft {
        &self.draft
    }

    pub fn enter(&self, index: usize) -> Result<World, Error> {
        let mut world = self.foundation.clone();
        let candidate = self
            .draft
            .candidates
            .get(index)
            .ok_or(Error::CandidateUnavailable {
                requested: index,
                available: self.draft.candidates.len(),
            })?;
        // The selected founder and its reserve are paid from the local patch.
        // Return the displaced provisional founder to its original soil first.
        let original = &world.organisms[0];
        world.soil.deposit(
            world.soil.column_at(original.position),
            original.biomass_mg() + original.energy_mg,
        );
        let column = world.soil.column_at(candidate.position);
        let columns: Vec<_> = world.soil.columns_within(column, 3).collect();
        let mut remaining = candidate.body.total_mass_mg() * 2;
        for column in columns {
            remaining -= world.soil.draw(column, remaining);
        }
        if remaining != 0 {
            return Err(Error::Invalid("founding material changed after admission"));
        }
        world
            .lineages
            .set_recipe(SpeciesId(1), candidate.recipe.clone());
        world
            .lineages
            .set_symmetry(SpeciesId(1), candidate.symmetry);
        let organism = &mut world.organisms[0];
        organism.phenotype = BodyPhenotype::seed(candidate.body.clone());
        organism.development_seed = candidate.seed;
        organism.life_history_mass_mg = candidate.body.total_mass_mg();
        organism.energy_mg = organism.life_history_mass_mg;
        organism.position = candidate.position;
        organism.guise = candidate.role;
        world.pending = world
            .organisms
            .iter()
            .map(|o| {
                crate::flow::Envelope::new(
                    0,
                    world.places.at(o.position),
                    crate::history::Event::Born {
                        organism: o.id,
                        species: o.species,
                        parent: None,
                    },
                )
            })
            .collect();
        world.frontier = world.controlled().map(|o| world.intricacy(o)).unwrap_or(0);
        Ok(world)
    }
}

#[cfg(test)]
mod tests;
