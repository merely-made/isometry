// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a host may read off a world.
//!
//! Split out of `world.rs` at the 600-line ceiling. Nothing here mutates
//! anything, which is the whole point of gathering it: a host projects a world
//! and never changes one, so the surface it is allowed to touch is worth being
//! able to see in one place.

use std::collections::{BTreeMap, BTreeSet};

use crate::body::{Aabb, BodyDocument, SpeciesId};
use crate::organism::{Organism, OrganismId};
use crate::places::{PlaceId, Places};
use crate::process::Unmet;
use crate::record::WorldRecord;

use super::{Ineligible, World};

impl World {
    /// Which organism the player is, if any.
    pub fn controlled_id(&self) -> Option<OrganismId> {
        self.controlled
    }

    /// The played organism.
    ///
    /// `None` when nobody is embodied, **and also when the subject is no
    /// longer alive**. A carcass is not a critter you can play: natural death
    /// leaves an organism in the roster as carrion until it is spent, and an
    /// earlier cut checked only that the id was still present, so a dead
    /// critter kept moving and eating while it decomposed.
    pub fn controlled(&self) -> Option<&Organism> {
        let id = self.controlled?;
        self.organisms.iter().find(|o| o.id == id && o.is_alive())
    }

    /// How many ticks running the player has done nothing.
    pub fn idle_run(&self) -> u32 {
        self.idle_run
    }

    /// The critter a hand is currently on, if any hand is.
    ///
    /// **Control and holding are not the same thing.** `controlled` says whose
    /// body a key would move; this says whether anybody has moved it lately.
    /// Past [`INSTINCT_IDLE_TICKS`] of unbroken idling the answer is `None`
    /// while control stays exactly where it was: the critter goes back to its
    /// own drives, the next keypress takes it back mid-stride, and nothing had
    /// to be handed over or reclaimed.
    ///
    /// [`INSTINCT_IDLE_TICKS`]: super::INSTINCT_IDLE_TICKS
    pub fn held(&self) -> Option<OrganismId> {
        if self.idle_run >= super::INSTINCT_IDLE_TICKS {
            return None;
        }
        self.controlled().map(|organism| organism.id)
    }

    /// Whether the played critter's budget has fallen far enough that a meal
    /// burns rather than builds. `false` with nobody embodied: a world with no
    /// hand in it is not hungry, it is empty.
    ///
    /// Public because the surface that shows the state should read the same
    /// predicate the rule uses, rather than re-deriving it from the number.
    pub fn is_starved(&self) -> bool {
        self.controlled()
            .is_some_and(|organism| organism.budget_below(super::STARVED_UPKEEP_TICKS))
    }

    pub(super) fn controlled_mut(&mut self) -> Option<&mut Organism> {
        let id = self.controlled?;
        self.organisms
            .iter_mut()
            .find(|o| o.id == id && o.is_alive())
    }

    /// Whether a given organism could be played.
    pub fn is_eligible(&self, organism: OrganismId) -> bool {
        self.eligibility(organism).is_ok()
    }

    /// How elaborate an organism's lineage is, as the frontier reads it.
    ///
    /// The **recipe's** intricacy, not the body's part count: repetition is
    /// cheap and vocabulary is expensive, so reaching a new lineage means
    /// reaching a richer recipe rather than a longer creature. Falls back to
    /// anatomy for an organism whose lineage the registry has lost, which a
    /// consistent world never has.
    pub fn intricacy(&self, organism: &Organism) -> i32 {
        self.lineages
            .get(organism.species)
            .map(|species| species.recipe.complexity() as i32)
            .unwrap_or_else(|| organism.complexity())
    }

