//! Isometry's half of the shared host: `init` plus the seven hooks.
//!
//! The host calls `init` once, after the window exists and before the first
//! frame, and hands back a runner over what it returns. Everything after that
//! is a hook: a plain closure over the `Rc<RefCell<App>>` this module builds
//! them from. Nothing here knows about winit, wgpu, netrender or a layout
//! engine — the boundary the migration bought.

use std::cell::RefCell;
use std::rc::Rc;

use cambium_genet_winit_host::{
    CloseDisposition, FocusedTextSlot, HostHooks, HostWake, HostWindow, Init, Key, KeyPress,
    NamedKey,
};
use genet_scripted_dom::NodeId;
use layout_dom_api::{LayoutDom as _, LocalName, Namespace};

use super::*;

/// Isometry's three editable lanes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Lane {
    /// The side panel's `>` command line.
    Command,
    /// The side panel's whisper composer.
    Whisper,
    /// The compendium index's filter.
    Search,
}

/// Which lane holds the caret, and the `<input>` node that carries it.
///
/// Recognized by the class of the field's wrapper, the way woodshed recognizes
/// its two: the DOM the host hands back is the only place the app and the host
/// can agree on which control has the caret. Those three class names are
/// therefore load-bearing — `panel.rs` and `compendium.rs` set them, and a
/// rename in either place breaks typing rather than styling.
fn focused_lane(runner: &Runner) -> Option<(NodeId, Lane)> {
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
        _ => return None,
    };
    Some((node, lane))
}

/// The text seam: which field has the caret, and how to reach its `TextInput`.
///
/// All three lanes ride `caret_text_field` as of M3, so the host owns the
/// caret, the selection, drag-selection, visual caret movement and IME in each
/// of them, and no key is rebuilt into a `String` on this side any more.
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
    })
}

/// Whether `press` is this character with no command chord held.
fn plain_char(press: &KeyPress, want: &str) -> bool {
    matches!(&press.key, Key::Character(c) if c == want) && !press.modifiers.ctrl
}

/// Whether `press` is this named key.
fn named(press: &KeyPress, want: NamedKey) -> bool {
    matches!(&press.key, Key::Named(k) if *k == want)
}

/// What a key press means before the tree sees it.
///
/// Everything that used to live in the desktop host's `key`, in the same order,
/// because the order *is* the policy: an open transient surface outranks the
/// text lanes, which outrank the single-letter verbs. Returning `true` consumes
/// the press.
///
/// What M3 changed is *how* a lane is recognised. There used to be three flag
/// branches here, each rebuilding a draft one character at a time. There is one
/// now, and it asks the caret: only one field can hold it, so the flags cannot
/// disagree about who is typing. Escape and Enter stay app commands and
/// everything else falls through, so the host routes it into
/// `caret_text_field` — which is where the text, the caret, the selection and
/// IME live now.
pub(crate) fn key_intercept(runner: &mut Runner, press: &KeyPress) -> bool {
    // Escape backs out of target-pick before anything else reads it, so an
    // armed attack is always cancellable without spending a turn.
    if runner.state().picking_target() && named(press, NamedKey::Escape) {
        runner.update(|ui| ui.cancel_action_pick());
        return true;
    }
    // An open token menu is the most recent thing on screen and the first
    // thing Escape takes back. A press elsewhere is the `overlay_surface`'s
    // dismissal layer; this is the keyboard half, which the surface's own
    // passive listener cannot serve because nothing inside the menu is focused.
    if runner.state().context_menu.is_some() && named(press, NamedKey::Escape) {
        runner.update(|ui| ui.close_context_menu());
        return true;
    }
    // A focused text lane owns every key but its own two.
    if let Some((_, lane)) = focused_lane(runner) {
        if named(press, NamedKey::Escape) {
            match lane {
                Lane::Command => runner.update(|ui| ui.command_cancel()),
                Lane::Whisper => runner.update(|ui| ui.compose_cancel()),
                Lane::Search => runner.update(|ui| ui.compendium_escape()),
            }
            return true;
        }
        if named(press, NamedKey::Enter) {
            match lane {
                Lane::Command => runner.update(|ui| ui.command_submit()),
                Lane::Whisper => runner.update(|ui| ui.compose_send()),
                // The filter has nothing to submit, but Enter must not reach
                // `end_turn` behind an open compendium either.
                Lane::Search => {}
            }
            return true;
        }
        return false;
    }
    // The compendium with no field focused: an entry page is open, so there is
    // no filter to type into. It still swallows the press — a single-letter
    // verb firing on the board behind an open page was never the policy —
    // and Escape still steps back to the index.
    if runner.state().compendium_open {
        if named(press, NamedKey::Escape) {
            runner.update(|ui| ui.compendium_escape());
        }
        return true;
    }
    // The command sigil opens the > line, the way `w` opens a whisper. The
    // draft starts empty; the ">" is the prompt.
    if plain_char(press, ">") {
        runner.update(|ui| ui.start_command());
        return true;
    }
    if plain_char(press, "w") {
        runner.update(|ui| ui.start_compose());
        return true;
    }
    if plain_char(press, "r") {
        runner.update(|ui| ui.rotate_selected());
        return true;
    }
    if plain_char(press, "f") {
        // Cycle the fog viewer: omniscient, then each side. Lets the DM preview
        // a player's view (and drives single-window fog verification without a
        // session).
        runner.update(|ui| ui.cycle_viewer());
        return true;
    }
    if named(press, NamedKey::Enter) {
        runner.update(|ui| ui.end_turn());
        return true;
    }
    if press.modifiers.ctrl {
        if matches!(&press.key, Key::Character(c) if c == "z") {
            runner.update(|ui| ui.undo());
            return true;
        }
        if matches!(&press.key, Key::Character(c) if c == "y") {
            runner.update(|ui| ui.redo());
            return true;
        }
    }
    let pan = match &press.key {
        Key::Named(NamedKey::ArrowLeft) => Some((-1.0, 1.0)),
        Key::Named(NamedKey::ArrowRight) => Some((1.0, -1.0)),
        Key::Named(NamedKey::ArrowUp) => Some((-1.0, -1.0)),
        Key::Named(NamedKey::ArrowDown) => Some((1.0, 1.0)),
        _ => None,
    };
    if let Some((dc, dr)) = pan {
        runner.update(|ui| ui.pan_tiles(dc, dr));
        return true;
    }
    false
}

