//! Tests for `surfaces.rs`: the overlay surfaces and the selection rows.
//!
//! A surface only ever sets view state and arms a request flag. These check
//! that it arms the right one, that the DM-only surfaces stay shut for a
//! viewer, and that the rows mirror the world they are drawn from.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn compendium_item_request_targets_the_open_sheet() {
    let mut ui = UiState::new(demo_map());
    ui.open_sheet = Some(TokenId(1));
    let item = ItemRow {
        key: "longsword".to_owned(),
        name: "Longsword".to_owned(),
        category: "Weapon".to_owned(),
        cost: "15 gp".to_owned(),
        weight: "3 lb.".to_owned(),
        detail: "1d8 slashing".to_owned(),
        desc: String::new(),
    };
    ui.request_compendium_item(&item);
    assert_eq!(
        ui.inventory_request,
        Some(InventoryRequest::AddCompendiumItem {
            token: TokenId(1),
            template: "longsword".to_owned(),
            name: "Longsword".to_owned(),
            category: "Weapon".to_owned(),
        })
    );
}

#[test]
fn generator_controls_keep_locks_visible_and_queue_host_work() {
    let mut ui = UiState::new(demo_map());
    ui.generator_choices.push(GeneratorChoice {
        id: "demo:forge_item".to_owned(),
        name: "Forge item".to_owned(),
        default_args: GenValue::Text {
            value: "river".to_owned(),
        },
        lock_presets: vec![isometry_campaign::GeneratorLockPreset {
            key: "culture".to_owned(),
            label: "River-clan culture".to_owned(),
            value: GenValue::Text {
                value: "river-clans".to_owned(),
            },
        }],
    });
    ui.open_generator();
    ui.toggle_generator_lock();
    assert_eq!(
        ui.generator_locks.get("culture"),
        Some(&GenValue::Text {
            value: "river-clans".to_owned()
        })
    );
    ui.request_generation();
    assert_eq!(ui.generation_request, Some(GenerationRequest::Generate));

    ui.toggle_generator_lock();
    assert!(!ui.generator_locks.contains_key("culture"));
}

#[test]
fn generator_choice_request_is_host_only_and_preserves_the_disclosed_inputs() {
    let mut ui = UiState::new(demo_map());
    ui.generator_choices.push(GeneratorChoice {
        id: "demo:npc".to_owned(),
        name: "NPC".to_owned(),
        default_args: GenValue::Text {
            value: "example".to_owned(),
        },
        lock_presets: Vec::new(),
    });

    ui.can_edit_inventory = false;
    ui.choose_generator(
        "session-4".to_owned(),
        "isometry.generator-preview/v1".to_owned(),
        "What should I prepare?".to_owned(),
    );
    assert!(ui.generator_selection_request.is_none());
    assert_eq!(ui.status, "generation requires the host");

    ui.can_edit_inventory = true;
    ui.choose_generator(
        "session-4".to_owned(),
        "isometry.generator-preview/v1".to_owned(),
        "What should I prepare?".to_owned(),
    );
    assert_eq!(
        ui.generator_selection_request,
        Some(GeneratorSelectionRequest {
            seed: "session-4".to_owned(),
            domain: "isometry.generator-preview/v1".to_owned(),
            prompt: "What should I prepare?".to_owned(),
        })
    );
}

#[test]
fn governance_conflict_queues_typed_adopt_and_branch_requests() {
    let mut ui = UiState::new(demo_map());
    ui.governance_conflict = Some(GovernanceConflict {
        candidates: vec![
            GovernanceBindingRow {
                proposal: [1; 32],
                moot: "North table".to_owned(),
                policy: "unanimous".to_owned(),
                endorsements: 2,
                required: 2,
                claims: 1,
            },
            GovernanceBindingRow {
                proposal: [2; 32],
                moot: "North table".to_owned(),
                policy: "threshold 2".to_owned(),
                endorsements: 2,
                required: 2,
                claims: 1,
            },
        ],
        can_adopt: true,
        can_branch: true,
        restriction: None,
    });

    ui.open_governance_conflict();
    ui.select_governance_candidate(1);
    ui.request_governance_adopt();
    assert_eq!(
        ui.governance_resolution_request,
        Some(GovernanceResolutionRequest::Adopt { selected: [2; 32] })
    );
    assert!(!ui.governance_conflict_open);

    ui.open_governance_conflict();
    ui.request_governance_branch();
    assert_eq!(
        ui.governance_resolution_request,
        Some(GovernanceResolutionRequest::Branch {
            candidates: vec![[1; 32], [2; 32]],
        })
    );
}

