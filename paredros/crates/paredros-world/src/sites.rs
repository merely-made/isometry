// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use mesocosm_core::places::{Grown, PlaceId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Layer {
    Surface,
    Underground,
}

/// A structural address. Its occupant may change without changing its
/// containment or routes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SlotId {
    pub place: PlaceId,
    pub layer: Layer,
}

impl SlotId {
    pub const fn surface(place: PlaceId) -> Self {
        Self {
            place,
            layer: Layer::Surface,
        }
    }

    pub const fn underground(place: PlaceId) -> Self {
        Self {
            place,
            layer: Layer::Underground,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HistoryFactId(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteKind {
    Wilds,
    Settlement,
    Ruin,
    Encounter,
    Dungeon,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SiteSource {
    Generated,
    Inherited(HistoryFactId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Site {
    pub slot: SlotId,
    pub kind: SiteKind,
    pub source: SiteSource,
    pub parent: Option<SlotId>,
}

/// Paredros's projection of generated topology. Routes and containment are
/// keyed by slots, so changing a site's meaning cannot move it accidentally.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldMap {
    sites: BTreeMap<SlotId, Site>,
    routes: BTreeMap<SlotId, BTreeSet<SlotId>>,
}

impl WorldMap {
    pub(crate) fn generate(grown: &Grown) -> Self {
        let settlement = grown
            .places
            .all()
            .map(|place| place.id)
            .min()
            .expect("world configuration guarantees places");
        let ruin = grown
            .places
            .all()
            .map(|place| place.id)
            .filter(|place| *place != settlement)
            .max_by_key(|place| (grown.places.hops(settlement, *place).unwrap_or(0), *place))
            .expect("world configuration guarantees several places");
        let encounter = grown
            .places
            .all()
            .map(|place| place.id)
            .filter(|place| *place != settlement && *place != ruin)
            .min_by_key(|place| (Reverse(grown.places.neighbours(*place).len()), *place))
            .expect("world configuration guarantees several places");

        let mut sites = BTreeMap::new();
        let mut routes = BTreeMap::new();
        for place in grown.places.all() {
            let slot = SlotId::surface(place.id);
            let kind = if place.id == settlement {
                SiteKind::Settlement
            } else if place.id == ruin {
                SiteKind::Ruin
            } else if place.id == encounter {
                SiteKind::Encounter
            } else {
                SiteKind::Wilds
            };
            sites.insert(
                slot,
                Site {
                    slot,
                    kind,
                    source: SiteSource::Generated,
                    parent: None,
                },
            );
            routes.entry(slot).or_insert_with(BTreeSet::new);
        }

        for place in grown.places.all() {
            let from = SlotId::surface(place.id);
            for neighbour in grown.places.neighbours(place.id) {
                Self::link(&mut routes, from, SlotId::surface(*neighbour));
            }
        }

        for nest in &grown.nests {
            let parent = SlotId::surface(nest.host);
            let slot = SlotId::underground(nest.host);
            sites.insert(
                slot,
                Site {
                    slot,
                    kind: SiteKind::Dungeon,
                    source: SiteSource::Generated,
                    parent: Some(parent),
                },
            );
            Self::link(&mut routes, slot, parent);
        }

        Self { sites, routes }
    }

    fn link(routes: &mut BTreeMap<SlotId, BTreeSet<SlotId>>, left: SlotId, right: SlotId) {
        routes.entry(left).or_default().insert(right);
        routes.entry(right).or_default().insert(left);
    }

    pub fn sites(&self) -> impl Iterator<Item = &Site> {
        self.sites.values()
    }

    pub fn site(&self, slot: SlotId) -> Option<&Site> {
        self.sites.get(&slot)
    }

    pub fn neighbours(&self, slot: SlotId) -> Option<&BTreeSet<SlotId>> {
        self.routes.get(&slot)
    }

    pub fn slots_of_kind(&self, kind: SiteKind) -> impl Iterator<Item = SlotId> + '_ {
        self.sites
            .values()
            .filter(move |site| site.kind == kind)
            .map(|site| site.slot)
    }

    pub(crate) fn inherit(&mut self, slot: SlotId, kind: SiteKind, fact: HistoryFactId) -> bool {
        let Some(site) = self.sites.get_mut(&slot) else {
            return false;
        };
        site.kind = kind;
        site.source = SiteSource::Inherited(fact);
        true
    }

    /// A deterministic shortest route over structural slots.
    pub fn route(&self, from: SlotId, to: SlotId) -> Option<Vec<SlotId>> {
        if !self.sites.contains_key(&from) || !self.sites.contains_key(&to) {
            return None;
        }
        let mut previous = BTreeMap::new();
        let mut seen = BTreeSet::from([from]);
        let mut queue = VecDeque::from([from]);
        while let Some(at) = queue.pop_front() {
            if at == to {
                let mut path = vec![to];
                let mut cursor = to;
                while let Some(parent) = previous.get(&cursor) {
                    path.push(*parent);
                    cursor = *parent;
                }
                path.reverse();
                return Some(path);
            }
            for next in self.routes.get(&at).into_iter().flatten() {
                if seen.insert(*next) {
                    previous.insert(*next, at);
                    queue.push_back(*next);
                }
            }
        }
        None
    }
}
