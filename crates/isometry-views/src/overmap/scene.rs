//! Scoring the pointcrawl, and fitting the solved scene into the viewport.
//!
//! `overmap_score` states the arrangement in Scenograph's portable terms;
//! Scenomise solves it and optionally relaxes it; the final unit-box fit
//! stays on this side because it is a property of Cambium's swatch viewport,
//! not of the score or the scene.
//!
//! Split out of `overmap.rs` on 2026-09-04; unchanged.

use super::*;

/// Adapt a pointcrawl to the shared score contract.
///
/// Authored coordinates choose the geographic arrangement. Uniform default
/// coordinates instead choose the generic spiral: the campaign owns the choice
/// of whether locations were authored, while Scenomise owns the analytic layout.
pub fn overmap_score(overmap: &Overmap) -> Score {
    let authored = overmap
        .nodes
        .first()
        .is_some_and(|first| overmap.nodes.iter().any(|node| node.at != first.at));
    let arrangement = if authored {
        Arrangement::Geographic(Geographic {
            invert_y: false,
            ..Geographic::default()
        })
    } else {
        Arrangement::Spiral(Spiral::default())
    };
    let mut score = Score::new(arrangement);
    score.items = overmap
        .nodes
        .iter()
        .enumerate()
        .map(|(ordinal, node)| ScoreItem {
            source: SourceRef::new(ISOMETRY_OVERMAP_ADAPTER, &node.id),
            ordinal: ordinal as u32,
            footprint: Footprint::Circle { radius: 6.0 },
            representation: Representation::Glyph,
            placement: if authored {
                Placement::Coordinate(Vec2::new(node.at.0 as f32, node.at.1 as f32))
            } else {
                Placement::Ordinal
            },
            layer: 0,
            visible: true,
            // Placement here is authored or ordinal, never derived from a
            // producer-side coordinate, so sceno's three optional placement
            // hints (added 0.0.4) stay unset.
            axis: None,
            embedding: None,
            weight: None,
        })
        .collect();
    score
}

/// Realize an overmap score, then fit the scene's product-neutral coordinates
/// into Cambium's normalized swatch viewport. This is deliberately a view
/// adapter, replacing the old `Overmap::layout` solver rather than relocating it
/// into the campaign model.
pub fn overmap_positions(overmap: &Overmap) -> BTreeMap<String, (f32, f32)> {
    overmap_positions_relaxed(overmap, None)
}

/// The same overmap, optionally loosened by the shared relaxation before it is
/// normalized: sites push apart, routes pull toward their rest length, and the
/// arrangement keeps drawing them back toward the slots it chose. Physics is a
/// capability of any graph surface, not of the one canvas that owns a rigid-body
/// world — a swatch gets the same reading at swatch cost, and `None` keeps the
/// purely analytic placement.
pub fn overmap_positions_relaxed(
    overmap: &Overmap,
    relaxation: Option<scenomise::Relaxation>,
) -> BTreeMap<String, (f32, f32)> {
    let mut scene = scenomise::solve(&overmap_score(overmap));
    if let Some(settings) = relaxation {
        scenomise::relax(&mut scene, &settings);
    }
    normalize_for_swatch(&scene)
}

fn normalize_for_swatch(scene: &sceno::Scene) -> BTreeMap<String, (f32, f32)> {
    const MARGIN: f32 = 0.08;
    let raw: Vec<_> = scene
        .items
        .iter()
        .filter_map(|item| {
            scene
                .sources
                .get(item.source.0 as usize)
                .map(|source| (source.id.clone(), item.transform.translate))
        })
        .collect();
    if raw.is_empty() {
        return BTreeMap::new();
    }
    let (mut min_x, mut min_y, mut max_x, mut max_y) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    for (_, point) in &raw {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }
    let span = 1.0 - 2.0 * MARGIN;
    let fit = |value: f32, low: f32, high: f32| {
        if high - low < 1e-4 {
            0.5
        } else {
            MARGIN + (value - low) / (high - low) * span
        }
    };
    raw.into_iter()
        .map(|(id, point)| (id, (fit(point.x, min_x, max_x), fit(point.y, min_y, max_y))))
        .collect()
}
#[cfg(test)]
mod relax_tests {
    use super::*;
    use isometry_core::OvermapNode;

    fn node(id: &str, at: (i32, i32)) -> OvermapNode {
        OvermapNode {
            id: id.to_owned(),
            name: id.to_owned(),
            at,
            site: None,
        }
    }

    /// A swatch can run physics. The shared relaxation loosens the overmap in
    /// place -- no rigid-body world at swatch scale -- and placement changes
    /// while membership and the normalized viewport fit do not.
    #[test]
    fn a_relaxed_overmap_still_places_every_site() {
        let mut overmap = Overmap::new("shore");
        overmap.nodes = vec![
            node("west", (0, 4)),
            node("east", (10, 0)),
            node("north", (5, 9)),
        ];
        let analytic = overmap_positions(&overmap);
        let relaxed = overmap_positions_relaxed(&overmap, Some(scenomise::Relaxation::default()));

        assert_eq!(
            analytic.keys().collect::<Vec<_>>(),
            relaxed.keys().collect::<Vec<_>>(),
            "relaxation changes placement, never membership"
        );
        for (_, (x, y)) in &relaxed {
            assert!(
                (0.0..=1.0).contains(x) && (0.0..=1.0).contains(y),
                "a relaxed site still fits the swatch viewport"
            );
        }
        assert_eq!(
            relaxed,
            overmap_positions_relaxed(&overmap, Some(scenomise::Relaxation::default())),
            "swatch physics is deterministic"
        );
    }
}