#[test]
fn governance_conflict_respects_host_restrictions() {
    let mut ui = UiState::new(demo_map());
    ui.governance_conflict = Some(GovernanceConflict {
        candidates: vec![
            GovernanceBindingRow {
                proposal: [1; 32],
                moot: "First table".to_owned(),
                policy: "unanimous".to_owned(),
                endorsements: 1,
                required: 1,
                claims: 1,
            },
            GovernanceBindingRow {
                proposal: [2; 32],
                moot: "Other table".to_owned(),
                policy: "unanimous".to_owned(),
                endorsements: 1,
                required: 1,
                claims: 1,
            },
        ],
        can_adopt: false,
        can_branch: false,
        restriction: Some("no shared founding electorate".to_owned()),
    });
    ui.open_governance_conflict();
    ui.request_governance_adopt();
    assert!(ui.governance_resolution_request.is_none());
    assert_eq!(ui.status, "no shared founding electorate");
}

#[test]
fn transfer_request_keeps_source_and_target_explicit() {
    let mut ui = UiState::new(demo_map());
    ui.open_sheet = Some(TokenId(1));
    ui.request_transfer(TokenId(2), ItemId::new("token-1.item-0"));
    assert_eq!(
        ui.inventory_request,
        Some(InventoryRequest::Transfer {
            from: TokenId(1),
            to: TokenId(2),
            item: ItemId::new("token-1.item-0"),
        })
    );
}

#[test]
fn the_storylet_surface_is_dm_only_and_plays_only_the_ready() {
    let mut ui = UiState::new(demo_map());
    // A joined player cannot open or play storylets (matching reads secrets).
    ui.can_edit_inventory = false;
    ui.open_storylets();
    assert!(
        !ui.storylet_open,
        "a client must not open the storylet surface"
    );

    // The DM can. A locked storylet cannot be played; a ready one arms a
    // request the host will commit.
    ui.can_edit_inventory = true;
    ui.storylets = vec![
        StoryletRow {
            key: "locked".to_owned(),
            entry: "The cult stirs.".to_owned(),
            available: false,
            status: "needs a faction tagged 'cult'".to_owned(),
            cast: Vec::new(),
        },
        StoryletRow {
            key: "ready".to_owned(),
            entry: "A stranger greets you.".to_owned(),
            available: true,
            status: "ready".to_owned(),
            cast: Vec::new(),
        },
    ];
    ui.open_storylets();
    assert!(ui.storylet_open);

    ui.storylet_selected = 0; // the locked one
    ui.play_storylet();
    assert_eq!(
        ui.storylet_request, None,
        "a locked storylet cannot be played"
    );

    ui.storylet_selected = 1; // the ready one
    ui.play_storylet();
    assert_eq!(ui.storylet_request.as_deref(), Some("ready"));
}

#[test]
fn the_downtime_surface_is_dm_only_and_commits_only_the_kept() {
    let mut ui = UiState::new(demo_map());
    // A joined player cannot open downtime (the roll reads the world and
    // spends host entropy), so nothing is armed.
    ui.can_edit_inventory = false;
    ui.open_downtime();
    assert!(
        !ui.downtime_open,
        "a client must not open the downtime surface"
    );
    assert!(!ui.downtime_roll_request);

    // The DM can: opening arms a roll request the host fills with rows.
    ui.can_edit_inventory = true;
    ui.open_downtime();
    assert!(ui.downtime_open && ui.downtime_roll_request);
    ui.downtime_roll_request = false; // the host consumed it and filled rows
    ui.faction_moves = vec![
        FactionMoveRow {
            faction: "tide".to_owned(),
            verb: "court".to_owned(),
            text: "Bran swore to the Tide Court.".to_owned(),
            has_change: true,
            struck: false,
        },
        FactionMoveRow {
            faction: "ash".to_owned(),
            verb: "raid".to_owned(),
            text: "The Ash Company raided a rival.".to_owned(),
            has_change: false,
            struck: false,
        },
    ];

    // Strike the raid; it will not commit.
    ui.downtime_selected = 1;
    ui.toggle_strike_downtime();
    assert!(ui.faction_moves[1].struck);
    ui.commit_downtime();
    assert!(ui.downtime_commit_request, "one kept move arms the commit");

    // Strike everything and commit refuses: an empty tick is no tick.
    ui.downtime_commit_request = false;
    ui.faction_moves.iter_mut().for_each(|m| m.struck = true);
    ui.commit_downtime();
    assert!(
        !ui.downtime_commit_request,
        "nothing kept, nothing to commit"
    );
}

