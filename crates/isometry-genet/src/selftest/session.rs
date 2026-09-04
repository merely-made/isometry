//! The session lanes: one end-turn over the net, and a door crossing.
//!
//! `ISOMETRY_NET_SELFTEST` and `ISOMETRY_TRAVEL_SELFTEST`. Both drive the
//! board through the shipping path — the session round trip, and the
//! Play-mode click that walks onto a door — rather than by poking state.
//!
//! Split out of `selftest.rs` on 2026-09-04; unchanged.

use super::*;

impl App {
    /// The env-gated self-test: after a warm-up, emit one end-turn as if
    /// the user pressed it, exercising the full session round-trip.
    pub(crate) fn maybe_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.net_selftest || self.selftest_fired {
            return;
        }
        let ready = self
            .started
            .map(|t| t.elapsed() > Duration::from_secs(3))
            .unwrap_or(false);
        if ready {
            self.selftest_fired = true;
            eprintln!("[isometry] selftest: firing end_turn");
            {
                let runner = &mut *ctx.runner;
                runner.update(|ui| ui.end_turn());
            }
            self.pump_net(ctx);
        }
    }

    /// `ISOMETRY_TRAVEL_SELFTEST=1`: prove C2 end to end in the app. The demo
    /// board becomes the stored map `field` with a door; a `hut` map waits on
    /// the other side; the knight (the whole party: everyone else is demoted to
    /// DM furniture) walks onto the door through the normal Play-mode click
    /// path, and the board follows it through.
    pub(crate) fn maybe_travel_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.travel_selftest || self.travel_fired {
            return;
        }
        let ready = self
            .started
            .is_some_and(|t| t.elapsed() > Duration::from_secs(2));
        if !ready {
            return;
        }
        self.travel_fired = true;
        let runner = &mut *ctx.runner;
        runner.update(|ui| {
            // The party is the knight alone; the rest is the DM's furniture.
            for t in ui.map.tokens.iter_mut() {
                if t.id != TokenId(1) {
                    t.owner = None;
                }
            }
            // The field: the live board, stored, with a door beside the knight.
            let field = isometry_campaign::CampaignMap {
                id: "field".to_owned(),
                scale: isometry_campaign::MapScale::Local,
                document: ui.map.clone(),
                spawn_zones: Vec::new(),
                transitions: vec![isometry_campaign::MapTransition {
                    id: "field-gate".to_owned(),
                    at: isometry_campaign::MapPoint { col: 12, row: 14 },
                    target_map: "hut".to_owned(),
                    target_entry: Some("hut-door".to_owned()),
                }],
                encounter_anchors: Vec::new(),
            };
            // The hut: a small stone room with one resident.
            let mut hut_doc = isometry_core::MapDocument::new("hut", 10, 10);
            let floor = hut_doc.intern_tile_kind("stone");
            for r in 0..10 {
                for c in 0..10 {
                    hut_doc.ground.set(c, r, floor);
                }
            }
            hut_doc.tokens.push(isometry_core::Token {
                id: TokenId(7),
                at: (6, 6),
                facing: isometry_core::Facing::South,
                sprite: "goblin".to_owned(),
                owner: None,
            });
            let hut = isometry_campaign::CampaignMap {
                id: "hut".to_owned(),
                scale: isometry_campaign::MapScale::Local,
                document: hut_doc,
                spawn_zones: Vec::new(),
                transitions: vec![isometry_campaign::MapTransition {
                    id: "hut-door".to_owned(),
                    at: isometry_campaign::MapPoint { col: 2, row: 2 },
                    target_map: "field".to_owned(),
                    target_entry: Some("field-gate".to_owned()),
                }],
                encounter_anchors: Vec::new(),
            };
            ui.campaign_maps.insert("field".to_owned(), field);
            ui.campaign_maps.insert("hut".to_owned(), hut);
            ui.active_map = Some("field".to_owned());
            // Time passes in the field before anyone crosses: the DM declares
            // a rest, so the two locations' clocks drift apart.
            ui.pass_time(4);
            eprintln!(
                "[isometry] travel selftest: on {:?}, knight@{:?}, door at (12, 14) | clocks {:?}",
                ui.active_map,
                ui.map.token(TokenId(1)).map(|t| t.at),
                ui.clocks,
            );
            // Walk through the door via the normal Play-mode click path.
            ui.mode = EditMode::Play;
            ui.select_token(TokenId(1));
            ui.click_tile((12, 14));
            eprintln!(
                "[isometry] travel selftest: {} | active {:?} board '{}' | knight here: {:?} | field still holds knight: {:?} | clocks {:?}",
                ui.status,
                ui.active_map,
                ui.map.name,
                ui.map.tokens.iter().find(|t| t.sprite == "knight").map(|t| t.at),
                ui.campaign_maps["field"].document.token(TokenId(1)).is_some(),
                ui.clocks,
            );
        });
    }
}
