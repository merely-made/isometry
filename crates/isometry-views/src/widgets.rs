//! Shared chrome widgets, extracted on second use. The stat shapes the
//! compendium's monster page and the character sheet both render live here so
//! there is one implementation. View compositions, host-agnostic; promote to
//! the cross-repo catalog only when another repo needs them.

use cambium::{el, on_pointer, text, PointerEvent};

use crate::board::UiChild;
use crate::state::UiState;

// The namespace nav used to be a hand-rolled `tab_strip` here. It is Cambium's
// `tab_strip` now (adopted 2026-07-25), which adds arrow-key switching and the
// ARIA tabs roles this one never had.

/// A floating titled panel over the board: a header (title plus action
/// buttons) above a body, positioned by `panel_class`. The compendium and the
/// character sheet are both instances.
///
/// The panel root carries an `on_pointer` that does nothing, and that is the
/// point: the host routes a pointer down to the *nearest* `on_pointer`
/// ancestor of the hit element and captures there, so a press inside a panel
/// stops at this handler instead of reaching `.pane` — a paint drag begun over
/// an open sheet no longer paints the tiles beneath it, and the drag stays
/// here for its whole life. It deliberately does not `prevent_default`: the
/// host's click-to-caret still has to reach the compendium's filter field.
/// Anything nested deeper with its own `on_pointer` (the overmap's canvas
/// drag) is nearer and still wins.
pub fn overlay_panel(
    panel_class: &'static str,
    title: String,
    actions: Vec<UiChild>,
    body: Vec<UiChild>,
) -> UiChild {
    let header = el::<_, UiState, ()>(
        "div",
        (
            el::<_, UiState, ()>("span", text(title)).attr("class", "overlay-title"),
            el::<_, UiState, ()>("div", actions).attr("class", "overlay-actions"),
        ),
    )
    .attr("class", "overlay-header");
    let mut kids: Vec<UiChild> = vec![Box::new(header)];
    kids.extend(body);
    Box::new(on_pointer(
        el::<_, UiState, ()>("div", kids).attr("class", panel_class),
        |_ui: &mut UiState, _event: PointerEvent| {},
    ))
}

/// A titled record: an entry name, an optional subtitle, then sections. The
/// compendium's monster/spell/item pages are the consumers.
pub fn record_card(name: &str, subtitle: &str, sections: Vec<UiChild>) -> UiChild {
    let mut kids: Vec<UiChild> = vec![Box::new(
        el::<_, UiState, ()>("div", text(name.to_owned())).attr("class", "entry-name"),
    )];
    if !subtitle.is_empty() {
        kids.push(Box::new(
            el::<_, UiState, ()>("div", text(subtitle.to_owned())).attr("class", "entry-sub"),
        ));
    }
    kids.extend(sections);
    Box::new(el::<_, UiState, ()>("div", kids))
}

// The compendium's filter used to be a display-only `search_field` here, with
// the host rebuilding the query one key at a time. It is a `caret_text_field`
// over a `TextInput` in `compendium.rs` now (2026-09-03), so the widget is
// gone rather than shimmed.

/// One read-only labeled value: a muted label beside an emphasised value
/// ("AC 13", "Reflex +2").
pub fn stat_row(label: &str, value: impl Into<String>) -> UiChild {
    Box::new(
        el::<_, UiState, ()>(
            "div",
            (
                el("span", text(label.to_owned())).attr("class", "stat-label"),
                el("span", text(value.into())).attr("class", "stat-val"),
            ),
        )
        .attr("class", "stat-row"),
    )
}

/// A container of labeled values under `container_class`. Shared by the monster
/// page (AC/HP/Speed/CR) and the sheet's derived modifiers.
pub fn stat_list(
    pairs: impl IntoIterator<Item = (String, String)>,
    container_class: &'static str,
) -> UiChild {
    let rows: Vec<UiChild> = pairs.into_iter().map(|(l, v)| stat_row(&l, v)).collect();
    Box::new(el::<_, UiState, ()>("div", rows).attr("class", container_class))
}
