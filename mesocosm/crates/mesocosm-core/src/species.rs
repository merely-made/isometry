// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which lineages exist, what they are called, and what they came from.
//!
//! Until now reproduction copied a parent's `SpeciesId` verbatim and nothing
//! else ever assigned one, so **lineages could never split and no new species
//! was ever born.** Three things quietly assumed otherwise: the complexity
//! frontier, multiple-lineage play, and any notion of how closely two creatures
//! are related.
//!
//! # Splitting is an act, not a threshold
//!
//! A lineage forks because something *happened*, not because a similarity
//! metric crossed a line. For the player that act is **naming**. Thrive
//! auto-speciates on trait divergence, which produces species nobody noticed
//! being born; Dwarf Fortress never speciates and its creature types are
//! eternal. An act gives the player a moment and the world a reason.
//!
//! It is also the same rule as one level down: a critter becomes a borg by
//! being named. Naming promotes an individual out of being a statistic and a
//! line out of being a variation.
//!
//! # A founder is one creature
//!
//! Forking takes the creature you are holding and nothing else. Its offspring
//! inherit the new line; its former kin keep the old one. That is a real
//! commitment rather than a free rename, and it is how a founder effect
//! actually works.
//!
//! # This is in-world descent, not fili
//!
//! Fili is lineage across *worlds*: forks, campaign descent, cross-moot grafts.
//! Biological descent inside one world is explicitly not that and needed its
//! own home, which is here. Built beside `chartulary::stemma` rather than on
//! it, per the standing rule.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::axis::Soma;
use crate::body::{BodyDocument, PartId, SpeciesId};
use crate::development::{DevelopmentError, PartPalette, develop_body};
use crate::plan::Symmetry;
use crate::process::IntakePort;

mod tissue;
pub use tissue::{InitialTissueRecipe, TissueRecipeError};

/// One lineage.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Species {
    pub id: SpeciesId,
    /// What this line's bodies are made from.
    ///
    /// The theme its members vary on: segments, tagmata, and the lexicon of
    /// appendage kinds it has acquired by eating. Heritable, so a fork
    /// inherits its parent's recipe and diverges from there.
    #[serde(default = "crate::axis::Recipe::default_founding")]
    pub recipe: crate::axis::Recipe,
    /// The lineage's declared tissue before it has eaten or grown anything.
    ///
    /// It is stored for founder provenance. Existing bodies retain their
    /// allocated stock when their anatomy or trophic reading later changes.
    pub initial_tissue: InitialTissueRecipe,
    /// The lineage's inherited trophic silhouette. It is part of the
    /// developer's declared input, so previews and births agree.
    #[serde(default)]
    pub symmetry: Symmetry,
    /// What it is called, if anyone named it.
    ///
    /// `None` for the lineages a world begins with: they were there before
    /// anybody arrived to name them, and an unnamed line is a variation rather
    /// than a thing you can point at.
    pub name: Option<String>,
    /// What it split from. `None` for a founding lineage.
    pub parent: Option<SpeciesId>,
    /// The tick it was founded on.
    pub founded: u64,
    /// Which tissue domain this line's flesh belongs to, as this world numbers
    /// them. (P3)
    ///
    /// **Heritable, and world data rather than a fact about biology.** The
    /// world's [`Affinity`](crate::graft::Affinity) table is what gives a
    /// domain meaning, and it decides whether a branch off this line can be
    /// carried into another one's body. A fork inherits it, because splitting a
    /// line does not change what it is made of.
    #[serde(default)]
    pub domain: crate::graft::Domain,
    /// What this line's descendants are born expressing. (P4/PD5)
    ///
    /// **Heritable, immutable, append-only.** A revision states declared sites
    /// rather than a body snapshot, so two descendants of one program may
    /// realize differently; a fork inherits the whole program, because
    /// splitting a line is a commitment rather than a fresh start. Empty is
    /// the founding program — allocation seeded from geometry, which is what
    /// every birth in this world does until a line commits.
    #[serde(default)]
    program: crate::program::Program,
    /// Explicit intake declarations inherited by every realization of this
    /// line. Geometry still seeds the ordinary defaults; this retains authored
    /// departures such as the roster omnivore through filial construction.
    #[serde(default)]
    intake_ports: Vec<(PartId, IntakePort)>,
}

