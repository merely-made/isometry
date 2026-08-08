//! Play: spawning, actions, time, travel, and beats.
//!
//! Everything the table does to the board once the map exists. The
//! keystroke-captured text lanes (`>` command line, whisper composer)
//! live in `lanes.rs`.
//!
//! Split out of `state.rs` on 2026-07-24, text lanes out on 2026-08-08;
//! behavior unchanged.

use super::*;

impl UiState {
    /// Spawn a bestiary monster as a token on the board (a Local editor
    /// action, reusing the token-placement path). Places on the selected tile
    /// if free, else the nearest free tile, then closes the compendium so the
    /// new token is visible.
    pub fn spawn_monster(&mut self, key: &str) {
        let Some(m) = self.bestiary.iter().find(|m| m.key == key) else {
            return;
        };
        let (sprite, name, key) = (m.sprite.clone(), m.name.clone(), m.key.clone());
        let at = self.free_spawn_tile();
        let id = self.next_token_id();
        let placed = SessionEvent::TokenPlaced(Token {
            id,
            at,
            facing: Facing::South,
            sprite,
            owner: None,
        });
        // Route through the authority like every other mutator, or the token
        // shows only on the DM's screen and is wiped by the next snapshot
        // mirror (orphaning the replicated sheet). Apply locally only when solo.
        if !self.net_emit(GameEvent::Map(placed.clone())) {
            self.apply_step(vec![placed]);
        }
        // Ask the host to bind the stat block: what fields a 5e creature has
        // is the system's business, not the board's.
        self.spawn_sheet_request = Some((id, key));
        self.status = format!("spawned {name}");
        self.compendium_open = false;
        self.compendium_selected = None;
    }

    /// A free tile to spawn onto: the selection if empty, else scanning a small
    /// block outward from it. Bounds-checked, because placing a token off-map is
    /// rejected (and on a narrow map the block can walk off the edge).
    pub(crate) fn free_spawn_tile(&self) -> TileCoord {
        let free =
            |at: TileCoord| self.map.ground.in_bounds(at.0, at.1) && self.token_at(at).is_none();
        let start = self.selected.filter(|&s| free(s)).unwrap_or((2, 2));
        for d in 0..64 {
            let at = (start.0 + (d % 8), start.1 + (d / 8));
            if free(at) {
                return at;
            }
        }
        let (w, h) = (
            self.map.ground.width() as i32,
            self.map.ground.height() as i32,
        );
        for row in 0..h {
            for col in 0..w {
                if free((col, row)) {
                    return (col, row);
                }
            }
        }
        (0, 0)
    }

    /// Queue a field edit (a stepper on the open sheet); the host applies
    /// and replicates it.
    pub fn request_sheet_edit(&mut self, key: &str, delta: i64) {
        if let Some(id) = self.open_sheet {
            self.sheet_edit = Some((id, key.to_owned(), delta));
        }
    }

    /// Queue an action roll; the host evaluates it against the system.
    /// Click an action on the open sheet.
    ///
    /// An untargeted action (an ability check) rolls immediately, as it always
    /// has. A targeted one cannot: it needs a victim, so it arms target-pick
    /// mode and the next click on a token becomes the intent.
    pub fn request_action(&mut self, key: &str) {
        let Some(id) = self.open_sheet else {
            return;
        };
        let targeted = self
            .sheet_schema
            .actions
            .iter()
            .any(|(k, _, targeted)| k == key && *targeted);
        if targeted {
            let label = self
                .sheet_schema
                .actions
                .iter()
                .find(|(k, _, _)| k == key)
                .map(|(_, l, _)| l.clone())
                .unwrap_or_else(|| key.to_owned());
            self.action_pick = Some((id, key.to_owned()));
            self.status = format!("{label}: pick a target (Esc to cancel)");
        } else {
            self.sheet_action = Some((id, key.to_owned()));
        }
    }

    /// Whether the board is waiting for the player to click a victim.
    pub fn picking_target(&self) -> bool {
        self.action_pick.is_some()
    }

