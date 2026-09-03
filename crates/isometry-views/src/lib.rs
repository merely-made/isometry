//! Isometry's Cambium view layer.
//!
//! View functions project [`isometry_core`] state into DOM-shaped views:
//! every visible tile, prop, and token is an element positioned by the
//! iso math, appearance bound through CSS class vocabulary so tilesets
//! arrive as stylesheets. Host-agnostic: the desktop host and the later
//! web host both drive [`board_root`].

mod board;
mod command;
mod compendium;
mod demo;
mod downtime;
mod generator;
mod governance;
mod overmap;
mod panel;
mod projection;
mod sheet;
mod state;
mod storylet;
mod theme;
mod widgets;

pub use board::{board_root, UiChild};
pub use demo::{demo_map, synth_map, synth_world, SYNTH_PARTY};
pub use overmap::{
    overmap_positions, overmap_score, overmap_swatch, OvermapNodeKind, ISOMETRY_OVERMAP_ADAPTER,
    OVERMAP_CANVAS, OVERMAP_LEAF_KEY,
};
pub use projection::{
    tile_board_cells, tile_board_scene, tile_board_score, ISOMETRY_TILE_BOARD_ADAPTER,
    ISOMETRY_TILE_BOARD_BACKDROP,
};
pub use state::{mode_items, pace_items, stance_items, PACE_PCTS, STANCE_KEYS};
pub use state::{
    ActionRow, CompendiumTab, EditMode, FactionMoveRow, FogLevel, GenerationRequest,
    GeneratorSelectionRequest, GovernanceBindingRow, GovernanceConflict,
    GovernanceResolutionRequest, InitiativeMode, InventoryRequest, ItemRow, MonsterRow, NetMode,
    SheetSchema, SpellRow, StoryletRow, UiState, BOARD_UNIT, MESSAGES_CAP, PANEL_W,
    WHEEL_BOARD_TILES, WHEEL_NOTCH_PX,
};
pub use theme::board_css;
