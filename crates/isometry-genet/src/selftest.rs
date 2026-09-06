//! The env-gated self-tests: scripted sessions the host drives itself.
//!
//! Each one arms from an `ISOMETRY_*` env var, waits out a warm-up so the
//! window and the first frames exist, fires once, and prints what it saw. They
//! are how a headed run proves a lane end to end without a human driving the
//! mouse, and they stay out of `main.rs` because they are scaffolding for
//! verification rather than part of the host loop.
//!
//! Split out of `main.rs` on 2026-07-24, and grouped into the modules below
//! on 2026-09-04; behavior unchanged both times.

use cambium_genet_winit_host::{HostPointer, Key, KeyPress, NamedKey};
use layout_dom_api::{LayoutDom as _, LocalName, Namespace};

use super::*;

mod adjudicate;
mod session;
mod surfaces;
mod world;
mod watchtower;

// The 2026-09-04 split moved the lanes into the modules above, grouped by
// what each one drives; this file keeps the shared imports and the four
// key-and-caret helpers they read through `use super::*`. Two doc comments
// moved with the lane they name rather than the lane they had drifted above:
// the combat one was sitting on `maybe_travel_selftest`, the convince one on
// `maybe_storylet_selftest`.

/// Deliver one key press exactly as the host's key arm does: isometry's own
/// `key_intercept` first, and only then the focused element's own handlers.
///
/// This is the self-driven twin of the harness's `key_char`, and it is
/// deliberately not a shortcut around the text lanes: a letter that reaches a
/// focused `caret_text_field` is turned into a [`cambium::TextCommand`] by the
/// field itself, so these self-tests exercise the shipping path rather than a
/// second editor of their own. No synthetic OS input is involved — the same
/// reason the other self-tests drive the runner directly.
fn deliver_key(runner: &mut Runner, press: &KeyPress) {
    if hooks::key_intercept(runner, press) {
        return;
    }
    if let Some(event) = press.to_runner_key() {
        runner.dispatch_key(event);
    }
}

/// Type a run of text, one press per character, the way a person would.
///
/// Space goes as the named key winit reports for it, not as a character, so a
/// lane that ever stops accepting `NamedKey::Space` fails here rather than
/// passing on a shape no keyboard produces.
fn type_text(runner: &mut Runner, text: &str) {
    for c in text.chars() {
        let key = if c == ' ' {
            Key::Named(NamedKey::Space)
        } else {
            Key::Character(c.to_string())
        };
        deliver_key(runner, &KeyPress::new(key));
    }
}

/// The class of the wrapper around whichever `<input>` holds the caret.
///
/// The same name `hooks::focused_text` recognises a lane by, read straight off
/// the retained tree: a receipt that says where the letters went should first
/// say that they had somewhere to go.
fn caret_lane(runner: &Runner) -> Option<String> {
    let node = runner.focus()?;
    let dom = runner.dom();
    let dom = dom.borrow();
    let parent = dom.parent(node)?;
    dom.attribute(parent, &Namespace::from(""), &LocalName::from("class"))
        .map(str::to_owned)
}

/// The rendered compendium index: how many entry rows the grid is drawing, and
/// the first row's name.
///
/// Read off the DOM rather than recomputed from `bestiary`, because what the
/// filter is for is fewer rows on screen — the same receipt the harness test
/// `typing_in_the_compendium_filters_its_index` takes, in the headed run.
fn compendium_index(runner: &Runner) -> (usize, Option<String>) {
    let dom = runner.dom();
    let dom = dom.borrow();
    let rows = dom.all_with_class(dom.document(), "compendium-link");
    let first = rows.first().and_then(|&node| {
        dom.dom_children(node)
            .find_map(|child| dom.text(child).map(str::to_owned))
    });
    (rows.len(), first)
}
