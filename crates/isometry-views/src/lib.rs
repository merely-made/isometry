//! Isometry's Cambium view layer.
//!
//! View functions project [`isometry_core`] state into DOM-shaped views:
//! every visible tile, prop, and token is an element positioned by the
//! iso math, appearance bound through CSS class vocabulary so tilesets
//! arrive as stylesheets. Host-agnostic: the desktop host and the later
//! web host both drive [`board_root`].

mod board;
mod character;
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

pub use board::{UiChild, board_root};
pub use demo::{SYNTH_PARTY, demo_map, synth_map, synth_world};
pub use overmap::{
    AtlasBounds, AtlasProjection, AtlasRoute, AtlasSite, AtlasTerrainCell,
    ISOMETRY_OVERMAP_ADAPTER, OVERMAP_CANVAS, OVERMAP_LEAF_KEY, OvermapNodeKind, atlas_projection,
    overmap_atlas, overmap_positions, overmap_score, overmap_swatch,
};
pub use projection::{
    ISOMETRY_TILE_BOARD_ADAPTER, ISOMETRY_TILE_BOARD_BACKDROP, tile_board_cells, tile_board_scene,
    tile_board_score,
};
pub use state::{
    ActionRow, BOARD_UNIT, CharacterCreateRequest, CompendiumTab, EditMode, FactionMoveRow,
    FogLevel, GenerationRequest, GeneratorSelectionRequest, GovernanceBindingRow,
    GovernanceConflict, GovernanceResolutionRequest, InitiativeMode, InventoryRequest, ItemRow,
    MESSAGES_CAP, MonsterRow, NetMode, PANEL_W, SheetSchema, SpellRow, StoryletRow, UiState,
    WHEEL_BOARD_TILES, WHEEL_NOTCH_PX,
};
pub use state::{OvermapMotionState, OvermapMotionTick};
pub use state::{PACE_PCTS, STANCE_KEYS, mode_items, pace_items, stance_items};
pub use theme::board_css;
