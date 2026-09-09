// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::PartId;
use paredros_identity::{BodyRevisionId, SubjectId};
use paredros_world::{fixtures::three_lives::wetland_body, *};

const SUBJECT: SubjectId = SubjectId(1);

fn setup(nested: bool) -> (GameState, Vec<ItemId>, [i32; 3]) {
    let mut game = GameState::new(World::generate(4242, WorldConfig::default()).unwrap());
    let at = game
        .items()
        .all()
        .find_map(|item| match item.location {
            ItemLocation::At(at)
                if item.kind == ItemKind::Dressing
                    && game
                        .world()
                        .ground()
                        .stands(at, mesocosm_core::places::WALKER_HEIGHT) =>
            {
                Some(at)
            },
            _ => None,
        })
        .unwrap();
    let items: Vec<_> = game
        .items()
        .at(at)
        .filter(|item| item.kind == ItemKind::Dressing)
        .map(|item| item.id)
        .collect();
    assert!(items.len() >= 2);
    game.apply(GameIntent::Generate {
        tick: game.next_tick(),
        subject: SUBJECT,
        body_seed: 7,
        at,
    })
    .unwrap();
    game.apply(GameIntent::Name {
        tick: game.next_tick(),
        subject: SUBJECT,
        name: Name::new("Keeper").unwrap(),
    })
    .unwrap();
    let mut document = wetland_body();
    if nested {
        document.parts[2].attachment.as_mut().unwrap().parent = PartId(1);
    }
    game.apply(GameIntent::AdmitAnatomy {
        tick: game.next_tick(),
        subject: SUBJECT,
        revision: BodyRevisionId(0),
        document: Box::new(document),
    })
    .unwrap();
    for item in &items {
        game.apply(GameIntent::Take {
            tick: game.next_tick(),
            subject: SUBJECT,
            item: *item,
        })
        .unwrap();
    }
    (game, items, at)
}

fn attach(game: &mut GameState, item: ItemId, part: u32, revision: u64) {
    game.apply(GameIntent::AttachItem {
        tick: game.next_tick(),
        subject: SUBJECT,
        item,
        part: PartId(part),
        revision: BodyRevisionId(revision),
    })
    .unwrap();
}

fn injure(game: &mut GameState) {
    game.apply(GameIntent::Fall {
        tick: game.next_tick(),
        subject: SUBJECT,
        distance: 5,
    })
    .unwrap();
}

fn reconcile(game: &mut GameState, from: u64, revision: u64, lost: Vec<PartId>) -> Vec<GameEvent> {
    game.apply(GameIntent::ReconcileAnatomy {
        tick: game.next_tick(),
        subject: SUBJECT,
        from_revision: BodyRevisionId(from),
        revision: BodyRevisionId(revision),
        severed_parts: lost,
    })
    .unwrap()
}

#[test]
fn attachment_refresh_release_and_replay_use_the_same_owned_items() {
    let (mut game, items, at) = setup(false);
    let mass = game.items().carried_mass_mg(SUBJECT);
    attach(&mut game, items[0], 1, 0);
    assert_eq!(game.items().carried_mass_mg(SUBJECT), mass);
    assert!(game.attachments(SUBJECT)[0].current);
    injure(&mut game);
    assert!(!game.attachments(SUBJECT)[0].current);
    let before = game.clone();
    assert!(
        game.apply(GameIntent::AttachItem {
            tick: game.next_tick(),
            subject: SUBJECT,
            item: items[1],
            part: PartId(2),
            revision: BodyRevisionId(1)
        })
        .is_err()
    );
    assert_eq!(game, before);
    // A stale body reference must never trap an item on the body.
    game.apply(GameIntent::DetachItem {
        tick: game.next_tick(),
        subject: SUBJECT,
        item: items[0],
    })
    .unwrap();
    assert_eq!(
        game.items().get(items[0]).unwrap().location,
        ItemLocation::Carried(SUBJECT)
    );
    reconcile(&mut game, 0, 1, vec![]);
    attach(&mut game, items[0], 1, 1);
    injure(&mut game);
    let events = reconcile(&mut game, 1, 2, vec![PartId(1)]);
    assert!(events.iter().any(|event| matches!(event, GameEvent::ItemReleased { item, at: dropped, .. } if *item == items[0] && *dropped == at)));
    assert_eq!(
        game.items().get(items[0]).unwrap().location,
        ItemLocation::At(at)
    );
    assert_eq!(
        game.items().carried_mass_mg(SUBJECT),
        mass - ItemKind::Dressing.mass_mg()
    );
    assert!(game.attachments(SUBJECT).is_empty());
    game.apply(GameIntent::Take {
        tick: game.next_tick(),
        subject: SUBJECT,
        item: items[0],
    })
    .unwrap();
    let before = game.clone();
    assert!(
        game.apply(GameIntent::AttachItem {
            tick: game.next_tick(),
            subject: SUBJECT,
            item: items[0],
            part: PartId(1),
            revision: BodyRevisionId(2)
        })
        .is_err()
    );
    assert_eq!(game, before);
    assert_eq!(GameState::restore(&game.save().unwrap()).unwrap(), game);
}

