//! The worldbuilding generator lane: preview, reroll, commit.
//!
//! Generation is host-side and seeded from an entropy tape, so a fixed
//! `ISOMETRY_GEN_SEED` replays the same world. A preview is not yet world
//! state; committing is what turns it into replicated events.
//!
//! Split out of `main.rs` on 2026-07-24; behavior unchanged.

use super::*;

impl App {
    pub(crate) fn pump_generators(&mut self, ctx: &mut Ctx<'_>) {
        let mut action = None;
        let mut locks = Default::default();
        let mut can_edit = false;
        let mut remote = false;
        let mut existing_ids = Vec::new();
        let mut preview = None;
        let mut choice = None;
        let mut local_snapshot = None;
        let mut selection_request = None;
        let journal = self.journal.clone();
        {
            let runner = &mut *ctx.runner;
            runner.update(|ui| {
                action = ui.generation_request.take();
                selection_request = ui.generator_selection_request.take();
                locks = ui.generator_locks.clone();
                can_edit = ui.can_edit_inventory;
                remote = ui.net_mode == NetMode::Remote;
                choice = ui.selected_generator().cloned();
                existing_ids = ui
                    .generations
                    .iter()
                    .map(|record| record.id.clone())
                    .collect();
                if action == Some(GenerationRequest::Commit) {
                    preview = ui.generator_preview.take();
                    local_snapshot = Some(GameSnapshot {
                        map: ui.map.clone(),
                        turns: ui.turns.clone(),
                        roll_log: ui.roll_log.clone(),
                        journal: journal.clone(),
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
                    });
                }
            });
        }
        if let Some(request) = selection_request {
            if !can_edit {
                {
                    let runner = &mut *ctx.runner;
                    runner.update(|ui| ui.status = "generation requires the host".to_owned());
                }
                return;
            }
            let choices = self.generator_catalog.choices();
            match crate::cleromancy_selection::select_generator(&choices, &request) {
                Ok(selection) => {
                    let choice_index = selection.choice_index;
                    let choice_name = choices[choice_index].name.clone();
                    let receipt_digest = selection
                        .reading
                        .receipt
                        .derivation_digest
                        .clone()
                        .unwrap_or_else(|| "unavailable".to_owned());
                    self.last_generator_selection = Some(selection);
                    {
                        let runner = &mut *ctx.runner;
                        runner.update(|ui| {
                            ui.generator_selected = choice_index;
                            ui.generator_preview = None;
                            ui.generator_locks.clear();
                            ui.generator_open = true;
                            // This is intentionally the existing host action:
                            // Cleromancy chooses a declaration, never an output.
                            ui.generation_request = Some(GenerationRequest::Generate);
                            ui.status = format!(
                                "receipt chose {choice_name} ({receipt_digest}); generating preview"
                            );
                        });
                    }
                }
                Err(error) => {
                    let runner = &mut *ctx.runner;
                    runner.update(|ui| ui.status = format!("generator selection failed: {error}"));
                }
            }
            return;
        }
        let Some(action) = action else {
            return;
        };
        if !can_edit {
            {
                let runner = &mut *ctx.runner;
                runner.update(|ui| ui.status = "generation requires the host".to_owned());
            }
            return;
        }
        match action {
            GenerationRequest::Generate => {
                let Some(choice) = choice else {
                    {
                        let runner = &mut *ctx.runner;
                        runner.update(|ui| ui.status = "no generator selected".to_owned());
                    }
                    return;
                };
                let mut ordinal = self.generation_ordinal;
                let record_id = loop {
                    ordinal = ordinal.wrapping_add(1);
                    let generator_slug: String = choice
                        .id
                        .chars()
                        .map(|c| if c.is_alphanumeric() { c } else { '.' })
                        .collect();
                    let id = format!("generated.{generator_slug}.{ordinal}");
                    if !existing_ids.iter().any(|existing| existing == &id) {
                        break id;
                    }
                };
                self.generation_ordinal = ordinal;
                let request = GeneratorRequest {
                    generator: choice.id,
                    args: choice.default_args,
                    locks,
                };
                let result = self.generator_catalog.generate(
                    record_id,
                    &request,
                    &mut self.generation_tape,
                    GeneratorLimits::default(),
                );
                {
                    let runner = &mut *ctx.runner;
                    runner.update(|ui| match result {
                        Ok(record) => {
                            ui.generator_preview = Some(record);
                            ui.status = "generated preview".to_owned();
                        }
                        Err(error) => ui.status = format!("generation failed: {error}"),
                    });
                }
            }
            GenerationRequest::Commit => {
                let Some(record) = preview else {
                    return;
                };
                if matches!(record.proposal, GenValue::Campaign { .. }) {
                    let item_owner = local_snapshot.as_ref().and_then(|snapshot| {
                        snapshot
                            .turns
                            .active()
                            .or_else(|| snapshot.map.tokens.first().map(|token| token.id))
                    });
                    if remote {
                        if let Some(net) = self.net.as_mut() {
                            let request = net.commit_campaign(record, item_owner);
                            {
                                let runner = &mut *ctx.runner;
                                runner.update(|ui| match request {
                                    Some(request) => {
                                        ui.status = format!(
                                            "committing campaign draft (request {})",
                                            request
                                        )
                                    }
                                    None => {
                                        ui.status = "campaign authority actor stopped".to_owned()
                                    }
                                });
                            }
                        }
                    } else if let Some(snapshot) = local_snapshot {
                        let mut host = HostSession::with_history(
                            snapshot,
                            self.campaign.clone(),
                            self.history.clone(),
                        );
                        match host.commit_campaign(record.clone(), item_owner) {
                            Ok(_) => {
                                self.campaign = host.campaign().clone();
                                self.history = host.history().clone();
                                self.journal = host.state().journal.clone();
                                let snapshot = host.state().clone();
                                {
                                    let runner = &mut *ctx.runner;
                                    runner.update(|ui| {
                                        ui.apply_snapshot(snapshot);
                                        ui.status = "committed campaign draft".to_owned();
                                    });
                                }
                            }
                            Err(error) => {
                                let runner = &mut *ctx.runner;
                                runner.update(|ui| {
                                    ui.generator_preview = Some(record);
                                    ui.status = format!("campaign commit failed: {error}");
                                });
                            }
                        }
                    }
                    return;
                }
                let mut events = vec![GameEvent::Generation(record.clone())];
                match &record.proposal {
                    GenValue::LocalMap { map } => match map.lower(MapScale::Local) {
                        Ok(mut campaign_map) => {
                            campaign_map.id = format!("{}.map", record.id);
                            let id = campaign_map.id.clone();
                            events.push(GameEvent::MapStored(campaign_map));
                            events.push(GameEvent::MapActivated { id });
                        }
                        Err(error) => {
                            {
                                let runner = &mut *ctx.runner;
                                runner.update(|ui| {
                                    ui.generator_preview = Some(record.clone());
                                    ui.status = format!("generated map is invalid: {error}");
                                });
                            }
                            return;
                        }
                    },
                    GenValue::WorldFact { fact } => events.push(GameEvent::Fact(fact.clone())),
                    GenValue::Item { item } => {
                        let target = local_snapshot.as_ref().and_then(|snapshot| {
                            snapshot
                                .turns
                                .active()
                                .or_else(|| snapshot.map.tokens.first().map(|token| token.id))
                        });
                        let Some(target) = target else {
                            {
                                let runner = &mut *ctx.runner;
                                runner.update(|ui| {
                                    ui.generator_preview = Some(record.clone());
                                    ui.status =
                                        "generated item needs a character on the map".to_owned();
                                });
                            }
                            return;
                        };
                        let mut inventory = local_snapshot
                            .as_ref()
                            .and_then(|snapshot| snapshot.inventories.get(&target))
                            .cloned()
                            .unwrap_or_default();
                        let instance = ItemInstance {
                            id: ItemId::new(format!("{}.item", record.id)),
                            template: item.template.clone(),
                            name: item.name.clone(),
                            quantity: 1,
                            tags: item.tags.clone(),
                            modifiers: Vec::new(),
                            appearance_layers: Vec::new(),
                        };
                        if let Err(error) = inventory.insert(instance) {
                            {
                                let runner = &mut *ctx.runner;
                                runner.update(|ui| {
                                    ui.generator_preview = Some(record.clone());
                                    ui.status = format!("generated item is invalid: {error:?}");
                                });
                            }
                            return;
                        }
                        events.push(GameEvent::InventorySet {
                            token: target,
                            inventory,
                        });
                    }
                    GenValue::Npc { npc } => {
                        // Lower a generated NPC into a *statted* creature. The
                        // proposal is thin (key, name, tags); its key doubles as
                        // a bestiary slug, so a generated "Skreek" is a goblin's
                        // stat block under a generated name. That reuse is what
                        // makes `>gen npc` end in something fightable rather than
                        // a nameless sprite. A key with no bestiary match falls
                        // back to a plain default sheet.
                        let snapshot = local_snapshot.as_ref();
                        let (at, id) = match snapshot {
                            Some(s) => (free_snapshot_tile(&s.map), next_snapshot_id(s)),
                            None => ((2, 2), TokenId(1)),
                        };
                        let monster = srd_bestiary().into_iter().find(|m| m.key == npc.key);
                        let (sprite, mut sheet) = match monster {
                            Some(m) => (m.sprite.clone(), monster_sheet(&m)),
                            None => {
                                let sheet = self
                                    .system
                                    .as_ref()
                                    .map(System::default_sheet)
                                    .unwrap_or_else(|| SheetData::new("5e-srd"));
                                ("knight".to_owned(), sheet)
                            }
                        };
                        // The generated name over the base creature's.
                        sheet.set_text("name", npc.name.clone());
                        events.push(GameEvent::Map(SessionEvent::TokenPlaced(Token {
                            id,
                            at,
                            facing: Facing::South,
                            sprite,
                            owner: None,
                        })));
                        events.push(GameEvent::SheetSet { token: id, sheet });
                        // A fightable NPC joins initiative.
                        events.push(GameEvent::TurnAdd(id));
                    }
                    _ => {}
                }
                if remote {
                    {
                        let runner = &mut *ctx.runner;
                        runner.update(|ui| {
                            ui.net_outbox.extend(events);
                            ui.status = "committing generated result".to_owned();
                        });
                    }
                } else if let Some(mut snapshot) = local_snapshot {
                    let result = events
                        .iter()
                        .try_for_each(|event| apply_game(&mut snapshot, event));
                    match result {
                        Ok(()) => {
                            self.journal = snapshot.journal.clone();
                            {
                                let runner = &mut *ctx.runner;
                                runner.update(|ui| {
                                    ui.apply_snapshot(snapshot);
                                    ui.status = "committed generated result".to_owned();
                                });
                            }
                        }
                        Err(error) => {
                            let runner = &mut *ctx.runner;
                            runner.update(|ui| {
                                ui.generator_preview = Some(record);
                                ui.status = format!("generation commit failed: {error:?}");
                            });
                        }
                    }
                }
            }
        }
    }
}