/// Build the starting state: the map, the campaign checkpoint, the game system,
/// the generator catalog, and the session bridge. Runs once, inside the host,
/// after the window exists but before the first frame.
pub(crate) fn init(
    app: &Rc<RefCell<App>>,
    window: &dyn HostWindow,
    wake: &HostWake,
) -> Init<UiState, Logic> {
    let mut app = app.borrow_mut();
    // `ISOMETRY_SYNTH=<n>` loads an n x n synthetic stress board (n>1,
    // default 30 = the probe P2 board) instead of the demo skirmish;
    // large n exercises viewport windowing.
    let map = match std::env::var("ISOMETRY_SYNTH") {
        Ok(v) => {
            let n = v
                .trim()
                .parse::<u32>()
                .ok()
                .filter(|&n| n > 1)
                .unwrap_or(30);
            synth_map(n, n)
        }
        Err(_) => demo_map(),
    };
    let can_restore = !matches!(app.net_intent.as_ref(), Some(NetIntent::Join(_)));
    let mut restore_status = None;
    let mut restored_public = None;
    if can_restore {
        if let Some(name) = app.campaign_arg.take() {
            match CampaignRepository::open(campaign_path(&name))
                .and_then(|repository| repository.load_checkpoint())
            {
                Ok(Some(checkpoint)) => {
                    app.campaign = checkpoint.private;
                    app.journal = checkpoint.public.journal.clone();
                    app.history = checkpoint.history;
                    app.history_origin = checkpoint.history_origin;
                    app.source_history_len = None;
                    app.source_history_attached = false;
                    restored_public = Some(checkpoint.public);
                    restore_status = Some(format!("restored campaign {name}"));
                }
                Ok(None) => restore_status = Some(format!("campaign {name} has no checkpoint")),
                Err(error) => restore_status = Some(format!("campaign restore failed: {error}")),
            }
        }
    }
    let mut ui = UiState::new(map);
    if let Some(snapshot) = restored_public {
        ui.apply_snapshot(snapshot);
    }
    ui.generator_choices = app.generator_catalog.choices();
    for diagnostic in app.generator_catalog.diagnostics() {
        eprintln!("[isometry] content pack: {diagnostic}");
    }
    if let Some(status) = restore_status {
        ui.status = status;
    }
    // Start with the board roughly centered in the pane, and every
    // token in the turn order (a skirmish ready to play; drop
    // tokens out via the panel for free movement).
    ui.camera = (420.0, 140.0);
    // Seed the pane size so the view can window tile emission to the
    // viewport (the frame hook keeps it current on resize).
    //
    // The window's scale factor is the *device* scale and knows nothing about
    // the design fit, so dividing by it alone would hand the view a viewport a
    // whole zoom factor wrong for one frame. The host measures `fit_design`
    // against the pre-zoom surface and lays out at `surface / zoom`; `fit_zoom`
    // is public precisely so a consumer can compute the same number rather than
    // a near miss, and `DESIGN_SIZE` is the one `HostOptions` was handed.
    let (width, height) = window.inner_size();
    let device = window.scale_factor() as f32;
    let available = (width as f32 / device, height as f32 / device);
    let zoom = cambium_genet_winit_host::fit_zoom(DESIGN_SIZE, available);
    let (logical_w, logical_h) = (available.0 / zoom, available.1 / zoom);
    ui.viewport = ((logical_w - PANEL_W).max(0.0), logical_h);
    app.last_viewport = ui.viewport;
    // The board's pixel grid, before the first frame rather than after it: a
    // board laid out at the raw fractional zoom for one frame and re-laid out
    // on the next is a visible jump on a slow boot.
    ui.set_pixel_grid((device, zoom));
    let ids: Vec<_> = ui.map.tokens.iter().map(|t| t.id).collect();
    for id in ids {
        ui.turns.add(id);
    }
    let initial_snapshot = app.snapshot_of(&ui);
    if !matches!(app.net_intent.as_ref(), Some(NetIntent::Join(_))) {
        app.ensure_history_origin(&initial_snapshot);
    }
    if let Some(origin) = app.history_origin.clone() {
        ui.set_overmap_source_history(Some(isometry_net::GameSourceHistory::new(
            origin,
            app.history.clone(),
        )));
        app.source_history_len = Some(app.history.len());
        app.source_history_attached = true;
    }

    // Session setup: host publishes this board; a client starts from
    // an empty view and fills in on the first snapshot. Either way the
    // view is Remote, so play routes through the session.
    //
    // The bridge's actor wakes this application through the host's own wake
    // handle, so a snapshot arriving on the network thread schedules one drain
    // turn (`after_wake`) and one redraw rather than waiting on a poll.
    match app.net_intent.take() {
        Some(NetIntent::Host) => {
            app.net_is_host = true;
            ui.net_mode = NetMode::Remote;
            let snapshot = GameSnapshot {
                map: ui.map.clone(),
                turns: ui.turns.clone(),
                roll_log: Vec::new(),
                journal: app.journal.clone(),
                inventories: ui.inventories.clone(),
                generations: ui.generations.clone(),
                maps: ui.campaign_maps.clone(),
                active_map: ui.active_map.clone(),
                world: ui.world.clone(),
                clocks: ui.clocks.clone(),

                party_cap: ui.party_cap,
                last_beats: Vec::new(),
                beat_seq: 0,
                applied_actions: Default::default(),
            };
            let campaign = app.campaign.clone();
            let history = app.history.clone();
            app.net = Some(NetBridge::spawn(
                Role::Host {
                    state: snapshot,
                    campaign,
                    history,
                },
                wake.callback(),
            ));
        }
        Some(NetIntent::Join(ticket)) => {
            ui.net_mode = NetMode::Remote;
            ui.can_edit_inventory = false;
            ui.status = "connecting...".to_owned();
            let name = app
                .viewer_arg
                .clone()
                .unwrap_or_else(|| "player".to_owned());
            app.net = Some(NetBridge::spawn(
                Role::Client { ticket, name },
                wake.callback(),
            ));
        }
        None => {}
    }
    // Boot clock. The net selftest waits on it, and so does the combat
    // selftest, which runs solo (there is no session to wait for).
    app.started = Some(Instant::now());
    // Fog viewer from `--as`. Applies in any mode: a client sees
    // through its player's tokens, and a solo run can preview a side.
    if let Some(v) = app.viewer_arg.take() {
        ui.viewer = Some(v);
        ui.recompute_fog();
    }
    // Seed the dice generator with real entropy so rolls differ per
    // launch (the clock is plenty for a friendly table).
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1);
    ui.reseed(seed);

    // Load the game system (5e SRD) and hand the view its schema so it
    // can render sheets without knowing any rules.
    let system = srd_5e();
    ui.sheet_schema = schema_of(&system);
    ui.bestiary = bestiary_of();
    ui.emotes = app.pack_emotes.clone();
    ui.spells = spells_of();
    ui.items = items_of();
    app.system = Some(system);

    Init {
        state: ui,
        logic: board_root as Logic,
        sheet: std::mem::take(&mut app.sheet),
    }
}

