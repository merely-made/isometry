// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mesocosm's view layer, adapter-first.
//!
//! The stack's projection posture, adopted from birth rather than migrated to:
//! this crate **discloses** world facts as a [`sceno::Score`], `scenomise`
//! solves placement, and the paint leaves here realize the solved scene. The
//! layout debt isometry carried (hand-rolled positions the proofs plan
//! schedules for deletion) is never incurred.
//!
//! What a scene element *means* stays on this side of the contract, per the
//! per-vessel ruling: sceno carries geometry and source references; this crate
//! decides that a region's tint is the lineage that dominates it.

//! The second lane, added 2026-08-29: the cambium chrome that says things.
//! [`vitals`] is Mesocosm's first consumer of it. Same posture as the paint
//! leaves — the view fn and its sheet live here, host-agnostic, and the host
//! decides only where the result lands.

pub mod body_menu;
pub mod caption;
pub mod dev;
pub mod leaf;
pub mod minimap;
pub mod review;
pub mod succession;
pub mod vitals;

pub use body_menu::{
    BODY_MENU_HEIGHT, BODY_MENU_WIDTH, BodyMenu, BodyMenuChild, BodyMenuRow, MAX_BODY_MENU_ROWS,
    body_menu_css, body_menu_root,
};
pub use caption::{Caption, CaptionChild, caption_css, caption_root};
pub use dev::{
    Dev, DevChild, Follow, Lost, PartInspection, PartReading, dev_css, dev_root, follow_of,
    lost_of, lost_words, part_of, role_word,
};
pub use leaf::MinimapLeaf;
pub use minimap::{
    MINIMAP_ADAPTER, dominant_lineages, lineage_tint, minimap_leaf, minimap_scene, minimap_score,
};
pub use review::{
    Board, BoardChild, BoardRow, board_css, board_root, commit_words, evidence_words,
    reading_words, row_words,
};
pub use succession::{Succession, SuccessionChild, succession_css, succession_root};
pub use vitals::{
    DiscoveryWords, Vitals, VitalsChild, discovery_words, notice_in, observation_words,
    refusal_words, vitals_css, vitals_of, vitals_root,
};