impl Species {
    pub fn is_named(&self) -> bool {
        self.name.is_some()
    }

    /// This line's development program.
    pub fn program(&self) -> &crate::program::Program {
        &self.program
    }

    /// Commits a revision on this line. **The only way a program moves**, and
    /// [`World::revise`](crate::World) is its only production caller: a
    /// revision is a world transaction with a record, not a field a host sets.
    pub(crate) fn commit(
        &mut self,
        cites: crate::program::Citation,
        sites: Vec<crate::program::DeclaredSite>,
        at: u64,
    ) -> crate::program::RevisionId {
        self.program.commit(cites, sites, at)
    }

    /// One founder, realized from this line's recipe and developed under its
    /// current revision.
    ///
    /// A prediction and an explanation receipt (phenotype plan §3): the same
    /// declared inputs reproduce it, and a different ground or provisioning
    /// may realize the same program as a legibly different body.
    pub fn preview(
        &self,
        registry: &crate::process::Registry,
        founder: crate::program::Founder,
        seed: u64,
    ) -> Result<crate::program::Preview, DevelopmentError> {
        crate::program::preview(self, registry, founder, seed)
    }

    /// Realizes one phenotype from this lineage's developmental program.
    ///
    /// Founders, offspring, regrowth, and adaptation previews all call this
    /// same function. A preview is therefore an ordinary unpublished body,
    /// not a renderer approximation that can disagree with birth.
    pub fn realize(
        &self,
        seed: u64,
        mass_mg: u64,
        palette: PartPalette,
    ) -> Result<BodyDocument, DevelopmentError> {
        let soma = Soma::develop(&self.recipe, seed);
        let mut body = develop_body(self.id, &self.recipe, &soma, mass_mg, palette)?;
        body.plan.symmetry = self.symmetry;
        Ok(body)
    }

    pub fn apply_intake_ports(&self, phenotype: &mut crate::phenotype::BodyPhenotype) {
        for (part, port) in &self.intake_ports {
            phenotype.declare_port(*part, *port);
        }
    }

    pub(crate) fn declare_intake_port(&mut self, part: PartId, port: IntakePort) {
        if let Some(existing) = self.intake_ports.iter_mut().find(|(held, _)| *held == part) {
            existing.1 = port;
        } else {
            self.intake_ports.push((part, port));
            self.intake_ports.sort_by_key(|(part, _)| *part);
        }
    }
}

/// Every lineage a world has had, including extinct ones.
///
/// Extinct lineages stay: removing them would erase the ancestry of everything
/// descended from them, and a distance measured against a forgotten ancestor is
/// not a distance.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lineages {
    species: BTreeMap<SpeciesId, Species>,
    next: u32,
}

impl Lineages {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a lineage that was there from the beginning.
    pub fn found(&mut self, id: SpeciesId) -> &Species {
        self.next = self.next.max(id.0 + 1);
        self.species.entry(id).or_insert(Species {
            id,
            recipe: crate::axis::Recipe::default_founding(),
            initial_tissue: InitialTissueRecipe::default(),
            symmetry: Symmetry::default(),
            name: None,
            parent: None,
            founded: 0,
            domain: crate::graft::Domain::default(),
            program: crate::program::Program::default(),
            intake_ports: Vec::new(),
        })
    }

