//! The campaign-world lanes: storylets and the overmap.
//!
//! `ISOMETRY_STORYLET_SELFTEST` and `ISOMETRY_OVERMAP_SELFTEST`, the latter
//! carrying `ISOMETRY_OVERMAP_SOURCE_TIME_SELFTEST` as its historical
//! variant. Both seed a world, open the surface, and leave it open for the
//! capture.
//!
//! Split out of `selftest.rs` on 2026-09-04; unchanged.

use super::*;

impl App {
    /// `ISOMETRY_STORYLET_SELFTEST=1`: seed a ready storylet and a locked one,
    /// open the surface, play the ready one, and confirm its fact committed.
    pub(crate) fn maybe_storylet_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.storylet_selftest || self.storylet_fired {
            return;
        }
        if !self
            .started
            .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
        {
            return;
        }
        self.storylet_fired = true;

        use isometry_campaign::{
            StoryletEffect, StoryletProposal, StoryletRequirements, WorldFact,
        };
        // A ready storylet (no requirements, no roles) and a locked one (needs a
        // faction that does not exist).
        let ready = StoryletProposal {
            key: "gate-greeting".to_owned(),
            entry: "A stranger greets you at the gate.".to_owned(),
            tags: Vec::new(),
            requirements: StoryletRequirements::default(),
            roles: Vec::new(),
            effects: vec![StoryletEffect::Fact {
                fact: WorldFact {
                    id: "gate-met".to_owned(),
                    kind: "event".to_owned(),
                    text: "The party met a stranger at the gate.".to_owned(),
                    tags: Vec::new(),
                },
            }],
        };
        let locked = StoryletProposal {
            key: "cult-rises".to_owned(),
            entry: "The eel cult stirs in the deep.".to_owned(),
            tags: Vec::new(),
            requirements: StoryletRequirements {
                faction_tags: vec!["cult".to_owned()],
                ..Default::default()
            },
            roles: Vec::new(),
            effects: Vec::new(),
        };
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                ui.world
                    .storylets
                    .insert("gate-greeting".to_owned(), ready.clone());
                ui.world
                    .storylets
                    .insert("cult-rises".to_owned(), locked.clone());
                ui.open_storylets();
            });
        }
        // Compute the rows.
        self.pump_storylets(ctx);
        {
            let runner = &*ctx.runner;
            for row in &runner.state().storylets {
                eprintln!(
                    "[isometry] storylet selftest: {} available={} status={:?} entry={:?}",
                    row.key, row.available, row.status, row.entry
                );
            }
        }
        // Select the ready one and play it.
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                let idx = ui
                    .storylets
                    .iter()
                    .position(|r| r.key == "gate-greeting")
                    .unwrap_or(0);
                ui.storylet_selected = idx;
                ui.play_storylet();
            });
        }
        self.pump_storylets(ctx);
        let committed = self.journal.iter().any(|f| f.id == "gate-met");
        let status = Some(&*ctx.runner)
            .map(|r| r.state().status.clone())
            .unwrap_or_default();
        eprintln!(
            "[isometry] storylet selftest: played | journal has 'gate-met': {committed} | status: {status}"
        );
    }

    /// `ISOMETRY_OVERMAP_SELFTEST=1`: seed a small overmap (four places joined by
    /// roads), stand the party at the village, reveal the map it would know, and
    /// open the overmap surface. A focus-free proof the C8 render draws.
    pub(crate) fn maybe_overmap_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.overmap_selftest || self.overmap_fired {
            return;
        }
        if !self
            .started
            .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
        {
            return;
        }
        self.overmap_fired = true;

        use isometry_campaign::{ItemId, ItemInstance, WorldPlace, WorldRoute};
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                // Positions left unset (`None`): the overmap relaxes a
                // force-directed layout from the routes, proving that path.
                let place = |id: &str, name: &str| WorldPlace {
                    id: id.to_owned(),
                    name: name.to_owned(),
                    tags: Vec::new(),
                    map: None,
                    position: None,
                };
                for (id, name) in [
                    ("village", "Village"),
                    ("forest", "Deepwood"),
                    ("ruins", "Old Ruins"),
                    ("keep", "Grey Keep"),
                    ("citadel", "Sky Citadel"),
                ] {
                    ui.world.places.insert(id.to_owned(), place(id, name));
                }
                let route = |id: &str, from: &str, to: &str, weight: u32| WorldRoute {
                    id: id.to_owned(),
                    from: from.to_owned(),
                    to: to.to_owned(),
                    tags: Vec::new(),
                    weight,
                };
                ui.world
                    .routes
                    .insert("r1".to_owned(), route("r1", "village", "forest", 2));
                ui.world
                    .routes
                    .insert("r2".to_owned(), route("r2", "forest", "ruins", 3));
                ui.world
                    .routes
                    .insert("r3".to_owned(), route("r3", "village", "keep", 5));
                ui.world
                    .routes
                    .insert("r4".to_owned(), route("r4", "keep", "citadel", 4));

                let party = ui.viewer.clone().unwrap_or_else(|| "dm".to_owned());
                ui.world
                    .party_node
                    .insert(party.clone(), "village".to_owned());
                // The party knows only its own ground and the near woods. The keep
                // is one route past the known (a frontier the "study map" read
                // finds); the Sky Citadel is two routes out, unreachable by that
                // read -- it is only learned from a *carried* map (below).
                for node in ["village", "forest", "ruins"] {
                    ui.world.reveal(&party, node);
                }
                // A party token to carry the pack: knight 1, deeded to the party,
                // holding a looted chart tagged with the place it depicts.
                if let Some(token) = ui.map.tokens.iter_mut().find(|t| t.id == TokenId(1)) {
                    token.owner = Some(party.clone());
                }
                ui.inventories.entry(TokenId(1)).or_default().items.insert(
                    ItemId::new("citadel-chart"),
                    ItemInstance {
                        id: ItemId::new("citadel-chart"),
                        template: "map".to_owned(),
                        name: "Chart to the Sky Citadel".to_owned(),
                        quantity: 1,
                        tags: vec!["map".to_owned(), "reveals:citadel".to_owned()],
                        modifiers: Vec::new(),
                        appearance_layers: Vec::new(),
                    },
                );
                ui.open_overmap();
                // Study the map at once so the capture shows the outcome: the
                // frontier read finds the keep, and the carried chart discloses
                // the citadel two routes out that no frontier read could reach.
                ui.request_map_read();
            });
        }
        // The `request_map_read` above only arms a read; its pump normally runs
        // on a window event, of which a headless selftest has none. Drive it once
        // here so the seeded chart resolves and the capture shows the outcome.
        self.pump_overmap_read(ctx);
        if self.overmap_source_time_selftest {
            // The C8 fixture is deliberately assembled directly so it can
            // exercise discovery and a carried map without a content pack. For
            // this receipt, make the finished public state the explicit origin,
            // then append one real authority event through the normal host path.
            // Selecting its empty prefix is consequently a truthful source
            // projection, never a reconstruction from current state.
            let origin = Some(&*ctx.runner).map(|runner| self.snapshot_of(runner.state()));
            if let Some(origin) = origin {
                self.history = Journal::new();
                self.history_origin = Some(origin);
                self.source_history_len = None;
                self.source_history_attached = false;
                self.emit_host_event(
                    ctx,
                    GameEvent::Fact(WorldFact {
                        id: "overmap-source-time-receipt".to_owned(),
                        kind: "history".to_owned(),
                        text: "The survey was filed after the route was drawn.".to_owned(),
                        tags: vec!["receipt".to_owned()],
                    }),
                );
                {
                    let runner = &mut *ctx.runner;
                    runner.update(|ui| {
                        ui.overmap_source_slider.value = 0.0;
                        ui.sync_overmap_source_time();
                        ui.status = "source-time receipt: historical prefix selected".to_owned();
                    });
                }
                eprintln!(
                    "[isometry] overmap source-time selftest: selected event 0 of 1; live authority retained"
                );
            }
        }
        eprintln!(
            "[isometry] overmap selftest: seeded 5 places, party at the village knowing 3, \
             carrying a chart to the citadel; reading the map reveals keep + citadel"
        );
    }
}
