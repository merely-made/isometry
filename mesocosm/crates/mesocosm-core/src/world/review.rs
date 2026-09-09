// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The played line's own turn: what it could commit, what each would cost, and
//! what a founder under it would look like. (PE3b)
//!
//! [`World::adapt_round`](crate::World) skips the played line because its turn
//! is the review. This is the reading that turn is made of — the same
//! candidates, scored by the same function, priced by the same expression a
//! birth pays, previewed through the same realization. A panel draws it; it
//! decides nothing.
//!
//! # Every figure here is somebody else's, asked once
//!
//! Nothing below invents a number. The score is [`World::score`], which is what
//! an unplayed line's turn is decided by. The price is
//! [`Filial::cost_mg`](crate::Filial) off
//! [`program::express`](crate::program::express) — literally the milligrams the
//! next descendant will pay out of its reserve and into the ground under it.
//! The preview is [`Species::preview`](crate::Species). The refusal is
//! [`Unexpressed`], carried whole rather than restated. A review that computed
//! its own version of any of them would be a second authority over what a
//! candidate is worth.
//!
//! # The status quo is a row
//!
//! `candidate: None` is the founding revision, or whatever the line has already
//! committed: no change, scored the same way, priced at what a descendant is
//! already paying. *Nothing beat what I have* is therefore something the player
//! reads off the table rather than infers from an absence — the same rule
//! [`Turn::considered`](super::Turn) already keeps for an unplayed line.
//!
//! # A candidate that cannot be taken is the ordinary case
//!
//! PE2's residue, answered here: a bulk consumer has nowhere to put a gland
//! until its line grows a plate, so an offer carries [`Untakeable`] and stays on
//! the table. Hiding it would leave a player unable to tell *this world has
//! nothing for me* from *this body is the wrong shape for it, yet*.
//!
//! # The budget is the founder's material, and it is real
//!
//! **Ruled flat** (epoch boundary plan §8 q4, 2026-09-01): committing a
//! revision costs nothing, and the descendant pays the ordinary development
//! price. So the finite thing a review is spending against is not an invented
//! currency but [`Conditions::material_mg`] — what a founder of this line will
//! have banked when it is born, which is the ecology's own provisioning rule
//! (`parent.energy_mg.min(parent.biomass_mg() / OFFSPRING_COST)`) and the
//! number [`Unexpressed::Unaffordable`] already refuses against. The other
//! candidate, the line's living bodies' reserves summed, was not taken: no
//! rule anywhere spends it, so a budget read off it would be a figure the game
//! never charges.

use serde::{Deserialize, Serialize};

use crate::body::SpeciesId;
use crate::discovery::ConditionId;
use crate::organism::Organism;
use crate::organism::ecology::{BIRTH_SCATTER, OFFSPRING_COST};
use crate::program::{Citation, Conditions, DeclaredSite, Founder, Unexpressed};
use crate::species::Species;

use super::{Score, World};

/// Why an offered candidate cannot be taken up.
///
/// `None` beside an offer means it can. Every arm names a boundary that already
/// exists somewhere else, so a reason here and a refusal at the commit or the
/// birth are the same fact said once.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Untakeable {
    /// This world's ruleset does not hold the definition the discovery cites,
    /// or the candidate claims no tissue. [`World::revise`]'s own
    /// [`Unrevised::Nothing`](super::Unrevised::Nothing), asked before the
    /// commit rather than after it.
    Nothing,
    /// The line has no living body, so there is nothing to grow a founder from
    /// and nothing to found the reading on.
    Extinct,
    /// A founder of this line, grown under the declared conditions, could not
    /// express it. Carried whole from [`program::express`](crate::program::express),
    /// which is the one place that decides.
    Unexpressed(Unexpressed),
}

impl Untakeable {
    /// The refusal in the plain sentence a panel prints.
    pub fn words(&self) -> String {
        match self {
            Untakeable::Nothing => "this world does not hold what it needs".to_owned(),
            Untakeable::Extinct => "no body of your line is left to grow it".to_owned(),
            Untakeable::Unexpressed(why) => why.words(),
        }
    }
}

/// One row of the review: a candidate, what it came to, and what it costs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offer {
    /// `None` is the status quo — the revision the line is already born under,
    /// which is a real answer and therefore a real row.
    pub candidate: Option<ConditionId>,
    /// What growing it earned, scored exactly as an unplayed line's turn is.
    pub score: Score,
    /// What the filial expression would pay, in milligrams: the development
    /// price the next descendant of this line is charged at its birth. Zero
    /// under a program that declares nothing.
    pub price_mg: u64,
    /// The founder preview's body, as its digest — the same number
    /// [`Species::preview`](crate::Species) answers under these declared
    /// conditions.
    pub preview: u64,
    /// The program digest that preview was grown under. It moves when a line
    /// commits and never when a descendant realizes the same program
    /// differently, which is what makes a preview a prediction rather than a
    /// promise.
    pub program: u64,
    /// Why it cannot be taken, when it cannot.
    pub why_not: Option<Untakeable>,
}