    /// Splits a new lineage off an existing one.
    ///
    /// Returns `None` if the parent is unknown, because a line cannot descend
    /// from something the world has never heard of.
    pub fn fork(&mut self, parent: SpeciesId, name: String, at: u64) -> Option<SpeciesId> {
        if !self.species.contains_key(&parent) {
            return None;
        }
        let id = SpeciesId(self.next);
        self.next += 1;
        // A fork inherits its parent's body recipe, vocabulary included: a
        // founder does not forget what its line had learned to grow.
        let recipe = self.species[&parent].recipe.clone();
        let initial_tissue = self.species[&parent].initial_tissue;
        let symmetry = self.species[&parent].symmetry;
        // A fork inherits what its parent is made of. Splitting a line is a
        // commitment, not a change of tissue.
        let domain = self.species[&parent].domain;
        // And what its parent had come to grow. Epoch-boundary plan §2: a
        // branch shares the parent revision it split from, so the program
        // carries across and the two lines diverge by committing rather than
        // by one of them forgetting.
        let program = self.species[&parent].program.clone();
        let intake_ports = self.species[&parent].intake_ports.clone();
        self.species.insert(
            id,
            Species {
                id,
                recipe,
                initial_tissue,
                symmetry,
                name: Some(name),
                parent: Some(parent),
                founded: at,
                domain,
                program,
                intake_ports,
            },
        );
        Some(id)
    }

    pub fn get(&self, id: SpeciesId) -> Option<&Species> {
        self.species.get(&id)
    }

    pub fn get_mut(&mut self, id: SpeciesId) -> Option<&mut Species> {
        self.species.get_mut(&id)
    }

    /// Installs a recipe on a lineage, which worldgen does when it seeds one.
    pub fn set_recipe(&mut self, id: SpeciesId, recipe: crate::axis::Recipe) {
        if let Some(species) = self.species.get_mut(&id) {
            species.recipe = recipe;
        }
    }

    /// Installs a lineage's explicit unparented-founder tissue declaration.
    pub fn set_initial_tissue(&mut self, id: SpeciesId, tissue: InitialTissueRecipe) {
        if let Some(species) = self.species.get_mut(&id) {
            species.initial_tissue = tissue;
        }
    }

    pub(crate) fn declare_intake_port(&mut self, id: SpeciesId, part: PartId, port: IntakePort) {
        if let Some(species) = self.species.get_mut(&id) {
            species.declare_intake_port(part, port);
        }
    }

    pub fn set_symmetry(&mut self, id: SpeciesId, symmetry: Symmetry) {
        if let Some(species) = self.species.get_mut(&id) {
            species.symmetry = symmetry;
        }
    }

    /// Assigns a lineage's tissue domain, which worldgen does when it seeds
    /// one. (P3)
    pub fn set_domain(&mut self, id: SpeciesId, domain: crate::graft::Domain) {
        if let Some(species) = self.species.get_mut(&id) {
            species.domain = domain;
        }
    }

    /// What a lineage is made of, as this world numbers domains.
    ///
    /// A line this world has never heard of answers with the default domain
    /// rather than with nothing, because every graft needs both ends of the
    /// edge and an absent donor line is already refused upstream.
    pub fn domain(&self, id: SpeciesId) -> crate::graft::Domain {
        self.species
            .get(&id)
            .map(|species| species.domain)
            .unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.species.len()
    }

    pub fn is_empty(&self) -> bool {
        self.species.is_empty()
    }

    pub fn all(&self) -> impl Iterator<Item = &Species> {
        self.species.values()
    }

    /// Lineages anyone bothered to name.
    pub fn named(&self) -> impl Iterator<Item = &Species> {
        self.species.values().filter(|s| s.is_named())
    }

    /// A lineage and everything it descends from, nearest first.
    pub fn ancestry(&self, id: SpeciesId) -> Vec<SpeciesId> {
        let mut line = Vec::new();
        let mut seen = BTreeSet::new();
        let mut current = Some(id);

        while let Some(at) = current {
            // A cycle is unreachable through `fork`, which only ever descends
            // from something already present, but a deserialized registry is
            // not this code's to trust.
            if !seen.insert(at) {
                break;
            }
            line.push(at);
            current = self.species.get(&at).and_then(|s| s.parent);
        }
        line
    }

