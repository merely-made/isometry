// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Far-tier population state.
//!
//! A cohort is a reversible compression of equivalent living bodies. It is
//! deliberately scalar: identity, body documents, and causal events remain
//! individual facts, while the far tier can carry abundance and conserved
//! quantities without pretending every distant body needs an embodied mind.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::organism::{Kingdom, Organism, OrganismId};
use crate::places::{PlaceId, Places, Tier};
use crate::process::FeedingMode;

const MASS_BAND_MG: u64 = 64;

/// What makes two far bodies the same body for the far tier's purposes.
///
/// **The kingdom and mode terms are anatomy now** (DC1.5). They used to be a
/// species-wide silhouette wearing two names, so grouping on them was grouping
/// on the lineage twice over; a body can now leave its class by growing or
/// losing a feeding organ, and the key follows it. A cohort therefore still
/// means "these earn and are eaten the same way", but it is a statement about
/// the bodies rather than about the line they came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CohortKey {
    pub species: crate::body::SpeciesId,
    pub place: PlaceId,
    pub kingdom: Kingdom,
    pub mode: FeedingMode,
    pub mass_band: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CohortMember {
    pub id: OrganismId,
    pub key: CohortKey,
    pub biomass_mg: u64,
    pub energy_mg: u64,
    pub age: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cohort {
    pub key: CohortKey,
    pub count: u32,
    pub biomass_mg: u64,
    pub energy_mg: u64,
    pub age_sum: u64,
}

impl Cohort {
    pub fn from_members(members: &[CohortMember]) -> Vec<Self> {
        let mut ordered = members.to_vec();
        ordered.sort_by_key(|member| (member.key, member.id));
        let mut out: Vec<Self> = Vec::new();
        for member in ordered {
            if let Some(last) = out.last_mut()
                && last.key.species == member.key.species
                && last.key.place == member.key.place
                && last.key.mode == member.key.mode
                && last.key.mass_band.abs_diff(member.key.mass_band) <= 1
            {
                last.add(member);
                continue;
            }
            out.push(Self {
                key: member.key,
                count: 1,
                biomass_mg: member.biomass_mg,
                energy_mg: member.energy_mg,
                age_sum: u64::from(member.age),
            });
        }
        out
    }

    fn add(&mut self, member: CohortMember) {
        self.count += 1;
        self.biomass_mg += member.biomass_mg;
        self.energy_mg += member.energy_mg;
        self.age_sum += u64::from(member.age);
    }

    /// Splits a cohort while conserving biomass, energy, and summed age.
    /// Remainders go to the lowest stable member ids.
    pub fn split(&self, ids: &[OrganismId]) -> Vec<CohortMember> {
        assert_eq!(ids.len(), self.count as usize, "split must name every body");
        let mut members = Vec::with_capacity(ids.len());
        let (mass, mass_remainder) = distribute(self.biomass_mg, ids.len());
        let (energy, energy_remainder) = distribute(self.energy_mg, ids.len());
        let (age, age_remainder) = distribute(self.age_sum, ids.len());
        for (index, id) in ids.iter().copied().enumerate() {
            members.push(CohortMember {
                id,
                key: self.key,
                biomass_mg: mass + u64::from(index < mass_remainder),
                energy_mg: energy + u64::from(index < energy_remainder),
                age: (age + u64::from(index < age_remainder)) as u32,
            });
        }
        members
    }
}

fn distribute(value: u64, count: usize) -> (u64, usize) {
    (value / count as u64, (value % count as u64) as usize)
}

/// Forms far-tier cohorts from the current individual world state. The
/// conversion is deterministic and leaves near bodies untouched.
pub fn from_organisms(organisms: &[Organism], places: &Places) -> Vec<Cohort> {
    let members: Vec<_> = organisms
        .iter()
        .filter(|organism| organism.is_alive() && organism.tier == Tier::Far)
        .filter_map(|organism| {
            let place = places.at(organism.position)?;
            let biomass = organism.biomass_mg();
            Some(CohortMember {
                id: organism.id,
                key: CohortKey {
                    species: organism.species,
                    place,
                    kingdom: organism.kingdom(),
                    mode: organism.feeding_mode(),
                    mass_band: biomass / MASS_BAND_MG,
                },
                biomass_mg: biomass,
                energy_mg: organism.energy_mg,
                age: organism.age,
            })
        })
        .collect();
    Cohort::from_members(&members)
}

/// A compact receipt for a far-tier formation pass.
pub fn conserved_totals(cohorts: &[Cohort]) -> (u64, u64, u64) {
    cohorts
        .iter()
        .fold((0, 0, 0), |(count, biomass, energy), cohort| {
            (
                count + u64::from(cohort.count),
                biomass + cohort.biomass_mg,
                energy + cohort.energy_mg,
            )
        })
}

/// Stable population counts by place, useful to a host without exposing the
/// individual far-tier roster as its working unit.
pub fn by_place(cohorts: &[Cohort]) -> BTreeMap<PlaceId, u32> {
    let mut out = BTreeMap::new();
    for cohort in cohorts {
        *out.entry(cohort.key.place).or_default() += cohort.count;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::SpeciesId;

    fn member(id: u32, mass: u64, age: u32) -> CohortMember {
        CohortMember {
            id: OrganismId(id),
            key: CohortKey {
                species: SpeciesId(1),
                place: PlaceId(2),
                kingdom: Kingdom::Consumer,
                mode: FeedingMode::Predator,
                mass_band: mass / MASS_BAND_MG,
            },
            biomass_mg: mass,
            energy_mg: mass / 2,
            age,
        }
    }

    #[test]
    fn merge_and_split_conserve_every_scalar() {
        let members = [member(1, 100, 3), member(2, 120, 5), member(3, 130, 7)];
        let cohorts = Cohort::from_members(&members);
        let (count, biomass, energy) = conserved_totals(&cohorts);
        assert_eq!((count, biomass, energy), (3, 350, 175));

        let ids = [OrganismId(1), OrganismId(2), OrganismId(3)];
        let expanded: Vec<_> = cohorts[0].split(&ids);
        assert_eq!(expanded.iter().map(|m| m.biomass_mg).sum::<u64>(), biomass);
        assert_eq!(expanded.iter().map(|m| m.energy_mg).sum::<u64>(), energy);
        assert_eq!(expanded.iter().map(|m| u64::from(m.age)).sum::<u64>(), 15);
    }
}