impl Offer {
    /// Whether committing this would change anything and be admitted.
    pub fn takeable(&self) -> bool {
        self.candidate.is_some() && self.why_not.is_none()
    }

    /// Whether a founder could pay for it out of what it will be born holding.
    pub fn affordable(&self, budget_mg: u64) -> bool {
        self.price_mg <= budget_mg
    }
}

/// Everything one founder preview is grown from, read off a world.
///
/// **The founder the played line would bear next**, and not a fixture: the mass
/// and the material are the ecology's own provisioning arithmetic over a living
/// parent of the line, the ground is the poorest column a birth off that parent
/// could land on, the palette is the world's, and the seed is the one the birth
/// pass would hand the next id it allocates. Nothing here is a number this file
/// chose.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prospect {
    pub founder: Founder,
    /// The development seed a birth would realize under.
    pub seed: u64,
    /// What a founder will have banked to pay a development with. The review's
    /// budget, and [`Conditions::material_mg`] restated for a caller that wants
    /// the number without the context.
    pub budget_mg: u64,
}

/// The review: the played line's own turn, as a reading.
impl World {
    /// A number drawn off this world's own stream, without moving it.
    ///
    /// **Host-owned entropy for an authored proposal** (PD4): a script draws
    /// nothing of its own, so the host draws for it, and the draw has to come
    /// from the game's one seeded stream rather than a second generator. Taken
    /// on a copy of the state, so asking twice answers twice the same and the
    /// world's own sequence is untouched — which is what lets a review be built
    /// twice and compared.
    pub fn draw(&self) -> u64 {
        let mut stream = self.rng;
        stream.next_u64()
    }

    /// The body a founder of this line would be born from.
    ///
    /// The played critter when it is of this line, and otherwise the first
    /// living body of it in id order — the same rule
    /// [`World::candidates`](crate::World) picks with, so a review and a round
    /// reason about one body.
    fn parent_of(&self, species: SpeciesId) -> Option<&Organism> {
        self.controlled()
            .filter(|organism| organism.species == species)
            .or_else(|| self.living().find(|organism| organism.species == species))
    }

    /// The poorest column a birth off `parent` could land on.
    ///
    /// **The quote is a promise a birth has to be able to keep** (playable
    /// ecology plan PE3, ruled by Mark 2026-09-02). A preview used to declare
    /// the column the parent was standing on, and a body that had stood still
    /// for a hundred ticks had returned its upkeep into exactly that column —
    /// so the table quoted five cells of a site where the descendant, landing
    /// up to [`BIRTH_SCATTER`] voxels away on ordinary soil, could afford one.
    /// The dormancy rule was doing its job; the quote was still a number the
    /// game would not charge.
    ///
    /// So the declared ground is the **minimum** over the dispersal
    /// neighbourhood — the same square
    /// [`ecology::bear`](crate::organism::ecology::bear) scatters into, read
    /// through the same [`Soil::column_at`](crate::places::Soil::column_at),
    /// which clamps at the wall exactly as the birth's own clamp does. The
    /// quote is therefore at most what any birth in reach pays, and on uniform
    /// ground it is exactly that.
    ///
    /// Deterministic and a pure read: a fixed square walked in a fixed order,
    /// no entropy, nothing moved. It draws no sample of where the birth will
    /// *actually* land, because a preview that guessed the scatter would
    /// change every time the world's stream did.
    fn poorest_ground_in_reach(&self, parent: [i32; 3]) -> u64 {
        let mut poorest = u64::MAX;
        for dz in -BIRTH_SCATTER..=BIRTH_SCATTER {
            for dx in -BIRTH_SCATTER..=BIRTH_SCATTER {
                let at = [parent[0] + dx, parent[1], parent[2] + dz];
                let held = self.soil.matter_mg(self.soil.column_at(at));
                poorest = poorest.min(held);
            }
        }
        // Unreachable — the square always holds the parent's own column — and
        // written out rather than unwrapped so a degenerate radius reads as
        // bare ground instead of an absurd budget.
        if poorest == u64::MAX { 0 } else { poorest }
    }

    /// What the next founder of a line would be provisioned with.
    ///
    /// `None` when the line has no living body: there is nothing to bear one.
    pub fn prospect(&self, species: SpeciesId) -> Option<Prospect> {
        let parent = self.parent_of(species)?;
        // The ecology's own arithmetic, not a copy of it with a different name:
        // a child's body is a quarter of its parent's and its opening budget is
        // whatever the parent can actually hand over.
        let mass_mg = parent.biomass_mg() / OFFSPRING_COST;
        let material_mg = parent.energy_mg.min(mass_mg);
        let ground_mg = self.poorest_ground_in_reach(parent.position);
        Some(Prospect {
            founder: Founder {
                mass_mg,
                palette: self.development_palette,
                conditions: Conditions {
                    ground_mg,
                    material_mg,
                },
            },
            // The seed the birth pass would hand the next id it allocates.
            seed: crate::organism::ecology::filial_seed(
                parent.development_seed,
                crate::organism::OrganismId(self.next_organism),
            ),
            budget_mg: material_mg,
        })
    }

