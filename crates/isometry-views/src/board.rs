//! The board screen: side panel plus the iso board pane.
//!
//! Every ground tile, elevation-column filler, prop, and token is one
//! absolutely-positioned element inside the `.board` container; the
//! container's inline `left`/`top` carries the camera, so a pan is a
//! single attribute change on one element. Depth comes from
//! [`isometry_core::depth_key`] as a plain z-index.

use std::collections::HashSet;

use cambium::{
    clickable, command_menu, el, lens, map_action, on_hover, on_pointer, on_wheel, overlay_surface,
    AnyView, CommandEvent, CommandItem, CommandState, ElementView, GenetCtx, GenetElement,
    HoverEvent, HoverPhase, OverlayDismiss, OverlayRole, OverlaySurface, Placement, PointerButton,
    PointerEvent, PointerPhase, WheelEvent,
};
use isometry_core::{depth_key, path_to, MapDocument, TileCoord, TileKindId, Token};

use crate::panel::side_panel;
use crate::projection::tile_board_cells;
use crate::state::{EditMode, FogLevel, UiState, PANEL_W};

pub type UiChild = Box<dyn AnyView<UiState, (), GenetCtx, GenetElement>>;

/// The unscaled box the stylesheet gives each kind of board element, in CSS
/// pixels. These restate `theme.rs`, because a board element's size and its
/// projected position have to move together and only one of the two can live
/// in a sheet; `the_board_screen_lays_out_where_the_gestures_expect` is the
/// receipt that the pair still agrees.
const TILE_BOX: (f32, f32) = (32.0, 16.0);
const PROP_BOX: (f32, f32) = (20.0, 24.0);
const TOKEN_BOX: (f32, f32) = (24.0, 36.0);
const MARKER_BOX: (f32, f32) = (28.0, 14.0);

/// One board element's inline geometry: where it sits, how big it is, and how
/// deep it stands.
///
/// `size` is the sheet's own unscaled number for the box, and it is emitted
/// only when the board is not at scale 1 — where it would restate what the
/// stylesheet already says, character for character. Above or below 1 it
/// overrides the sheet, which is what keeps the diamonds meeting: the
/// projection in [`UiState::geo`] and the boxes drawn on it are one scale, or
/// the board tears along every tile edge.
fn placed(ui: &UiState, (x, y): (f32, f32), size: (f32, f32), z: i32) -> String {
    let mut style = format!("left: {x}px; top: {y}px; z-index: {z};");
    if ui.board_scale != 1.0 {
        let (w, h) = (size.0 * ui.board_scale, size.1 * ui.board_scale);
        style.push_str(&format!(" width: {w}px; height: {h}px;"));
    }
    style
}

/// Report the tile a board element stands on as the pointer enters and leaves
/// it, so the play-mode path preview and the measure template follow the
/// cursor.
///
/// The granularity comes from the tree because it cannot come from the host:
/// the shared host routes `on_hover` Enter and Leave as the *hit element*
/// changes and deliberately routes no Move, so a single handler on the pane
/// would only ever learn that the pointer is somewhere over the board. Every
/// element that stands on a tile carries this instead, and one crossing is one
/// Leave followed by one Enter. [`UiState::hover_tile_enter`] holds the gate
/// that decides whether the change is worth a rebuild.
fn standing_on<V>(child: V, at: TileCoord) -> UiChild
where
    V: ElementView<UiState, ()> + 'static,
{
    Box::new(on_hover(
        child,
        move |ui: &mut UiState, event: HoverEvent| match event.phase {
            HoverPhase::Enter => ui.hover_tile_enter(Some(at)),
            HoverPhase::Leave => ui.hover_tile_enter(None),
            HoverPhase::Move => {}
        },
    ))
}

/// One diamond at tile `at`, drawn at `elevation`, with `class` deciding
/// its paint. Clicking selects the tile.
fn tile_el(ui: &UiState, at: TileCoord, elevation: i32, class: String) -> UiChild {
    let geo = &ui.geo;
    let (cx, cy) = geo.tile_to_screen(at, elevation);
    let (x, y) = (cx - geo.tile_w / 2.0, cy - geo.tile_h / 2.0);
    let z = depth_key(at, elevation);
    standing_on(
        clickable(
            el("div", ())
                .attr("class", class)
                .attr("style", placed(ui, (x, y), TILE_BOX, z)),
            move |ui: &mut UiState, _| {
                ui.click_tile(at);
            },
        ),
        at,
    )
}

