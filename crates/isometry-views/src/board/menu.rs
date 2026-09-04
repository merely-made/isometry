//! The token context menu, and the overlay that carries it.
//!
//! The rows come from the packs' emote vocabulary and only ever *ask*; the
//! catalog's `command_menu` draws the card and its `overlay_surface` supplies
//! the dismissal layer around it.
//!
//! Split out of `board.rs` on 2026-09-04; unchanged.

use super::*;

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
pub(super) fn context_menu_overlay(ui: &UiState) -> Option<UiChild> {
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
