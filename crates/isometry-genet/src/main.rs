//! Isometry's desktop application.
//!
//! There is no native host in this crate any more. `cambium-genet-winit-host`
//! owns the winit lifecycle, the genet surface, the retained layout, the paint
//! pass, hit testing, pointer/keyboard/IME/wheel routing, frame capture, and
//! the AccessKit lifecycle — all of it extracted from woodshed's donor
//! assembly, and all of it previously hand-assembled here in `host.rs`,
//! `render.rs` and `input.rs`. Those three are gone; nothing shims them.
//!
//! What is left is isometry: which views to render, which state they run over,
//! the campaign store, the session bridge, the self-tests, and the overmap's
//! painted leaf. It reaches the host through seven plain closures
//! ([`HostHooks`]) with its own state in their captured environment.
//!
//! | hook | isometry's half |
//! |------|-----------------|
//! | `frame` | pane viewport, overmap leaf, beat hold, capture arming ([`hooks`]) |
//! | `after_dispatch` | save/load, the pumps, source-time attach, the net outbox ([`dispatch`]) |
//! | `after_frame` | the `ISOMETRY_*_SELFTEST` drivers and the profile line ([`selftest`]) |
//! | `after_wake` | drain the session bridge the armillary actor woke us for ([`net`]) |
//! | `close_request` | exit: nothing here outlives the window |
//! | `focused_text` | whichever of the three text lanes holds the caret ([`hooks`]) |
//! | `key_intercept` | Escape policy, the text lanes, the single-letter verbs |
//!
//! Board gestures — drag painting, token drag, the path-preview hover, wheel
//! pan and the token context menu — are not here either. They are `on_pointer`,
//! `on_hover` and `on_wheel` handlers on the board pane in `isometry-views`,
//! which is where the tile-under-a-pointer projection already lived.
//!
//! Sessions (I4): `--host` binds an iroh session and prints a join ticket;
//! `--join <ticket>` dials it. `--campaign <name>` restores that campaign's
//! durable checkpoint before a host accepts peers. In a session the view is
//! Remote — play routes through the host authority (`net` bridges the async
//! session to this sync loop). Env hooks: `ISOMETRY_PROFILE=1` (frame timers +
//! net trace), `ISOMETRY_CAPTURE_DIR` (self-capture), `ISOMETRY_SYNTH=1`
//! (stress board), `ISOMETRY_NET_SELFTEST=1` (fire one end-turn after warm-up
//! to verify the session round-trip without OS input automation),
//! `ISOMETRY_OVERMAP_SELFTEST=1` (overmap capture),
//! `ISOMETRY_OVERMAP_SOURCE_TIME_SELFTEST=1` (historical-overmap capture),
//! `ISOMETRY_COMPENDIUM_SELFTEST=1` and `ISOMETRY_WHISPER_SELFTEST=1` (the two
//! M3 text lanes, typed through the field and held open for a capture).

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use isometry_campaign::{
    CampaignStore, CampaignWorld, EntropyTape, FactionMove, GenValue, GeneratorRequest, ItemId,
    ItemInstance, MapScale, WorldEvent, WorldFact,
};
use isometry_core::{
    Facing, FieldValue, MapDocument, Rng, SessionEvent, SheetData, TileCoord, Token, TokenId,
};
use isometry_net::{
    apply_game, ActionIntent, ActionResolved, GameEvent, GameSnapshot, HostSession, RequestId,
};
use isometry_system::{
    monster_sheet, sheet_with_conditions, sheet_with_turn_counters, srd_5e, srd_bestiary,
    srd_items, srd_spells, ActionError, GeneratorCatalog, GeneratorLimits, System,
};
use isometry_views::{
    board_css, board_root, demo_map, synth_map, ActionRow, EditMode, FactionMoveRow,
    GenerationRequest, InventoryRequest, ItemRow, MonsterRow, NetMode, SheetSchema, SpellRow,
    StoryletRow, UiChild, UiState, PANEL_W,
};
use muniment::Journal;

mod adjudicate;
mod campaign_store;
mod catalog;
mod cleromancy_selection;
mod dispatch;
mod generators;
mod hooks;
// Host-routing receipts, driven through the shared host with no window.
#[cfg(test)]
mod host_routing;
// The design fit and the board's pixel grid, on the same harness.
#[cfg(test)]
mod host_zoom;
mod net;
mod overmap;
mod selection_rows;
mod selftest;
mod sheets;
mod source_time;
mod storylets;

