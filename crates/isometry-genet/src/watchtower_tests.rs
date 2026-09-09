//! The inhabited watchtower through the desktop host's real dispatch seam.

mod atlas;
mod performance;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use cambium_genet_winit_host::{Harness, Init};
use genet_probe::Selector;
use isometry_core::TokenId;
use layout_dom_api::{LayoutDom as _, LocalName, Namespace};

use super::*;

type WatchtowerHarness = Harness<UiState, Logic, UiChild>;

const WINDOW: (f32, f32) = (1_100.0, 820.0);

#[test]
fn campaign_creation_places_the_selected_party_on_the_overmap() {
    for viewer in [None, Some("player")] {
        let (mut harness, _) = watchtower();
        harness.update(|ui| {
            ui.viewer = viewer.map(str::to_owned);
            ui.start_generator("watchtower");
        });
        harness.after_dispatch();
        harness.update(|ui| ui.commit_generation_preview());
        harness.after_dispatch();
        harness.update(|ui| ui.open_overmap());
        harness.relayout();
        let ui = harness.state();
        let party = viewer.unwrap_or("dm");
        assert_eq!(
            ui.world.party_at(party),
            Some("watchtower:ruined-watchtower")
        );
        assert_eq!(
            ui.world.party_node.len(),
            1,
            "only the selected party is placed"
        );
        let known = ui.world.overmap_for(party);
        let ids: Vec<_> = known.nodes.iter().map(|node| node.id.as_str()).collect();
        assert_eq!(
            ids,
            ["watchtower:forest-region", "watchtower:ruined-watchtower"]
        );
        assert_eq!(known.edges.len(), 1);
        assert!(
            isometry_views::overmap_swatch(ui).is_some(),
            "the native overmap has content"
        );
    }
}

#[test]
fn map_only_campaign_creation_does_not_require_a_regional_party() {
    let (mut harness, _) = watchtower();
    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    harness.update(|ui| {
        let isometry_campaign::GenValue::Campaign { campaign } =
            &mut ui.generator_preview.as_mut().unwrap().proposal
        else {
            panic!("campaign preview")
        };
        campaign.world.places.clear();
        campaign.world.routes.clear();
        ui.commit_generation_preview();
    });
    harness.after_dispatch();
    let ui = harness.state();
    assert_eq!(
        ui.active_map.as_deref(),
        Some("watchtower:ruined-watchtower")
    );
    assert!(ui.world.party_node.is_empty());
    assert!(ui.world.party_known.is_empty());
    assert_eq!(ui.map.tokens.len(), 5);
}

#[test]
fn generated_forest_sites_reach_the_native_board() {
    let (mut harness, app) = watchtower();
    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    harness.update(|ui| ui.commit_generation_preview());
    harness.after_dispatch();
    let snapshot = app.borrow().snapshot_of(harness.state());
    for map in snapshot.maps.values() {
        let mut visit = snapshot.clone();
        visit.active_map = Some(map.id.clone());
        visit.map = map.document.clone();
        harness.update(|ui| ui.apply_snapshot(visit));
        harness.relayout();
        let labels = harness.with_dom(|dom| {
            genet_probe::matching(dom, &Selector::class("tile-encounter"))
                .into_iter()
                .map(|node| {
                    dom.attribute(node, &Namespace::from(""), &LocalName::from("aria-label"))
                        .unwrap_or_default()
                        .to_owned()
                })
                .collect::<Vec<_>>()
        });
        for anchor in &map.encounter_anchors {
            assert!(
                labels.iter().any(|label| label.contains(&anchor.id)),
                "{} exposes authored site {}: {labels:?}",
                map.id,
                anchor.id
            );
        }
    }
}