impl App {
    /// Keep the view's pane size current so windowing culls to the actual
    /// viewport. Cheap-checked: writing the same two floats every frame would
    /// rebuild the retained tree for nothing.
    fn sync_viewport(&mut self, ctx: &mut Ctx<'_>) {
        let (width, height) = ctx.logical_size;
        let viewport = ((width - PANEL_W).max(0.0), height);
        if self.last_viewport == viewport {
            return;
        }
        self.last_viewport = viewport;
        ctx.runner.update(|ui| ui.viewport = viewport);
    }

    /// Keep the board's pixel grid current: the device scale the window is on
    /// and the zoom the host is laying out at.
    ///
    /// It belongs in a hook rather than in the view because only the host knows
    /// either number — `AppCtx::ui_zoom` is the effective zoom, fit times user
    /// offset, and the scale factor is the window's. `zoom_changed` is the edge
    /// the host raises, but a window dragged to a monitor of another density
    /// moves the device scale as well, so the compare is on the pair: two float
    /// tests per frame, and an update only when the board really must move.
    fn sync_board_scale(&mut self, ctx: &mut Ctx<'_>) {
        let device = ctx
            .window
            .map_or(1.0, |window| window.scale_factor() as f32);
        let grid = (device, ctx.ui_zoom);
        if ctx.runner.state().pixel_grid == grid {
            return;
        }
        ctx.runner.update(|ui| ui.set_pixel_grid(grid));
    }