use campaign_store::{CampaignCheckpoint, CampaignRepository};
use catalog::{bestiary_of, items_of, schema_of, spells_of};
use net::{NetBridge, Role};
use sprigging::ColorF;

/// The view logic the runner diffs: one screen root over one state.
pub(crate) type Logic = fn(&UiState) -> UiChild;
/// The runner the shared host drives.
pub(crate) type Runner = cambium_genet_winit_host::Runner<UiState, Logic, UiChild>;
/// What a hook is handed: the runner, plus the host's per-frame handles.
pub(crate) type Ctx<'a> = cambium_genet_winit_host::AppCtx<'a, UiState, Logic, UiChild>;

/// How long a staged beat is held on the board before its classes are dropped.
///
/// The old host asked its layout engine (`has_active_animations`) and cleared
/// on the frame the last `@keyframes` expired. The shared host's retained
/// layout reports no animation state to an application hook, so the hold is a
/// wall clock instead, set past the longest beat the bundled packs declare
/// (620ms, `cheer`). Its job is unchanged: drop the classes so the *next*
/// strike is a genuine attribute change and re-triggers, rather than restyling
/// nothing and standing still.
pub(crate) const BEAT_HOLD: Duration = Duration::from_millis(750);

/// The logical size the interface is drawn for, and the design the host fits
/// the window to.
///
/// The side panel is a fixed column of sections ending in the whisper composer,
/// the status line and the key hint, and it needs 820 logical pixels of height
/// to show all of them. The host clamps the window it opens to the display's
/// height less 48, so this laptop (1280x800 logical) gives 752 and the composer
/// used to fall off the bottom. Naming it as `fit_design` scales the whole
/// interface to whatever the display offers instead of cutting it off; on a
/// display that has the room the factor is 1 and nothing moves.
pub(crate) const DESIGN_SIZE: (f32, f32) = (1_100.0, 820.0);

/// The overmap palette: the party's place reads warm-green, the rest of the
/// discovered map cool-gray. The host owns the palette so no product-specific
/// node kind leaks into Cambium.
fn overmap_node_color(kind: &isometry_views::OvermapNodeKind) -> ColorF {
    match kind {
        isometry_views::OvermapNodeKind::Here => ColorF {
            r: 0.62,
            g: 0.83,
            b: 0.54,
            a: 1.0,
        },
        isometry_views::OvermapNodeKind::Elsewhere => ColorF {
            r: 0.72,
            g: 0.75,
            b: 0.82,
            a: 1.0,
        },
    }
}