#[test]
fn board_tile_clips_its_hit_area_to_the_visible_diamond() {
    let mut map = isometry_core::MapDocument::new("one tile", 1, 1);
    map.tile_kinds.push("grass".to_owned());
    map.ground.set(0, 0, isometry_core::TileKindId(1));
    let mut ui = UiState::new(map);
    ui.camera = (160.0, 140.0);
    ui.viewport = (WINDOW.0 - PANEL_W, WINDOW.1);
    let mut harness = Harness::new(board_css(), ui, board_root as Logic);
    for zoom in [1.0, 0.93] {
        harness.set_ui_zoom(zoom);
        harness.update(|ui| ui.set_pixel_grid((2.0, zoom)));
        harness.layout_at(WINDOW.0, WINDOW.1);
        let tile = harness.with_dom(|dom| {
            let tiles = genet_probe::matching(dom, &Selector::class("tile"));
            assert_eq!(tiles.len(), 1);
            tiles[0]
        });
        let (x, y, w, h) = harness.painted_rect(tile).unwrap();
        for (fx, fy) in [(0.1, 0.1), (0.9, 0.1), (0.1, 0.9), (0.9, 0.9)] {
            harness.update(|ui| ui.selected = None);
            harness.relayout();
            harness.click_at(x + w * fx, y + h * fy);
            assert_ne!(harness.hit(), Some(tile), "invisible corner at zoom {zoom}");
            assert_eq!(harness.state().selected, None);
        }
        harness.click_at(x + w * 0.5, y + h * 0.5);
        assert_eq!(harness.state().selected, Some((0, 0)));
    }
}

/// Assemble the same application-owned pieces [`hooks::init`] supplies to the shared
/// host, without a window or process-wide environment changes.
fn watchtower() -> (WatchtowerHarness, Rc<RefCell<App>>) {
    let mut app = App::boot();
    let mut ui = UiState::new(demo_map());
    ui.viewport = (WINDOW.0 - PANEL_W, WINDOW.1);
    ui.camera = (ui.viewport.0 / 2.0, 110.0);
    ui.generator_choices = app.generator_catalog.choices();
    assert!(
        ui.generator_choices
            .iter()
            .any(|choice| choice.id.contains("watchtower")),
        "the bundled watchtower pack is available to the desktop generator"
    );
    let system = srd_5e();
    ui.sheet_schema = schema_of(&system);
    app.system = Some(system);

    let app = Rc::new(RefCell::new(app));
    let sheet = std::mem::take(&mut app.borrow_mut().sheet);
    let mut harness = Harness::with_hooks(
        Init {
            state: ui,
            logic: board_root as Logic,
            sheet,
        },
        hooks::hooks(&app),
    );
    harness.layout_at(WINDOW.0, WINDOW.1);
    (harness, app)
}

