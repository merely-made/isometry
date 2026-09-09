// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where things happen.
//!
//! [`Scale`] has had three variants since the world record was built and no way
//! to tell them apart, because an enclosure was a continuous field of positions
//! with nowhere in it. `CROWD_CELL` looked like the missing piece and is not: it
//! is an eight-voxel bucket for counting neighbours, recomputed from scratch
//! every tick, with no identity to remember and no notion of what is next to
//! what.
//!
//! # A partition, and a graph over it
//!
//! Two facts are needed and they are not the same fact.
//!
//! Everything in the enclosure is somewhere, so places must **cover** it:
//! [`at`](Places::at) always answers. Sites are scattered one per cell of a
//! coarse grid, jittered inside it, and a position belongs to the nearest site.
//! Stratifying rather than scattering freely is what keeps regions from
//! collapsing into slivers, and the jitter is what keeps the result from looking
//! like the grid it came from.
//!
//! Places are also **next to** each other, and that is the half a partition
//! cannot express. Three places in a row and three scattered across the
//! enclosure are the same count and a different fact: one is a spread, the other
//! is an outbreak. Adjacency is the grid's, so the graph is connected by
//! construction and no seed can produce an enclosure with an unreachable corner.
//!
//! Height is not a place. The enclosure is a few voxels deep and nothing lives
//! in a layer, so places are x/z regions, which is the same reading
//! `organism::ecology` already takes of position.
//!
//! # Derived, like everything else that is a verdict
//!
//! A place has no name and no character of its own. What a place *is* comes from
//! what lives in it, which the world can compute at any moment and which changes
//! as the ecology does. Naming arrives with the reward that grants it, the same
//! promotion that makes a critter a borg and a lineage a species.
//!
//! # This is the wing's place granularity
//!
//! Isometry's `Overmap` is the same shape one vessel over: sites, links, and a
//! search across them. Kept separate rather than shared, because that one
//! carries travel weights and prepared maps and this one is a partition of a
//! continuous space. Two consumers of a shape is the bar for extracting it, and
//! these are not yet the same shape.

mod bricks;
mod grown;
mod near;
mod relief;
mod soil;

#[cfg(test)]
pub(crate) use bricks::nest_entry;
pub use bricks::{AIR, BRICK, Brick, Ground, NestEntry, ROCK, SOIL, SURFACE_BAND};
pub use grown::{Grown, Nest};
pub use near::{
    BODY_VOXELS_PER_GROUND_VOXEL, Tier, TierLine, WALKER_HEIGHT, WalkerShape, route_step,
    route_step_for, spot, spot_for, step, step_for, surface_stance_for,
};
pub use relief::Relief;
pub use soil::{Column, FORAGE_RADIUS, Soil, SoilError};

use std::collections::{BTreeSet, VecDeque};

use serde::{Deserialize, Serialize};

use crate::record::Scale;
use crate::rng::Rng;

/// One region of the enclosure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceId(pub u16);

/// A region, and where it is centred.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    pub id: PlaceId,
    /// Centre in x and z. Not a boundary: a place ends where the next one is
    /// nearer, so the regions tile the enclosure without storing any edges.
    pub centre: [i32; 2],
}

/// The enclosure, divided.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Places {
    places: Vec<Place>,
    /// Neighbours by index, ascending. Ordered, so traversal is deterministic.
    links: Vec<Vec<PlaceId>>,
}