/// Isometry's own state: everything the runner does not hold.
///
/// The host owns the window, the surface, the retained layout, the leaf
/// registry and the `UiState` behind the runner. What is left here is the
/// application's: the private campaign store, the authority's history, the
/// session bridge, the rules system, the generator tape, and the self-test
/// scaffolding. It lives in an `Rc<RefCell<..>>` captured by the hooks, which
/// is how a plain-closure host lets an application keep state.
struct App {
    /// GM-only state saved beside the public map through Muniment. It never
    /// enters the view, map JSON, or replicated snapshot.
    campaign: CampaignStore,
    /// Public campaign facts. The view does not render the journal yet, but
    /// the checkpoint must retain it for replay and host handoff.
    journal: Vec<WorldFact>,
    /// The host's append-only Journal history. It is empty for local editing
    /// until a session begins, then mirrors the authority actor.
    history: Journal<GameEvent>,
    /// Public state immediately before the first entry in `history`. It is the
    /// required replay origin for a truthful source-time projection.
    history_origin: Option<GameSnapshot>,
    /// The log length last attached to the view's source-time adapter. Keeps
    /// ordinary dispatches from cloning/reinstalling an unchanged history.
    source_history_len: Option<usize>,
    /// Whether the current source-time availability has been projected into the
    /// view. This distinguishes an unavailable source from a stale projection
    /// that still needs clearing after a checkpoint load.
    source_history_attached: bool,
    /// The overmap swatch the registered leaf was last built from. Compared per
    /// frame while the surface is open so the leaf is re-registered (and thus
    /// repainted) only when the model actually changed; also the "any leaf is
    /// live" gate for the per-frame leaf-box walk.
    last_overmap_swatch:
        Option<cambium::GraphCanvasSwatch<String, isometry_views::OvermapNodeKind>>,
    /// The exact inputs the storylet rows were last built from: the world and
    /// the host's secret-fact ids. Compared per dispatch while the surface is
    /// open, so the rows re-resolve only when an answer could have changed. Kept
    /// across a close so reopening is free; the compare catches anything that
    /// moved while it was shut.
    last_storylet_inputs: Option<(CampaignWorld, Vec<String>)>,
    /// Host entropy for adjudication. Every die an action rolls comes from here,
    /// so a fixed seed replays a session's combat exactly; peers never roll.
    action_rng: Rng,
    /// Nonces for this process's *own* asks: the DM's swings and solo play,
    /// which arrive over no connection and so carry [`RequestId::host`]. A
    /// joined player's ask is numbered by its own `ClientSession` and
    /// attributed by the host, so it never passes through here.
    own_requests: u64,
    /// When the beat currently on screen should be dropped. `None` when the
    /// board is still. See [`BEAT_HOLD`].
    beat_until: Option<Instant>,
    /// The stylesheet, from app CSS plus whatever the content packs declared.
    /// Handed to the host once, by `init`.
    sheet: String,
    /// The pane size last pushed into the view, so an unchanged frame does not
    /// rebuild the retained tree to write the same two floats.
    last_viewport: (f32, f32),
    profile: bool,
    /// `ISOMETRY_CAPTURE_DIR`: overwrite `<dir>/isometry_capture.png`
    /// with every presented frame, read back from the app's own texture.
    /// Screen grabs lose to overlapping windows; this cannot.
    capture_dir: Option<std::path::PathBuf>,
    /// What session, if any, this process runs (from `--host`/`--join`),
    /// consumed once by `init`.
    net_intent: Option<NetIntent>,
    /// True when this process is the authority. Only the host adjudicates, so a
    /// client must *ask* rather than resolve: otherwise two machines would each
    /// roll their own dice for the same swing.
    net_is_host: bool,
    /// `--as <player>`: the fog viewer this process plays as. `None` is
    /// omniscient (the DM / a spectator).
    viewer_arg: Option<String>,
    /// `--campaign <name>`: restore this named campaign checkpoint at boot.
    campaign_arg: Option<String>,
    /// The live session bridge in networked mode.
    net: Option<NetBridge>,
    /// Last session version pulled into the UI; a bump means redraw.
    last_net_version: u64,
    /// `ISOMETRY_NET_SELFTEST=1`: fire one end-turn from inside the app a
    /// few seconds after a session starts, so the UI→net→republish→UI
    /// round-trip is verifiable without OS input automation (Windows
    /// foreground-lock makes driving one of two windows unreliable).
    net_selftest: bool,
    /// Emotes the loaded packs offer, handed to the view at boot. The app owns
    /// no beat vocabulary of its own.
    pack_emotes: Vec<(String, String)>,
    /// Tokens whose standing-on-a-door state has already produced a travel
    /// event, so the sweep emits once per crossing rather than once per poll.
    travel_emitted: Vec<TokenId>,
    /// `ISOMETRY_TRAVEL_SELFTEST`: register two campaign maps joined by a door
    /// and walk the knight through it. Focus-free, like the others.
    travel_selftest: bool,
    travel_fired: bool,
    /// `ISOMETRY_CMD_SELFTEST`: drive the `>` command line: spawn, find, and a
    /// full `>gen npc` generate/commit into a statted NPC.
    cmd_selftest: bool,
    cmd_fired: bool,
    /// `ISOMETRY_CONVINCE_SELFTEST`: a bard wins a goblin over, then hits the
    /// party cap on the next one. Proves allegiance + the cap + fog.
    convince_selftest: bool,
    convince_fired: bool,
    /// `ISOMETRY_STORYLET_SELFTEST`: seed two storylets (one ready, one locked),
    /// open the surface, play the ready one, and confirm its fact commits.
    storylet_selftest: bool,
    storylet_fired: bool,
    /// `ISOMETRY_OVERMAP_SELFTEST`: seed a small overmap, place the party, and
    /// open the overmap surface for a capture (C8).
    overmap_selftest: bool,
    overmap_fired: bool,
    /// `ISOMETRY_OVERMAP_SOURCE_TIME_SELFTEST`: after the normal overmap seed,
    /// capture its first durable source prefix. This stays opt-in so the C8
    /// screenshot mode remains a live exploration proof.
    overmap_source_time_selftest: bool,
    /// `ISOMETRY_COMPENDIUM_SELFTEST`: open the compendium and type into its
    /// filter through the field's own `TextInput`, then hold the surface open
    /// for a capture. The headed half of M3's compendium lane.
    compendium_selftest: bool,
    compendium_fired: bool,
    /// `ISOMETRY_WHISPER_SELFTEST`: open the whisper composer with `w` and type
    /// a draft into it, then hold it open — unsent, caret in it — for a
    /// capture. The headed half of M3's composer lane.
    whisper_selftest: bool,
    whisper_fired: bool,
    /// `ISOMETRY_COMBAT_SELFTEST`: drive a short adjudicated exchange on boot.
    combat_selftest: bool,
    /// Swings left to throw, when the last one landed, and whether the winner
    /// has taken its bow.
    combat_swings: u8,
    last_swing: Option<Instant>,
    combat_emoted: bool,
    /// Session start instant, for the self-test delay.
    started: Option<Instant>,
    selftest_fired: bool,
    /// The loaded game system (owns the Lua interpreter); character
    /// sheets evaluate through it.
    system: Option<System>,
    /// Last open sheet, so derived stats recompute only on change.
    last_sheet_open: Option<isometry_core::TokenId>,
    /// Entropy remains host-owned even while a preview is uncommitted. Each
    /// accepted record carries its exact draw; no peer evaluates the pack.
    generation_tape: EntropyTape,
    generation_ordinal: u64,
    generator_catalog: GeneratorCatalog,
    /// The last Cleromancy-backed generator choice. This remains a host-local
    /// receipt: previews and commits continue through their normal Isometry
    /// path, while campaign state never contains this selection record.
    last_generator_selection: Option<cleromancy_selection::GeneratorSelection>,
    /// The real faction moves behind the downtime surface's display rows,
    /// index-aligned with them. The DM strikes rows in the view; on commit the
    /// host keeps the moves whose row survived. Rolled from `generation_tape`.
    faction_turn_batch: Vec<FactionMove>,
}