    /// What a founder of this line will have banked to pay a development with.
    ///
    /// The review's budget. Zero when the line has nothing living, which is
    /// the honest reading rather than an absent one.
    pub fn lineage_budget(&self, species: SpeciesId) -> u64 {
        self.prospect(species)
            .map(|prospect| prospect.budget_mg)
            .unwrap_or(0)
    }

    /// Every candidate this line could weigh, priced and previewed.
    ///
    /// The status quo first, then this world's discoveries the line does not
    /// already hold — **including the ones it cannot take**, each carrying its
    /// reason. [`World::candidates`](crate::World) is the round's list and it
    /// drops those; a review has to show them, because a candidate that cannot
    /// be taken yet is the ordinary case and the thing a player acts on next.
    ///
    /// Deterministic: every figure is a pure function of this world, the
    /// discovery order is the order they landed in, and the scoring copies are
    /// discarded. Building it twice gives the same rows.
    pub fn offers(&self, species: SpeciesId) -> Vec<Offer> {
        let mut offers = vec![self.offer(species, None)];
        let Some(line) = self.lineages.get(species) else {
            return offers;
        };
        let held = line
            .program()
            .current()
            .map(|revision| revision.cites.condition);
        for discovery in &self.discoveries {
            if Some(discovery.condition) == held {
                continue;
            }
            offers.push(self.offer(species, Some(discovery.condition)));
        }
        offers
    }

    /// One row.
    fn offer(&self, species: SpeciesId, candidate: Option<ConditionId>) -> Offer {
        // Scored first and unconditionally, through the same function the round
        // uses. An untakeable candidate scores the world as it stands — which
        // is truthful, and the reason beside it says why the figures did not
        // move.
        let score = self.score(species, candidate);
        let mut offer = Offer {
            candidate,
            score,
            price_mg: 0,
            preview: 0,
            program: 0,
            why_not: None,
        };

        let Some(line) = self.lineages.get(species) else {
            offer.why_not = Some(Untakeable::Extinct);
            return offer;
        };
        // The line as it would be if this were committed. A clone, because a
        // reading may not move a program: `Program::commit` appends, and the
        // copy is dropped with the row.
        let proposed = match candidate {
            None => line.clone(),
            Some(condition) => match self.proposed_line(line, condition) {
                Ok(proposed) => proposed,
                Err(why) => {
                    offer.why_not = Some(why);
                    return offer;
                },
            },
        };
        let Some(prospect) = self.prospect(species) else {
            offer.why_not = Some(Untakeable::Extinct);
            return offer;
        };
        let Ok(preview) = proposed.preview(self.ruleset(), prospect.founder, prospect.seed) else {
            offer.why_not = Some(Untakeable::Extinct);
            return offer;
        };

        offer.preview = preview.phenotype.digest();
        offer.program = preview.program;
        match preview.filial {
            // The founding revision declares nothing, so nothing is charged.
            None => {},
            Some(Ok(filial)) => offer.price_mg = filial.cost_mg,
            // A price the body could not meet is still a price, and it is the
            // one number that would let a player go and earn it.
            Some(Err(why @ Unexpressed::Unaffordable { needed_mg, .. })) => {
                offer.price_mg = needed_mg;
                offer.why_not = Some(Untakeable::Unexpressed(why));
            },
            Some(Err(why)) => offer.why_not = Some(Untakeable::Unexpressed(why)),
        }
        offer
    }

    /// The line as it would stand with this candidate committed.
    ///
    /// The same two refusals [`World::revise`](crate::World) gives, asked
    /// before a program is cloned — so a reason on a row and a rejection at the
    /// commit cannot disagree.
    fn proposed_line(&self, line: &Species, condition: ConditionId) -> Result<Species, Untakeable> {
        let discovery = self
            .discoveries
            .iter()
            .find(|discovery| discovery.condition == condition)
            .ok_or(Untakeable::Nothing)?;
        if discovery.candidate.cells == 0
            || self
                .ruleset()
                .resolve(discovery.candidate.process)
                .is_none()
        {
            return Err(Untakeable::Nothing);
        }
        let mut proposed = line.clone();
        proposed.commit(
            Citation::of(discovery),
            vec![DeclaredSite::of(&discovery.candidate)],
            self.tick,
        );
        Ok(proposed)
    }
}

#[cfg(test)]
mod tests;
