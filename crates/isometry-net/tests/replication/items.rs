//! W1: inventories replicate, and a hidden modifier does not.
//!
//! Players receive equipped public items; a generated curse waits for the DM's
//! reveal. A transfer moves an equipped item atomically.
//!
//! Split out of `replication.rs` on 2026-09-04; unchanged.

use super::*;

/// W1's visibility guard: players receive equipped public items, but a
/// generated curse does not enter their snapshots until the DM reveals it.
#[test]
fn hidden_item_modifiers_stay_private_until_dm_reveal() {
    const CURSE_NAME: &str = "VOID-THIRST-CURSE";

    let mut state = snapshot();
    state.inventories.insert(TokenId(1), sword_inventory());
    let mut host = HostSession::new(state);
    let hidden = HiddenItemModifier {
        id: "reward-03.sword.curse".to_owned(),
        item: ItemId::new("reward-03.sword"),
        modifier: ItemModifier {
            id: "reward-03.sword.curse.void-thirst".to_owned(),
            kind: ItemModifierKind::Curse,
            name: CURSE_NAME.to_owned(),
            stats: BTreeMap::from([("attack_bonus".to_owned(), -1)]),
            appearance_layer: Some("effect:void".to_owned()),
        },
        reveal: RevealCondition::Identify,
    };
    host.campaign_mut()
        .insert_hidden_item_modifier(hidden.clone());

    let mut sim = Sim::new(host);
    sim.connect(PeerId(10));
    sim.connect(PeerId(11));

    // A player gets the sword and its equipped slot, but cannot learn or
    // forge the still-private curse.
    for client in sim.clients.values() {
        let state = client.state().unwrap();
        assert_eq!(
            state.inventories[&TokenId(1)].equipped[&EquipmentSlot::MainHand],
            ItemId::new("reward-03.sword")
        );
        let bytes = postcard::to_allocvec(state).unwrap();
        assert!(!bytes
            .windows(CURSE_NAME.len())
            .any(|w| w == CURSE_NAME.as_bytes()));
    }
    sim.client_intent(
        PeerId(10),
        GameEvent::ItemModifierRevealed(hidden.public_face()),
    );
    sim.client_intent(
        PeerId(10),
        GameEvent::InventorySet {
            token: TokenId(1),
            inventory: Inventory::default(),
        },
    );
    sim.client_intent(
        PeerId(10),
        GameEvent::ItemTransfer {
            from: TokenId(1),
            to: TokenId(2),
            item: ItemId::new("reward-03.sword"),
        },
    );
    assert!(
        sim.host.state().inventories[&TokenId(1)].items[&ItemId::new("reward-03.sword")]
            .modifiers
            .is_empty()
    );
    assert!(sim.host.state().inventories[&TokenId(1)]
        .items
        .contains_key(&ItemId::new("reward-03.sword")));

    sim.host_reveal_item_modifier("reward-03.sword.curse")
        .expect("DM reveals the curse");
    assert!(sim.host.campaign().is_empty());
    for client in sim.clients.values() {
        let item = &client.state().unwrap().inventories[&TokenId(1)].items
            [&ItemId::new("reward-03.sword")];
        assert_eq!(item.modifiers[0].name, CURSE_NAME);
        assert_eq!(
            item.appearance_layers().collect::<Vec<_>>(),
            vec!["weapon:longsword", "effect:void"]
        );
    }
    assert_converged(&sim);
}

#[test]
fn host_transfers_an_equipped_item_atomically() {
    let mut state = snapshot();
    state.inventories.insert(TokenId(1), sword_inventory());
    let mut sim = Sim::new(HostSession::new(state));
    sim.connect(PeerId(10));
    sim.connect(PeerId(11));

    sim.host_event(GameEvent::ItemTransfer {
        from: TokenId(1),
        to: TokenId(2),
        item: ItemId::new("reward-03.sword"),
    });

    let source = &sim.host.state().inventories[&TokenId(1)];
    let target = &sim.host.state().inventories[&TokenId(2)];
    assert!(!source.items.contains_key(&ItemId::new("reward-03.sword")));
    assert!(source.equipped.is_empty());
    assert!(target.items.contains_key(&ItemId::new("reward-03.sword")));
    assert!(target.equipped.is_empty());
    assert_converged(&sim);
}

#[test]
fn restored_host_reconciles_a_pending_item_modifier_once() {
    let mut campaign = isometry_campaign::CampaignStore::new();
    let hidden = HiddenItemModifier {
        id: "reward-03.sword.curse".to_owned(),
        item: ItemId::new("reward-03.sword"),
        modifier: ItemModifier {
            id: "reward-03.sword.curse.void-thirst".to_owned(),
            kind: ItemModifierKind::Curse,
            name: "Void Thirst".to_owned(),
            stats: BTreeMap::new(),
            appearance_layer: None,
        },
        reveal: RevealCondition::Manual,
    };
    campaign.insert_hidden_item_modifier(hidden);
    campaign
        .begin_item_modifier_reveal("reward-03.sword.curse")
        .expect("modifier exists");

    let mut state = snapshot();
    state.inventories.insert(TokenId(1), sword_inventory());
    let mut host = HostSession::with_campaign(state, campaign);
    let out = host
        .reconcile_pending_reveals()
        .expect("reconciles pending modifier");
    assert_eq!(out.len(), 1);
    assert_eq!(host.history().len(), 1);
    assert_eq!(
        host.state().inventories[&TokenId(1)].items[&ItemId::new("reward-03.sword")]
            .modifiers
            .len(),
        1
    );
    assert!(host.campaign().is_empty());
}