/// Parsed session role from the command line.
enum NetIntent {
    Host,
    Join(String),
}

fn document_slug(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// The public, reviewable map document.
fn map_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("maps").join(format!("{}.json", document_slug(name)))
}

/// The private GM store paired with a map. Muniment's redb backend makes the
/// slot durable; it is intentionally outside the map's shareable JSON.
fn campaign_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("campaigns").join(format!("{}.redb", document_slug(name)))
}

/// Search the bundled example, a project-local pack root, and user-selected
/// roots. Entries may be pack directories or directories containing packs.
/// How many tokens a player owns across the *whole* campaign: the active board
/// plus every stored map. The party cap is a limit on a person's followers, and
/// a split party (C3) has them spread over several maps, so counting only the
/// active board would let the cap be dodged by recruiting on each map in turn.
fn owner_token_count(ui: &UiState, owner: &str) -> u32 {
    let on = |m: &isometry_core::MapDocument| {
        m.tokens
            .iter()
            .filter(|t| t.owner.as_deref() == Some(owner))
            .count()
    };
    let active = on(&ui.map);
    let stored: usize = ui
        .campaign_maps
        .iter()
        // The active map is also mirrored into the stored registry, so skip it
        // there to avoid double-counting its tokens.
        .filter(|(id, _)| Some(id.as_str()) != ui.active_map.as_deref())
        .map(|(_, m)| on(&m.document))
        .sum();
    (active + stored) as u32
}

