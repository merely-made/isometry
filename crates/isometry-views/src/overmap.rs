//! The overmap surface (C8, exploration mode): the party's pointcrawl, drawn.
//!
//! The board above the tactical maps. The host projects the world's places and
//! routes into an `Overmap` (`CampaignWorld::overmap`), filtered to what the
//! party has discovered (`overmap_for`); this draws that graph through Cambium's
//! `graph_canvas_swatch` -- painted nodes and edges on a retained Sprigging paint
//! leaf, with one native hit target per node -- and lets the table click a place
//! to travel there. The click only *asks*; the host rolls the navigation, spends
//! the time, and moves the party (`resolve_travel` -> `TravelResolved`), so the
//! view never decides a trip's outcome.
//!
//! Isometry adapts the campaign's pointcrawl to Scenograph's portable score and
//! realizes it with Scenomise. The final unit-box fit remains here because it is
//! a property of Cambium's swatch viewport, not of the score or scene. The leaf
//! key and the swatch model are shared with the host through [`overmap_swatch`],
//! so the painted leaf and these hit targets project through one identical
//! layout.

use std::collections::BTreeMap;

use cambium::{
    clickable, el, graph_canvas_swatch_with_drag_and_relations, lens, segmented_control, slider,
    text, GraphCanvasNode, GraphCanvasRelation, GraphCanvasSubgraph, GraphCanvasSwatch, Slider,
};
use isometry_core::{Overmap, OvermapEdge};
use sceno::{
    Arrangement, Footprint, Geographic, Placement, Representation, Score, ScoreItem, SourceRef,
    Spiral, Vec2,
};

use crate::board::UiChild;
use crate::state::UiState;

/// The Sprigging `LeafRegistry` key the host registers the overmap's painted
/// graph leaf under. Shared so the view's `custom_leaf` and the host's
/// `paint_leaf` name the same leaf.
pub const OVERMAP_LEAF_KEY: u64 = 8001;

/// The two node roles the palette colors: the party's current place, and the
/// rest of what it has discovered. The host resolves these to paint; the plain
/// vocabulary keeps rules and product-specific kinds out of the component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OvermapNodeKind {
    /// Where the party stands now.
    Here,
    /// A discovered place the party is not standing on.
    Elsewhere,
}

/// The canvas size, in logical pixels. Shared by the view (the `custom_leaf`
/// box) and the host (the leaf's intrinsic size), so the two never disagree.
pub const OVERMAP_CANVAS: (u32, u32) = (440, 300);

/// The adapter id persisted in a score's opaque source refs. It identifies a
/// boundary, not an Isometry type in the portable scene schema.
pub const ISOMETRY_OVERMAP_ADAPTER: &str = "isometry.overmap";

/// A local relation-cell id for the present overmap projection. `OvermapEdge`
/// does not yet carry the campaign route id, so the source order distinguishes
/// parallel routes. A future projection with campaign ids can replace this
/// adapter-local identity without changing Cambium's relation contract.
fn overmap_relation_id(index: usize, edge: &OvermapEdge) -> String {
    format!(
        "route:{index}:{}:{}:{}:{}",
        edge.from, edge.to, edge.weight, edge.directed
    )
}

mod overlay;
mod scene;
mod swatch;

// The 2026-09-04 split moved the bodies into the modules above; this file
// keeps the shared imports, the canvas constants, and the relation id all
// three build. Re-exported here so `overmap::` still names what it did.
// `overmap_positions_relaxed` is deliberately not among them: it is
// `overmap_positions`' own knob, and only `scene.rs` ever calls it.
pub use overlay::overmap_overlay;
pub use scene::{overmap_positions, overmap_score};
pub use swatch::overmap_swatch;
