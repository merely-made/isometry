// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a receipt sheet writes under a picture: a name, and one line saying
//! what the picture is of.
//!
//! **The smallest cambium surface in the crate, and it exists for a rule.**
//! No text rendering lives anywhere in this repo — words are Livery's and the
//! sentences are this crate's — so a receipt example that needs three words
//! over three captures cannot draw them itself. It asks for them here, and
//! rasterizes the result through the same chrome path every lane uses.
//!
//! Not a chrome lane: nothing in the running game draws a caption, and if
//! something ever should, that is a ruling and not a side effect of a
//! contact sheet. It is a receipt surface, the way the vitals panel is
//! borrowed by `p3_receipt` for the same reason.

use cambium::{AnyView, GenetCtx, GenetElement, el, text};

pub type CaptionChild = Box<dyn AnyView<Caption, (), GenetCtx, GenetElement>>;

/// One pane's title and its one-line reading.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Caption {
    /// What the pane is, in a word or two.
    pub label: String,
    /// What it is of, in a line. Empty draws nothing rather than a gap.
    pub note: String,
}

impl Caption {
    pub fn new(label: impl Into<String>, note: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            note: note.into(),
        }
    }
}

/// The caption: a name, and under it the line that qualifies it.
pub fn caption_root(state: &Caption) -> CaptionChild {
    let mut children: Vec<CaptionChild> = vec![Box::new(
        el::<_, Caption, ()>("div", text(state.label.clone())).attr("class", "caption-label"),
    )];
    if !state.note.is_empty() {
        children.push(Box::new(
            el::<_, Caption, ()>("div", text(state.note.clone())).attr("class", "caption-note"),
        ));
    }
    Box::new(el::<_, Caption, ()>("div", children).attr("class", "caption"))
}

/// The sheet. Deliberately plainer than any lane's: a caption sits under a
/// picture on a receipt, where the picture is the thing being looked at.
pub fn caption_css() -> &'static str {
    r#"
.caption {
    padding: 8px 14px;
    background-color: #11161a;
    color: #dfe6dd;
    font-family: sans-serif;
    font-size: 15px;
}
.caption-label {
    color: #e6d9a8;
    font-size: 19px;
    font-weight: bold;
    margin-bottom: 3px;
}
.caption-note {
    color: #8fa08c;
    font-size: 13px;
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An empty note draws no second line, rather than a blank one that
    /// reads as a missing sentence.
    #[test]
    fn a_caption_with_no_note_is_just_the_name() {
        assert_eq!(
            Caption::new("side", ""),
            Caption {
                label: "side".into(),
                note: String::new(),
            }
        );
        assert_eq!(Caption::new("side", "the control").note, "the control");
    }
}