/// A free tile to place a generated token on, scanning outward from (2, 2) past
/// anything occupied. The host commit path holds only a snapshot, so this is the
/// snapshot twin of the view's `free_spawn_tile`.
fn free_snapshot_tile(map: &MapDocument) -> TileCoord {
    let occupied: std::collections::HashSet<TileCoord> = map.tokens.iter().map(|t| t.at).collect();
    let free = |at: TileCoord| map.ground.in_bounds(at.0, at.1) && !occupied.contains(&at);
    // Prefer a free interior tile scanning outward from (2, 2), but never leave
    // the board: a narrow or short map has no col/row 2..17, and placing a token
    // off-map fails the whole commit (TokenPlaced rejects out-of-bounds).
    for d in 0..256 {
        let at = (2 + (d % 16), 2 + (d / 16));
        if free(at) {
            return at;
        }
    }
    // The window missed (a small or packed map): take any free in-bounds tile.
    let (w, h) = (map.ground.width() as i32, map.ground.height() as i32);
    for row in 0..h {
        for col in 0..w {
            if free((col, row)) {
                return (col, row);
            }
        }
    }
    (0, 0) // board is full or empty; (0,0) is in-bounds for any non-empty map
}

/// The next free token id across the whole campaign, so a generated NPC never
/// collides with a resident of another stored map (inventories key on `TokenId`
/// globally). Same discipline as travel's id minting.
fn next_snapshot_id(snapshot: &GameSnapshot) -> TokenId {
    let max = snapshot
        .maps
        .values()
        .flat_map(|m| m.document.tokens.iter())
        .chain(snapshot.map.tokens.iter())
        .map(|t| t.id.0)
        .chain(snapshot.inventories.keys().map(|id| id.0))
        .max()
        .unwrap_or(0);
    TokenId(max + 1)
}

fn generator_pack_roots() -> Vec<std::path::PathBuf> {
    // The `core` pack ships the default beat vocabulary (strike, recoil, fall,
    // cheer...). It is a pack like any other, so a campaign overrides a beat
    // simply by declaring the same name: the app owns no choreography.
    let mut roots = vec![
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../isometry-system/examples/packs/core"),
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../isometry-system/examples/packs/demo"),
    ];
    let local = std::path::PathBuf::from("packs");
    if local.is_dir() {
        roots.push(local);
    }
    if let Some(paths) = std::env::var_os("ISOMETRY_PACK_DIRS") {
        roots.extend(std::env::split_paths(&paths));
    }
    roots
}

/// Parse `--host` or `--join <ticket>` from the command line.
fn parse_net_intent() -> Option<NetIntent> {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--host") {
        Some(NetIntent::Host)
    } else if let Some(i) = args.iter().position(|a| a == "--join") {
        args.get(i + 1).map(|t| NetIntent::Join(t.clone()))
    } else {
        None
    }
}

/// Parse `--as <player>` from the command line.
fn parse_viewer() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--as")
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// Parse `--campaign <name>` for checkpoint restore. The name shares the map
/// slug convention, so `--campaign "Demo Skirmish"` resolves its paired store.
fn parse_campaign() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--campaign")
        .and_then(|i| args.get(i + 1))
        .cloned()
}

