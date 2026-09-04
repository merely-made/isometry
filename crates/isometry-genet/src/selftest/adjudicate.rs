//! The adjudication lanes: a recruit, and a swing.
//!
//! `ISOMETRY_CONVINCE_SELFTEST` and `ISOMETRY_COMBAT_SELFTEST`. Both are
//! focus-free, for the reason the combat lane's own doc comment gives.
//!
//! Split out of `selftest.rs` on 2026-09-04. The two doc comments below came
//! with it: each had drifted one lane up the file, onto a neighbour it did
//! not describe.

use super::*;

impl App {
    /// `ISOMETRY_CONVINCE_SELFTEST=1`: a bard recruits a goblin, then meets the
    /// party cap on the next. Focus-free.
    pub(crate) fn maybe_convince_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.convince_selftest || self.convince_fired {
            return;
        }
        if !self
            .started
            .is_some_and(|t| t.elapsed() > Duration::from_secs(2))
        {
            return;
        }
        self.convince_fired = true;

        let Some(system) = self.system.as_mut() else {
            return;
        };
        // A silver-tongued bard: CHA 18 (+4) and proficiency, so the pitch is
        // 1d20+6 against a goblin's low resolve.
        let mut bard = system.default_sheet();
        bard.set_text("name", "Bard");
        bard.set_int("cha", 18);
        bard.set_int("prof", 2);
        let goblin = |will: i64| {
            let mut s = srd_bestiary()
                .iter()
                .find(|m| m.key == "goblin")
                .map(monster_sheet)
                .unwrap_or_else(|| system.default_sheet());
            s.set_int("will", will);
            s
        };
        let (g1, g2) = (goblin(4), goblin(4));

        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                // Knight 1 is player A's; make it the bard. Goblins 2 and 4 are
                // the DM's furniture (owner None) standing in talking range.
                ui.map.set_sheet(TokenId(1), bard.clone());
                ui.map.set_sheet(TokenId(2), g1.clone());
                ui.map.set_sheet(TokenId(4), g2.clone());
                let anchor = ui.map.token(TokenId(1)).map(|t| t.at).unwrap_or((10, 14));
                for (id, dx) in [(TokenId(2), 2), (TokenId(4), 3)] {
                    if let Some(g) = ui.map.tokens.iter_mut().find(|t| t.id == id) {
                        g.at = (anchor.0 + dx, anchor.1);
                        g.owner = None; // DM furniture, up for grabs
                    }
                }
                // A owns knight 1 and knight 3 on this board, plus one companion
                // stashed on a *stored* map (a split party, C3). The cap counts
                // the whole campaign, so that third token matters: with cap 4, A
                // can take exactly one goblin before the party fills.
                let mut away = isometry_core::MapDocument::new("waystation", 6, 6);
                away.tokens.push(isometry_core::Token {
                    id: TokenId(50),
                    at: (2, 2),
                    facing: isometry_core::Facing::South,
                    sprite: "knight".to_owned(),
                    owner: Some("A".to_owned()),
                });
                ui.campaign_maps.insert(
                    "waystation".to_owned(),
                    isometry_campaign::CampaignMap {
                        id: "waystation".to_owned(),
                        scale: isometry_campaign::MapScale::Local,
                        document: away,
                        spawn_zones: Vec::new(),
                        transitions: Vec::new(),
                        encounter_anchors: Vec::new(),
                    },
                );
                ui.party_cap = 4;
                ui.viewer = Some("A".to_owned());
                ui.recompute_fog();
                let a_active = ui.map.tokens.iter().filter(|t| t.owner.as_deref() == Some("A")).count();
                eprintln!(
                    "[isometry] convince selftest: A owns {a_active} here + 1 stored = 3 global, cap {}",
                    ui.party_cap
                );
            });
        }

        // First pitch: goblin 2 joins A (A goes 2 -> 3 tokens, at the cap).
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                ui.action_intent = Some((TokenId(1), TokenId(2), "convince".to_owned()))
            });
        }
        self.pump_sheets(ctx);
        // Second pitch: goblin 4 would make 4 > cap 3, so it fails to hold.
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                ui.action_intent = Some((TokenId(1), TokenId(4), "convince".to_owned()))
            });
        }
        self.pump_sheets(ctx);

        {
            let runner = &*ctx.runner;
            let ui = runner.state();
            let owner = |id| ui.map.token(id).and_then(|t| t.owner.clone());
            let a_here = ui
                .map
                .tokens
                .iter()
                .filter(|t| t.owner.as_deref() == Some("A"))
                .count();
            let a_global = a_here
                + ui.campaign_maps
                    .values()
                    .flat_map(|m| m.document.tokens.iter())
                    .filter(|t| t.owner.as_deref() == Some("A"))
                    .count();
            eprintln!(
                "[isometry] convince selftest: goblin2 owner {:?} | goblin4 owner {:?} | A owns {a_here} here, {a_global} global (cap 4) | status: {}",
                owner(TokenId(2)),
                owner(TokenId(4)),
                ui.status,
            );
        }
    }

    /// `ISOMETRY_COMBAT_SELFTEST=1`: a focus-free proof of the adjudication
    /// loop. It stats both duelists, stands the goblin in reach, and swings.
    ///
    /// The app drives itself rather than being driven by synthetic clicks,
    /// because SendKeys loses the foreground race on a machine someone is
    /// actually using and silently types into their editor. Same rationale as
    /// `ISOMETRY_NET_SELFTEST`.
    pub(crate) fn maybe_combat_selftest(&mut self, ctx: &mut Ctx<'_>) {
        if !self.combat_selftest || (self.combat_swings == 0 && self.combat_emoted) {
            return;
        }
        // Wait 2s for the first swing, then one per second: long enough for a
        // 420ms beat to finish and be cleared, so the *next* swing has to
        // genuinely restart the animation rather than find its class still set.
        let due = match self.last_swing {
            None => self
                .started
                .is_some_and(|t| t.elapsed() > Duration::from_secs(2)),
            Some(last) => last.elapsed() > Duration::from_millis(1000),
        };
        if !due {
            return;
        }
        let first = self.last_swing.is_none();
        self.last_swing = Some(Instant::now());

        // The swings are spent: the winner celebrates. An emote is the same beat
        // primitive, with no resolution behind it and nothing to adjudicate.
        if self.combat_swings == 0 {
            self.combat_emoted = true;
            {
                let runner = &mut *ctx.runner;
                runner.update(|ui| ui.emote(TokenId(1), "cheer"));
                eprintln!(
                    "[isometry] combat selftest: emote | beats = {:?}",
                    runner.state().beats
                );
            }
            return;
        }
        self.combat_swings -= 1;
        let swings_left = self.combat_swings;

        let Some(system) = self.system.as_mut() else {
            return;
        };
        let mut knight = system.default_sheet();
        knight.set_text("name", "Knight");
        knight.set_int("str", 18); // +4
        knight.set_int("prof", 3); // so the swing is 1d20+7 against AC 15
        let Some(goblin) = srd_bestiary()
            .iter()
            .find(|m| m.key == "goblin")
            .map(monster_sheet)
        else {
            eprintln!("[isometry] combat selftest: no goblin in the bestiary");
            return;
        };
        let runner = &mut *ctx.runner;
        runner.update(|ui| {
            if first {
                // Stand the goblin within reach of the knight, and stat them
                // both. After that the board carries its own state: each swing
                // hits whatever hit points the last one left behind.
                if let Some(at) = ui.map.token(TokenId(1)).map(|t| t.at) {
                    if let Some(g) = ui.map.tokens.iter_mut().find(|t| t.id == TokenId(2)) {
                        g.at = (at.0 + 1, at.1);
                    }
                }
                ui.map.set_sheet(TokenId(1), knight.clone());
                ui.map.set_sheet(TokenId(2), goblin.clone());
                ui.open_sheet = Some(TokenId(1));
                ui.recompute_fog();
                eprintln!(
                    "[isometry] combat selftest: knight@{:?} vs goblin@{:?} | goblin hp {:?}, ac {:?} (1d20+7 to hit)",
                    ui.map.token(TokenId(1)).map(|t| t.at),
                    ui.map.token(TokenId(2)).map(|t| t.at),
                    ui.map.sheet(TokenId(2)).and_then(|s| s.int("hp_current")),
                    ui.map.sheet(TokenId(2)).and_then(|s| s.int("ac")),
                );
            }
            // Trip first (a condition: prone halves speed, truth on every
            // peer), then attacks. The prone goblin keeps its tile, unlike the
            // shove run: a condition changes what it can DO, not where it is.
            // The fixed tape rolls 11 then 22: the first swing misses whatever
            // it is, so the trip goes second, where it connects.
            let action = if swings_left == 2 { "trip" } else { "attack" };
            ui.action_intent = Some((TokenId(1), TokenId(2), action.to_owned()));
        });
        self.pump_sheets(ctx);
        {
            let runner = &*ctx.runner;
            let ui = runner.state();
            eprintln!(
                "[isometry] combat selftest: {} | goblin hp {:?} conds {:?} mobility {:?} | beats = {:?}",
                ui.status,
                ui.map.sheet(TokenId(2)).and_then(|s| s.int("hp_current")),
                ui.map.conditions.get(&TokenId(2)),
                ui.map.effective_mobility(TokenId(2), (5, 6)),
                ui.beats,
            );
        }
    }
}
