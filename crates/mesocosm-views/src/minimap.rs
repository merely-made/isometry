// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The minimap adapter: who holds where, disclosed as a score.
//!
//! # The cells are simulation truth
//!
//! `Places::at` assigns a position to its nearest site; `Arrangement::Hulls`
//! partitions by the same rule. So the cells a solved minimap scene draws are
//! **exactly** the regions the world itself reasons with, not a cartographer's
//! approximation of them. A congruence test holds the two rules together.
//!
//! # Dominance is derived, never stored
//!
//! Which lineage holds a region is read off the living roster at projection
//! time, the same discipline as capability, temperament, and the possibility
//! space: a stored verdict drifts from the facts it was drawn from, and a
//! derived one cannot.

use std::collections::BTreeMap;

use mesocosm_core::{PlaceId, SpeciesId, World};
use sceno::{
    Arrangement, Footprint, Hulls, Placement, Rect, Representation, Scene, Score, ScoreItem, Size2,
    SourceRef, Vec2,
};
use sprigging::ColorF;

/// The adapter name every minimap source reference carries.
pub const MINIMAP_ADAPTER: &str = "mesocosm";

/// Discloses the enclosure's places as a hulls score.
///
/// World x maps to scene x and world z to scene y, untransformed: the score's
/// units are voxels, `invert_y` is off, and a host that wants a different
/// framing scales the realized scene rather than the facts.
pub fn minimap_score(world: &World) -> Score {
    let extent = mesocosm_core::world::ENCLOSURE as f32;
    let mut score = Score::new(Arrangement::Hulls(Hulls {
        origin: Vec2::ZERO,
        units_per_coordinate: 1.0,
        invert_y: false,
        bounds: Rect::new(
            Vec2::new(-extent, -extent),
            Size2::new(extent * 2.0, extent * 2.0),
        ),
    }));

    for place in world.places().all() {
        score.items.push(ScoreItem {
            source: SourceRef::new(MINIMAP_ADAPTER, format!("place:{}", place.id.0)),
            ordinal: place.id.0 as u32,
            footprint: Footprint::Point,
            representation: Representation::Glyph,
            placement: Placement::Coordinate(Vec2::new(
                place.centre[0] as f32,
                place.centre[1] as f32,
            )),
            layer: 0,
            visible: true,
            // A place discloses where it is and nothing else. The score's other
            // disclosure fields exist for arrangements that place along an axis
            // or from precomputed coordinates; Hulls reads the coordinate above.
            axis: None,
            embedding: None,
            weight: None,
        });
    }
    score
}

/// The solved minimap scene: one region per place, cells tiling the enclosure.
pub fn minimap_scene(world: &World) -> Scene {
    scenomise::solve(&minimap_score(world))
}

/// Which lineage holds each region, by biomass of the living.
///
/// `None` where nothing lives. Ties break toward the older (lower) id, so the
/// answer is deterministic and an insurgent has to actually pass the incumbent
/// rather than merely match it.
pub fn dominant_lineages(world: &World) -> BTreeMap<PlaceId, Option<SpeciesId>> {
    let mut biomass: BTreeMap<(PlaceId, SpeciesId), u64> = BTreeMap::new();
    for organism in world.living() {
        if let Some(place) = world.places().at(organism.position) {
            *biomass.entry((place, organism.species)).or_default() += organism.biomass_mg();
        }
    }

    let mut holders: BTreeMap<PlaceId, Option<SpeciesId>> =
        world.places().all().map(|place| (place.id, None)).collect();
    let mut best: BTreeMap<PlaceId, u64> = BTreeMap::new();
    for ((place, species), mass) in biomass {
        // Strictly-greater keeps the tie with the incumbent: BTreeMap yields
        // lower species ids first.
        if mass > best.get(&place).copied().unwrap_or(0) {
            best.insert(place, mass);
            holders.insert(place, Some(species));
        }
    }
    holders
}

/// A lineage's colour, everywhere it appears.
///
/// Golden-angle hues off the species id: deterministic, unbounded, and no two
/// nearby ids look alike. Desaturated toward moss rather than neon, because
/// the minimap sits over a living enclosure.
pub fn lineage_tint(species: SpeciesId) -> ColorF {
    let hue = (species.0 as f32 * 137.507_77) % 360.0;
    let (r, g, b) = hsl(hue, 0.45, 0.55);
    ColorF::new(r, g, b, 1.0)
}

