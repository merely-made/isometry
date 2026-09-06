//! The focused text-control seam shared by the host and Isometry's key policy.

use cambium_genet_winit_host::FocusedTextSlot;
use genet_scripted_dom::NodeId;
use layout_dom_api::{LayoutDom as _, LocalName, Namespace};

use super::super::{Runner, UiState};

/// Isometry's editable text lanes.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Lane {
    /// The side panel's `>` command line.
    Command,
    /// The side panel's whisper composer.
    Whisper,
    /// The compendium index's filter.
    Search,
    /// The first-character display name.
    CharacterName,
    /// The optional first-character token owner.
    CharacterOwner,
}

/// Which lane holds the caret, and the `<input>` node that carries it.
///
/// The wrapper classes are the host/view contract. A rename in a view breaks
/// typing here rather than silently sending letters to board shortcuts.
pub(super) fn focused_lane(runner: &Runner) -> Option<(NodeId, Lane)> {
    let node = runner.focus()?;
    let dom = runner.dom();
    let dom = dom.borrow();
    if dom.element_name(node)?.local.as_ref() != "input" {
        return None;
    }
    let parent = dom.parent(node)?;
    let lane = match dom.attribute(parent, &Namespace::from(""), &LocalName::from("class"))? {
        "cmd-line" => Lane::Command,
        "compose-line" => Lane::Whisper,
        "search-field" => Lane::Search,
        "character-name" => Lane::CharacterName,
        "character-owner" => Lane::CharacterOwner,
        _ => return None,
    };
    Some((node, lane))
}

/// The text seam: which field has the caret, and how to reach its `TextInput`.
/// Every lane rides `caret_text_field`, so the shared host owns editing, IME,
/// selection and caret rendering.
pub(crate) fn focused_text(runner: &Runner) -> Option<FocusedTextSlot<UiState>> {
    let (node, lane) = focused_lane(runner)?;
    Some(match lane {
        Lane::Command => FocusedTextSlot {
            node,
            get: Box::new(|ui: &UiState| &ui.command_draft),
            get_mut: Box::new(|ui: &mut UiState| &mut ui.command_draft),
        },
        Lane::Whisper => FocusedTextSlot {
            node,
            get: Box::new(|ui: &UiState| &ui.whisper_draft),
            get_mut: Box::new(|ui: &mut UiState| &mut ui.whisper_draft),
        },
        Lane::Search => FocusedTextSlot {
            node,
            get: Box::new(|ui: &UiState| &ui.compendium_search),
            get_mut: Box::new(|ui: &mut UiState| &mut ui.compendium_search),
        },
        Lane::CharacterName => FocusedTextSlot {
            node,
            get: Box::new(|ui: &UiState| &ui.character_name),
            get_mut: Box::new(|ui: &mut UiState| &mut ui.character_name),
        },
        Lane::CharacterOwner => FocusedTextSlot {
            node,
            get: Box::new(|ui: &UiState| &ui.character_owner),
            get_mut: Box::new(|ui: &mut UiState| &mut ui.character_owner),
        },
    })
}