#[test]
fn the_overmap_surface_opens_and_arms_a_travel_request() {
    let mut ui = UiState::new(demo_map());
    assert!(!ui.overmap_open);
    // Anyone may look at the overmap (unlike the DM-only downtime surface).
    ui.open_overmap();
    assert!(ui.overmap_open);
    // Clicking a place arms a one-shot the host resolves; the view decides
    // nothing about the trip.
    ui.request_travel("forest".to_owned());
    assert_eq!(ui.overmap_travel_request.as_deref(), Some("forest"));
    // Choosing a pace and a stance arm their own one-shots for the host.
    ui.request_pace(50);
    assert_eq!(ui.overmap_pace_request, Some(50));
    ui.request_stance("scout");
    assert_eq!(ui.overmap_stance_request.as_deref(), Some("scout"));
    ui.request_map_read();
    assert!(ui.overmap_read_request, "studying the map arms a read");
    ui.close_overmap();
    assert!(!ui.overmap_open);
}

/// The selection rows are a mirror of truth, never the truth itself.
///
/// The host bridge dispatches when a row disagrees with the world, so a sync
/// that failed to track truth would look exactly like a user click and fire a
/// spurious request every dispatch. These pin the one-way push.
#[test]
fn selection_rows_mirror_mode_and_world() {
    let mut ui = UiState::new(demo_map());

    ui.mode = EditMode::Measure;
    ui.sync_selection_rows();
    assert_eq!(
        ui.mode_selection.selected,
        vec![EditMode::ALL
            .iter()
            .position(|m| *m == EditMode::Measure)
            .unwrap()],
        "the mode row must follow ui.mode"
    );

    // Pace 200 is "Slow", index 2 of PACE_PCTS.
    ui.world.party_pace.insert("dm".to_owned(), 200);
    ui.sync_selection_rows();
    assert_eq!(ui.pace_selection.selected, vec![2]);
    assert_eq!(PACE_PCTS[2], 200, "index and value must agree");

    // An unset pace reads as normal, not as whatever was there before.
    ui.world.party_pace.remove("dm");
    ui.sync_selection_rows();
    assert_eq!(ui.pace_selection.selected, vec![1]);
    assert_eq!(PACE_PCTS[1], 100);
}

/// The compendium's namespace strip is a mirror on the same terms as the
/// selection rows, and its bridge is the one that does more than assign a
/// field: switching a namespace also clears the open page, sort, scroll, and
/// filter, so the strip must never be treated as the truth.
#[test]
fn compendium_tab_strip_mirrors_the_namespace() {
    use crate::state::CompendiumTab;
    let mut ui = UiState::new(demo_map());
    ui.sync_selection_rows();
    assert_eq!(ui.compendium_tabs.selected, 0, "Monsters is the first tab");

    ui.set_compendium_tab(CompendiumTab::Items);
    ui.sync_selection_rows();
    assert_eq!(
        ui.compendium_tabs.selected,
        CompendiumTab::ALL
            .iter()
            .position(|t| *t == CompendiumTab::Items)
            .unwrap(),
        "the strip must follow compendium_tab"
    );

    // What the bridge has to run, and why poking the field would not do: a
    // namespace switch resets everything scoped to the old namespace.
    ui.compendium_selected = Some("goblin".to_owned());
    ui.compendium_search = cambium::TextInput::new("gob");
    ui.compendium_scroll = 40.0;
    ui.set_compendium_tab(CompendiumTab::Spells);
    assert!(ui.compendium_selected.is_none(), "the open page follows");
    assert!(
        ui.compendium_search.text().is_empty(),
        "the filter follows"
    );
    assert_eq!(ui.compendium_scroll, 0.0, "the scroll follows");
}

/// Stance is per-token and the row speaks for the lead token, so an unset
/// stance must land on "Walk" (the empty key) rather than a stale index.
#[test]
fn stance_row_defaults_to_walking() {
    let mut ui = UiState::new(demo_map());
    ui.sync_selection_rows();
    assert_eq!(ui.stance_selection.selected, vec![STANCE_KEYS.len() - 1]);
    assert_eq!(STANCE_KEYS[STANCE_KEYS.len() - 1], "");

    let lead = ui
        .map
        .tokens
        .first()
        .map(|t| t.id)
        .expect("demo has tokens");
    ui.map.stances.insert(lead, "forage".to_owned());
    ui.sync_selection_rows();
    assert_eq!(ui.stance_selection.selected, vec![2]);
    assert_eq!(STANCE_KEYS[2], "forage");
}
