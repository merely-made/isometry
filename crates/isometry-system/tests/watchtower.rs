use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use isometry_campaign::{CampaignDraft, EntropyTape, GenValue, GeneratorRequest};
use isometry_core::{HeightSightRules, MoveRules, TokenId, reachable, visible_from_height};
use isometry_system::{GeneratorCatalog, GeneratorLimits};
use serde_json::Value;

fn pack_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs/watchtower")
}

fn request(locks: BTreeMap<String, GenValue>) -> GeneratorRequest {
    GeneratorRequest {
        generator: "watchtower:ruined_tower".to_owned(),
        args: GenValue::Text {
            value: "ash-and-bells".to_owned(),
        },
        locks,
    }
}

fn generate(seed: u64, locks: BTreeMap<String, GenValue>) -> CampaignDraft {
    let catalog = GeneratorCatalog::discover([pack_root()]);
    let mut tape = EntropyTape::from_seed(seed);
    let record = catalog
        .generate(
            format!("generated.watchtower.{seed}"),
            &request(locks),
            &mut tape,
            GeneratorLimits::default(),
        )
        .unwrap();
    let GenValue::Campaign { campaign } = record.proposal else {
        panic!("watchtower generator should return a campaign");
    };
    assert_eq!(tape.draws.len(), 1);
    campaign
}

fn json(campaign: &CampaignDraft) -> Value {
    serde_json::to_value(campaign).unwrap()
}

fn map_json(campaign: &CampaignDraft) -> Value {
    json(campaign)["maps"][0]["map"].clone()
}

fn cell_prop(campaign: &CampaignDraft, prop: &str) -> Option<(u32, u32)> {
    map_json(campaign)["cells"]
        .as_array()
        .unwrap()
        .iter()
        .find(|cell| cell["prop"] == prop)
        .map(|cell| {
            (
                cell["col"].as_u64().unwrap() as u32,
                cell["row"].as_u64().unwrap() as u32,
            )
        })
}

fn beast_at(campaign: &CampaignDraft) -> (i32, i32) {
    json(campaign)["maps"][0]["inhabitants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|inhabitant| inhabitant["sprite"] == "tower-beast")
        .map(|inhabitant| {
            (
                inhabitant["at"]["col"].as_i64().unwrap() as i32,
                inhabitant["at"]["row"].as_i64().unwrap() as i32,
            )
        })
        .expect("tower-beast inhabitant")
}

fn campaign_map<'a>(campaign: &'a CampaignDraft, id: &str) -> &'a isometry_campaign::DraftMap {
    campaign
        .maps
        .iter()
        .find(|map| map.map.id == id)
        .unwrap_or_else(|| panic!("missing campaign map {id}"))
}

#[test]
fn watchtower_pack_loads_and_declares_one_breach_lock() {
    let catalog = GeneratorCatalog::discover([pack_root()]);
    assert!(catalog.diagnostics().is_empty(), "pack should load cleanly");
    let choice = catalog
        .choices()
        .into_iter()
        .find(|choice| choice.id == "watchtower:ruined_tower")
        .expect("watchtower generator choice");
    assert_eq!(choice.lock_presets.len(), 1);
    assert_eq!(choice.lock_presets[0].key, "breach");
    assert_eq!(
        choice.lock_presets[0].value,
        GenValue::Text {
            value: "east".to_owned()
        }
    );
}

#[test]
fn watchtower_generation_varies_and_replays_deterministically() {
    let first = generate(3, BTreeMap::new());
    let replay = generate(3, BTreeMap::new());
    first.validate().unwrap();
    replay.validate().unwrap();
    assert!(first.world.storylets.contains_key("parley-at-the-breach"));
    assert!(first.world.storylets.contains_key("steel-in-the-stones"));
    assert!(first.world.storylets.contains_key("the-last-bell"));
    assert!(
        !first.world.storylets["parley-at-the-breach"]
            .effects
            .is_empty()
    );
    assert!(
        !first.world.storylets["steel-in-the-stones"]
            .effects
            .is_empty()
    );
    assert_eq!(json(&first), json(&replay));
    let variants: BTreeSet<String> = (0..32)
        .map(|seed| {
            let campaign = generate(seed, BTreeMap::new());
            campaign.validate().unwrap();
            let map = campaign.maps[0].lower().unwrap();
            let rules = MoveRules {
                budget: 30,
                step_up: 1,
                step_down: 1,
                passable: &|kind| kind != "water",
            };
            let breach = cell_prop(&campaign, "tower-wall-east")
                .or_else(|| cell_prop(&campaign, "tower-wall-west"))
                .expect("generated breach");
            let reachable = reachable(&map.document, (2, 7), &rules, TokenId(1));
            assert!(
                reachable.contains_key(&(breach.0 as i32, breach.1 as i32)),
                "hero should reach either generated breach"
            );
            serde_json::to_string(&map_json(&campaign)).unwrap()
        })
        .collect();
    assert!(
        variants.len() >= 2,
        "seed range should produce distinct breach or nest layouts"
    );
    assert!(
        cell_prop(&first, "tower-wall-east").is_some()
            || cell_prop(&first, "tower-wall-west").is_some()
    );
}

