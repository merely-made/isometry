//! Line-of-sight visibility over the tile grid: which tiles a token (or a
//! set of a player's tokens) can currently see. Substrate geometry, the
//! same shape as movement's [`MoveRules`](crate::MoveRules): a sight
//! radius plus an opacity predicate the caller supplies. What blocks
//! sight (a tree, a wall kind) is the caller's business; the geometry is
//! ours.
//!
//! This is the raw visibility set. Fog-of-war presentation (explored
//! memory, dimming, hiding enemy tokens) is a view concern layered on
//! top, and whether the map is *filtered on the wire* (server-authority
//! hidden information) versus *filtered at render* is a session-policy
//! choice above this module. This module just answers "what can a token
//! at X see, out to radius R, through these walls."

use std::collections::HashSet;

use crate::iso::TileCoord;
use crate::map::MapDocument;

/// Sight constraints: how far a token sees and what stops its line of
/// sight. `opaque` is applied to a tile's blocking kind name (its prop if
/// it has one, else its ground kind).
pub struct SightRules<'a> {
    pub radius: u32,
    pub opaque: &'a dyn Fn(&str) -> bool,
}

/// Finite obstacles for a height-field map. Heights use the same units as
/// map elevation; the caller supplies material policy. A zero obstacle
/// height still allows raised terrain itself to hide lower ground.
pub struct HeightSightRules<'a> {
    pub radius: u32,
    pub eye_height: u32,
    pub obstacle_height: &'a dyn Fn(&str) -> u32,
}

/// Visibility between eye points above tile surfaces, using the same grid
/// rays as flat visibility. Nearby canopy can hide a walker while a lookout
/// sees over it. This is a height-field approximation, not volume ray tracing.
pub fn visible_from_height(
    map: &MapDocument,
    origin: TileCoord,
    rules: &HeightSightRules,
) -> HashSet<TileCoord> {
    let mut out = HashSet::new();
    if !map.ground.in_bounds(origin.0, origin.1) {
        return out;
    }
    let elevation =
        |at: TileCoord| f64::from(*map.elevation.get(at.0 as u32, at.1 as u32).unwrap_or(&0));
    let eye = f64::from(rules.eye_height);
    let start = elevation(origin) + eye;
    // Clamp enumeration to the map rather than allocating off-map rays.
    let radius = i64::from(rules.radius);
    let min_col = (i64::from(origin.0) - radius).max(0) as u32;
    let max_col = (i64::from(origin.0) + radius).min(i64::from(map.ground.width()) - 1) as u32;
    let min_row = (i64::from(origin.1) - radius).max(0) as u32;
    let max_row = (i64::from(origin.1) + radius).min(i64::from(map.ground.height()) - 1) as u32;
    for row in min_row..=max_row {
        for col in min_col..=max_col {
            let target = (col as i32, row as i32);
            let dx = i64::from(target.0) - i64::from(origin.0);
            let dy = i64::from(target.1) - i64::from(origin.1);
            if i128::from(dx) * i128::from(dx) + i128::from(dy) * i128::from(dy)
                > i128::from(radius) * i128::from(radius)
            {
                continue;
            }
            let line = bresenham(origin, target);
            let end = elevation(target) + eye;
            let steps = line.len().saturating_sub(1);
            let clear = line
                .iter()
                .enumerate()
                .skip(1)
                .take(steps.saturating_sub(1))
                .all(|(index, &at)| {
                    let (c, r) = (at.0 as u32, at.1 as u32);
                    let obstacle = [map.props.get(c, r), map.ground.get(c, r)]
                        .into_iter()
                        .flatten()
                        .filter(|kind| kind.0 != 0)
                        .map(|kind| (rules.obstacle_height)(kind_name(map, *kind)))
                        .max()
                        .unwrap_or(0);
                    let top = elevation(at) + f64::from(obstacle);
                    let ray = start + (end - start) * index as f64 / steps as f64;
                    top < ray
                });
            if clear {
                out.insert(target);
            }
        }
    }
    out
}

fn kind_name(map: &MapDocument, kind: crate::map::TileKindId) -> &str {
    map.tile_kinds
        .get(kind.0 as usize)
        .map(String::as_str)
        .unwrap_or("empty")
}

/// Whether tile `at` stops a line of sight passing through it. Out of
/// bounds blocks. A tile blocks if its prop kind is opaque, or (no
/// opaque prop) its ground kind is opaque.
fn blocks(map: &MapDocument, at: TileCoord, opaque: &dyn Fn(&str) -> bool) -> bool {
    if !map.ground.in_bounds(at.0, at.1) {
        return true;
    }
    let (c, r) = (at.0 as u32, at.1 as u32);
    if let Some(prop) = map.props.get(c, r) {
        if prop.0 != 0 && opaque(kind_name(map, *prop)) {
            return true;
        }
    }
    match map.ground.get(c, r) {
        Some(g) => g.0 != 0 && opaque(kind_name(map, *g)),
        None => true,
    }
}

/// The cells a Bresenham line from `a` to `b` passes through, inclusive.
fn bresenham(a: TileCoord, b: TileCoord) -> Vec<TileCoord> {
    let (mut x0, mut y0) = a;
    let (x1, y1) = b;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut pts = vec![(x0, y0)];
    while (x0, y0) != (x1, y1) {
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
        pts.push((x0, y0));
    }
    pts
}

/// Whether `from` has clear line of sight to `to`: every cell strictly
/// between them is non-blocking. The target itself may be opaque — you
/// see the wall you're looking at, just not past it.
fn los_clear(
    map: &MapDocument,
    from: TileCoord,
    to: TileCoord,
    opaque: &dyn Fn(&str) -> bool,
) -> bool {
    let line = bresenham(from, to);
    if line.len() <= 2 {
        return true; // adjacent or same tile
    }
    line[1..line.len() - 1]
        .iter()
        .all(|&cell| !blocks(map, cell, opaque))
}