impl Places {
    /// Scatters `side * side` regions across an enclosure of the given extent.
    ///
    /// One site per grid cell, jittered inside it. Drawn from a stream of the
    /// caller's choosing, so a world can give places their own and leave the
    /// ecology's sequence of draws untouched.
    pub fn scatter(rng: &mut Rng, side: u16, extent: i32) -> Self {
        assert!(side > 0, "an enclosure has at least one place in it");
        let n = side as usize;
        let edge = |i: usize| -extent + (2 * extent * i as i32) / side as i32;

        let mut places = Vec::with_capacity(n * n);
        for row in 0..n {
            for column in 0..n {
                let (x, z) = (edge(column), edge(row));
                places.push(Place {
                    id: PlaceId((row * n + column) as u16),
                    centre: [
                        rng.range_i32(x, (edge(column + 1) - 1).max(x)),
                        rng.range_i32(z, (edge(row + 1) - 1).max(z)),
                    ],
                });
            }
        }

        let links = (0..n * n)
            .map(|index| {
                let (row, column) = (index / n, index % n);
                let mut out = Vec::new();
                if row > 0 {
                    out.push(PlaceId((index - n) as u16));
                }
                if column > 0 {
                    out.push(PlaceId((index - 1) as u16));
                }
                if column + 1 < n {
                    out.push(PlaceId((index + 1) as u16));
                }
                if row + 1 < n {
                    out.push(PlaceId((index + n) as u16));
                }
                out
            })
            .collect();

        Self { places, links }
    }

    pub fn len(&self) -> usize {
        self.places.len()
    }

    pub fn is_empty(&self) -> bool {
        self.places.is_empty()
    }

    pub fn all(&self) -> impl Iterator<Item = &Place> {
        self.places.iter()
    }

    pub fn get(&self, id: PlaceId) -> Option<&Place> {
        self.places.get(id.0 as usize)
    }

    /// Which region a position falls in.
    ///
    /// The nearest site, by exact integer distance. `None` only for an
    /// enclosure with no places in it, which a real world never is.
    pub fn at(&self, position: [i32; 3]) -> Option<PlaceId> {
        self.places
            .iter()
            .min_by_key(|place| {
                let dx = (position[0] - place.centre[0]) as i64;
                let dz = (position[2] - place.centre[1]) as i64;
                // The id breaks ties, so an equidistant point does not depend
                // on iteration order to decide where it is.
                (dx * dx + dz * dz, place.id)
            })
            .map(|place| place.id)
    }

