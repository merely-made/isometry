//! The board's tile layers: ground, shroud, and props.
//!
//! Each pass returns the elements for one layer over the visible window,
//! already culled and depth-keyed. Nothing here decides *where* a cell sits:
//! `placed` and `standing_on` stay at the roof, because the tokens and the
//! menu place things through exactly the same two helpers.
//!
//! Split out of `board.rs` on 2026-09-04; unchanged.

use super::*;

/// The ground layer: a column of filler diamonds up to the tile's
/// elevation, then the top diamond carrying the kind class. In Play
/// mode the selected token's reach tints blue and the hovered path
/// tints lighter still.
pub(super) fn ground_tiles(ui: &UiState) -> Vec<UiChild> {
    let map = &ui.map;
    // The portable board scene decides which authored tile instances are live;
    // the local isometric painter keeps responsibility for elevation, fog, and
    // DOM interaction.
    let projected_tiles = tile_board_cells(map);
    let playing = ui.mode == EditMode::Play;
    let path: HashSet<TileCoord> = if playing {
        ui.hover_tile
            .filter(|t| ui.reach.contains_key(t))
            .map(|t| path_to(&ui.reach, t).into_iter().collect())
            .unwrap_or_default()
    } else {
        HashSet::new()
    };
    let template: HashSet<TileCoord> = ui.template_preview();
    let doors = ui.door_tiles();
    let mut out: Vec<UiChild> = Vec::new();
    for (col, row, kind) in map.ground.iter() {
        if kind.0 == 0 {
            continue;
        }
        let at: TileCoord = (col as i32, row as i32);
        if !projected_tiles.contains(&at) {
            continue;
        }
        if !in_view(ui, at) {
            continue; // outside the pane: windowing skips the emit
        }
        let fog = ui.fog_level(at);
        if fog == FogLevel::Hidden {
            continue; // unexplored: the dark pane shows through
        }
        let elev = *map.elevation.get(col, row).unwrap_or(&0) as i32;
        for step in 0..elev {
            out.push(tile_el(ui, at, step, "tile tile-under".to_owned()));
        }
        let mut class = format!("tile tile-{}", kind_name(map, *kind));
        if (col + row) % 2 == 1 {
            class.push_str(" alt");
        }
        if ui.selected == Some(at) {
            class.push_str(" tile-selected");
        }
        if playing {
            if path.contains(&at) {
                class.push_str(" tile-path");
            } else if ui.reach.contains_key(&at) {
                class.push_str(" tile-reach");
            }
        }
        if template.contains(&at) {
            class.push_str(" tile-template");
        }
        // A transition point: walk onto it and you are on the other map.
        if doors.contains(&at) {
            class.push_str(" tile-door");
        }
        out.push(tile_el(ui, at, elev, class));
        if fog == FogLevel::Dim {
            out.push(shroud_el(ui, at, elev)); // remembered terrain, dimmed
        }
    }
    out
}

/// A dim overlay diamond over an explored-but-unseen tile.
fn shroud_el(ui: &UiState, at: TileCoord, elev: i32) -> UiChild {
    let geo = &ui.geo;
    let (cx, cy) = geo.tile_to_screen(at, elev);
    let (x, y) = (cx - geo.tile_w / 2.0, cy - geo.tile_h / 2.0);
    let z = depth_key(at, elev) + 2;
    standing_on(
        el("div", ())
            .attr("class", "fog-shroud")
            .attr("style", placed(ui, (x, y), TILE_BOX, z)),
        at,
    )
}

/// Props stand on their tile: anchored bottom-center on the diamond,
/// one depth step above the ground they occupy.
pub(super) fn prop_tiles(ui: &UiState) -> Vec<UiChild> {
    let map = &ui.map;
    let geo = &ui.geo;
    let mut out: Vec<UiChild> = Vec::new();
    for (col, row, kind) in map.props.iter() {
        if kind.0 == 0 {
            continue;
        }
        let at: TileCoord = (col as i32, row as i32);
        if !in_view(ui, at) {
            continue;
        }
        if ui.fog_level(at) == FogLevel::Hidden {
            continue;
        }
        let elev = *map.elevation.get(col, row).unwrap_or(&0) as i32;
        let (cx, cy) = geo.tile_to_screen(at, elev);
        let z = depth_key(at, elev) + 1;
        let class = format!("prop prop-{}", kind_name(map, *kind));
        // 20x24 body, base at the diamond center. The anchor rides the board's
        // own scale with the box, or a scaled prop would float off its tile.
        let s = ui.board_scale;
        let (x, y) = (cx - PROP_BOX.0 / 2.0 * s, cy - PROP_BOX.1 * s);
        out.push(standing_on(
            el("div", ())
                .attr("class", class)
                .attr("style", placed(ui, (x, y), PROP_BOX, z)),
            at,
        ));
    }
    out
}
