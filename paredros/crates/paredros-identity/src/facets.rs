// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Character and faction facets, kept separate.
//!
//! A role is held, not been. An office is a record in its own right, a holding
//! is a tenure with a start and an end, and both outlive the person. That is
//! half of what community continuity means: the office survives its holder, so
//! succession has something to succeed to.
//!
//! Nothing here is a universal character schema. Each facet is its own store
//! keyed by [`SubjectId`], and a person is the join, not a struct.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::{BodyRevisionId, FactionId, IdentityError, OfficeId, SubjectId, Tick};

/// A role that exists apart from whoever holds it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Office {
    pub id: OfficeId,
    pub faction: FactionId,
    pub name: String,
}

/// One person's tenure in one office. Appended, then closed by an `until`;
/// never rewritten, so the line of holders stays readable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Holding {
    pub office: OfficeId,
    pub holder: SubjectId,
    pub since: Tick,
    pub until: Option<Tick>,
}

impl Holding {
    pub fn current(&self) -> bool {
        self.until.is_none()
    }
}

/// The facet stores. Ordered containers throughout: iteration order is part of
/// the contract, because replays compare outcomes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Facets {
    bodies: BTreeMap<SubjectId, BodyRevisionId>,
    memberships: BTreeSet<(SubjectId, FactionId)>,
    offices: BTreeMap<OfficeId, Office>,
    holdings: Vec<Holding>,
}

impl Facets {
    pub fn new() -> Self {
        Self::default()
    }

    /// Points a subject at a body revision. Replacing the chassis replaces
    /// this and nothing else.
    pub fn wears(&mut self, subject: SubjectId, body: BodyRevisionId) {
        self.bodies.insert(subject, body);
    }

    pub fn body_of(&self, subject: SubjectId) -> Option<BodyRevisionId> {
        self.bodies.get(&subject).copied()
    }

    pub fn join(&mut self, subject: SubjectId, faction: FactionId) {
        self.memberships.insert((subject, faction));
    }

    pub fn leave(&mut self, subject: SubjectId, faction: FactionId) {
        self.memberships.remove(&(subject, faction));
    }

    pub fn belongs_to(&self, subject: SubjectId, faction: FactionId) -> bool {
        self.memberships.contains(&(subject, faction))
    }

    pub fn factions_of(&self, subject: SubjectId) -> Vec<FactionId> {
        self.memberships
            .iter()
            .filter(|(who, _)| *who == subject)
            .map(|(_, faction)| *faction)
            .collect()
    }

    pub fn found(&mut self, office: Office) -> Result<(), IdentityError> {
        if self.offices.contains_key(&office.id) {
            return Err(IdentityError::OfficeExists(office.id));
        }
        self.offices.insert(office.id, office);
        Ok(())
    }

    pub fn office(&self, id: OfficeId) -> Option<&Office> {
        self.offices.get(&id)
    }

    pub fn offices(&self) -> impl Iterator<Item = &Office> {
        self.offices.values()
    }

    /// Takes up an office. Refuses rather than displaces: a holder is vacated
    /// first, which is the transition worth recording.
    pub fn hold(
        &mut self,
        office: OfficeId,
        holder: SubjectId,
        since: Tick,
    ) -> Result<(), IdentityError> {
        if !self.offices.contains_key(&office) {
            return Err(IdentityError::NoSuchOffice(office));
        }
        if let Some(sitting) = self.holder_of(office) {
            return Err(IdentityError::OfficeHeld(office, sitting));
        }
        self.holdings.push(Holding {
            office,
            holder,
            since,
            until: None,
        });
        Ok(())
    }

    /// Closes the current tenure. The office record is untouched, which is the
    /// point: it is still there for the next holder.
    pub fn vacate(&mut self, office: OfficeId, at: Tick) -> Result<SubjectId, IdentityError> {
        if !self.offices.contains_key(&office) {
            return Err(IdentityError::NoSuchOffice(office));
        }
        let held = self
            .holdings
            .iter_mut()
            .rev()
            .find(|holding| holding.office == office && holding.current())
            .ok_or(IdentityError::OfficeVacant(office))?;
        held.until = Some(at);
        Ok(held.holder)
    }

    pub fn holder_of(&self, office: OfficeId) -> Option<SubjectId> {
        self.holdings
            .iter()
            .rev()
            .find(|holding| holding.office == office && holding.current())
            .map(|holding| holding.holder)
    }

    pub fn offices_held_by(&self, subject: SubjectId) -> Vec<OfficeId> {
        self.holdings
            .iter()
            .filter(|holding| holding.holder == subject && holding.current())
            .map(|holding| holding.office)
            .collect()
    }

    /// Every tenure of one office, oldest first. The line of holders.
    pub fn line_of(&self, office: OfficeId) -> Vec<Holding> {
        self.holdings
            .iter()
            .filter(|holding| holding.office == office)
            .copied()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WARDEN: OfficeId = OfficeId(1);
    const HOLD: FactionId = FactionId(1);
    const FIRST: SubjectId = SubjectId(1);
    const SECOND: SubjectId = SubjectId(2);

    fn wardenship() -> Facets {
        let mut facets = Facets::new();
        facets
            .found(Office {
                id: WARDEN,
                faction: HOLD,
                name: "warden".into(),
            })
            .unwrap();
        facets
    }

    #[test]
    fn an_office_survives_its_holder() {
        let mut facets = wardenship();
        facets.hold(WARDEN, FIRST, Tick(1)).unwrap();
        assert_eq!(facets.vacate(WARDEN, Tick(9)).unwrap(), FIRST);
        facets.hold(WARDEN, SECOND, Tick(9)).unwrap();

        assert_eq!(facets.holder_of(WARDEN), Some(SECOND));
        assert!(facets.office(WARDEN).is_some());
        let line = facets.line_of(WARDEN);
        assert_eq!(line.len(), 2);
        assert_eq!(line[0].holder, FIRST);
        assert_eq!(line[0].until, Some(Tick(9)));
    }

    #[test]
    fn an_office_is_vacated_and_not_displaced() {
        let mut facets = wardenship();
        facets.hold(WARDEN, FIRST, Tick(1)).unwrap();
        assert_eq!(
            facets.hold(WARDEN, SECOND, Tick(2)),
            Err(IdentityError::OfficeHeld(WARDEN, FIRST))
        );
    }

    #[test]
    fn the_facets_stay_four() {
        // A body swap moves nothing else, and a faction change moves nothing
        // else. That separation is the whole reason these are four stores.
        let mut facets = wardenship();
        facets.wears(FIRST, BodyRevisionId(7));
        facets.join(FIRST, HOLD);
        facets.hold(WARDEN, FIRST, Tick(1)).unwrap();

        facets.wears(FIRST, BodyRevisionId(8));
        assert!(facets.belongs_to(FIRST, HOLD));
        assert_eq!(facets.offices_held_by(FIRST), vec![WARDEN]);

        facets.leave(FIRST, HOLD);
        assert_eq!(facets.body_of(FIRST), Some(BodyRevisionId(8)));
        assert_eq!(facets.holder_of(WARDEN), Some(FIRST));
    }
}