    /// Register (or clear) the overmap's painted graph leaf so the view's
    /// `<custom-leaf>` gets nodes + edges.
    ///
    /// The swatch is only *built* while the surface is open (building it
    /// projects the world and runs the force layout — never pay that on an
    /// ordinary board frame), and the leaf is only *re-registered* when the
    /// swatch model changed: a fresh `GraphCanvas` is born dirty, so an
    /// unconditional insert would defeat the leaf-tier retention gate and
    /// repaint every frame. Both gates are load-bearing; read the 2026-07-20
    /// perf plan before loosening either.
    fn sync_overmap_leaf(&mut self, ctx: &mut Ctx<'_>) {
        if ctx.runner.state().overmap_open {
            match isometry_views::overmap_swatch(ctx.runner.state()) {
                Some(swatch) => {
                    if self.last_overmap_swatch.as_ref() != Some(&swatch) {
                        ctx.leaves.insert(
                            isometry_views::OVERMAP_LEAF_KEY,
                            Box::new(swatch.paint_leaf(overmap_node_color)),
                        );
                        self.last_overmap_swatch = Some(swatch);
                    }
                }
                None => {
                    ctx.leaves.remove(&isometry_views::OVERMAP_LEAF_KEY);
                    self.last_overmap_swatch = None;
                }
            }
        } else if self.last_overmap_swatch.is_some() {
            ctx.leaves.remove(&isometry_views::OVERMAP_LEAF_KEY);
            self.last_overmap_swatch = None;
        }
    }

    /// Hold a staged beat for [`BEAT_HOLD`], then drop its classes. Returns
    /// whether frames should keep coming.
    ///
    /// The drop is what makes the *next* strike a genuine attribute change: an
    /// unchanged class restyles nothing, so without it the second swing would
    /// stand still.
    fn drive_beats(&mut self, ctx: &mut Ctx<'_>) -> bool {
        if ctx.runner.state().beats.is_empty() {
            self.beat_until = None;
            return false;
        }
        match self.beat_until {
            Some(until) if Instant::now() >= until => {
                self.beat_until = None;
                ctx.runner.update(|ui| ui.clear_beats());
                false
            }
            Some(_) => true,
            None => {
                self.beat_until = Some(Instant::now() + BEAT_HOLD);
                true
            }
        }
    }

