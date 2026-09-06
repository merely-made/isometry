// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The other things living here.
//!
//! These used to be `Morsel`s: inert matter with a mass, waiting to be
//! collected. That name was the design flaw written down — a morsel is *a
//! small piece of food*, which is what you call something that exists to be
//! eaten. Mark's diagnosis: "we're just kinda munchin'. A free meal, talk
//! about the opposite of a game."
//!
//! An organism runs the same loop the player runs. It grows, matures,
//! reproduces, ages, dies, and rots, whether or not anyone is watching. Eating
//! one is a decision with a cost and a moment, because the thing in front of
//! you is going somewhere on its own.

use serde::{Deserialize, Serialize};

use crate::body::{Attachment, BodyDocument, Provenance, Yaw};
use crate::phenotype::BodyPhenotype;
use crate::places::{Tier, WalkerShape};
use crate::plan::{Role, classify};
use crate::process::{FeedingMode, NisKind, Process};

mod behavior;
pub mod ecology;
mod kingdom;
mod ledger;

pub use behavior::{
    FaunaDecisionTrace, FaunaDrive, FaunaDriveScores, FaunaPolicy, FaunaSenses, FaunaTraits,
};
pub use ecology::step;
use ecology::{OFFSPRING_COST, STARVATION_MG};
pub use kingdom::Kingdom;
pub use ledger::Upkeep;

use crate::body::{SpeciesId, VolumeRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct OrganismId(pub u32);

/// A recent direct observation. It is simulation state: losing sight changes
/// what an embodied creature does next, so it must replay with the organism
/// rather than live in a host-side perception cache.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastSeen {
    pub target: OrganismId,
    pub position: [i32; 3],
    pub ticks_left: u8,
}

/// What an organism advertises about itself.
///
/// Signalling and counter-signalling: an advertisement is a claim, and a claim
/// can be false. This is what makes choosing a meal a decision rather than a
/// collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Signal {
    /// Claims nothing. Ordinary, unremarkable, probably safe.
    Plain,
    /// Claims to be dangerous. Bright, loud, conspicuous.
    Warning,
}

/// Where an organism is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    /// Growing. Worth less now than it will be shortly.
    Juvenile,
    /// Grown, and able to reproduce.
    Mature,
    /// Dead, and not yet returned. Food for decomposers, poor food for others.
    Carrion,
    /// Fully returned to the world. Removed at the end of the tick.
    Spent,
}