    /// Whether an organism could be played, and why not if not.
    ///
    /// **This is where the complexity frontier finally binds.** The rule was
    /// ruled long ago -- an unlocked lineage must be *more* metabolically
    /// complex than the target, so stepping downward into a newly viable niche
    /// is the point and minting an unearned peer at the frontier is not -- and
    /// until then it lived in the deleted `epoch::can_switch_to`, which nothing
    /// outside its own tests called. Control could take anything alive, however
    /// elaborate.
    ///
    /// It binds here rather than there because that function reasoned over the
    /// provisional scalar lineage and its trait array, while control reasons
    /// over organisms. Since P1 every organism has a body, so complexity can
    /// come from anatomy and the two models no longer have to agree first. The
    /// module it lived in was deleted on 2026-09-04 (phenotype plan §D4), so
    /// this is now the only statement of the rule anywhere.
    pub fn eligibility(&self, organism: OrganismId) -> Result<(), Ineligible> {
        let Some(target) = self.organisms.iter().find(|o| o.id == organism) else {
            return Err(Ineligible::NoSuchOrganism);
        };
        if !target.is_alive() {
            return Err(Ineligible::NotAlive);
        }
        // A line you have already lived is always yours to return to. The
        // frontier gates reaching *outward*, not going home.
        if self.unlocked.contains(&target.species) {
            return Ok(());
        }

        let frontier = self.frontier();
        let reach = self.intricacy(target);
        if reach < frontier {
            Ok(())
        } else {
            Err(Ineligible::AboveTheFrontier {
                frontier,
                target: reach,
            })
        }
    }

    /// Who could carry the line on from `of`: its living descendants, eldest
    /// first, that this world would let anyone inhabit.
    ///
    /// **The succession path's one new question**, and it is answered by the
    /// two authorities that already exist rather than a third. Descent comes
    /// out of the past, because `Event::Born` has always carried its parent;
    /// eligibility comes out of [`Self::eligibility`], because control has one
    /// gate and a descendant does not get a private one. The past is a
    /// parameter for the reason [`reckon`] takes one: a world can say what
    /// is, never what happened.
    ///
    /// This is a **reading**, not a roster: it names living organisms that go
    /// on eating, breeding and dying whether or not anyone looks at them, and
    /// nothing here reserves, removes or freezes one. Siblings stay in the
    /// ecology. (PE1.)
    ///
    /// [`reckon`]: World::reckon
    pub fn heirs(&self, history: &crate::history::History, of: OrganismId) -> Vec<OrganismId> {
        history
            .descendants(of)
            .into_iter()
            .filter(|heir| self.is_eligible(*heir))
            .collect()
    }

    /// The most elaborate thing the player has ever held.
    ///
    /// The ceiling a new lineage must sit below, and it only goes up.
    pub fn frontier(&self) -> i32 {
        self.frontier
    }

    /// Which lineages exist, and how they are related.
    pub fn lineages(&self) -> &crate::species::Lineages {
        &self.lineages
    }

    /// The lineage registry, mutably. The adaptation phase edits recipes
    /// through here; ordinary play does not.
    pub fn lineages_mut(&mut self) -> &mut crate::species::Lineages {
        &mut self.lineages
    }

    /// Part vocabulary this world uses when a lineage realizes a body.
    pub fn development_palette(&self) -> crate::development::PartPalette {
        self.development_palette
    }

    /// How far apart two creatures' ancestries are, in forks.
    ///
    /// One of the axes graft compatibility was ruled to scale with. It was
    /// uncomputable until lineages could split: every pair was identical.
    pub fn kinship(&self, a: OrganismId, b: OrganismId) -> Option<u32> {
        let species = |id: OrganismId| {
            self.organisms
                .iter()
                .find(|o| o.id == id)
                .map(|o| o.species)
        };
        self.lineages.distance(species(a)?, species(b)?)
    }