#[test]
fn watchtower_lock_holds_breach_and_keeps_approach_and_nest_reachable() {
    let locked = BTreeMap::from([(
        "breach".to_owned(),
        GenValue::Text {
            value: "east".to_owned(),
        },
    )]);
    for seed in [1, 2, 17, 42] {
        let campaign = generate(seed, locked.clone());
        campaign.validate().unwrap();
        assert!(cell_prop(&campaign, "tower-wall-east").is_some());
        assert!(cell_prop(&campaign, "tower-wall-west").is_none());

        let map = campaign.maps[0].lower().unwrap();
        let rules = MoveRules {
            budget: 30,
            step_up: 1,
            step_down: 1,
            passable: &|kind| kind != "water",
        };
        let reachable = reachable(&map.document, (2, 7), &rules, TokenId(1));
        assert!(
            reachable.contains_key(&(7, 7)),
            "approach must reach the tower"
        );
        let beast = beast_at(&campaign);
        assert_eq!(
            map.document.elevation.get(beast.0 as u32, beast.1 as u32),
            Some(&4),
            "the creature perches on the surviving upper ledge"
        );
        let adjacent = [
            (beast.0 - 1, beast.1),
            (beast.0 + 1, beast.1),
            (beast.0, beast.1 - 1),
            (beast.0, beast.1 + 1),
        ]
        .into_iter()
        .find(|tile| {
            map.document.ground.in_bounds(tile.0, tile.1)
                && !map.document.tokens.iter().any(|token| token.at == *tile)
                && map.document.elevation.get(tile.0 as u32, tile.1 as u32) == Some(&3)
        })
        .expect("an unoccupied tile adjacent to the tower-beast");
        assert!(
            reachable.contains_key(&adjacent),
            "a tile beside the nest must be reachable"
        );
        assert!(
            reachable.contains_key(&(10, 5)) || reachable.contains_key(&(10, 9)),
            "the empty upper ledge leads to a lookout"
        );
        let mut sealed = map.document.clone();
        sealed.ground.set(12, 7, isometry_core::TileKindId(0));
        let outside = isometry_core::reachable(&sealed, (2, 7), &rules, TokenId(1));
        for row in 4..=10 {
            for col in 8..=12 {
                if (row == 4 || row == 10 || col == 8 || col == 12)
                    && map.document.elevation.get(col, row) == Some(&4)
                {
                    assert!(
                        !outside.contains_key(&(col as i32, row as i32)),
                        "the outer wall cannot be climbed without entering the breach"
                    );
                }
            }
        }

        let value = json(&campaign);
        let inhabitants = value["maps"][0]["inhabitants"].as_array().unwrap();
        assert_eq!(inhabitants.len(), 5);
        for inhabitant in inhabitants {
            let at = &inhabitant["at"];
            assert!(at["col"].as_u64().unwrap() < 15);
            assert!(at["row"].as_u64().unwrap() < 15);
            assert_eq!(inhabitant["system"], "5e-srd");
            let stats = inhabitant["stats"].as_object().unwrap();
            for key in [
                "hp_current",
                "hp_max",
                "ac",
                "speed",
                "sight",
                "prof",
                "str",
                "dex",
                "con",
                "int",
                "wis",
                "cha",
            ] {
                assert!(stats.contains_key(key), "missing 5e stat {key}");
            }
        }
        assert_eq!(
            inhabitants
                .iter()
                .find(|inhabitant| inhabitant["id"] == 1)
                .unwrap()["owner"],
            "player"
        );
    }
}