#[test]
fn watchtower_preview_commit_projects_inhabitants_and_reopens_checkpoint() {
    let (mut harness, app) = watchtower();

    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    let expected = {
        let preview = harness
            .state()
            .generator_preview
            .as_ref()
            .expect("watchtower request creates a campaign preview");
        let isometry_campaign::GenValue::Campaign { campaign } = &preview.proposal else {
            panic!("watchtower generator produces a campaign draft");
        };
        campaign
            .maps
            .iter()
            .find(|map| map.map.id == campaign.starting_map)
            .expect("campaign identifies a starting map")
            .inhabitants
            .len()
    };
    assert!(expected > 1, "the watchtower has a party and a creature");

    harness.update(|ui| ui.commit_generation_preview());
    harness.after_dispatch();
    harness.relayout();

    let beast = harness
        .state()
        .map
        .tokens
        .iter()
        .find(|token| token.sprite == "tower-beast")
        .expect("the committed map includes the tower beast")
        .id;
    assert_eq!(harness.state().map.tokens.len(), expected);
    assert_eq!(
        harness
            .state()
            .map
            .sheet(beast)
            .map(|sheet| sheet.system.as_str()),
        Some("5e-srd")
    );
    assert!(
        harness.state().map.tokens.iter().all(|token| harness
            .state()
            .map
            .sheet(token.id)
            .is_some()),
        "each generated inhabitant receives its authored sheet"
    );
    assert!(
        harness.state().turns.entries().contains(&beast),
        "the activated campaign puts the creature into the live turn list"
    );
    assert!(
        harness.state().turns.entries().contains(&TokenId(1)),
        "the generated hero is also in the live turn list"
    );
    assert!(
        harness.with_dom(|dom| {
            !genet_probe::matching(dom, &Selector::class("token-tower-beast")).is_empty()
        }),
        "the projected board renders the creature's pack sprite class"
    );

    assert_eq!(
        harness
            .state()
            .map
            .token(TokenId(1))
            .unwrap()
            .owner
            .as_deref(),
        Some("player"),
        "the generated hero belongs to the default joining player"
    );
    harness.update(|ui| {
        ui.viewer = Some("player".to_owned());
        ui.recompute_fog();
    });
    assert!(harness.state().commands(Some("player")));
    assert!(
        harness.state().visible.contains(&(7, 5)),
        "the player can see the approach"
    );
    assert!(
        !harness.state().visible.contains(&(9, 6)),
        "the tower wall blocks the interior"
    );
    harness.update(|ui| {
        ui.viewer = None;
        ui.recompute_fog();
    });

    // Stage an adjacent encounter with the normal editor move, then ask the
    // actual sheet/action pump to resolve it. The generated stats, rather than
    // substituted test sheets, must be sufficient to play.
    let beast_at = harness.state().map.token(beast).unwrap().at;
    let adjacent = [(beast_at.0 - 1, beast_at.1), (beast_at.0 + 1, beast_at.1)]
        .into_iter()
        .find(|at| {
            !harness
                .state()
                .map
                .tokens
                .iter()
                .any(|token| token.at == *at)
        })
        .expect("space beside the creature");
    let hp = harness
        .state()
        .map
        .sheet(beast)
        .unwrap()
        .int("hp_current")
        .unwrap();
    app.borrow_mut().action_rng = Rng::new(3);
    harness.update(|ui| {
        ui.drag_move_token(TokenId(1), adjacent);
        ui.open_sheet = Some(TokenId(1));
        ui.request_action("attack");
        ui.pick_action_target(beast);
    });
    harness.after_dispatch();
    assert!(
        harness
            .state()
            .map
            .sheet(beast)
            .unwrap()
            .int("hp_current")
            .unwrap()
            < hp,
        "an attack through the sheet pump must injure the generated creature: {}",
        harness.state().status
    );

    let public = app.borrow().snapshot_of(harness.state());
    let (private, history, history_origin) = {
        let app = app.borrow();
        (
            app.campaign.clone(),
            app.history.clone(),
            app.history_origin.clone(),
        )
    };
    let checkpoint = CampaignCheckpoint::new(public, private, history, history_origin);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after the Unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "isometry-watchtower-{}-{nonce}.redb",
        std::process::id()
    ));
    {
        let repository = CampaignRepository::open(&path).expect("open fresh campaign repository");
        repository
            .save_checkpoint(&checkpoint)
            .expect("save populated campaign checkpoint");
    }
    let restored = {
        let repository = CampaignRepository::open(&path).expect("reopen campaign repository");
        repository
            .load_checkpoint()
            .expect("load populated campaign checkpoint")
            .expect("the saved checkpoint exists")
    };
    assert_eq!(restored.public, checkpoint.public);
    assert_eq!(restored.history, checkpoint.history);
    drop(restored);
    std::fs::remove_file(path).expect("remove temporary campaign repository");
}

