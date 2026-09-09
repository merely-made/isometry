// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable things and where they are.

use std::collections::BTreeMap;

use mesocosm_core::PartId;
use paredros_identity::SubjectId;
use serde::{Deserialize, Serialize};

use crate::{Layer, SiteKind, World};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ItemId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemKind {
    Food,
    Dressing,
    Scrap,
}

impl ItemKind {
    pub const fn mass_mg(self) -> u32 {
        match self {
            Self::Food => 250,
            Self::Dressing => 100,
            Self::Scrap => 900,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemLocation {
    At([i32; 3]),
    Carried(SubjectId),
    Attached { subject: SubjectId, part: PartId },
    Consumed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub kind: ItemKind,
    pub location: ItemLocation,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Items {
    items: BTreeMap<ItemId, Item>,
}

impl Items {
    pub(crate) fn generate(world: &World) -> Self {
        let mut items = BTreeMap::new();
        let mut next = 0u32;
        for site in world.map().sites() {
            if site.slot.layer != Layer::Surface {
                continue;
            }
            let Some(place) = world.grown().places.get(site.slot.place) else {
                continue;
            };
            let [x, z] = place.centre;
            let Some(top) = world.ground().surface(x, z) else {
                continue;
            };
            let at = [x, top + 1, z];
            let mut kinds = vec![ItemKind::Food];
            match site.kind {
                SiteKind::Settlement => {
                    kinds.extend([
                        ItemKind::Food,
                        ItemKind::Food,
                        ItemKind::Dressing,
                        ItemKind::Dressing,
                    ]);
                },
                SiteKind::Ruin => kinds.push(ItemKind::Scrap),
                _ => {},
            }
            for kind in kinds {
                let id = ItemId(next);
                next += 1;
                items.insert(
                    id,
                    Item {
                        id,
                        kind,
                        location: ItemLocation::At(at),
                    },
                );
            }
        }
        Self { items }
    }

    pub fn get(&self, id: ItemId) -> Option<&Item> {
        self.items.get(&id)
    }

    pub fn all(&self) -> impl Iterator<Item = &Item> {
        self.items.values()
    }

    pub fn at(&self, position: [i32; 3]) -> impl Iterator<Item = &Item> {
        self.items
            .values()
            .filter(move |item| item.location == ItemLocation::At(position))
    }

    pub fn carried_by(&self, subject: SubjectId) -> impl Iterator<Item = &Item> {
        self.items.values().filter(move |item| {
            matches!(
                item.location,
                ItemLocation::Carried(owner)
                    | ItemLocation::Attached { subject: owner, .. }
                    if owner == subject
            )
        })
    }

    pub fn carried_mass_mg(&self, subject: SubjectId) -> u32 {
        self.carried_by(subject)
            .map(|item| item.kind.mass_mg())
            .sum()
    }

    pub(crate) fn take(&mut self, id: ItemId, subject: SubjectId) -> Result<(), ItemError> {
        let item = self.items.get_mut(&id).ok_or(ItemError::Missing(id))?;
        item.location = ItemLocation::Carried(subject);
        Ok(())
    }

    pub(crate) fn attach(
        &mut self,
        id: ItemId,
        subject: SubjectId,
        part: PartId,
    ) -> Result<(), ItemError> {
        let item = self.items.get_mut(&id).ok_or(ItemError::Missing(id))?;
        match item.location {
            ItemLocation::Carried(owner) if owner == subject => {},
            ItemLocation::Attached { .. } => return Err(ItemError::AlreadyAttached(id)),
            _ => return Err(ItemError::NotCarried(id, subject)),
        }
        if item.kind != ItemKind::Dressing {
            return Err(ItemError::WrongKind(id, item.kind));
        }
        item.location = ItemLocation::Attached { subject, part };
        Ok(())
    }

    pub(crate) fn detach(&mut self, id: ItemId, subject: SubjectId) -> Result<(), ItemError> {
        let item = self.items.get_mut(&id).ok_or(ItemError::Missing(id))?;
        match item.location {
            ItemLocation::Attached { subject: owner, .. } if owner == subject => {
                item.location = ItemLocation::Carried(subject);
                Ok(())
            },
            _ => Err(ItemError::NotCarried(id, subject)),
        }
    }

    pub(crate) fn release_attached(
        &mut self,
        subject: SubjectId,
        severed: &[PartId],
        at: [i32; 3],
    ) -> Vec<ItemId> {
        let mut released = Vec::new();
        for item in self.items.values_mut() {
            if let ItemLocation::Attached {
                subject: owner,
                part,
            } = item.location
            {
                if owner == subject && severed.contains(&part) {
                    released.push(item.id);
                    item.location = ItemLocation::At(at);
                }
            }
        }
        released
    }

    pub(crate) fn consume(&mut self, id: ItemId) -> Result<(), ItemError> {
        let item = self.items.get_mut(&id).ok_or(ItemError::Missing(id))?;
        item.location = ItemLocation::Consumed;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemError {
    Missing(ItemId),
    NotHere(ItemId),
    NotCarried(ItemId, SubjectId),
    WrongKind(ItemId, ItemKind),
    AlreadyAttached(ItemId),
    OverCapacity { capacity_mg: u32, attempted_mg: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    const ACTOR: SubjectId = SubjectId(7);
    const OTHER: SubjectId = SubjectId(8);

    fn items() -> Items {
        Items {
            items: BTreeMap::from([
                (
                    ItemId(1),
                    Item {
                        id: ItemId(1),
                        kind: ItemKind::Dressing,
                        location: ItemLocation::Carried(ACTOR),
                    },
                ),
                (
                    ItemId(2),
                    Item {
                        id: ItemId(2),
                        kind: ItemKind::Dressing,
                        location: ItemLocation::Carried(ACTOR),
                    },
                ),
                (
                    ItemId(3),
                    Item {
                        id: ItemId(3),
                        kind: ItemKind::Food,
                        location: ItemLocation::Carried(ACTOR),
                    },
                ),
                (
                    ItemId(4),
                    Item {
                        id: ItemId(4),
                        kind: ItemKind::Dressing,
                        location: ItemLocation::Carried(ACTOR),
                    },
                ),
            ]),
        }
    }

    #[test]
    fn attached_items_are_carried_once_and_mass_is_exact() {
        let mut items = items();
        items.attach(ItemId(1), ACTOR, PartId(2)).unwrap();
        assert_eq!(items.carried_by(ACTOR).count(), 4);
        assert_eq!(items.carried_mass_mg(ACTOR), 550);
    }

    #[test]
    fn attachment_rejects_wrong_kind_and_wrong_owner_atomically() {
        let mut items = items();
        assert_eq!(
            items.attach(ItemId(3), ACTOR, PartId(1)),
            Err(ItemError::WrongKind(ItemId(3), ItemKind::Food))
        );
        assert_eq!(
            items.get(ItemId(3)).unwrap().location,
            ItemLocation::Carried(ACTOR)
        );
        assert_eq!(
            items.attach(ItemId(1), OTHER, PartId(1)),
            Err(ItemError::NotCarried(ItemId(1), OTHER))
        );
        assert_eq!(
            items.get(ItemId(1)).unwrap().location,
            ItemLocation::Carried(ACTOR)
        );
        items.attach(ItemId(1), ACTOR, PartId(1)).unwrap();
        assert_eq!(
            items.attach(ItemId(1), ACTOR, PartId(2)),
            Err(ItemError::AlreadyAttached(ItemId(1)))
        );
    }

    #[test]
    fn detach_requires_same_owner_and_returns_to_carried() {
        let mut items = items();
        items.attach(ItemId(1), ACTOR, PartId(1)).unwrap();
        assert_eq!(
            items.detach(ItemId(1), OTHER),
            Err(ItemError::NotCarried(ItemId(1), OTHER))
        );
        items.detach(ItemId(1), ACTOR).unwrap();
        assert_eq!(
            items.get(ItemId(1)).unwrap().location,
            ItemLocation::Carried(ACTOR)
        );
    }

    #[test]
    fn release_is_deterministic_and_consume_clears_attachment() {
        let mut items = items();
        items.attach(ItemId(2), ACTOR, PartId(4)).unwrap();
        items.attach(ItemId(1), ACTOR, PartId(4)).unwrap();
        assert_eq!(
            items.release_attached(ACTOR, &[PartId(4)], [9, 8, 7]),
            vec![ItemId(1), ItemId(2)]
        );
        assert_eq!(
            items.get(ItemId(1)).unwrap().location,
            ItemLocation::At([9, 8, 7])
        );
        items.attach(ItemId(4), ACTOR, PartId(1)).unwrap();
        items.consume(ItemId(4)).unwrap();
        assert_eq!(
            items.get(ItemId(4)).unwrap().location,
            ItemLocation::Consumed
        );
    }
}