#[test]
fn invalid_reconciliation_is_atomic_and_parent_loss_releases_descendants_in_order() {
    let (mut game, items, at) = setup(true);
    attach(&mut game, items[0], 1, 0);
    attach(&mut game, items[1], 2, 0);
    injure(&mut game);
    for lost in [
        vec![PartId(0)],
        vec![PartId(99)],
        vec![PartId(1), PartId(1)],
    ] {
        let before = game.clone();
        assert!(
            game.apply(GameIntent::ReconcileAnatomy {
                tick: game.next_tick(),
                subject: SUBJECT,
                from_revision: BodyRevisionId(0),
                revision: BodyRevisionId(1),
                severed_parts: lost
            })
            .is_err()
        );
        assert_eq!(game, before);
    }
    let events = reconcile(&mut game, 0, 1, vec![PartId(1)]);
    let released: Vec<_> = events
        .iter()
        .filter_map(|event| {
            if let GameEvent::ItemReleased { item, .. } = event {
                Some(*item)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(released, items[..2]);
    assert!(
        game.current_anatomy(SUBJECT)
            .unwrap()
            .document
            .part(PartId(2))
            .unwrap()
            .severed
    );
    assert!(
        items[..2]
            .iter()
            .all(|id| game.items().get(*id).unwrap().location == ItemLocation::At(at))
    );
    assert_eq!(GameState::restore(&game.save().unwrap()).unwrap(), game);
}

#[test]
fn attached_dressing_consumption_clears_its_location() {
    let (mut game, items, _) = setup(false);
    attach(&mut game, items[0], 1, 0);
    injure(&mut game);
    game.apply(GameIntent::Rest {
        tick: game.next_tick(),
        subject: SUBJECT,
    })
    .unwrap();
    assert_eq!(
        game.items().get(items[0]).unwrap().location,
        ItemLocation::Consumed
    );
    assert!(game.attachments(SUBJECT).is_empty());
    assert_eq!(GameState::restore(&game.save().unwrap()).unwrap(), game);
}

#[test]
fn invalid_attachment_addresses_and_reconciliation_revisions_are_atomic() {
    let (mut game, items, _) = setup(false);
    for (part, revision) in [(99, 0), (1, 1)] {
        let before = game.clone();
        assert!(
            game.apply(GameIntent::AttachItem {
                tick: game.next_tick(),
                subject: SUBJECT,
                item: items[0],
                part: PartId(part),
                revision: BodyRevisionId(revision),
            })
            .is_err()
        );
        assert_eq!(game, before);
    }
    attach(&mut game, items[0], 1, 0);
    assert_eq!(GameState::restore(&game.save().unwrap()).unwrap(), game);
    injure(&mut game);
    for (from, revision) in [(0, 0), (0, 2), (1, 1)] {
        let before = game.clone();
        assert!(
            game.apply(GameIntent::ReconcileAnatomy {
                tick: game.next_tick(),
                subject: SUBJECT,
                from_revision: BodyRevisionId(from),
                revision: BodyRevisionId(revision),
                severed_parts: vec![PartId(1)],
            })
            .is_err()
        );
        assert_eq!(game, before);
    }
    assert_eq!(GameState::restore(&game.save().unwrap()).unwrap(), game);
}

#[test]
fn another_subject_cannot_detach_owned_equipment_and_old_save_versions_reject() {
    let (mut game, items, at) = setup(false);
    attach(&mut game, items[0], 1, 0);
    let peer = SubjectId(2);
    game.apply(GameIntent::Generate {
        tick: game.next_tick(),
        subject: peer,
        body_seed: 8,
        at,
    })
    .unwrap();
    game.apply(GameIntent::Name {
        tick: game.next_tick(),
        subject: peer,
        name: Name::new("Peer").unwrap(),
    })
    .unwrap();
    let before = game.clone();
    assert!(
        game.apply(GameIntent::DetachItem {
            tick: game.next_tick(),
            subject: peer,
            item: items[0]
        })
        .is_err()
    );
    assert_eq!(game, before);
    for version in [1, 2] {
        let mut save = game.save_record().unwrap();
        save.version = version;
        assert_eq!(
            GameState::restore_record(save),
            Err(GameError::VersionDiverged {
                saved: version,
                current: GAME_STATE_VERSION
            })
        );
    }
}