#[test]
fn forest_region_doors_carry_the_character_and_sheet_between_sites() {
    let (mut harness, _) = watchtower();
    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    harness.update(|ui| ui.commit_generation_preview());
    harness.after_dispatch();
    let hero = TokenId(1);
    let sheet = harness.state().map.sheet(hero).unwrap().clone();
    assert_eq!(harness.state().campaign_maps.len(), 4);
    // The first crossing uses the shipping Play click from Mira's spawn.
    harness.update(|ui| {
        ui.mode = EditMode::Play;
        ui.select_token(hero);
        ui.click_tile((1, 7));
    });
    assert_eq!(
        harness.state().active_map.as_deref(),
        Some("watchtower:forest-region"),
        "{}",
        harness.state().status
    );
    assert_eq!(harness.state().map.sheet(hero), Some(&sheet));
    assert_eq!(
        harness.state().world.party_at("dm"),
        Some("watchtower:forest-region")
    );
    assert_eq!(harness.state().world.overmap_for("dm").nodes.len(), 4);
    // Stage each further departure with the editor, then resolve the normal
    // doorway. This proves entry lookup and transfer, independently of budget.
    for (door, destination) in [
        ("stream-road", "watchtower:stream-clearing"),
        ("stream-road", "watchtower:forest-region"),
        ("entry-road", "watchtower:forest-entry"),
        ("entry-road", "watchtower:forest-region"),
        ("tower-road", "watchtower:ruined-watchtower"),
    ] {
        harness.update(|ui| {
            let active = ui.active_map.as_ref().unwrap();
            let at = ui.campaign_maps[active]
                .transitions
                .iter()
                .find(|transition| transition.id == door)
                .unwrap()
                .at;
            ui.drag_move_token(hero, (at.col as i32, at.row as i32));
            ui.travel(hero);
        });
        assert_eq!(
            harness.state().active_map.as_deref(),
            Some(destination),
            "{door}: {}",
            harness.state().status
        );
        assert_eq!(harness.state().world.party_at("dm"), Some(destination));
        assert_eq!(harness.state().map.sheet(hero), Some(&sheet));
        assert_eq!(
            harness.state().map.token(hero).unwrap().owner.as_deref(),
            Some("player")
        );
    }
    assert!(
        harness
            .state()
            .map
            .tokens
            .iter()
            .any(|token| token.sprite == "tower-beast"),
        "the creature remains at the tower while Mira travels"
    );
}

#[test]
fn created_character_has_system_sheet_and_survives_snapshot_roundtrip() {
    let (mut harness, app) = watchtower();
    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    harness.update(|ui| ui.commit_generation_preview());
    harness.after_dispatch();
    let before = harness.state().map.tokens.len();
    let system = app.borrow_mut().system.take();
    harness.update(|ui| {
        ui.selected = Some((3, 7));
        ui.character_name = cambium::TextInput::new("Rowan");
        ui.character_owner = cambium::TextInput::new("rowan-player");
        ui.create_character();
    });
    harness.after_dispatch();
    assert_eq!(
        harness.state().map.tokens.len(),
        before,
        "missing system leaves no token"
    );
    assert!(harness.state().character_create_request.is_none());
    app.borrow_mut().system = system;
    harness.update(|ui| ui.create_character());
    harness.after_dispatch();
    let ui = harness.state();
    assert_eq!(ui.map.tokens.len(), before + 1);
    let id = ui.selected_token.expect("created character is selected");
    assert_eq!(ui.open_sheet, Some(id));
    assert_eq!(ui.map.token(id).unwrap().at, (3, 7));
    assert_eq!(
        ui.map.token(id).unwrap().owner.as_deref(),
        Some("rowan-player")
    );
    let sheet = ui.map.sheet(id).expect("system sheet is attached");
    assert_eq!(sheet.text("name"), Some("Rowan"));
    assert_eq!(sheet.system, "5e-srd");
    assert!(sheet.int("hp_max").unwrap() > 0);
    let snapshot = app.borrow().snapshot_of(ui);
    let bytes = serde_json::to_vec(&snapshot).unwrap();
    let restored: isonetry::GameSnapshot = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored.map.sheet(id), Some(sheet));
    assert_eq!(restored.map.token(id), ui.map.token(id));
}
