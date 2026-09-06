//! World events, their errors, and the editable campaign draft.
//!
//! A draft is what a DM is still authoring; committing turns it into the
//! replicated events every peer folds.
//!
//! Split out of `world.rs` on 2026-07-24; behavior unchanged.

use std::collections::{BTreeMap, BTreeSet};

use super::*;

use crate::{CampaignMap, MapPoint, MapProposalError};
use isometry_core::{Facing, SheetData, Token, TokenId};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorldEvent {
    Faction(WorldFaction),
    Place(WorldPlace),
    Character(WorldCharacter),
    Route(WorldRoute),
    Law(WorldLaw),
    History(HistoryEvent),
    Storylet(StoryletProposal),
    /// Bind or update a faction's mutable numbers. Overwrites, because a
    /// faction's resources and banked time change as it acts.
    FactionSheet {
        faction: String,
        sheet: BTreeMap<String, i64>,
    },
    /// Grant (or, with `None`, revoke) a player's control of a faction's channel.
    /// The DM's ruling; every peer applies it, and a controlling player may then
    /// command the faction's tokens.
    FactionControlSet {
        faction: String,
        player: Option<String>,
    },
    /// Move a party (keyed by its owner) to an overmap node (a place id). The
    /// substrate records the position; adjacency and travel cost are the
    /// resolver's and the host's, not this event's.
    PartyMoved {
        party: String,
        node: String,
    },
    /// Set a party's travel pace, as a percent of normal time (100/50/200).
    PartyPaceSet {
        party: String,
        pace: i64,
    },
    /// Reveal an overmap place to a party (a rumour, a guide's directions, a map
    /// read the reader passed). The DM commits it; travel discovers on its own.
    NodeRevealed {
        party: String,
        node: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldError {
    MissingId,
    ConflictingId(String),
    UnknownRouteEndpoint(String),
    MissingStartingMap(String),
    DuplicateMap(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftMap {
    pub scale: MapScale,
    pub map: LocalMapProposal,
    /// Public people and creatures that begin on this generated map. Their
    /// numbers remain system-owned sheet fields; campaign only carries them.
    #[serde(default)]
    pub inhabitants: Vec<MapInhabitant>,
}

/// One generated token and its opaque system sheet. This is portable campaign
/// data, not a game-rule model: `stats` are named integers the selected system
/// may interpret after the campaign is accepted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapInhabitant {
    pub id: u32,
    pub name: String,
    pub sprite: String,
    pub at: MapPoint,
    pub system: String,
    pub stats: BTreeMap<String, i64>,
    #[serde(default)]
    pub owner: Option<String>,
}

impl DraftMap {
    /// Lower a portable campaign map into the substrate document that peers
    /// replicate. Inhabitants are validated before they are installed in the
    /// returned document, so a bad generated person cannot escape as a partial
    /// map.
    pub fn lower(&self) -> Result<CampaignMap, MapProposalError> {
        let mut lowered = self.map.lower(self.scale)?;
        self.validate_inhabitants()?;

        for inhabitant in &self.inhabitants {
            let id = TokenId(inhabitant.id);
            let mut sheet = SheetData::new(&inhabitant.system);
            for (key, value) in &inhabitant.stats {
                sheet.set_int(key, *value);
            }
            // A sheet's display name is ordinary text data. Keep it separate
            // from the generated numeric vocabulary even if a generator chose
            // the same key in its number map.
            sheet.set_text("name", &inhabitant.name);
            lowered.document.tokens.push(Token {
                id,
                at: (inhabitant.at.col as i32, inhabitant.at.row as i32),
                facing: Facing::South,
                sprite: inhabitant.sprite.clone(),
                owner: inhabitant.owner.clone(),
            });
            lowered.document.set_sheet(id, sheet);
        }
        Ok(lowered)
    }

    fn validate_inhabitants(&self) -> Result<(), MapProposalError> {
        let mut ids = BTreeSet::new();
        let mut positions = BTreeSet::new();
        for inhabitant in &self.inhabitants {
            if inhabitant.id == 0 {
                return Err(MapProposalError::InvalidInhabitantId);
            }
            if !ids.insert(inhabitant.id) {
                return Err(MapProposalError::DuplicateInhabitantId(inhabitant.id));
            }
            if inhabitant.name.trim().is_empty() {
                return Err(MapProposalError::MissingInhabitantField("name"));
            }
            if inhabitant.sprite.trim().is_empty() {
                return Err(MapProposalError::MissingInhabitantField("sprite"));
            }
            if inhabitant.system.trim().is_empty() {
                return Err(MapProposalError::MissingInhabitantField("system"));
            }
            if inhabitant.at.col >= self.map.width || inhabitant.at.row >= self.map.height {
                return Err(MapProposalError::OutOfBounds(inhabitant.at));
            }
            if !positions.insert((inhabitant.at.col, inhabitant.at.row)) {
                return Err(MapProposalError::DuplicateInhabitantPosition(inhabitant.at));
            }
        }
        Ok(())
    }
}

/// One host-private, inspectable proposal. Its public pieces lower into world
/// and map events; `secrets` lower only into the private campaign store.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignDraft {
    pub id: String,
    pub name: String,
    pub world: CampaignWorld,
    #[serde(default)]
    pub maps: Vec<DraftMap>,
    #[serde(default)]
    pub secrets: Vec<SecretFact>,
    #[serde(default)]
    pub rewards: Vec<ItemProposal>,
    pub starting_map: String,
    pub final_storylet: String,
}

impl CampaignDraft {
    pub fn validate(&self) -> Result<(), WorldError> {
        if self.id.trim().is_empty() {
            return Err(WorldError::MissingId);
        }
        if !self.maps.iter().any(|map| map.map.id == self.starting_map) {
            return Err(WorldError::MissingStartingMap(self.starting_map.clone()));
        }
        if !self.world.storylets.contains_key(&self.final_storylet) {
            return Err(WorldError::ConflictingId(self.final_storylet.clone()));
        }
        let mut rebuilt = CampaignWorld::default();
        for event in self.public_world_events() {
            rebuilt.apply(&event)?;
        }
        let mut map_ids = BTreeSet::new();
        for map in &self.maps {
            if !map_ids.insert(map.map.id.as_str()) {
                return Err(WorldError::DuplicateMap(map.map.id.clone()));
            }
            map.lower()
                .map_err(|_| WorldError::ConflictingId(map.map.id.clone()))?;
        }
        Ok(())
    }

    pub fn public_world_events(&self) -> Vec<WorldEvent> {
        self.world
            .factions
            .values()
            .cloned()
            .map(WorldEvent::Faction)
            .chain(self.world.places.values().cloned().map(WorldEvent::Place))
            .chain(
                self.world
                    .characters
                    .values()
                    .cloned()
                    .map(WorldEvent::Character),
            )
            .chain(self.world.routes.values().cloned().map(WorldEvent::Route))
            .chain(self.world.laws.values().cloned().map(WorldEvent::Law))
            .chain(self.world.history.iter().cloned().map(WorldEvent::History))
            .chain(
                self.world
                    .storylets
                    .values()
                    .cloned()
                    .map(WorldEvent::Storylet),
            )
            .collect()
    }
}
