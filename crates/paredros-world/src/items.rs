// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable things and where they are.

use std::collections::BTreeMap;

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
        self.items
            .values()
            .filter(move |item| item.location == ItemLocation::Carried(subject))
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
    OverCapacity { capacity_mg: u32, attempted_mg: u32 },
}