impl App {
    /// Everything the command line and the environment decided, before a window
    /// exists. The content packs are read here because the stylesheet the host
    /// is handed in `init` is the app sheet plus whatever choreography they
    /// declared.
    fn boot() -> Self {
        let generator_catalog = GeneratorCatalog::discover(generator_pack_roots());
        // Choreography is pack data: the stylesheet the packs supply is appended
        // to the app's, and the emote menu is built from whichever beats they
        // marked emotable. A table with no packs still plays a correct game; it
        // just plays it without flourishes, which is safe precisely because no
        // rule may read a beat.
        let (pack_beats, beat_diagnostics) = generator_catalog.choreography();
        for diagnostic in &beat_diagnostics {
            eprintln!("[isometry] choreography: {diagnostic}");
        }
        let mut sheet = board_css();
        for beat in &pack_beats {
            sheet.push('\n');
            sheet.push_str(&beat.css);
        }
        let pack_emotes: Vec<(String, String)> = pack_beats
            .iter()
            .filter_map(|b| b.emote.clone().map(|label| (b.name.clone(), label)))
            .collect();
        Self {
            campaign: CampaignStore::new(),
            journal: Vec::new(),
            history: Journal::new(),
            history_origin: None,
            source_history_len: None,
            source_history_attached: false,
            last_overmap_swatch: None,
            last_storylet_inputs: None,
            // A fixed seed keeps a solo session reproducible and makes the headed
            // verification deterministic. A real table seeds this per session.
            action_rng: Rng::new(0x15D_0BE),
            own_requests: 0,
            beat_until: None,
            sheet,
            last_viewport: (0.0, 0.0),
            profile: std::env::var_os("ISOMETRY_PROFILE").is_some(),
            capture_dir: std::env::var_os("ISOMETRY_CAPTURE_DIR").map(Into::into),
            net_intent: parse_net_intent(),
            net_is_host: false,
            viewer_arg: parse_viewer(),
            campaign_arg: parse_campaign(),
            net: None,
            last_net_version: 0,
            net_selftest: std::env::var_os("ISOMETRY_NET_SELFTEST").is_some(),
            travel_selftest: std::env::var_os("ISOMETRY_TRAVEL_SELFTEST").is_some(),
            travel_fired: false,
            cmd_selftest: std::env::var_os("ISOMETRY_CMD_SELFTEST").is_some(),
            cmd_fired: false,
            convince_selftest: std::env::var_os("ISOMETRY_CONVINCE_SELFTEST").is_some(),
            convince_fired: false,
            storylet_selftest: std::env::var_os("ISOMETRY_STORYLET_SELFTEST").is_some(),
            storylet_fired: false,
            overmap_selftest: std::env::var_os("ISOMETRY_OVERMAP_SELFTEST").is_some(),
            overmap_fired: false,
            overmap_source_time_selftest: std::env::var_os("ISOMETRY_OVERMAP_SOURCE_TIME_SELFTEST")
                .is_some(),
            compendium_selftest: std::env::var_os("ISOMETRY_COMPENDIUM_SELFTEST").is_some(),
            compendium_fired: false,
            whisper_selftest: std::env::var_os("ISOMETRY_WHISPER_SELFTEST").is_some(),
            whisper_fired: false,
            combat_selftest: std::env::var_os("ISOMETRY_COMBAT_SELFTEST").is_some(),
            combat_swings: 4,
            last_swing: None,
            combat_emoted: false,
            travel_emitted: Vec::new(),
            started: None,
            selftest_fired: false,
            system: None,
            last_sheet_open: None,
            // `ISOMETRY_GEN_SEED` fixes the generator tape so `>gen` previews and
            // rerolls are reproducible (headed verification, and a table that wants
            // a deterministic session); otherwise the wall clock seeds it as before.
            generation_tape: EntropyTape::from_seed(
                std::env::var("ISOMETRY_GEN_SEED")
                    .ok()
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or_else(|| {
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|duration| duration.as_nanos() as u64)
                            .unwrap_or(1)
                    }),
            ),
            generation_ordinal: 0,
            generator_catalog,
            last_generator_selection: None,
            faction_turn_batch: Vec::new(),
            pack_emotes,
        }
    }
}

fn main() {
    let app = Rc::new(RefCell::new(App::boot()));
    let init_app = app.clone();
    let options = cambium_genet_winit_host::HostOptions {
        title: "Isometry".into(),
        // The pre-migration window size, which every receipt under
        // `testing/isometry/images` was taken at; the host's own default is
        // shorter (1100x664) and cuts the side panel off below the Dice rows.
        // The host clamps this to the primary monitor's logical height less
        // 48, so a short display opens smaller than asked.
        initial_logical_size: (DESIGN_SIZE.0 as f64, DESIGN_SIZE.1 as f64),
        // ...and when it does, the interface is scaled to fit rather than cut
        // off. The panel is laid out for 820 and this laptop offers 752, so
        // the host lays out at `surface / fit` on every resize and isometry
        // does no zoom arithmetic of its own: `AppCtx::logical_size`, the
        // pointer, hit testing and the harness are all already in that space.
        // On a display with the room the factor is 1 and this is inert.
        fit_design: Some(DESIGN_SIZE),
        // Host decorations: isometry draws no client-side chrome, so the
        // platform's own title bar and window buttons stay (`WindowFrame::Host`,
        // the default). `initial_geometry` stays unset: isometry does not
        // persist window geometry in this pass.
        ..Default::default()
    };
    cambium_genet_winit_host::run(
        options,
        move |window, _commands, wake| hooks::init(&init_app, window, wake),
        hooks::hooks(&app),
    )
    .expect("run app");
}