#[test]
fn forest_region_links_sites_and_keeps_crossings_walkable() {
    let campaign = generate(91, BTreeMap::new());
    campaign.validate().unwrap();
    assert_eq!(campaign.maps.len(), 4);
    assert_eq!(campaign.starting_map, "watchtower:ruined-watchtower");

    let region = campaign_map(&campaign, "watchtower:forest-region");
    assert_eq!(region.scale, isometry_campaign::MapScale::Region);
    let region_ids: BTreeSet<_> = region
        .map
        .transitions
        .iter()
        .map(|transition| transition.target_map.as_str())
        .collect();
    assert_eq!(region_ids.len(), 3);
    for target in [
        "watchtower:ruined-watchtower",
        "watchtower:forest-entry",
        "watchtower:stream-clearing",
    ] {
        assert!(region_ids.contains(target), "region link to {target}");
    }
    for local in [
        "watchtower:ruined-watchtower",
        "watchtower:forest-entry",
        "watchtower:stream-clearing",
    ] {
        let map = campaign_map(&campaign, local);
        assert!(
            map.map
                .transitions
                .iter()
                .any(|transition| transition.target_map == "watchtower:forest-region"),
            "local site {local} must return to the region"
        );
    }
    for source in &campaign.maps {
        for transition in &source.map.transitions {
            let target = campaign_map(&campaign, &transition.target_map);
            if let Some(entry) = &transition.target_entry {
                assert!(
                    target.map.transitions.iter().any(|door| &door.id == entry),
                    "{} entry {} must exist on {}",
                    source.map.id,
                    entry,
                    target.map.id
                );
            }
        }
    }
    let tower = campaign_map(&campaign, "watchtower:ruined-watchtower");
    assert!(
        tower
            .map
            .cells
            .iter()
            .any(|cell| cell.prop.as_deref() == Some("forest-tree")),
        "tower approach should sit in physical forest"
    );

    let region_map = region.lower().unwrap();
    let rules = MoveRules {
        budget: 30,
        step_up: 1,
        step_down: 1,
        passable: &|kind| kind != "water",
    };
    let from_entry = reachable(&region_map.document, (0, 4), &rules, TokenId(999));
    assert!(
        from_entry.contains_key(&(5, 1)),
        "entry road reaches tower road"
    );
    let from_stream = reachable(&region_map.document, (10, 4), &rules, TokenId(999));
    assert!(
        from_stream.contains_key(&(8, 4)),
        "stone crossing remains reachable"
    );

    let stream = campaign_map(&campaign, "watchtower:stream-clearing")
        .lower()
        .unwrap();
    let crossing = reachable(&stream.document, (0, 3), &rules, TokenId(999));
    assert!(
        crossing.contains_key(&(4, 3)),
        "stone crossing reaches midstream"
    );
    assert!(
        crossing.contains_key(&(8, 3)),
        "stone crossing reaches far bank"
    );
    assert!(!crossing.contains_key(&(4, 2)), "water remains impassable");

    let tree_count = region
        .map
        .cells
        .iter()
        .filter(|cell| cell.prop.as_deref() == Some("forest-tree"))
        .count();
    assert!(tree_count >= 6, "forest should carry physical canopy props");
}

#[test]
fn generated_tower_lookout_clears_forest_canopy() {
    let campaign = generate(7, BTreeMap::new());
    let tower = campaign_map(&campaign, "watchtower:ruined-watchtower")
        .lower()
        .unwrap();
    let rules = HeightSightRules {
        radius: 8,
        eye_height: 1,
        obstacle_height: &|kind| match kind {
            "forest-tree" => 3,
            "wall" | "tower-wall-east" | "tower-wall-west" => 1,
            _ => 0,
        },
    };
    let walker = visible_from_height(&tower.document, (10, 0), &rules);
    let lookout = visible_from_height(&tower.document, (10, 5), &rules);
    assert!(
        walker.contains(&(10, 2)),
        "the nearby canopy itself is visible"
    );
    assert!(
        !walker.contains(&(10, 3)),
        "ground-level sight stops at the canopy"
    );
    assert!(
        lookout.contains(&(10, 0)),
        "raised lookout clears nearby canopy"
    );
}
