//! The typed-surface lanes: the command line, the compendium, the composer.
//!
//! `ISOMETRY_CMD_SELFTEST`, `ISOMETRY_COMPENDIUM_SELFTEST` and
//! `ISOMETRY_WHISPER_SELFTEST`. All three send their letters down the
//! shipping key path, so the field's own `TextInput` is what edits the text;
//! nothing here appends a character.
//!
//! Split out of `selftest.rs` on 2026-09-04; unchanged.

use super::*;

impl App {
    /// `ISOMETRY_TURNS_SELFTEST=1`: collapse the Turns disclosure through the
    /// same laid-out trigger and host pointer route as a person's click, then
    /// leave the short panel standing for the headed capture.
    pub(crate) fn maybe_turns_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.turns_selftest || self.turns_fired {
            return;
        }
        if !self
            .started
            .is_some_and(|started| started.elapsed() > Duration::from_secs(2))
        {
            return;
        }

        let trigger = {
            let dom = ctx.runner.dom();
            let dom = dom.borrow();
            dom.all_with_class(dom.document(), "disclosure-trigger")
                .into_iter()
                .next()
        };
        let Some((x, y, width, height)) = trigger.and_then(|node| ctx.painted_rect(node)) else {
            eprintln!("[isometry] turns selftest: disclosure trigger has no painted box");
            self.turns_fired = true;
            return;
        };
        let (x, y) = (x + width / 2.0, y + height / 2.0);
        ctx.pointer.push(HostPointer::Moved(x, y));
        ctx.pointer.push(HostPointer::Press(x, y));
        ctx.pointer.push(HostPointer::Release(x, y));
        self.turns_fired = true;
        eprintln!("[isometry] turns selftest: clicked disclosure trigger at ({x:.1}, {y:.1})");
    }

    /// `ISOMETRY_CMD_SELFTEST=1` (pair with `ISOMETRY_GEN_SEED` for a fixed
    /// NPC): drive the whole `>` command surface once, focus-free.
    pub(crate) fn maybe_cmd_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.cmd_selftest || self.cmd_fired {
            return;
        }
        if !self
            .started
            .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
        {
            return;
        }
        self.cmd_fired = true;
        let before = Some(&*ctx.runner)
            .map(|r| r.state().map.tokens.len())
            .unwrap_or(0);

        // >spawn: a statted goblin, resolved from a free-text query.
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| ui.spawn_query("gobl"));
        }
        self.pump_sheets(ctx); // binds the stat block

        // >find: a unified compendium search.
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| ui.find_query("sword"));
            eprintln!(
                "[isometry] cmd selftest: find 'sword' -> {} results, first: {:?}",
                runner.state().command_results.len(),
                runner.state().command_results.first(),
            );
        }

        // The receipt path only selects an existing declaration; the following
        // two pumps reuse the normal preview call. Its result stays host-local.
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                ui.choose_generator(
                    "cmd-selftest".to_owned(),
                    "isometry.cmd-selftest/v1".to_owned(),
                    "What should I prepare for the next scene?".to_owned(),
                )
            });
        }
        self.pump_generators(ctx); // receipt -> selected declaration -> Generate
        self.pump_generators(ctx); // Generate -> preview
        eprintln!(
            "[isometry] cmd selftest: receipt selection = {:?}",
            self.last_generator_selection
                .as_ref()
                .map(|selection| (&selection.reading.candidate_id, &selection.reading.receipt))
        );
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| ui.discard_generation_preview());
        }

        // >gen npc: open the generator, generate a preview, commit it.
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| ui.start_generator("npc"));
        }
        self.pump_generators(ctx); // Generate -> preview
        let previewed = Some(&*ctx.runner).and_then(|r| r.state().generator_preview.clone());
        eprintln!("[isometry] cmd selftest: gen npc preview = {previewed:?}");
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| ui.commit_generation_preview());
        }
        self.pump_generators(ctx); // Commit -> lower into a statted token
        self.pump_sheets(ctx);

        {
            let runner = &*ctx.runner;
            let ui = runner.state();
            let newest = ui.map.tokens.last();
            eprintln!(
                "[isometry] cmd selftest: tokens {} -> {} | newest {:?} sheet name {:?} hp {:?} | status: {}",
                before,
                ui.map.tokens.len(),
                newest.map(|t| (t.id.0, t.sprite.clone(), t.at)),
                newest
                    .and_then(|t| ui.map.sheet(t.id))
                    .and_then(|s| s.text("name").map(str::to_owned)),
                newest
                    .and_then(|t| ui.map.sheet(t.id))
                    .and_then(|s| s.int("hp_current")),
                ui.status,
            );
        }
    }

    /// `ISOMETRY_COMPENDIUM_SELFTEST=1`: open the compendium and type "gob"
    /// into its filter, then leave the surface open for the capture.
    ///
    /// M3 turned that filter into a `TextInput` under `caret_text_field`, and
    /// the harness receipt for it can only assert that the field has a non-zero
    /// box. This is the half a windowless test cannot reach: whether the lane
    /// actually *draws* — chrome, caret, the typed query, and an index narrowed
    /// to what the query matched — in a frame that was really presented.
    ///
    /// The letters ride the shipping key path, so the field's own `TextInput`
    /// is what edits the query; nothing here appends a character, and the
    /// per-character `search_char` this replaced is gone rather than shimmed.
    pub(crate) fn maybe_compendium_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.compendium_selftest || self.compendium_fired {
            return;
        }
        if !self
            .started
            .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
        {
            return;
        }
        self.compendium_fired = true;

        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| ui.open_compendium());
        }
        // Opening the index requests the caret, so the filter is focused before
        // a single letter is sent. Printed first: without it the row counts
        // below would be a receipt for keys that went nowhere.
        let seam = hooks::focused_text(&*ctx.runner).is_some();
        let lane = caret_lane(&*ctx.runner);
        let (before, _) = compendium_index(&*ctx.runner);

        {
            let runner = &mut *ctx.runner;
            type_text(runner, "gob");
        }

        let (rows, first) = compendium_index(&*ctx.runner);
        let query = ctx.runner.state().compendium_search.text().to_owned();
        eprintln!(
            "[isometry] compendium selftest: caret in {lane:?} (host seam sees it: {seam}) | \
             typed {query:?} | index {before} -> {rows} rows | first row {first:?}"
        );
    }

    /// `ISOMETRY_WHISPER_SELFTEST=1`: open the whisper composer with `w`, type
    /// a draft into it, and leave it open — unsent, with the caret in it — for
    /// the capture.
    ///
    /// The composer is the other M3 lane, and the same gap applies: the harness
    /// proves the keys land in the draft, and only a headed frame can show that
    /// the field is legible with the caret sitting at the end of what was
    /// typed. Enter is deliberately never pressed: sending would close the lane
    /// and the capture would be of an empty panel row.
    pub(crate) fn maybe_whisper_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.whisper_selftest || self.whisper_fired {
            return;
        }
        if !self
            .started
            .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
        {
            return;
        }
        self.whisper_fired = true;

        const DRAFT: &str = "meet me at the gate";
        {
            let runner = &mut *ctx.runner;
            // `w` is the verb that opens the lane, and it goes through the same
            // intercept a real press does — so the receipt covers the open as
            // well as the typing.
            deliver_key(runner, &KeyPress::new(Key::Character("w".to_owned())));
        }
        let opened = ctx.runner.state().composing;
        let seam = hooks::focused_text(&*ctx.runner).is_some();
        let lane = caret_lane(&*ctx.runner);

        {
            let runner = &mut *ctx.runner;
            type_text(runner, DRAFT);
        }

        // The composer sits near the foot of the side panel, so this is also the
        // headed receipt for Z5's fit: the layout height is what decides whether
        // it is on screen at all, and it is `surface / zoom` rather than the
        // window's own height. `AppCtx` carries no laid-out geometry, so the
        // painted bottom edge of the panel's last elements is measured in the
        // harness instead (`host_zoom.rs`) at this same surface — the same
        // layout, since layout runs in CSS pixels and the device scale is not
        // in it.
        let (logical_w, logical_h) = ctx.logical_size;
        let zoom = ctx.ui_zoom;
        let runner = &*ctx.runner;
        let ui = runner.state();
        eprintln!(
            "[isometry] whisper selftest: layout {logical_w:.1}x{logical_h:.1} at zoom \
             {zoom:.4} (device scale {:.2}, board scale {:.5}, pixel grid {}) | \
             `w` opened the lane: {opened} | caret in {lane:?} \
             (host seam sees it: {seam}) at byte {} of {} | draft {:?} | to {:?} | \
             unsent (messages {}, outbox {}) | status: {}",
            ui.pixel_grid.0,
            ui.board_scale,
            if ui.integer_pixel_rounding { "on" } else { "off" },
            ui.whisper_draft.caret(),
            ui.whisper_draft.text().len(),
            ui.whisper_draft.text(),
            ui.whisper_target,
            ui.messages.len(),
            ui.whisper_outbox.len(),
            ui.status,
        );
    }
}
