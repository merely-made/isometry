// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The settlement, in S2's minimal form: dwellings, and the tenancies that
//! agreements create.
//!
//! **Residence is derived, never bookkept.** A tenancy records only which
//! dwelling, which tenant, which agreement put them there, and when. Where
//! someone lives *now* is folded from that record and the agreement's own
//! state: a tenancy is current exactly while its agreement stands. Ending the
//! agreement *is* moving out. There is no second copy of "who lives where" to
//! fall out of step with the arrangement that answers for it, the same shape
//! [`crate::Society`] gives standing.
//!
//! The daily round derives the same way: home from the current tenancy, work
//! from the agreement under it. An accepted agreement is therefore the one
//! thing that changes where someone lives and what they do daily, which is
//! S2's done-condition made structural.
//!
//! This is the settlement tested as **peer agency**, not as production. What
//! a dwelling yields, what the work produces, and who may make offers on the
//! settlement's behalf are S5's questions and are deliberately absent.

use std::collections::BTreeMap;

use paredros_identity::{SubjectId, Tick};
use serde::{Deserialize, Serialize};

use crate::agreement::AgreementId;
use crate::offer::{Offer, Work};
use crate::response::{Ruling, RulingKind};
use crate::society::{SocialError, Society};

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct DwellingId(pub u64);

/// A place someone could live. A record in its own right, like an office: it
/// outlives every tenant it ever has.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dwelling {
    pub id: DwellingId,
    pub name: String,
}

/// One person's residence in one dwelling, and the agreement that put them
/// there. Appended and never rewritten: the line of tenants stays readable,
/// and `under` is the trace from a home back to the bargain it rests on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tenancy {
    pub dwelling: DwellingId,
    pub tenant: SubjectId,
    pub under: AgreementId,
    pub since: Tick,
}

/// Where someone wakes and what they do, read at a moment. Both are `None`
/// for anyone the settlement has not housed, which is every peer before an
/// offer and every past tenant after their arrangement ends.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DailyRound {
    pub home: Option<DwellingId>,
    pub does: Option<Work>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SettleError {
    Social(SocialError),
    NoSuchDwelling(DwellingId),
    /// Someone already lives there under a standing agreement. Dwellings are
    /// vacated by an ending, not displaced by a newer offer.
    Occupied(DwellingId, SubjectId),
}

impl From<SocialError> for SettleError {
    fn from(err: SocialError) -> Self {
        Self::Social(err)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settlement {
    dwellings: BTreeMap<DwellingId, Dwelling>,
    tenancies: Vec<Tenancy>,
}

impl Settlement {
    pub fn new() -> Self {
        Self::default()
    }

    /// Raises a dwelling. Founding is a world act rather than a deed done to
    /// a person, so it moves nobody's standing.
    pub fn found(&mut self, dwelling: Dwelling) {
        self.dwellings.insert(dwelling.id, dwelling);
    }

    pub fn dwelling(&self, id: DwellingId) -> Option<&Dwelling> {
        self.dwellings.get(&id)
    }

    pub fn dwellings(&self) -> impl Iterator<Item = &Dwelling> {
        self.dwellings.values()
    }

    /// Every tenancy ever, oldest first. History, not occupancy: use
    /// [`Settlement::tenant_of`] for who lives somewhere now.
    pub fn tenancies(&self) -> &[Tenancy] {
        &self.tenancies
    }

    /// Offers someone housing and work in one arrangement. The answer is
    /// theirs, weighed exactly as any other offer; a tenancy exists only if
    /// the agreement formed, so a home nobody agreed to cannot be created.
    pub fn offer_home(
        &mut self,
        society: &mut Society,
        offer: &Offer,
        dwelling: DwellingId,
        at: Tick,
    ) -> Result<Ruling, SettleError> {
        if !self.dwellings.contains_key(&dwelling) {
            return Err(SettleError::NoSuchDwelling(dwelling));
        }
        if let Some(sitting) = self.tenant_of(society, dwelling) {
            return Err(SettleError::Occupied(dwelling, sitting));
        }

        let ruling = society.form(offer, at)?;
        if let RulingKind::Formed(id) = ruling.kind {
            self.tenancies.push(Tenancy {
                dwelling,
                tenant: offer.asked_of,
                under: id,
                since: at,
            });
        }
        Ok(ruling)
    }

    /// The tenancy someone currently lives under: the newest whose agreement
    /// still stands.
    fn current_of(&self, society: &Society, subject: SubjectId) -> Option<&Tenancy> {
        self.tenancies
            .iter()
            .rev()
            .filter(|tenancy| tenancy.tenant == subject)
            .find(|tenancy| Self::stands(society, tenancy))
    }

    fn stands(society: &Society, tenancy: &Tenancy) -> bool {
        society
            .agreement(tenancy.under)
            .is_some_and(|agreement| agreement.standing())
    }

    /// Where someone lives, or `None` for the unhoused and the moved-out.
    pub fn home_of(&self, society: &Society, subject: SubjectId) -> Option<DwellingId> {
        self.current_of(society, subject)
            .map(|tenancy| tenancy.dwelling)
    }

    /// Who lives somewhere, or `None` for a vacant dwelling.
    pub fn tenant_of(&self, society: &Society, dwelling: DwellingId) -> Option<SubjectId> {
        self.tenancies
            .iter()
            .rev()
            .filter(|tenancy| tenancy.dwelling == dwelling)
            .find(|tenancy| Self::stands(society, tenancy))
            .map(|tenancy| tenancy.tenant)
    }

    /// Where someone wakes and what they do daily, both read through the
    /// agreement that houses them.
    pub fn daily(&self, society: &Society, subject: SubjectId) -> DailyRound {
        let Some(tenancy) = self.current_of(society, subject) else {
            return DailyRound::default();
        };
        DailyRound {
            home: Some(tenancy.dwelling),
            does: society
                .agreement(tenancy.under)
                .map(|agreement| agreement.work),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unfounded_dwelling_cannot_be_offered() {
        let mut settlement = Settlement::new();
        let mut society = Society::new();
        let offer = Offer::new(
            SubjectId(1),
            SubjectId(2),
            Work::new(crate::companion::Craft::Hauling, 1, 0),
            crate::offer::Terms::new(1, 1),
        );
        assert_eq!(
            settlement.offer_home(&mut society, &offer, DwellingId(9), Tick(1)),
            Err(SettleError::NoSuchDwelling(DwellingId(9)))
        );
    }

    #[test]
    fn nobody_lives_anywhere_until_an_agreement_says_so() {
        let mut settlement = Settlement::new();
        settlement.found(Dwelling {
            id: DwellingId(1),
            name: "the gatehouse".into(),
        });
        let society = Society::new();
        assert_eq!(settlement.home_of(&society, SubjectId(2)), None);
        assert_eq!(settlement.tenant_of(&society, DwellingId(1)), None);
        assert_eq!(
            settlement.daily(&society, SubjectId(2)),
            DailyRound::default()
        );
    }
}