    /// The nearest lineage both descend from.
    pub fn common_ancestor(&self, a: SpeciesId, b: SpeciesId) -> Option<SpeciesId> {
        let theirs: BTreeSet<SpeciesId> = self.ancestry(b).into_iter().collect();
        self.ancestry(a).into_iter().find(|id| theirs.contains(id))
    }

    /// How far apart two lineages are, in forks.
    ///
    /// The longer of the two walks to their common ancestor, so a parent and
    /// child are one apart and two siblings are also one, which is what
    /// "generations since they diverged" means.
    ///
    /// `None` when they share no ancestor at all, which is a real answer: two
    /// founding lineages of one world are not related.
    pub fn distance(&self, a: SpeciesId, b: SpeciesId) -> Option<u32> {
        if a == b {
            return Some(0);
        }
        let shared = self.common_ancestor(a, b)?;
        let legs = |from: SpeciesId| {
            self.ancestry(from)
                .iter()
                .position(|id| *id == shared)
                .map(|steps| steps as u32)
        };
        Some(legs(a)?.max(legs(b)?))
    }

    /// Whether one lineage descends from another.
    pub fn descends_from(&self, id: SpeciesId, ancestor: SpeciesId) -> bool {
        id != ancestor && self.ancestry(id).contains(&ancestor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree() -> (Lineages, [SpeciesId; 4]) {
        // root
        //  |- a
        //  |   `- a2
        //  `- b
        let mut lineages = Lineages::new();
        let root = SpeciesId(1);
        lineages.found(root);
        let a = lineages.fork(root, "a".into(), 10).unwrap();
        let a2 = lineages.fork(a, "a2".into(), 20).unwrap();
        let b = lineages.fork(root, "b".into(), 30).unwrap();
        (lineages, [root, a, a2, b])
    }

    #[test]
    fn a_world_begins_with_unnamed_lineages() {
        // They were there before anybody arrived to name them, and an unnamed
        // line is a variation rather than a thing you can point at.
        let mut lineages = Lineages::new();
        let founding = lineages.found(SpeciesId(3)).clone();

        assert!(!founding.is_named());
        assert_eq!(founding.parent, None);
        assert_eq!(lineages.named().count(), 0);
    }

    #[test]
    fn forking_names_a_line_and_records_what_it_came_from() {
        let (lineages, [root, a, ..]) = tree();
        let forked = lineages.get(a).unwrap();

        assert_eq!(forked.name.as_deref(), Some("a"));
        assert_eq!(forked.parent, Some(root));
        assert_eq!(forked.founded, 10);
        assert_eq!(
            lineages.named().count(),
            3,
            "the founding line stays unnamed"
        );
    }

    #[test]
    fn a_line_cannot_descend_from_nothing() {
        let mut lineages = Lineages::new();
        assert_eq!(lineages.fork(SpeciesId(99), "orphan".into(), 1), None);
        assert!(lineages.is_empty());
    }

    #[test]
    fn ancestry_walks_back_to_the_founder() {
        let (lineages, [root, a, a2, _]) = tree();
        assert_eq!(lineages.ancestry(a2), vec![a2, a, root], "nearest first");
        assert_eq!(lineages.ancestry(root), vec![root]);
    }

    #[test]
    fn siblings_meet_at_their_parent() {
        let (lineages, [root, a, _, b]) = tree();
        assert_eq!(lineages.common_ancestor(a, b), Some(root));
        assert_eq!(lineages.common_ancestor(a, a), Some(a));
    }

    #[test]
    fn distance_is_generations_since_they_diverged() {
        // The axis Mark named for graft compatibility: how far apart two
        // creatures' ancestries are. It was uncomputable before, because
        // lineages never split and every pair was identical.
        let (lineages, [root, a, a2, b]) = tree();

        assert_eq!(
            lineages.distance(a, a),
            Some(0),
            "a line is no distance from itself"
        );
        assert_eq!(lineages.distance(root, a), Some(1), "parent and child");
        assert_eq!(
            lineages.distance(a, b),
            Some(1),
            "siblings diverged one fork ago"
        );
        assert_eq!(
            lineages.distance(a2, b),
            Some(2),
            "a cousin is further than a sibling"
        );
        assert_eq!(lineages.distance(a2, root), Some(2));
    }

    #[test]
    fn unrelated_founders_have_no_distance() {
        // A real answer rather than a large number: two lineages a world began
        // with are not related, and pretending they are some number of forks
        // apart would be inventing a shared ancestor.
        let mut lineages = Lineages::new();
        lineages.found(SpeciesId(1));
        lineages.found(SpeciesId(2));

        assert_eq!(lineages.common_ancestor(SpeciesId(1), SpeciesId(2)), None);
        assert_eq!(lineages.distance(SpeciesId(1), SpeciesId(2)), None);
    }

    #[test]
    fn descent_is_directional() {
        let (lineages, [root, a, a2, b]) = tree();
        assert!(lineages.descends_from(a2, root));
        assert!(lineages.descends_from(a2, a));
        assert!(
            !lineages.descends_from(root, a2),
            "an ancestor does not descend from its heir"
        );
        assert!(
            !lineages.descends_from(a, b),
            "siblings do not descend from each other"
        );
        assert!(!lineages.descends_from(a, a), "nor from themselves");
    }

    #[test]
    fn ids_are_never_reused() {
        // A distance measured against a recycled id would be nonsense.
        let mut lineages = Lineages::new();
        lineages.found(SpeciesId(7));
        let first = lineages.fork(SpeciesId(7), "one".into(), 1).unwrap();
        let second = lineages.fork(SpeciesId(7), "two".into(), 2).unwrap();

        assert_ne!(first, second);
        assert!(
            first.0 > 7 && second.0 > first.0,
            "and they never collide with a founder"
        );
    }

    #[test]
    fn the_extinct_keep_their_place_in_the_tree() {
        // Nothing removes a lineage, because removing one erases the ancestry
        // of everything descended from it.
        let (lineages, [root, a, a2, _]) = tree();
        assert_eq!(lineages.distance(a2, root), Some(2));
        assert!(
            lineages.get(a).is_some(),
            "the middle of a line is not prunable"
        );
    }

    #[test]
    fn a_registry_round_trips() {
        let (lineages, _) = tree();
        let bytes = crate::snapshot::encode(&lineages).unwrap();
        assert_eq!(
            crate::snapshot::decode::<Lineages>(&bytes).unwrap(),
            lineages
        );
    }

    #[test]
    fn a_fork_inherits_what_its_parent_is_made_of() {
        // A tissue domain is heritable for the same reason a recipe is:
        // splitting a line is a commitment, not a change of flesh. P3's graft
        // affinity reads it, so a fork that forgot it would become graftable
        // with things its parent was not.
        let mut lineages = Lineages::new();
        let root = SpeciesId(1);
        lineages.found(root);
        lineages.set_domain(root, crate::graft::Domain(2));

        let forked = lineages.fork(root, "a".into(), 10).unwrap();
        assert_eq!(lineages.domain(forked), crate::graft::Domain(2));
        assert_eq!(
            lineages.domain(SpeciesId(99)),
            crate::graft::Domain::default(),
            "a line this world has never heard of answers with the default"
        );
    }

    #[test]
    fn a_fork_inherits_its_initial_tissue_declaration() {
        let mut lineages = Lineages::new();
        let parent = SpeciesId(7);
        let tissue = InitialTissueRecipe::try_new([0, 3, 2, 0]).unwrap();
        lineages.found(parent);
        lineages.set_initial_tissue(parent, tissue);

        let child = lineages.fork(parent, "mixed founder".into(), 1).unwrap();
        assert_eq!(lineages.get(child).unwrap().initial_tissue, tissue);
    }
}