fn kind_name(map: &MapDocument, kind: TileKindId) -> &str {
    map.tile_kinds
        .get(kind.0 as usize)
        .map(String::as_str)
        .unwrap_or("empty")
}

/// Viewport culling margins (logical px). A tile is kept when its diamond,
/// elevation column, or standing sprite can still touch the pane, so the
/// margins are generous and asymmetric: a tile above the pane only pokes
/// down by half a diamond, while a tile below the pane can poke up by a
/// full elevation column plus a sprite. Over-emitting a thin ring is cheap;
/// clipping a visible tile is a bug.
const CULL_MARGIN_X: f32 = 32.0;
const CULL_MARGIN_ABOVE: f32 = 24.0;
const CULL_MARGIN_BELOW: f32 = 176.0;

/// Whether tile `at` can touch the board pane under the current camera, so
/// the whole-grid emitters only build elements the viewport can show. Until
/// the host reports a viewport (`(0, 0)`), this returns `true`, so an unset
/// viewport degrades to the pre-windowing "emit everything" behavior.
/// Battle-to-region scale is the design center; a pathologically tall tile
/// (elevation past ~18) below the pane edge is the one case the fixed bottom
/// margin does not cover, and that is outside the aesthetic.
fn in_view(ui: &UiState, at: TileCoord) -> bool {
    let (vw, vh) = ui.viewport;
    if vw <= 0.0 || vh <= 0.0 {
        return true;
    }
    let (bx, by) = ui.geo.tile_to_screen(at, 0);
    let px = bx + ui.camera.0;
    let py = by + ui.camera.1;
    px >= -CULL_MARGIN_X
        && px <= vw + CULL_MARGIN_X
        && py >= -CULL_MARGIN_ABOVE
        && py <= vh + CULL_MARGIN_BELOW
}

