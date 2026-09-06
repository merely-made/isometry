//! The board's tile layers: ground, shroud, and props.
//!
//! Each pass returns the elements for one layer over the visible window,
//! already culled and depth-keyed. Nothing here decides *where* a cell sits:
//! `placed` and `standing_on` stay at the roof, because the tokens and the
//! menu place things through exactly the same two helpers.
//!
//! Split out of `board.rs` on 2026-09-04; unchanged.

use super::*;

/// The ground layer: explicit foreground cliff faces below an elevated tile,
/// then its top diamond carrying the kind class. In Play mode the selected
/// token's reach tints blue and the hovered path tints lighter still.
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
        // The two neighbours deeper into the screen own the visible seam.
        // Faces are emitted immediately before their roof at the same depth,
        // so the roof caps them without depending on diamond raster seams.
        for face in visible_faces(map, at, elev) {
            out.push(face_el(ui, at, elev, face));
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

/// A visible vertical side of an elevated tile. In this locked projection,
/// `col + 1` is the lower-right neighbour and `row + 1` the lower-left one.
/// Those are the two edges that can face the viewer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FaceSide {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VisibleFace {
    side: FaceSide,
    /// Height units exposed between this roof and the foreground neighbour.
    steps: i32,
}

/// Return exactly the exposed foreground faces for `at`. A flat seam produces
/// no element, and a missing/empty neighbour is ground level. Keeping this
/// structural rule separate from DOM emission makes it hard to drift back to
/// the old stack-of-diamonds illusion.
fn visible_faces(map: &MapDocument, at: TileCoord, elevation: i32) -> Vec<VisibleFace> {
    [(FaceSide::Right, (1, 0)), (FaceSide::Left, (0, 1))]
        .into_iter()
        .filter_map(|(side, (dc, dr))| {
            let neighbour = (at.0 + dc, at.1 + dr);
            let steps = elevation - tile_elevation(map, neighbour);
            (steps > 0).then_some(VisibleFace { side, steps })
        })
        .collect()
}

/// The height visible at a neighbouring board cell. Authored elevation on an
/// empty cell is not terrain, so it must not conceal a cliff face.
fn tile_elevation(map: &MapDocument, (col, row): TileCoord) -> i32 {
    if col < 0 || row < 0 {
        return 0;
    }
    let (col, row) = (col as u32, row as u32);
    map.ground
        .get(col, row)
        .filter(|kind| kind.0 != 0)
        .and_then(|_| map.elevation.get(col, row))
        .copied()
        .unwrap_or(0) as i32
}

/// One explicit trapezoid below a roof edge. Its upper edge is the roof edge;
/// its lower edge lands exactly on the shorter foreground neighbour's roof.
/// It shares the roof's painter key and is emitted first, which keeps tokens
/// and foreground roofs on their established depth order.
fn face_el(ui: &UiState, at: TileCoord, elevation: i32, face: VisibleFace) -> UiChild {
    let geo = &ui.geo;
    let (cx, cy) = geo.tile_to_screen(at, elevation);
    let half_w = geo.tile_w / 2.0;
    let roof_half_h = geo.tile_h / 2.0;
    let drop = face.steps as f32 * geo.elev_step;
    let height = roof_half_h + drop;
    let (x, class, clip) = match face.side {
        FaceSide::Right => (
            cx,
            "tile-face tile-face-right",
            format!("polygon(100% 0%, 0% {roof_half_h}px, 0% 100%, 100% {drop}px)"),
        ),
        FaceSide::Left => (
            cx - half_w,
            "tile-face tile-face-left",
            format!("polygon(0% 0%, 100% {roof_half_h}px, 100% 100%, 0% {drop}px)"),
        ),
    };
    let z = depth_key(at, elevation);
    let kind = ui
        .map
        .ground
        .get(at.0 as u32, at.1 as u32)
        .copied()
        .unwrap_or_default();
    let class = format!("{class} tile-face-{}", kind_name(&ui.map, kind));
    let style = format!(
        "left: {x}px; top: {cy}px; width: {half_w}px; height: {height}px; \
         z-index: {z}; clip-path: {clip};"
    );
    standing_on(
        clickable(
            el("div", ()).attr("class", class).attr("style", style),
            move |ui: &mut UiState, _| {
                ui.click_tile(at);
            },
        ),
        at,
    )
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
        let name = kind_name(map, *kind);
        let class = format!("prop prop-{name}");
        let size = if name == "forest-tree" {
            (32.0, 40.0)
        } else {
            PROP_BOX
        };
        // Base at the diamond center. The anchor rides the board's
        // own scale with the box, or a scaled prop would float off its tile.
        let s = ui.board_scale;
        let (x, y) = (cx - size.0 / 2.0 * s, cy - size.1 * s);
        out.push(standing_on(
            el("div", ())
                .attr("class", class)
                .attr("style", placed(ui, (x, y), size, z)),
            at,
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grassy_map() -> MapDocument {
        let mut map = MapDocument::new("faces", 3, 3);
        let grass = map.intern_tile_kind("grass");
        for row in 0..3 {
            for col in 0..3 {
                map.ground.set(col, row, grass);
            }
        }
        map
    }

    #[test]
    fn equal_foreground_heights_emit_no_internal_faces() {
        let mut map = grassy_map();
        map.elevation.set(1, 1, 2);
        map.elevation.set(2, 1, 2);
        map.elevation.set(1, 2, 2);

        assert!(visible_faces(&map, (1, 1), 2).is_empty());
    }

    #[test]
    fn faces_only_cover_the_exposed_foreground_height() {
        let mut map = grassy_map();
        map.elevation.set(1, 1, 4);
        map.elevation.set(2, 1, 1);
        map.elevation.set(1, 2, 3);

        assert_eq!(
            visible_faces(&map, (1, 1), 4),
            vec![
                VisibleFace {
                    side: FaceSide::Right,
                    steps: 3,
                },
                VisibleFace {
                    side: FaceSide::Left,
                    steps: 1,
                },
            ]
        );
    }

    #[test]
    fn an_empty_foreground_cell_exposes_the_full_cliff() {
        let mut map = MapDocument::new("faces", 2, 2);
        let grass = map.intern_tile_kind("grass");
        map.ground.set(0, 0, grass);
        map.elevation.set(0, 0, 3);
        // A stale elevation value cannot turn an empty cell into terrain.
        map.elevation.set(1, 0, 9);

        assert_eq!(
            visible_faces(&map, (0, 0), 3),
            vec![
                VisibleFace {
                    side: FaceSide::Right,
                    steps: 3,
                },
                VisibleFace {
                    side: FaceSide::Left,
                    steps: 3,
                },
            ]
        );
    }
}