impl Stage {
    pub fn is_alive(self) -> bool {
        matches!(self, Stage::Juvenile | Stage::Mature)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Organism {
    pub id: OrganismId,
    /// Which lineage this belongs to. Incorporation carries it forward, so a
    /// part you take always knows whose it was.
    pub species: SpeciesId,
    /// This organism's anatomy **and its process allocation**.
    ///
    /// **Every organism has one**, played or not. Before P1 only the critter
    /// the player inhabited had a body and everything else was a `VolumeRef`
    /// and a `half_extent`, which meant anatomy could never constrain an
    /// unplayed creature and prey had no parts to lose. The pair it replaces
    /// are now readings of the root part, so there is one place a shape is
    /// written down.
    ///
    /// Most organisms carry a single root part. That is a body, not a special
    /// case, and it grows by the same rules as any other.
    ///
    /// **It was a bare `BodyDocument` until PD1b.** The wrapper's own fields
    /// are private, so no caller reaches `&mut BodyDocument` through an
    /// organism and no attach, sever or rearrangement can split anatomy from
    /// phenotype. Read the anatomy through [`Organism::body`].
    pub phenotype: BodyPhenotype,
    /// Entropy that realized this individual from its lineage recipe.
    ///
    /// Stored because the body is causal state, not a renderer accident. A
    /// founder preview using this seed and the same declared inputs must grow
    /// the same body, while descendants derive distinct seeds from it.
    #[serde(default)]
    pub development_seed: u64,
    /// Mass at the life-history transition that founded this individual.
    ///
    /// Rates are allometric, but maturity and senescence must not move away
    /// from an organism as it grows. This is the individual's life-history
    /// reference mass, not a second live biomass account.
    #[serde(default)]
    pub life_history_mass_mg: u64,
    pub position: [i32; 3],
    /// Which simulation tier currently owns this body's next decision.
    ///
    /// It is state, not a cache: hysteresis means the same position can have
    /// different ownership depending on where the organism came from.
    #[serde(default)]
    pub tier: Tier,
    /// The last directly perceived living target, while its short pursuit
    /// window remains. This is generic perception, not a Hunter FSM state.
    #[serde(default)]
    pub last_seen: Option<LastSeen>,
    /// The inherited, quantized proposal policy used only by near fauna.
    /// Movement legality and ecology resolution remain outside it.
    #[serde(default)]
    pub fauna_policy: FaunaPolicy,
    /// Inspectable evidence for the policy's most recent proposal.
    #[serde(default)]
    pub last_fauna_decision: Option<FaunaDecisionTrace>,
    /// What this organism can spend: its **budget**.
    ///
    /// Distinct from what it weighs, which is its body's business. Before P1
    /// only the played critter had a budget, so nothing else could run out of
    /// one.
    pub energy_mg: u64,
    pub stage: Stage,
    pub age: u32,
    /// Ticks since this organism last reproduced.
    pub since_offspring: u32,
    /// What this organism *advertises*.
    pub signal: Signal,
    /// What it actually does to something that eats it, in milligrams.
    ///
    /// The gap between this and [`Self::signal`] is the whole mechanic. An
    /// honest organism's claim matches its bite. A **Batesian** mimic warns
    /// without a bite: safe, and eaten only by something that learned better.
    /// An **aggressive** mimic looks plain and bites hard: the trap.
    pub venom_mg: u64,
    /// The kingdom this organism *appears* to belong to.
    ///
    /// Usually its own. A mimic's differs, which is what breaks the shape
    /// contract on purpose: roles are read from geometry, so the game teaches
    /// that form tells you function, and a simulacrum violates exactly that
    /// lesson.
    ///
    /// **Since DC1.5 the claim is about anatomy.** It used to be a claim about
    /// silhouette, because that is what a kingdom was; it is now a claim to
    /// carry the organs of a way of making a living — "there is a leaf on me" —
    /// told by a body that does not. What tells the lie is unchanged and no
    /// reader of it moved; what the lie is *about* did.
    pub guise: Kingdom,
}

/// The smallest part of each role a founding body can be given a feeding organ
/// out of: a frond, a crop, a jaw. Smallest because a fixture's organ should
/// make the reading and nothing else — a larger one would move the body's mass
/// ceiling, and with it every rate derived from it.
const FROND: [i32; 3] = [3, 2, 1];
const CROP: [i32; 3] = [2, 1, 1];
const JAW: [i32; 3] = [3, 1, 1];

impl Organism {
    /// A minimal organism: a root part, the feeding organ that makes it the
    /// kingdom it is asked for, and the scalars the ecology moves.
    ///
    /// **The organ is not decoration.** Since DC1.5 a kingdom is a reading of
    /// feeding anatomy, so a body that was handed a kingdom has to carry the
    /// anatomy that reads as one; a bare root reads `Decomposer` whatever it
    /// was asked for. A consumer built out of long thin bulk gets a jaw and one
    /// built out of any other shape gets a crop, which keeps a fixture the
    /// feeding mode its caller's half-extent has always asked for.
    ///
    /// The organ's milligram comes out of the root, so the body still weighs
    /// exactly what it was given.
    #[allow(clippy::too_many_arguments)]
    pub fn founding(
        id: OrganismId,
        species: SpeciesId,
        kingdom: Kingdom,
        volume: VolumeRef,
        half_extent: [i32; 3],
        position: [i32; 3],
        mass_mg: u64,
    ) -> Self {
        let development_seed = u64::from(id.0) << 32 | u64::from(species.0);
        let organ = match kingdom {
            Kingdom::Producer => Some((FROND, 1)),
            Kingdom::Consumer if classify(half_extent) == Role::Limb => Some((JAW, -1)),
            Kingdom::Consumer => Some((CROP, -1)),
            Kingdom::Decomposer => None,
        };
        let organ_mg = u64::from(organ.is_some()).min(mass_mg);
        let mut body = BodyDocument::new(species, volume, mass_mg - organ_mg, half_extent);
        body.plan.symmetry = kingdom.symmetry();
        if let Some((shape, side)) = organ {
            let root = body.root;
            let offset = [0, side * (half_extent[1].abs() + shape[1].abs()), 0];
            body.attach(
                volume,
                organ_mg,
                shape,
                Attachment {
                    parent: root,
                    offset,
                    yaw: Yaw::Zero,
                },
                Provenance::founding(),
            )
            .expect("the organ attaches to a root that was just built");
        }
        Self {
            id,
            species,
            phenotype: BodyPhenotype::seed(body),
            development_seed,
            life_history_mass_mg: mass_mg,
            position,
            tier: Tier::Near,
            last_seen: None,
            fauna_policy: FaunaPolicy::default(),
            last_fauna_decision: None,
            energy_mg: mass_mg,
            stage: Stage::Juvenile,
            age: 0,
            since_offspring: 0,
            signal: Signal::Plain,
            venom_mg: 0,
            guise: kingdom,
        }
    }

    /// The same organism, grown up. A fixture convenience: worldgen staggers
    /// ages, and a test that needs an adult should not have to.
    pub fn matured(mut self) -> Self {
        self.stage = Stage::Mature;
        self
    }

    /// This organism's anatomy.
    ///
    /// The structural reading everything that draws, weighs, places or
    /// projects a body wants. Its allocation is beside it, through
    /// [`Organism::phenotype`], and neither can move without the other.
    pub fn body(&self) -> &BodyDocument {
        self.phenotype.body()
    }

    /// The volume a projection should draw for this organism: its root part's.
    pub fn volume(&self) -> VolumeRef {
        self.body()
            .part(self.body().root)
            .map(|p| p.volume)
            .unwrap_or(VolumeRef([0; 32]))
    }

    /// This organism's overall half-extent, read off its root part.
    ///
    /// A reading rather than a field, so a body and the shape the world sees
    /// cannot disagree.
    pub fn half_extent(&self) -> [i32; 3] {
        self.body()
            .part(self.body().root)
            .map(|p| p.half_extent)
            .unwrap_or([1, 1, 1])
    }

    /// The cross-section this body's live anatomy presents to Ground.
    ///
    /// This is recomputed from the collision box so incorporation, growth, or
    /// injury can change where the organism fits without synchronizing a
    /// second locomotion size field.
    pub fn walker_shape(&self) -> WalkerShape {
        WalkerShape::from_aabb(self.body().aabb())
    }

    /// What this organism weighs: the sum of its surviving parts.
    ///
    /// **The only body mass there is.** Until 2026-08-01 an `Organism` also
    /// carried a scalar `mass_mg` that the ecology moved, so anatomy and the
    /// ecology kept separate accounts of the same quantity and neither could
    /// see the other. That is why a forty-part critter cost exactly what a
    /// single cell cost.
    pub fn biomass_mg(&self) -> u64 {
        self.body().total_mass_mg()
    }

    pub(crate) fn life_history_mass_mg(&self) -> u64 {
        if self.life_history_mass_mg == 0 {
            self.biomass_mg()
        } else {
            self.life_history_mass_mg
        }
    }

    /// Trophic role read from the organs this body feeds with, not retained as
    /// a genesis decree. A reshaped body therefore changes the role the ecology
    /// sees. See [`kingdom`] for the rules and why they replaced symmetry.
    pub fn kingdom(&self) -> Kingdom {
        Kingdom::of(&self.phenotype)
    }

    /// What this body does with matter, read from its active declared intake
    /// ports. Geometry seeds a founding declaration but does not remain the
    /// prey-set rule.
    pub fn feeding_mode(&self) -> FeedingMode {
        FeedingMode::of(&self.phenotype)
    }

    /// The one trophic admission question. Both NPC targeting and played
    /// consumption ask it, so a port declaration cannot make one route accept
    /// food the other refuses.
    pub fn admits(&self, kind: NisKind, deadstock: bool) -> bool {
        let ports = self.phenotype.intake_ports();
        if deadstock {
            ports.admits_deadstock()
        } else {
            ports.admits_live(kind)
        }
    }

    /// How far this body's actuators swing, in voxels: each living
    /// contractile part's longest half-extent, summed.
    ///
    /// **Zero for a body with no actuator**, which is what a sessile plan
    /// honestly reads. [`Self::locomotion`] floors the same number at one for
    /// the drive selector's arithmetic; rent prices this one, so a plant pays
    /// nothing for a machinery it does not carry. (TD7)
    pub fn actuator_span(&self) -> u32 {
        self.body()
            .living()
            .filter(|part| self.body().processes(part.id).contains(&Process::Contract))
            .map(|part| {
                part.half_extent
                    .iter()
                    .map(|v| v.unsigned_abs())
                    .max()
                    .unwrap_or(0)
            })
            .sum::<u32>()
    }

    /// How far this body's sense organs extend, in voxels: each living
    /// sensing part's longest half-extent, summed. The exact shape of
    /// [`Self::actuator_span`], read off [`Process::Sense`] instead of
    /// [`Process::Contract`]. (TD11)
    ///
    /// **Zero for a body with no sense organ**, which is what a blind plan
    /// honestly reads — see [`ecology::sight_for_body`], where zero is the
    /// near horizon a body has always had rather than no horizon at all.
    ///
    /// Span rather than [`FaunaTraits::sensory_parts`]'s count, for the reason
    /// rent reads span: a count cannot say that a bigger organ senses more, and
    /// the two coincide anyway on the primitive palette, whose sensor template
    /// is `[1, 1, 1]`.
    pub fn sensor_span(&self) -> u32 {
        self.body()
            .living()
            .filter(|part| self.body().processes(part.id).contains(&Process::Sense))
            .map(|part| {
                part.half_extent
                    .iter()
                    .map(|v| v.unsigned_abs())
                    .max()
                    .unwrap_or(0)
            })
            .sum::<u32>()
    }

    /// What a bite of this body costs whatever took it, on the ground this
    /// body is standing on. (PD2)
    ///
    /// **Two toxins, one number.** [`Self::venom_mg`] is what this line was
    /// born with and passes on; the rest is what its glands are making right
    /// now, which is [`Self::charged_mg`] and depends on where it is standing.
    /// Every eater — the played one and the ecology's — asks this, so a gland
    /// deters the same way whoever is holding the mouth.
    pub fn bite_mg(&self, ground_mg: u64) -> u64 {
        self.venom_mg.saturating_add(self.charged_mg(ground_mg))
    }

    /// The dose this body's glands hold, or zero when they are dry.
    ///
    /// **The dormancy rule, and it is a world condition.** A gland makes its
    /// toxin out of the ground it stands over, so it is charged only where
    /// that column could replace what the gland holds. A big gland needs
    /// richer ground than a small one, which is the whole of the rule and is
    /// derived rather than tuned: the threshold *is* the potency.
    ///
    /// Allocation does not move when a gland goes dry (plan §4: a changing
    /// environment does not rewrite the mosaic). The tissue is still there,
    /// still costs its rent, and starts working again on better ground.
    pub fn charged_mg(&self, ground_mg: u64) -> u64 {
        let held = self.phenotype.secretory_mg();
        if held > 0 && ground_mg >= held {
            held
        } else {
            0
        }
    }

    /// A compact locomotion reading used by the drive selector. It is based
    /// on the same contractile geometry that makes a body a predator.
    pub fn locomotion(&self) -> u32 {
        self.actuator_span().max(1)
    }

    pub fn is_alive(&self) -> bool {
        self.stage.is_alive()
    }

    /// Whether this organism is pretending to be something it is not.
    pub fn is_mimic(&self) -> bool {
        self.guise != self.kingdom() || self.signals_falsely()
    }

    /// Whether its advertisement is a lie, in either direction.
    pub fn signals_falsely(&self) -> bool {
        match self.signal {
            Signal::Warning => self.venom_mg == 0,
            Signal::Plain => self.venom_mg > 0,
        }
    }

    /// The tell.
    ///
    /// A thing wearing a producer's look but living a consumer's life does not
    /// gain mass in open ground, because it is not fixing anything. Watch it
    /// for a while and the lie shows. Unfair is fine here; unknowable is not,
    /// so every mimic leaves something a second encounter can find.
    ///
    /// **The claim got sharper at DC1.5**, and stayed true. It used to mean "a
    /// producer's silhouette over a body the symmetry field says is not one";
    /// it now means "a producer's claim over a body that carries no part
    /// performing [`Process::Fix`]" — which is precisely why the tell exists,
    /// rather than being a coincidence of two enums lining up.
    pub fn betrays_itself(&self) -> bool {
        self.guise == Kingdom::Producer && self.kingdom() != Kingdom::Producer
    }

    /// Whether this organism is ready to produce an offspring.
    ///
    /// **Adult mass, not an absolute floor** (TD8, and the missing half of
    /// TD6). The mass clause used to be a flat `STARVATION_MG * OFFSPRING_COST`
    /// — 80 mg, a number that knows nothing about the body asking. TD6 derived
    /// an adult mass from the body plan and made growth determinate; this makes
    /// breeding ask about it, so a big-plan body has to grow up first and a body
    /// stalled at a fifth of its ceiling stops shedding broods it cannot afford.
    ///
    /// The gestation clock is unchanged, and the 80 mg clause stays underneath
    /// as what it always really was: the guarantee that a brood costing a
    /// quarter of the parent is born above [`ecology::STARVATION_MG`] rather
    /// than already starving. The two do not conflict — a small plan's share of
    /// its ceiling can fall under 80 mg, and then the floor is the binding one.
    pub fn can_reproduce(&self) -> bool {
        self.stage == Stage::Mature
            && self.since_offspring >= ecology::gestation_for_mass(self.life_history_mass_mg())
            && self.biomass_mg() >= ecology::breeding_mass_mg(self.mass_ceiling_mg())
            && self.biomass_mg() > STARVATION_MG * OFFSPRING_COST
    }
}

/// What a tick did to the world's organisms, so a host can show it and a test
/// can assert on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub matured: u32,
    pub born: u32,
    pub died: u32,
    pub returned: u32,
    pub moved: u32,
    pub far_cohorts: u32,
    pub far_members: u32,
    pub far_biomass_mg: u64,
    pub promoted: u32,
    pub demoted: u32,
}
