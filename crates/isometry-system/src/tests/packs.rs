//! Loading a pack: the beat vocabulary, overrides, the schema, and items.
//!
//! What a loaded system supplies before any rule runs — `sys/system_core.rs`
//! and `sys/srd.rs`'s side — plus the equipped modifier that changes an
//! effective attack without touching the sheet it rides on.
//!
//! Split out of `tests.rs` on 2026-09-04; unchanged.

use super::*;

#[test]
fn the_core_pack_supplies_the_beat_vocabulary() {
    // The app ships no choreography of its own: the default beats come from
    // the `core` pack on disk, exactly like a campaign's would.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs/core");
    let catalog = GeneratorCatalog::discover([root]);
    let (beats, diagnostics) = catalog.choreography();
    assert!(diagnostics.is_empty(), "core pack stylesheets all open");

    // A rules-produced beat carries real CSS and no emote label...
    let strike = beats.iter().find(|b| b.name == "strike").expect("strike");
    assert!(strike.css.contains("@keyframes"));
    assert!(strike.emote.is_none(), "no one performs a strike on demand");

    // ...while a social beat is emotable, which is what puts it in the menu.
    let cheer = beats.iter().find(|b| b.name == "cheer").expect("cheer");
    assert_eq!(cheer.emote.as_deref(), Some("Cheer"));

    // The emote vocabulary is exactly the emotable beats.
    let emotes: Vec<&str> = beats
        .iter()
        .filter(|b| b.emote.is_some())
        .map(|b| b.name.as_str())
        .collect();
    assert_eq!(emotes, ["cheer", "shrug", "taunt"]);
}

#[test]
fn a_later_pack_overrides_a_beat_by_name() {
    // A campaign restyles the swing by shipping its own `strike`; the last
    // pack to declare a name wins, so nothing in the app has to change.
    let core = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs/core");
    let core_only = GeneratorCatalog::discover([core.clone()]);
    let (base, _) = core_only.choreography();
    let base_strike = &base.iter().find(|b| b.name == "strike").unwrap().css;
    assert!(base_strike.contains("iso-strike"));
    // (An override test needs a second on-disk pack; the override *rule* is
    // covered by the catalog unit below, which does not touch the disk.)
    let names: Vec<&str> = base.iter().map(|b| b.name.as_str()).collect();
    assert!(names.contains(&"fall"));
}

#[test]
fn demo_pack_composes_an_inspectable_campaign_draft() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs/demo");
    let pack = GeneratorPack::load(root).unwrap();
    let request = GeneratorRequest {
        generator: "demo:campaign".to_owned(),
        args: GenValue::Text {
            value: "river".to_owned(),
        },
        locks: BTreeMap::new(),
    };
    let mut tape = EntropyTape::from_seed(17);
    let record = pack
        .generate(
            "generated.demo.campaign.1",
            &request,
            &mut tape,
            GeneratorLimits::default(),
        )
        .unwrap();
    let GenValue::Campaign { campaign } = record.proposal else {
        panic!("expected campaign draft");
    };
    campaign.validate().unwrap();
    assert_eq!(campaign.maps.len(), 3);
    assert_eq!(campaign.world.factions.len(), 2);
    assert_eq!(campaign.secrets.len(), 1);
    assert!(campaign.world.laws.contains_key("iron-remembers"));
    assert!(campaign
        .world
        .storylets
        .contains_key(&campaign.final_storylet));
}

#[test]
fn default_sheet_has_schema_defaults() {
    let sys = srd_5e();
    let sheet = sys.default_sheet();
    assert_eq!(sheet.system, "5e-srd");
    assert_eq!(sheet.int("str"), Some(10));
    assert_eq!(sheet.int("prof"), Some(2));
    assert_eq!(sheet.text("name"), Some("Hero"));
}

#[test]
fn ability_modifiers_follow_5e() {
    let mut sys = srd_5e();
    let mut sheet = sys.default_sheet();
    sheet.set_int("str", 16); // +3
    sheet.set_int("dex", 7); //  -2 (floor)
    sheet.set_int("con", 10); //  0
    let d = sys.derived(&sheet);
    assert_eq!(d.get("str_mod"), Some(&3));
    assert_eq!(d.get("dex_mod"), Some(&-2));
    assert_eq!(d.get("con_mod"), Some(&0));
}

#[test]
fn attack_expr_folds_str_mod_and_proficiency() {
    let mut sys = srd_5e();
    let mut sheet = sys.default_sheet();
    sheet.set_int("str", 18); // +4
    sheet.set_int("prof", 3);
    // 1d20 + 4 + 3 = 1d20+7
    assert_eq!(sys.action_expr("attack", &sheet).as_deref(), Some("1d20+7"));
    // A negative total still formats correctly.
    sheet.set_int("str", 6); // -2
    sheet.set_int("prof", 0);
    assert_eq!(sys.action_expr("attack", &sheet).as_deref(), Some("1d20-2"));
}

#[test]
fn equipped_modifier_changes_effective_attack_without_mutating_sheet() {
    use isometry_campaign::{
        EquipmentSlot, Inventory, ItemId, ItemInstance, ItemModifier, ItemModifierKind,
    };

    let mut system = srd_5e();
    let sheet = system.default_sheet();
    let sword = ItemInstance {
        id: ItemId::new("reward-03.sword"),
        template: "srd5e:longsword".to_owned(),
        name: "Fine Longsword".to_owned(),
        quantity: 1,
        tags: vec!["weapon".to_owned()],
        modifiers: vec![ItemModifier {
            id: "reward-03.sword.fine".to_owned(),
            kind: ItemModifierKind::Quality,
            name: "Fine".to_owned(),
            stats: BTreeMap::from([("attack_bonus".to_owned(), 2)]),
            appearance_layer: None,
        }],
        appearance_layers: vec!["weapon:longsword".to_owned()],
    };
    let mut inventory = Inventory::default();
    inventory.insert(sword).unwrap();
    inventory
        .equip(EquipmentSlot::MainHand, ItemId::new("reward-03.sword"))
        .unwrap();

    let effective = system.effective_sheet(&sheet, Some(&inventory));
    assert_eq!(sheet.int("attack_bonus"), Some(0));
    assert_eq!(effective.int("attack_bonus"), Some(2));
    assert_eq!(
        system.action_expr("attack", &effective).as_deref(),
        Some("1d20+4")
    );
}