    /// Lineages the player has inhabited.
    pub fn unlocked(&self) -> impl Iterator<Item = SpeciesId> + '_ {
        self.unlocked.iter().copied()
    }

    /// The enclosure, divided into regions.
    /// The ground: brick truth for occupancy, sight, and carving.
    pub fn ground(&self) -> &crate::places::Ground {
        &self.ground
    }

    /// Takes the bricks changed since the last drain, for a projection's
    /// region-upload discipline.
    ///
    /// `&mut` and yet not a world change: `Ground::dirty` is `serde(skip)`
    /// and outside `Ground`'s `PartialEq`, so draining alters neither the
    /// snapshot nor the replay hash. Hosts drain at their own frame rate;
    /// the revision and the brick bytes carry the authoritative change.
    pub fn drain_ground_dirty(&mut self) -> Vec<[i16; 3]> {
        self.ground.drain_dirty()
    }

    /// Takes the events of the most recent tick, leaving the world empty of
    /// them. Recording them is a caller's business; the world only reports.
    pub fn drain_events(&mut self) -> Vec<crate::flow::RecordedEvent> {
        std::mem::take(&mut self.pending)
    }

    /// The events of the most recent tick, without taking them.
    pub fn events(&self) -> &[crate::flow::RecordedEvent] {
        &self.pending
    }

    /// Takes the matter movements of the most recent tick.
    ///
    /// **`&mut` and yet not a world change**, exactly as
    /// [`Self::drain_ground_dirty`] is: the ledger is `serde(skip)` and its
    /// equality is unconditional, so a run that reduces readings every tick and
    /// one that never looks are the same world and the same state hash. That is
    /// what keeps a dense presentation stream out of the snapshot (PE0).
    ///
    /// The buffer is reopened at the top of each tick, so this returns that
    /// tick's flows whether or not the last one was ever taken.
    pub fn drain_flows(&mut self) -> Vec<crate::flow::RecordedFlow> {
        self.flows.take()
    }

    /// The most recent tick's matter movements, without taking them.
    pub fn flows(&self) -> &[crate::flow::RecordedFlow] {
        self.flows.records()
    }

    /// Which region an act by `actor` happened in, read after it resolved.
    ///
    /// Every act event names the played critter or a body standing where it
    /// stands, so one lookup answers for all of them. `None` when the actor no
    /// longer exists, which is honest: nothing can say where it was.
    pub(super) fn acted_at(&self, actor: Option<OrganismId>) -> Option<PlaceId> {
        let actor = actor?;
        let position = self.organisms.iter().find(|o| o.id == actor)?.position;
        self.places.at(position)
    }

    /// The enclosure's matter store.
    ///
    /// Read-only from outside: matter moves through the tick and the recorded
    /// intents, never by a host reaching in. Its total plus every organism's
    /// substance and reserve is the conserved quantity TD6 rests on, which is
    /// what a conservation test reads through here.
    pub fn soil(&self) -> &crate::places::Soil {
        &self.soil
    }

    /// Every milligram in the enclosure: ground, living substance, carrion,
    /// and banked reserves.
    ///
    /// **The invariant.** It is constant across a run from genesis onward,
    /// because matter has nowhere else to be — light is the only open input
    /// and light is not matter.
    pub fn total_matter_mg(&self) -> u64 {
        self.soil.total_mg()
            + self
                .organisms
                .iter()
                .map(|o| o.biomass_mg() + o.energy_mg)
                .sum::<u64>()
    }

    pub fn places(&self) -> &Places {
        &self.places
    }

    /// Which region the played critter is in.
    pub fn place(&self) -> Option<PlaceId> {
        self.places.at(self.position()?)
    }

    /// The current far-tier population projection. Cohorts conserve their
    /// member count and scalar matter; near bodies are intentionally absent.
    pub fn far_cohorts(&self) -> Vec<crate::cohort::Cohort> {
        crate::cohort::from_organisms(&self.organisms, &self.places)
    }

    /// Everywhere each lineage has been.
    pub fn ranges(&self) -> &BTreeMap<SpeciesId, BTreeSet<PlaceId>> {
        &self.ranges
    }

    /// Everywhere one lineage has been.
    pub fn range(&self, species: SpeciesId) -> BTreeSet<PlaceId> {
        self.ranges.get(&species).cloned().unwrap_or_default()
    }

    /// What this world has seen anyone do.
    pub fn record(&self) -> &WorldRecord {
        &self.record
    }

    /// The played critter's anatomy.
    pub fn body(&self) -> Option<&BodyDocument> {
        self.controlled().map(|o| o.body())
    }

    /// The played critter's anatomy **and its allocation**.
    ///
    /// Added for PE2's inspector and PD2's receipt: `body()` answers what a
    /// creature is made of, and this answers what that tissue is doing.
    pub fn phenotype(&self) -> Option<&crate::phenotype::BodyPhenotype> {
        self.controlled().map(|o| &o.phenotype)
    }

    /// PD2's process, in the four states a receipt has to tell apart.
    ///
    /// `None` when the played critter has never had one, which is every body
    /// a world founds — nothing seeds a gland. The other three states are all
    /// `Some`, and are told apart by the fields: allocated (`sites` non-empty),
    /// charged or dry (`charged`), and lost with its branch (`sites` empty and
    /// `lost` not).
    pub fn gland(&self) -> Option<Gland> {
        let me = self.controlled()?;
        let sites = me.phenotype.glands();
        let lost = me.phenotype.lost_glands();
        if sites.is_empty() && lost.is_empty() {
            return None;
        }
        let potency_mg = me.phenotype.secretory_mg();
        let ground_mg = self.soil.matter_mg(self.soil.column_at(me.position));
        Some(Gland {
            // What carrying it costs every tick: the difference between this
            // body's rent and the rent the same body would pay without it.
            // A difference rather than a term, because the term is inside one
            // integer division and a player is owed the number they actually
            // pay.
            rent_mg: me
                .upkeep_mg()
                .saturating_sub(crate::organism::ecology::upkeep_for_body(
                    me.biomass_mg(),
                    me.actuator_span(),
                    me.mass_ceiling_mg(),
                    0,
                )),
            cells: sites.iter().map(|(_, cells)| cells).sum(),
            sites,
            lost,
            potency_mg,
            ground_mg,
            charged: me.charged_mg(ground_mg) > 0,
        })
    }

    /// Where the played critter is.
    pub fn position(&self) -> Option<[i32; 3]> {
        self.controlled().map(|o| o.position)
    }

    /// The played critter's budget.
    pub fn energy_mg(&self) -> Option<u64> {
        self.controlled().map(|o| o.energy_mg)
    }

    /// Whether anyone is being played.
    pub fn is_embodied(&self) -> bool {
        self.controlled().is_some()
    }

    /// Who the world stopped being able to play on the most recent tick.
    ///
    /// The seam the design calls for: losing a body is where witnessing, world
    /// examination, adaptation, and choosing another eligible critter happen.
    pub fn control_lost(&self) -> Option<OrganismId> {
        self.control_lost
    }

    /// Whether the played critter could touch `target`, and why not if not.
    ///
    /// **Anatomy answers this now.** It used to be `REACH = 8` regardless of
    /// what a critter was shaped like, so a twelve-limbed creature and a cube
    /// reached exactly as far.
    pub(super) fn reach_to(&self, target: [i32; 3]) -> Result<(), Unmet> {
        let Some(me) = self.controlled() else {
            return Err(Unmet::TooFar {
                reach: 0,
                distance: i32::MAX,
            });
        };
        let distance = (0..3)
            .map(|axis| (target[axis] - me.position[axis]).abs())
            .max()
            .unwrap_or(0);
        me.body().can_reach(distance)
    }

    /// Whether the played critter can touch a point. Anatomy decides.
    pub fn in_reach(&self, target: [i32; 3]) -> bool {
        self.within_reach(target)
    }

    /// How far the played critter can touch, from its anatomy.
    pub fn reach(&self) -> i32 {
        self.body().map(|b| b.reach()).unwrap_or(0)
    }

    fn within_reach(&self, target: [i32; 3]) -> bool {
        self.reach_to(target).is_ok()
    }

    /// The played body's collision extent in body space, when there is one.
    pub fn collision(&self) -> Option<Aabb> {
        self.body().map(|b| b.aabb())
    }

    pub fn total_mass_mg(&self) -> u64 {
        self.body().map(|b| b.total_mass_mg()).unwrap_or(0)
    }
}

/// PD2's played process, read off a body and the ground it is standing on.
///
/// **A reading, not state.** Nothing here is stored, enters the trace, or
/// reaches the state hash: the tissue is in the mosaic, the ground is in the
/// soil, and this is what you get when you ask both at once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gland {
    /// The living parts carrying secretory tissue, and how many cells each.
    /// Empty once the branch is gone.
    pub sites: Vec<(crate::body::PartId, u32)>,
    /// Cells across all of them: the tissue this body is spending on poison
    /// instead of on what the part used to do.
    pub cells: u32,
    /// What a bite costs an eater when the gland is charged, in milligrams.
    pub potency_mg: u64,
    /// What the column under this body holds.
    pub ground_mg: u64,
    /// Whether the ground can supply what the gland holds. **Dormancy**: a dry
    /// gland has lost none of its tissue and none of its cost.
    pub charged: bool,
    /// The standing rent this gland adds, per tick, charged or not.
    pub rent_mg: u64,
    /// Parts that carried a gland and were severed.
    pub lost: Vec<crate::body::PartId>,
}