    /// Cancel target-pick without spending anything.
    pub fn cancel_action_pick(&mut self) {
        if self.action_pick.take().is_some() {
            self.status = "action cancelled".to_owned();
        }
    }

    /// Commit the victim. This only *asks*: the host validates reach and turn
    /// ownership and the system decides the outcome. Nothing is resolved here.
    pub fn pick_action_target(&mut self, target: TokenId) {
        let Some((actor, key)) = self.action_pick.take() else {
            return;
        };
        if actor == target {
            self.status = "cannot target yourself".to_owned();
            return;
        }
        if self.map.is_defeated(target) {
            self.status = "that one is already down".to_owned();
            return;
        }
        self.action_intent = Some((actor, target, key));
    }

    /// The DM declares time passing on the active location: the downtime verb
    /// beside the automatic round tick. In a session it routes to the host;
    /// solo it applies directly. No stored map means no clock to keep.
    pub fn pass_time(&mut self, ticks: u64) {
        let Some(active) = self.active_map.clone() else {
            self.status = "no campaign clock without a stored map".to_owned();
            return;
        };
        if self.net_emit(GameEvent::TimeAdvanced { ticks }) {
            return;
        }
        *self.clocks.entry(active.clone()).or_insert(0) += ticks;
        self.status = format!("time passes: {} on {active}", self.clock_now());
    }

    /// The active location's clock, in ticks.
    pub fn clock_now(&self) -> u64 {
        self.active_map
            .as_ref()
            .and_then(|id| self.clocks.get(id))
            .copied()
            .unwrap_or(0)
    }

    /// The transition point on the active map at `at`, if any: the door the
    /// board renders and a token walks through.
    pub fn transition_at(&self, at: TileCoord) -> bool {
        let Some(active) = &self.active_map else {
            return false;
        };
        self.campaign_maps
            .get(active)
            .map(|m| {
                m.transitions
                    .iter()
                    .any(|t| (t.at.col as i32, t.at.row as i32) == at)
            })
            .unwrap_or(false)
    }