    /// The regions bordering this one.
    pub fn neighbours(&self, id: PlaceId) -> &[PlaceId] {
        self.links
            .get(id.0 as usize)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// How many borders lie between two regions.
    pub fn hops(&self, from: PlaceId, to: PlaceId) -> Option<u32> {
        if self.get(from).is_none() || self.get(to).is_none() {
            return None;
        }
        if from == to {
            return Some(0);
        }

        let mut seen = BTreeSet::from([from]);
        let mut queue = VecDeque::from([(from, 0u32)]);
        while let Some((at, walked)) = queue.pop_front() {
            for next in self.neighbours(at) {
                if !seen.insert(*next) {
                    continue;
                }
                if *next == to {
                    return Some(walked + 1);
                }
                queue.push_back((*next, walked + 1));
            }
        }
        None
    }

    /// How far apart the furthest two regions of a set are.
    ///
    /// **How confined something was**, which is a different question from how
    /// much of the world it covered. Three adjacent regions are a spread; three
    /// scattered ones are everywhere at once, and the count cannot tell them
    /// apart.
    pub fn spread(&self, touched: &BTreeSet<PlaceId>) -> u32 {
        let mut widest = 0;
        for near in touched {
            for far in touched {
                if far <= near {
                    continue;
                }
                widest = widest.max(self.hops(*near, *far).unwrap_or(0));
            }
        }
        widest
    }

    /// How far a set of regions reaches, as the world record measures it.
    ///
    /// One region is [`Scale::Local`]; most of the enclosure is
    /// [`Scale::Worldwide`]; anything between is [`Scale::Regional`]. A strict
    /// majority is the least arbitrary reading of "most of the world" available,
    /// and picking a fraction was the alternative.
    pub fn scale(&self, touched: &BTreeSet<PlaceId>) -> Scale {
        match touched.len() {
            0 | 1 => Scale::Local,
            reached if reached * 2 > self.places.len() => Scale::Worldwide,
            _ => Scale::Regional,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enclosure() -> Places {
        Places::scatter(&mut Rng::from_seed(4_242), 3, 16)
    }

    #[test]
    fn every_position_is_somewhere() {
        // Places cover the enclosure rather than dotting it, because everything
        // in the world is already at a position and has to be in a region.
        let places = enclosure();
        for x in -20..=20 {
            for z in -20..=20 {
                assert!(places.at([x, 0, z]).is_some(), "({x}, {z}) is nowhere");
            }
        }
    }

    #[test]
    fn a_site_is_in_its_own_region() {
        let places = enclosure();
        for place in places.all() {
            let at = [place.centre[0], 0, place.centre[1]];
            assert_eq!(places.at(at), Some(place.id));
        }
    }

    #[test]
    fn height_is_not_a_place() {
        // The enclosure is a few voxels deep. Being higher up is not being
        // somewhere else.
        let places = enclosure();
        assert_eq!(places.at([3, -2, -7]), places.at([3, 9, -7]));
    }

    #[test]
    fn regions_are_irregular_rather_than_a_visible_grid() {
        // The jitter earns its keep: sites sit off-centre, so the boundaries
        // between regions do not line up into the lattice they came from.
        let places = enclosure();
        let side = 32 / 3;
        assert!(
            places.all().any(|place| {
                let offset = |v: i32| (v.rem_euclid(side) - side / 2).abs();
                offset(place.centre[0]) > 1 || offset(place.centre[1]) > 1
            }),
            "a stratified scatter that never strays is just a grid"
        );
    }

    #[test]
    fn the_enclosure_is_connected() {
        // No seed produces a corner nothing can reach, because adjacency is the
        // grid's rather than a function of where the sites happened to land.
        let places = enclosure();
        for near in places.all() {
            for far in places.all() {
                assert!(
                    places.hops(near.id, far.id).is_some(),
                    "{near:?} to {far:?}"
                );
            }
        }
    }

    #[test]
    fn neighbours_are_the_regions_next_door() {
        let places = enclosure();
        // The middle of a three-by-three has four; a corner has two.
        assert_eq!(places.neighbours(PlaceId(4)).len(), 4);
        assert_eq!(places.neighbours(PlaceId(0)).len(), 2);
        assert_eq!(
            places.hops(PlaceId(0), PlaceId(8)),
            Some(4),
            "corner to corner"
        );
    }

    #[test]
    fn spread_separates_a_march_from_an_outbreak() {
        // The fact a count cannot carry, and the reason this is a graph.
        let places = enclosure();
        let adjacent = BTreeSet::from([PlaceId(0), PlaceId(1), PlaceId(2)]);
        let scattered = BTreeSet::from([PlaceId(0), PlaceId(4), PlaceId(8)]);

        assert_eq!(adjacent.len(), scattered.len(), "the same count");
        assert!(
            places.spread(&scattered) > places.spread(&adjacent),
            "and a different fact"
        );
    }

    #[test]
    fn scale_reads_reach_rather_than_area() {
        let places = enclosure();
        let of = |ids: &[u16]| places.scale(&ids.iter().copied().map(PlaceId).collect());

        assert_eq!(of(&[]), Scale::Local, "nowhere is not worldwide");
        assert_eq!(of(&[3]), Scale::Local);
        assert_eq!(of(&[0, 1]), Scale::Regional);
        assert_eq!(
            of(&[0, 1, 2, 3]),
            Scale::Regional,
            "four of nine is not most"
        );
        assert_eq!(of(&[0, 1, 2, 3, 4]), Scale::Worldwide, "five of nine is");
    }

    #[test]
    fn the_same_seed_divides_the_world_the_same_way() {
        assert_eq!(enclosure(), enclosure());
        assert_ne!(enclosure(), Places::scatter(&mut Rng::from_seed(9), 3, 16));
    }

    #[test]
    fn a_division_round_trips() {
        let places = enclosure();
        let bytes = crate::snapshot::encode(&places).unwrap();
        assert_eq!(crate::snapshot::decode::<Places>(&bytes).unwrap(), places);
    }
}
