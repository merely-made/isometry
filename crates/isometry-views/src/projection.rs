//! Isometry's adapters to the product-free Scenograph score contract.
//!
//! Campaign data and isometric paint stay local. These functions only choose
//! portable arrangements, measured footprints, and opaque source references.

use std::collections::BTreeSet;

use isometry_core::{MapDocument, TileCoord};
use sceno::{
    Arrangement, Backdrop, Footprint, Grid, Placement, Rect, Representation, Scene, Score,
    ScoreItem, Size2, SourceRef, Transform2, Vec2,
};

/// Opaque source lane for tactical-board tiles.
pub const ISOMETRY_TILE_BOARD_ADAPTER: &str = "isometry.tile-board";

/// Open face used for the authored board surface behind selectable tiles.
pub const ISOMETRY_TILE_BOARD_BACKDROP: &str = "isometry:tile-board";

/// Score the authored ground tiles as a regular board. The isometric painter
/// consumes the resulting scene for membership while retaining its own diamond
/// geometry, elevation, fog, and interaction rules.
pub fn tile_board_score(map: &MapDocument) -> Score {
    let mut score = Score::new(Arrangement::Grid(Grid {
        origin: Vec2::ZERO,
        cell: Vec2::new(32.0, 32.0),
        columns: map.ground.width(),
        gap: 0.0,
    }));
    score.items = map
        .ground
        .iter()
        .filter(|(_, _, kind)| kind.0 != 0)
        .enumerate()
        .map(|(ordinal, (column, row, _))| ScoreItem {
            source: SourceRef::new(ISOMETRY_TILE_BOARD_ADAPTER, format!("{column}:{row}")),
            ordinal: ordinal as u32,
            footprint: Footprint::Rect {
                size: Size2::new(32.0, 32.0),
            },
            representation: Representation::Sprite,
            placement: Placement::Cell {
                column: column as i32,
                row: row as i32,
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

/// Realize the board through the same score-to-scene path as the overmap.
pub fn tile_board_scene(map: &MapDocument) -> Scene {
    let mut scene = scenomise::solve(&tile_board_score(map));
    let board_size = Size2::new(
        map.ground.width() as f32 * 32.0,
        map.ground.height() as f32 * 32.0,
    );
    let board_origin = Vec2::new(-16.0, -16.0);
    let source = scene.intern_source(SourceRef::new(
        ISOMETRY_TILE_BOARD_ADAPTER,
        format!("map:{}", map.name),
    ));
    scene.backdrops.push(Backdrop {
        source,
        space: Scene::WORLD,
        transform: Transform2::translation(
            board_origin.x + board_size.w * 0.5,
            board_origin.y + board_size.h * 0.5,
        ),
        footprint: Footprint::Rect { size: board_size },
        kind: ISOMETRY_TILE_BOARD_BACKDROP.into(),
        visible: true,
        collidable: false,
    });
    scene.bounds = Rect::new(board_origin, board_size);
    scene
}

/// Extract the ground members the board renderer should paint. The solver owns
/// which score instances become scene items; board-specific paint owns how a
/// tile is drawn.
pub fn tile_board_cells(map: &MapDocument) -> BTreeSet<TileCoord> {
    let scene = tile_board_scene(map);
    scene
        .items
        .iter()
        .filter(|item| item.visible)
        .filter_map(|item| scene.sources.get(item.source.0 as usize))
        .filter(|source| source.adapter == ISOMETRY_TILE_BOARD_ADAPTER)
        .filter_map(|source| source.id.split_once(':'))
        .filter_map(|(column, row)| Some((column.parse().ok()?, row.parse().ok()?)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_core::TileKindId;

    #[test]
    fn tile_board_round_trips_through_the_shared_scene_contract() {
        let mut map = MapDocument::new("receipt", 3, 2);
        map.ground.set(2, 1, TileKindId(1));
        map.ground.set(0, 0, TileKindId(2));
        let score = tile_board_score(&map);
        assert!(matches!(score.arrangement, Arrangement::Grid(_)));
        let scene = tile_board_scene(&map);
        assert_eq!(scene.backdrops.len(), 1);
        assert_eq!(scene.backdrops[0].kind, ISOMETRY_TILE_BOARD_BACKDROP);
        assert!(scene.backdrops[0].visible);
        assert!(!scene.backdrops[0].collidable);
        assert_eq!(tile_board_cells(&map), BTreeSet::from([(0, 0), (2, 1)]));
        let tile = scene
            .items
            .iter()
            .find(|item| scene.sources[item.source.0 as usize].id == "2:1")
            .unwrap();
        assert_eq!(tile.transform.translate, Vec2::new(64.0, 32.0));
    }
}