    /// Every door tile on the active map, for the board to render.
    pub fn door_tiles(&self) -> HashSet<TileCoord> {
        let Some(active) = &self.active_map else {
            return HashSet::new();
        };
        self.campaign_maps
            .get(active)
            .map(|m| {
                m.transitions
                    .iter()
                    .map(|t| (t.at.col as i32, t.at.row as i32))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Walk `token` through the door it stands on (solo / hot-seat path).
    ///
    /// Deliberately *not* a reimplementation: it rules the crossing with the
    /// shared resolver, applies the verdict through the same `apply_game` every
    /// networked peer runs, and copies the outcome back. Solo play *is* the
    /// authority, so this is the host's door sweep, minus the network.
    pub fn travel(&mut self, token: TokenId) {
        // Refresh the stored copy of the active map: a stale board would rule
        // the departure against tokens that have since moved.
        if let Some(id) = &self.active_map {
            if let Some(m) = self.campaign_maps.get_mut(id) {
                m.document = self.map.clone();
            }
        }
        let mut snap = GameSnapshot {
            map: self.map.clone(),
            turns: self.turns.clone(),
            roll_log: Vec::new(),
            journal: Vec::new(),
            inventories: self.inventories.clone(),
            generations: Vec::new(),
            maps: self.campaign_maps.clone(),
            active_map: self.active_map.clone(),
            world: Default::default(),
            // The clocks must cross too, or reconciliation runs against empty
            // time and wipes the ledger on copy-back.
            clocks: self.clocks.clone(),
            party_cap: self.party_cap,
            last_beats: Vec::new(),
            beat_seq: 0,
            applied_actions: Default::default(),
        };
        // One crossing, one ruling. The nonce is this state's own, against the
        // reserved host identity: there is no connection to attribute it to.
        self.travel_requests += 1;
        let request = isometry_net::RequestId::host(self.travel_requests);
        let ruled = isometry_net::resolve_transition(&snap, token, request)
            .and_then(|res| apply_game(&mut snap, &GameEvent::TransitionResolved(res)));
        match ruled {
            Ok(()) => {
                let switched = snap.active_map != self.active_map;
                self.map = snap.map;
                self.turns = snap.turns;
                self.inventories = snap.inventories;
                self.campaign_maps = snap.maps;
                self.clocks = snap.clocks;
                self.party_cap = snap.party_cap;
                self.active_map = snap.active_map;
                if switched {
                    self.selected_token = None;
                    self.selected = None;
                    self.reach.clear();
                    self.explored.clear();
                    self.status = format!(
                        "the party moves on: {}",
                        self.active_map.as_deref().unwrap_or("?")
                    );
                } else {
                    self.status = "through the door".to_owned();
                }
                self.recompute_fog();
            }
            Err(error) => self.status = format!("cannot travel: {error:?}"),
        }
    }

    /// Play a beat on a token for its own sake: a cheer, a shrug, a taunt.
    ///
    /// Reuses the beat the combat lane already defined, so it costs no new
    /// replication and no rules. Unlike an action it needs no adjudication,
    /// which is why a player may throw one on their own token without asking
    /// the host.
    pub fn emote(&mut self, token: TokenId, beat: &str) {
        if self.map.token(token).is_none() || self.map.is_defeated(token) {
            return;
        }
        self.close_context_menu();
        if self.net_emit(GameEvent::Emoted {
            token,
            beat: beat.to_owned(),
        }) {
            return;
        }
        let seq = self.beat_seq.wrapping_add(1);
        self.stage_beats(seq, &[isometry_core::Beat::new(token, beat)]);
    }

    /// Stage the beats of a freshly applied action, so the board plays the
    /// exchange. Idempotent per `seq`: a re-delivered snapshot does not replay.
    pub fn stage_beats(&mut self, seq: u64, beats: &[isometry_core::Beat]) {
        if seq == self.beat_seq {
            return;
        }
        self.beat_seq = seq;
        self.beats.clear();
        for beat in beats {
            self.beats.insert(beat.token, beat.name.clone());
        }
    }

    /// Drop every playing beat. The host calls this once the engine's animation
    /// clock reports nothing is animating, which is what lets the *next* strike
    /// restart the animation instead of finding the class already set.
    pub fn clear_beats(&mut self) {
        self.beats.clear();
    }

    /// Append a whisper to the message log, dropping the oldest past the cap.
    /// Unlike the roll log this is not replicated (the net side drains its
    /// whispers), so the cap is local view state rather than a protocol
    /// constant.
    pub fn push_message(&mut self, message: String) {
        self.messages.push(message);
        let overflow = self.messages.len().saturating_sub(MESSAGES_CAP);
        if overflow > 0 {
            self.messages.drain(0..overflow);
        }
    }

    /// Append to the local shared log, dropping the oldest past the cap. In
    /// session the authority's log arrives with the snapshot instead.
    pub fn push_roll(&mut self, record: RollRecord) {
        self.roll_log.push(record);
        let overflow = self.roll_log.len().saturating_sub(ROLL_LOG_CAP);
        if overflow > 0 {
            self.roll_log.drain(0..overflow);
        }
    }

    /// Roll `expr`, logging it under `by` with `label` (the shared-log
    /// path the host uses for a system action; solo appends locally).
    pub fn roll_labeled(&mut self, by: &str, label: &str, expr: &str) {
        let Some((total, dice)) = roll(expr, &mut self.rng) else {
            self.status = format!("bad roll: {expr}");
            return;
        };
        let record = RollRecord {
            by: by.to_owned(),
            expr: label.to_owned(),
            dice,
            total,
        };
        self.status = format!("{by} {label} = {total}");
        if self.net_mode == NetMode::Remote {
            self.net_outbox.push(GameEvent::Rolled(record));
        } else {
            self.push_roll(record);
        }
    }
}
