//! Opt-in desktop receipt for the bundled inhabited campaign.

use super::*;

impl App {
    pub(crate) fn maybe_watchtower_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.watchtower_selftest || self.watchtower_fired {
            return;
        }
        if !self
            .started
            .is_some_and(|start| start.elapsed() > Duration::from_secs(2))
        {
            return;
        }
        self.watchtower_fired = true;
        // This receipt is solo; network convergence has its own session tests.
        assert!(
            self.net.is_none(),
            "run watchtower receipt without --host/--join"
        );
        ctx.runner.update(|ui| ui.start_generator("watchtower"));
        self.pump_generators(ctx);
        let preview = ctx
            .runner
            .state()
            .generator_preview
            .as_ref()
            .expect("watchtower generator must produce a preview");
        let isometry_campaign::GenValue::Campaign { campaign } = &preview.proposal else {
            panic!("watchtower preview must be a campaign");
        };
        let expected = campaign
            .maps
            .iter()
            .find(|map| map.map.id == campaign.starting_map)
            .expect("starting map")
            .inhabitants
            .len();
        ctx.runner.update(|ui| ui.commit_generation_preview());
        self.pump_generators(ctx);
        let ui = ctx.runner.state();
        assert_eq!(ui.map.tokens.len(), expected);
        assert!(
            ui.map
                .tokens
                .iter()
                .any(|token| token.sprite == "tower-beast")
        );
        assert!(
            ui.map
                .tokens
                .iter()
                .all(|token| ui.map.sheet(token.id).is_some())
        );
        eprintln!(
            "[isometry] watchtower receipt: {} inhabitants committed on {} with sheets; status={}",
            expected, ui.map.name, ui.status
        );
        ctx.runner.update(|ui| {
            ui.camera = (ui.viewport.0 / 2.0, 110.0);
            ui.close_generator();
            match std::env::var("ISOMETRY_WATCHTOWER_VIEW").as_deref() {
                Ok("region") => {
                    ui.drag_move_token(TokenId(1), (1, 7));
                    ui.travel(TokenId(1));
                    assert_eq!(ui.active_map.as_deref(), Some("watchtower:forest-region"));
                    ui.camera = (ui.viewport.0 / 2.0, 160.0);
                },
                Ok("character") => {
                    ui.open_character();
                    ui.character_name = cambium::TextInput::new("Rowan");
                    ui.character_owner = cambium::TextInput::new("player");
                },
                _ => {},
            }
        });
    }
}