/// The ground layer: a column of filler diamonds up to the tile's
/// elevation, then the top diamond carrying the kind class. In Play
/// mode the selected token's reach tints blue and the hovered path
/// tints lighter still.
fn ground_tiles(ui: &UiState) -> Vec<UiChild> {
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
fn prop_tiles(ui: &UiState) -> Vec<UiChild> {
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

fn token_el(ui: &UiState, token: &Token) -> UiChild {
    let geo = &ui.geo;
    let elev = *ui
        .map
        .elevation
        .get(token.at.0.max(0) as u32, token.at.1.max(0) as u32)
        .unwrap_or(&0) as i32;
    let (cx, cy) = geo.tile_to_screen(token.at, elev);
    let z = depth_key(token.at, elev) + 2;
    // 8x12 sprite at 3x (24x36), feet at the diamond center. The anchor takes
    // the board's scale with the box: the feet must stay on the diamond.
    let s = ui.board_scale;
    let (x, y) = (cx - TOKEN_BOX.0 / 2.0 * s, cy - 32.0 * s);
    let mut class = format!("token token-{}", token.sprite);
    // Equipment appearance remains pack CSS: public layer keys become stable
    // token classes, while the future voxel compositor can replace the same
    // projection with a baked recipe without changing campaign data.
    if let Some(inventory) = ui.inventories.get(&token.id) {
        for item_id in inventory.equipped.values() {
            if let Some(item) = inventory.items.get(item_id) {
                for layer in item.appearance_layers() {
                    class.push_str(" token-layer-");
                    class.push_str(&layer_class(layer));
                }
            }
        }
    }
    // One drawn side, mirrored for the other two facings (the GBA
    // economy): E/N flip, S/W stay.
    let flipped = matches!(
        token.facing,
        isometry_core::Facing::East | isometry_core::Facing::North
    );
    if flipped {
        class.push_str(" token-flip");
    }
    let id = token.id;

    // A1: the beat rides on a wrapper, not on the sprite. `.token-flip` already
    // owns the sprite's `transform` for facing, and a CSS animation on
    // `transform` outranks a normal declaration, so a beat on the same box would
    // strip a west/south-facing token of its mirror mid-swing. Two boxes: the
    // wrapper is placed and beats, the sprite inside is drawn and flipped. They
    // compose instead of fighting.
    let mut wrapper = "beat".to_owned();
    if let Some(beat) = ui.beats.get(&id) {
        wrapper.push_str(" beat-");
        wrapper.push_str(beat);
    }
    let down = ui.map.is_defeated(id);
    if down {
        wrapper.push_str(" beat-down");
    }
    // Conditions render as classes, like beats and equipment layers, so a pack
    // can style `cond-prone` the way it styles a swing. A graded condition also
    // emits `cond-frightened-2`, so a pack can show magnitude without the board
    // knowing what the number means.
    if let Some(conditions) = ui.map.conditions.get(&id) {
        for (name, value) in conditions {
            wrapper.push_str(" cond-");
            wrapper.push_str(name);
            if *value > 1 {
                wrapper.push_str(&format!(" cond-{name}-{value}"));
            }
        }
    }
    // A corpse is not a target, so it does not offer itself as one.
    if ui.picking_target() && !down {
        wrapper.push_str(" beat-targetable");
    }
    // The sprite is the one board box whose size is not the whole story: it
    // also carries `.token-flip`'s mirror, and that transform pre-translates by
    // the sprite's own width (genet conjugates at the box origin). A scaled
    // sprite therefore needs a scaled translate, so both ride the same inline
    // declaration rather than the class. At scale 1 nothing is emitted and the
    // sheet's rule stands untouched.
    let sprite_el = el("div", ()).attr("class", class);
    let sprite: Vec<UiChild> = vec![if s == 1.0 {
        Box::new(sprite_el)
    } else {
        let mut style = format!("width: {}px; height: {}px;", TOKEN_BOX.0 * s, TOKEN_BOX.1 * s);
        if flipped {
            style.push_str(&format!(
                " transform: translateX({}px) scaleX(-1);",
                TOKEN_BOX.0 * s
            ));
        }
        Box::new(sprite_el.attr("style", style))
    }];
    standing_on(
        clickable(
            el("div", sprite)
                .attr("class", wrapper)
                .attr("style", placed(ui, (x, y), TOKEN_BOX, z)),
            move |ui: &mut UiState, _| {
                // In target-pick mode a click on a token names the victim rather
                // than selecting it.
                if ui.picking_target() {
                    ui.pick_action_target(id);
                } else {
                    ui.click_token(id);
                }
            },
        ),
        token.at,
    )
}

/// Make a pack layer key safe for the CSS-class vocabulary. Different raw
/// punctuation collapses intentionally: appearance keys are authored ids, and
/// pack validation can reject collisions once packs gain a formal manifest.
fn layer_class(key: &str) -> String {
    key.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod layer_class_tests {
    use super::layer_class;

    #[test]
    fn equipment_layer_keys_map_to_stable_css_classes() {
        assert_eq!(layer_class("effect:Flame"), "effect-flame");
        assert_eq!(layer_class("weapon/longsword +1"), "weapon-longsword--1");
    }
}

#[cfg(test)]
mod token_menu_tests {
    use super::*;
    use isometry_core::{MapDocument, TokenId};

    fn ui_with_token() -> UiState {
        let mut map = MapDocument::new("t", 4, 4);
        map.intern_tile_kind("grass");
        map.tokens.push(Token {
            id: TokenId(1),
            at: (1, 1),
            facing: isometry_core::Facing::South,
            sprite: "knight".to_owned(),
            owner: None,
        });
        UiState::new(map)
    }

    /// The activation path is an index into the rows, so the two lists must stay
    /// the same length and the same order. If a row is ever added to one and not
    /// the other, every action past it fires the wrong thing -- silently, since
    /// an index is always "valid".
    #[test]
    fn every_row_has_exactly_one_action() {
        let mut ui = ui_with_token();
        ui.emotes = vec![("cheer".to_owned(), "Cheer".to_owned())];
        ui.map
            .conditions
            .entry(TokenId(1))
            .or_default()
            .insert("frightened".to_owned(), 2);

        let (items, actions) = token_menu(&ui, TokenId(1));
        assert_eq!(items.len(), actions.len());
        // Sheet, End turn, one emote, one condition, Remove, Close.
        assert_eq!(items.len(), 6);
        assert!(
            items.iter().any(|i| i.label == "Clear: frightened 2"),
            "a graded condition names its magnitude: {:?}",
            items.iter().map(|i| &i.label).collect::<Vec<_>>()
        );
    }

    /// The capability the hand-rolled menu could not express: a row that is
    /// present, visibly unavailable, and says why. It used to be equally
    /// clickable and equally silent.
    #[test]
    fn end_turn_says_why_it_is_unavailable() {
        let mut ui = ui_with_token();

        let (items, _) = token_menu(&ui, TokenId(1));
        let end = &items[1];
        assert!(end.disabled, "no turn order: nothing to end");
        assert_eq!(end.disabled_reason.as_deref(), Some("no turn order"));

        // Somebody else's turn: still disabled, with the reason that fits.
        ui.turns.add(TokenId(2));
        let (items, _) = token_menu(&ui, TokenId(1));
        assert_eq!(
            items[1].disabled_reason.as_deref(),
            Some("it is not this token's turn")
        );

        // Its own turn: live.
        ui.turns = isometry_core::TurnList::new();
        ui.turns.add(TokenId(1));
        let (items, _) = token_menu(&ui, TokenId(1));
        assert!(!items[1].disabled, "a token may end its own turn");
    }
}

/// A ground marker diamond under a token (turn-active gold, selection
/// green), one depth step above the tile it stands on.
fn marker_el(ui: &UiState, token_id: isometry_core::TokenId, class: &str) -> Option<UiChild> {
    let token = ui.map.token(token_id)?;
    let elev = *ui
        .map
        .elevation
        .get(token.at.0.max(0) as u32, token.at.1.max(0) as u32)
        .unwrap_or(&0) as i32;
    let (cx, cy) = ui.geo.tile_to_screen(token.at, elev);
    let z = depth_key(token.at, elev) + 1;
    let s = ui.board_scale;
    let (x, y) = (cx - MARKER_BOX.0 / 2.0 * s, cy - MARKER_BOX.1 / 2.0 * s);
    Some(standing_on(
        el("div", ())
            .attr("class", class.to_owned())
            .attr("style", placed(ui, (x, y), MARKER_BOX, z)),
        token.at,
    ))
}

/// What a context-menu row does when it is activated.
///
/// Built in lockstep with the [`CommandItem`]s in [`token_menu`], because the
/// catalog surface reports an activation as an *index* into the row list. Two
/// separate literals could drift and silently fire the wrong action; one builder
/// emitting both cannot. (The same reason `PACE_PCTS` indexes the pace row.)
enum TokenMenuAction {
    Sheet,
    EndTurn,
    Emote(String),
    ClearCondition(String),
    Remove,
    Close,
}

/// The rows of a token's context menu, paired with what each one does.
///
/// The emote vocabulary is the *packs'*: a pack declares which beats are
/// emotable (a `name` plus an `emote` label in its manifest), so a campaign can
/// add a rude gesture or remove one without the app knowing. The condition rows
/// only *ask*; the host recomputes what the token can do.
fn token_menu(
    ui: &UiState,
    id: isometry_core::TokenId,
) -> (Vec<CommandItem>, Vec<TokenMenuAction>) {
    let mut items = Vec::new();
    let mut actions = Vec::new();

    items.push(CommandItem::new("Sheet").with_id("sheet"));
    actions.push(TokenMenuAction::Sheet);

    // Ending a turn that is not this token's does nothing, so the row says why
    // instead of failing quietly. The hand-rolled menu had no way to express
    // this: every row was equally clickable and equally silent.
    let end_turn = CommandItem::new("End turn").with_id("end-turn");
    items.push(match ui.turns.active() {
        Some(active) if active == id => end_turn,
        Some(_) => end_turn.disabled_because("it is not this token's turn"),
        None => end_turn.disabled_because("no turn order"),
    });
    actions.push(TokenMenuAction::EndTurn);

    for (beat, label) in &ui.emotes {
        items.push(CommandItem::new(label.clone()).with_id(format!("emote-{beat}")));
        actions.push(TokenMenuAction::Emote(beat.clone()));
    }

    if let Some(set) = ui.map.conditions.get(&id) {
        for (name, value) in set {
            // Show the magnitude for a graded condition, so "Clear: frightened
            // 2" reads true; a plain on/off stays bare.
            let label = if *value > 1 {
                format!("Clear: {name} {value}")
            } else {
                format!("Clear: {name}")
            };
            items.push(CommandItem::new(label).with_id(format!("clear-{name}")));
            actions.push(TokenMenuAction::ClearCondition(name.clone()));
        }
    }

    items.push(CommandItem::new("Remove").with_id("remove"));
    actions.push(TokenMenuAction::Remove);
    items.push(CommandItem::new("Close").with_id("close"));
    actions.push(TokenMenuAction::Close);

    (items, actions)
}

/// The right-click context menu, or `None` when closed.
///
/// The catalog's `command_menu` (adopted 2026-07-25) draws the card: it brings
/// `role="menu"` / `menuitem`, `aria-activedescendant`, disabled-with-reason
/// rows, and arrow-key navigation, none of which the hand-roll had. It sits
/// inside the catalog's `overlay_surface` (adopted 2026-09-03), which supplies
/// the half the component cannot: a transparent dismissal layer over the whole
/// window, so a press *anywhere* outside the card closes the menu.
///
/// That layer is why this hangs off `.app` rather than `.pane`, and why the
/// anchor gains [`PANEL_W`]: `.pane` is `overflow: hidden`, so a layer sized to
/// the window inside it would be clipped back to the pane and a press on the
/// side panel would go on missing the menu — which is exactly the regression
/// the migration left behind when the host's own left-click-off branch went
/// away with `input.rs`. Pane-local stays the space
/// [`UiState::open_context_menu`] records; the translation happens here, once.
///
/// The surface is anchored at the window origin with a zero-size panel, so the
/// card lands at exactly the pane point the press recorded: `command_menu`
/// positions itself absolutely inside the panel box, the same as before. The
/// surface's own Escape listener is passive (it needs a focused descendant),
/// so the Escape half is `hooks::key_intercept`'s.
fn context_menu_overlay(ui: &UiState) -> Option<UiChild> {
    let (id, (mx, my)) = ui.context_menu?;
    ui.map.token(id)?;
    let (items, actions) = token_menu(ui, id);
    // Window coordinates: the pane starts at the panel's edge.
    let (mx, my) = (mx + PANEL_W, my);
    let (pane_w, pane_h) = ui.viewport;
    let surface = OverlaySurface::new(
        (0.0, 0.0, 0.0, 0.0),
        (0.0, 0.0),
        (0.0, 0.0, pane_w + PANEL_W, pane_h),
    )
    .with_placement(Placement::Below)
    .with_role(OverlayRole::Region)
    .with_label("Token actions");
    let menu = map_action(
        lens(
            move |cmd: &mut CommandState| command_menu(cmd, &items, mx, my),
            |ui: &mut UiState| &mut ui.context_menu_state,
        ),
        move |ui: &mut UiState, event: CommandEvent| {
            if let CommandEvent::Activate(path) = event {
                if let Some(action) = path.first().and_then(|i| actions.get(*i)) {
                    match action {
                        TokenMenuAction::Sheet => ui.open_or_bind_sheet(),
                        TokenMenuAction::EndTurn => ui.end_turn(),
                        TokenMenuAction::Emote(beat) => ui.emote(id, beat),
                        TokenMenuAction::ClearCondition(name) => {
                            ui.clear_condition_request = Some((id, name.clone()));
                        }
                        TokenMenuAction::Remove => ui.remove_token(id),
                        TokenMenuAction::Close => {}
                    }
                }
            }
            // Every row closes the menu, and so does a dismissal. `remove_token`
            // already closes it; doing so twice is harmless.
            ui.close_context_menu();
        },
    );
    Some(Box::new(overlay_surface(
        &surface,
        menu,
        |ui: &mut UiState, _reason: OverlayDismiss| ui.close_context_menu(),
    )))
}

/// The screen root the runner diffs.
pub fn board_root(ui: &UiState) -> UiChild {
    let mut layers: Vec<UiChild> = ground_tiles(ui);
    layers.extend(prop_tiles(ui));
    // Markers and tokens follow fog: a marker only shows on a token the
    // viewer can currently see.
    let marker_shown = |id: isometry_core::TokenId| {
        ui.map
            .token(id)
            .map(|t| ui.token_visible(t))
            .unwrap_or(false)
    };
    if let Some(id) = ui.selected_token {
        if marker_shown(id) {
            layers.extend(marker_el(ui, id, "marker marker-select"));
        }
    }
    if let Some(active) = ui.turns.active() {
        if marker_shown(active) {
            layers.extend(marker_el(ui, active, "marker marker-turn"));
        }
    }
    layers.extend(
        ui.map
            .tokens
            .iter()
            .filter(|t| ui.token_visible(t))
            .map(|t| token_el(ui, t)),
    );
    // Windowing metric: with `ISOMETRY_PROFILE` on, report how many
    // elements the viewport emits. It should stay bounded by the pane, not
    // grow with the board (see the windowing plan).
    if std::env::var_os("ISOMETRY_PROFILE").is_some() {
        eprintln!("[isometry] board elements emitted: {}", layers.len());
    }
    let (camx, camy) = ui.camera;
    let mut pane_children: Vec<UiChild> = vec![Box::new(
        el("div", layers)
            .attr("class", "board")
            .attr("style", format!("left: {camx}px; top: {camy}px;")),
    )];
    if let Some(overlay) = crate::sheet::sheet_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::compendium::compendium_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::generator::generator_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::storylet::storylet_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::downtime::downtime_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::overmap::overmap_overlay(ui) {
        pane_children.push(overlay);
    }
    if let Some(overlay) = crate::governance::governance_overlay(ui) {
        pane_children.push(overlay);
    }
    // The token menu is the one overlay that is *not* a pane child: its
    // dismissal layer has to cover the side panel too, and `.pane` clips.
    let mut screen: Vec<UiChild> = vec![side_panel(ui), board_pane(pane_children)];
    if let Some(menu) = context_menu_overlay(ui) {
        screen.push(menu);
    }
    Box::new(el("div", screen).attr("class", "app"))
}

/// The board pane, carrying the board's own pointer and wheel gestures.
///
/// Both handlers hang here rather than on the window, which is what keeps the
/// side panel out of them: a drag can never spam the panel's buttons and a
/// wheel notch over the panel never pans the board, by construction rather
/// than by a coordinate test. `local` is measured against this element's
/// painted box, so a handler receives the pointer in the pane's own
/// coordinates — the space [`UiState::open_context_menu`] already anchors in,
/// and the space every gesture on [`UiState`] takes.
fn board_pane(children: Vec<UiChild>) -> UiChild {
    let pane = el("div", children).attr("class", "pane");
    let pane = on_wheel(pane, |ui: &mut UiState, event: WheelEvent| {
        // The wheel pans, so the host's scrolling default must not also run:
        // `.pane` is `overflow: hidden`, and an unconsumed notch would chain
        // out to the document viewport and drag the whole board with it.
        event.prevent_default();
        ui.board_wheel(event.delta.0, event.delta.1);
    });
    Box::new(on_pointer(pane, |ui: &mut UiState, event: PointerEvent| {
        match (event.phase, event.button) {
            // A secondary press is one-shot: it captures nothing and no Move
            // or Up follows it, so opening the menu is the whole gesture.
            (PointerPhase::Down, PointerButton::Secondary) => ui.board_context_menu(event.local),
            (PointerPhase::Down, PointerButton::Primary) => {
                if std::env::var_os("ISOMETRY_PROFILE").is_some() {
                    eprintln!(
                        "[isometry] board press at {:?} tile {:?}",
                        event.local,
                        ui.tile_at_pane(event.local)
                    );
                }
                ui.board_press(event.local);
            }
            (PointerPhase::Move, _) => ui.board_drag(event.local),
            (PointerPhase::Up, _) => ui.board_release(event.local),
        }
    }))
}