    /// Arm a readback of the next presented frame for `ISOMETRY_CAPTURE_DIR`.
    ///
    /// The host runs it inside the frame, while the rasterized view is still
    /// alive, so the receipt is the frame that was actually presented — no
    /// compositor, no foreground window, and no chance of photographing
    /// something else.
    fn arm_capture(&self, ctx: &mut Ctx<'_>) {
        let Some(dir) = self.capture_dir.clone() else {
            return;
        };
        *ctx.capture = Some(Box::new(move |surface, view, width, height| {
            let Some(frame) = cambium_genet_winit_host::read_frame(surface, view, width, height)
            else {
                return;
            };
            let path = dir.join("isometry_capture.png");
            if let Err(e) = std::fs::create_dir_all(&dir).and_then(|_| {
                let file = std::fs::File::create(&path)?;
                let mut enc = png::Encoder::new(std::io::BufWriter::new(file), width, height);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                let mut writer = enc.write_header().map_err(std::io::Error::other)?;
                writer
                    .write_image_data(&frame.rgba)
                    .map_err(std::io::Error::other)?;
                Ok(())
            }) {
                eprintln!("[isometry] capture failed: {e}");
            }
        }));
    }

    /// Whether an armed self-test still needs frames to reach its deadline.
    ///
    /// A still board parks on `Wait`, which blocks until input arrives, so an
    /// armed selftest would never reach its own deadline. Asking for frames is
    /// how the shared host's hook says the same thing the old `WaitUntil` did.
    fn selftests_pending(&self) -> bool {
        (self.travel_selftest && !self.travel_fired)
            || (self.cmd_selftest && !self.cmd_fired)
            || (self.convince_selftest && !self.convince_fired)
            || (self.storylet_selftest && !self.storylet_fired)
            || (self.overmap_selftest && !self.overmap_fired)
            || (self.compendium_selftest && !self.compendium_fired)
            || (self.whisper_selftest && !self.whisper_fired)
            || (self.net_selftest && !self.selftest_fired)
            || (self.combat_selftest && !(self.combat_swings == 0 && self.combat_emoted))
    }

    /// Run every armed self-test driver. Each one waits out its own warm-up.
    fn drive_selftests(&mut self, ctx: &mut Ctx<'_>) {
        self.maybe_combat_selftest(ctx);
        self.maybe_travel_selftest(ctx);
        self.maybe_cmd_selftest(ctx);
        self.maybe_convince_selftest(ctx);
        self.maybe_storylet_selftest(ctx);
        self.maybe_overmap_selftest(ctx);
        self.maybe_compendium_selftest(ctx);
        self.maybe_whisper_selftest(ctx);
        if self.net.is_some() {
            self.maybe_selftest(ctx);
        }
    }
}

/// Isometry's seven closures over one shared `App`.
pub(crate) fn hooks(app: &Rc<RefCell<App>>) -> HostHooks<UiState, Logic, UiChild> {
    let frame_app = app.clone();
    let dispatch_app = app.clone();
    let after_frame_app = app.clone();
    let wake_app = app.clone();
    HostHooks {
        frame: Box::new(move |ctx: &mut Ctx<'_>| {
            let mut app = frame_app.borrow_mut();
            // Before the viewport: culling measures the pane against the board
            // geometry, and the two must agree on the same frame.
            app.sync_board_scale(ctx);
            app.sync_viewport(ctx);
            app.sync_overmap_leaf(ctx);
            app.arm_capture(ctx);
            let beating = app.drive_beats(ctx);
            beating || app.selftests_pending()
        }),
        after_dispatch: Box::new(move |ctx: &mut Ctx<'_>| {
            dispatch_app.borrow_mut().after_dispatch(ctx);
        }),
        after_frame: Box::new(move |ctx: &mut Ctx<'_>| {
            let mut app = after_frame_app.borrow_mut();
            app.drive_selftests(ctx);
            if app.profile {
                if let Some(profile) = ctx.frame_profile {
                    eprintln!("[isometry] frame {}", profile.summary());
                }
            }
        }),
        // The session actor woke us: drain what it sent, on the UI thread, in
        // one turn. The pumps that used to ride a 10Hz idle tick ride this
        // instead, because a wake is exactly the moment they have work.
        after_wake: Box::new(move |ctx: &mut Ctx<'_>| {
            let mut app = wake_app.borrow_mut();
            if app.net.is_none() {
                return;
            }
            app.pump_net(ctx);
            app.pump_sheets(ctx);
            app.pump_generators(ctx);
            app.pump_storylets(ctx);
            app.refresh_source_history(ctx);
        }),
        // Nothing here outlives the window: the campaign checkpoint is written
        // on an explicit save, and the session actor dies with the process.
        close_request: Box::new(|_ctx, _request| CloseDisposition::Exit),
        focused_text: Box::new(focused_text),
        key_intercept: Box::new(key_intercept),
    }
}