/// Every tile visible from `origin` within the sight radius. A tile is
/// visible if it is in range and line of sight to it is clear. The origin
/// tile is always visible.
pub fn visible_from(
    map: &MapDocument,
    origin: TileCoord,
    rules: &SightRules,
) -> HashSet<TileCoord> {
    let mut out = HashSet::new();
    out.insert(origin);
    let r = rules.radius as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy > r * r {
                continue;
            }
            let target = (origin.0 + dx, origin.1 + dy);
            if map.ground.in_bounds(target.0, target.1)
                && los_clear(map, origin, target, rules.opaque)
            {
                out.insert(target);
            }
        }
    }
    out
}

/// The union of what every origin (a player's tokens) can see.
pub fn visible_tiles(
    map: &MapDocument,
    origins: &[TileCoord],
    rules: &SightRules,
) -> HashSet<TileCoord> {
    let mut out = HashSet::new();
    for &origin in origins {
        out.extend(visible_from(map, origin, rules));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::TileKindId;

    fn field(w: u32, h: u32) -> MapDocument {
        let mut m = MapDocument::new("sight", w, h);
        let grass = m.intern_tile_kind("grass");
        for r in 0..h {
            for c in 0..w {
                m.ground.set(c, r, grass);
            }
        }
        m
    }

    fn rules(radius: u32) -> SightRules<'static> {
        SightRules {
            radius,
            opaque: &|kind| kind == "wall" || kind == "tree",
        }
    }

    #[test]
    fn open_field_sees_everything_in_radius() {
        let m = field(11, 11);
        let seen = visible_from(&m, (5, 5), &rules(3));
        assert!(seen.contains(&(5, 5)));
        assert!(seen.contains(&(8, 5))); // 3 east, in range
        assert!(seen.contains(&(5, 2))); // 3 north
        assert!(!seen.contains(&(9, 5))); // 4 east, out of range
        assert!(!seen.contains(&(8, 8))); // ~4.2 diagonal, out of round range
    }

    #[test]
    fn a_wall_blocks_tiles_behind_it() {
        let mut m = field(11, 11);
        let wall = m.intern_tile_kind("wall");
        // A wall segment due east of the viewer at (5,5).
        m.props.set(7, 5, wall);
        let seen = visible_from(&m, (5, 5), &rules(5));
        assert!(seen.contains(&(7, 5)), "the wall tile itself is seen");
        assert!(
            !seen.contains(&(9, 5)),
            "the tile behind the wall is hidden"
        );
        assert!(seen.contains(&(7, 3)), "an off-axis tile is still seen");
    }

    #[test]
    fn union_over_two_tokens_covers_both_neighborhoods() {
        let m = field(20, 20);
        let seen = visible_tiles(&m, &[(2, 2), (15, 15)], &rules(2));
        assert!(seen.contains(&(2, 2)));
        assert!(seen.contains(&(3, 2)));
        assert!(seen.contains(&(15, 15)));
        assert!(seen.contains(&(14, 15)));
        assert!(!seen.contains(&(8, 8)), "the gap between them stays dark");
    }

    #[test]
    fn ground_opacity_also_blocks() {
        let mut m = field(11, 11);
        let wall = m.intern_tile_kind("wall");
        m.ground.set(6, 5, wall); // opaque ground, not a prop
        let seen = visible_from(&m, (5, 5), &rules(5));
        assert!(!seen.contains(&(8, 5)));
        assert_eq!(TileKindId(0).0, 0); // empty sanity
    }

    #[test]
    fn lookout_sees_over_nearby_canopy_but_walker_cannot() {
        let mut m = field(9, 1);
        let tree = m.intern_tile_kind("forest-tree");
        m.props.set(2, 0, tree);
        let rules = HeightSightRules {
            radius: 8,
            eye_height: 1,
            obstacle_height: &|kind| if kind == "forest-tree" { 3 } else { 0 },
        };
        let ground = visible_from_height(&m, (0, 0), &rules);
        assert!(ground.contains(&(2, 0)), "canopy itself is visible");
        assert!(!ground.contains(&(8, 0)), "canopy hides the far clearing");
        m.elevation.set(0, 0, 4);
        let lookout = visible_from_height(&m, (0, 0), &rules);
        assert!(
            lookout.contains(&(8, 0)),
            "tower eye line clears nearby canopy"
        );
        m.props.set(2, 0, TileKindId(0));
        m.props.set(6, 0, tree);
        assert!(
            !visible_from_height(&m, (0, 0), &rules).contains(&(8, 0)),
            "distant canopy still hides ground immediately behind it"
        );
    }

    #[test]
    fn elevated_terrain_blocks_and_radius_still_bounds_sight() {
        let mut m = field(9, 1);
        m.elevation.set(3, 0, 2);
        let rules = HeightSightRules {
            radius: 5,
            eye_height: 1,
            obstacle_height: &|_| 0,
        };
        let seen = visible_from_height(&m, (0, 0), &rules);
        assert!(seen.contains(&(3, 0)), "ridge surface is visible");
        assert!(!seen.contains(&(4, 0)), "ridge hides its far side");
        m.elevation.set(3, 0, 0);
        let seen = visible_from_height(&m, (0, 0), &rules);
        assert!(seen.contains(&(5, 0)));
        assert!(!seen.contains(&(6, 0)));
        assert!(visible_from_height(&m, (-1, 0), &rules).is_empty());
    }
}