fn hsl(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match h as u32 {
        0..60 => (c, x, 0.0),
        60..120 => (x, c, 0.0),
        120..180 => (0.0, c, x),
        180..240 => (0.0, x, c),
        240..300 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (r + m, g + m, b + m)
}

/// The whole minimap, assembled: solved scene, resolved tints, player mark.
///
/// The one call a host makes per refresh. Tint resolution walks scene regions
/// back through their member's source ref, so the leaf never learns what a
/// place is.
pub fn minimap_leaf(world: &World) -> crate::leaf::MinimapLeaf {
    let scene = minimap_scene(world);
    let holders = dominant_lineages(world);
    let tints = scene
        .regions
        .iter()
        .map(|region| {
            let item = &scene.items[region.members[0].0 as usize];
            let source = &scene.sources[item.source.0 as usize];
            let id: u16 = source
                .id
                .trim_start_matches("place:")
                .parse()
                .expect("minimap sources are places");
            holders
                .get(&PlaceId(id))
                .copied()
                .flatten()
                .map(lineage_tint)
        })
        .collect();
    let player = world.position().map(|p| (p[0] as f32, p[2] as f32));
    crate::leaf::MinimapLeaf::new(scene, tints, player)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::Intent;

    fn world() -> World {
        let mut world = World::new(4_242, 40);
        for _ in 0..50 {
            world.apply(Intent::Idle);
        }
        world
    }

    #[test]
    fn every_place_is_disclosed_as_a_site() {
        let world = world();
        let score = minimap_score(&world);
        assert_eq!(score.items.len(), world.places().len());
        assert!(matches!(score.arrangement, Arrangement::Hulls(_)));
    }

    #[test]
    fn the_scene_cells_are_the_simulation_regions() {
        // The claim the whole minimap rests on: the solver's nearest-site rule
        // and `Places::at` are the same rule, so a sampled position lands in
        // the scene cell of exactly the place the world says it is in.
        let world = world();
        let scene = minimap_scene(&world);
        assert_eq!(scene.regions.len(), world.places().len());

        for (x, z) in [(-14, -9), (0, 0), (11, 3), (15, -15), (-2, 13)] {
            let simulated = world
                .places()
                .at([x, 0, z])
                .expect("everywhere is somewhere");
            let drawn = scene
                .regions
                .iter()
                .find(|region| match &region.contour {
                    Some(Footprint::Polygon { points }) => {
                        polygon_contains(points, x as f32, z as f32)
                    },
                    _ => false,
                })
                .expect("the cells tile the enclosure");
            let member = drawn.members[0];
            let source = &scene.sources[scene.items[member.0 as usize].source.0 as usize];
            assert_eq!(
                source.id,
                format!("place:{}", simulated.0),
                "scene and simulation disagree about ({x}, {z})"
            );
        }
    }

    #[test]
    fn dominance_is_a_reading_of_the_living() {
        let world = world();
        let holders = dominant_lineages(&world);

        assert_eq!(holders.len(), world.places().len(), "every region answers");
        assert!(
            holders.values().any(Option::is_some),
            "forty critters dominate somewhere"
        );

        // Derived, so reading twice changes nothing.
        assert_eq!(holders, dominant_lineages(&world));
    }

    #[test]
    fn an_empty_region_has_no_holder() {
        // A fresh world with a single critter holds exactly one region.
        let world = World::new(7, 0);
        let holders = dominant_lineages(&world);
        let held = holders.values().filter(|h| h.is_some()).count();
        assert_eq!(held, 1, "one critter, one held region");
    }

    #[test]
    fn tints_are_deterministic_and_distinct() {
        let a = lineage_tint(SpeciesId(1));
        assert_eq!(a, lineage_tint(SpeciesId(1)));
        for other in 2..30u32 {
            let b = lineage_tint(SpeciesId(other));
            assert!(
                (a.r - b.r).abs() + (a.g - b.g).abs() + (a.b - b.b).abs() > 0.05,
                "species 1 and {other} are indistinguishable"
            );
        }
    }

    pub(super) fn polygon_contains(points: &[Vec2], x: f32, y: f32) -> bool {
        let mut inside = false;
        for (index, a) in points.iter().enumerate() {
            let b = points[(index + 1) % points.len()];
            if (a.y > y) != (b.y > y) && x < a.x + (b.x - a.x) * (y - a.y) / (b.y - a.y) {
                inside = !inside;
            }
        }
        inside
    }
}
