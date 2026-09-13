// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What rules a world actually realized. (PD3)
//!
//! The playable ecology plan §2 asks a durable world to record its admitted
//! definitions, schedules, generator version and digests, because a seed alone
//! is insufficient once the code that reads it can change. `WorldRules` is that
//! record's working label, and this is its **first proven component**: the
//! digest of the [`Registry`](crate::process::Registry) a world was founded
//! under.
//!
//! # Identity, not a copy
//!
//! A world carries the digest, not the definitions. That is the same
//! arrangement `ProcessRef` already uses one scale down — a content address
//! rather than an inline copy — and it keeps a snapshot the size of a world
//! rather than the size of a world plus its biology. What the digest buys is
//! the refusal: a save restored against a ruleset that is not the one it ran
//! under is refused by name (`SnapshotError::Ruleset`) instead of quietly
//! simulating something else.
//!
//! # What is rule-bearing
//!
//! Everything [`Registry::digest`](crate::process::Registry::digest) folds,
//! and nothing else. Each definition contributes its identity, its site
//! requirement and its seeding; the set is folded in sorted order, so neither
//! the order a pack declared its files in nor a definition's plain label,
//! explanation text or native binding can move this number.
//!
//! # The second component: what ends an epoch (PE3)
//!
//! [`EpochRule`] is the first rule here that is not a digest, and it belongs
//! beside the ruleset for the same reason the ruleset does: a world that ended
//! its epochs on a different budget is a different game, and a save restored
//! against the wrong one would quietly re-time the whole run.

use serde::{Deserialize, Serialize};

use crate::process::Registry;

/// Ticks an epoch runs for by default.
///
/// **A hundred seconds at the canonical ten ticks a second, and a third of a
/// 1,000 mg starter's 3,000-tick lifespan** — so a played line reviews itself
/// about three times in the life of one body, and a recorded demo of a few
/// thousand steps crosses several boundaries rather than none. Shorter, and
/// the review interrupts a life that has not had time to become anything;
/// longer, and a body lives and dies without its line ever taking a turn.
pub const DEFAULT_EPOCH_TICKS: u64 = 1_000;

/// Version of the trophic semantics embodied in a world and its trace.
/// Zero names the pre-port grammar, so a decoded historical world can be
/// distinguished at the `WorldRules` admission gate rather than replayed as
/// TG1. Revision 2 introduces typed soil; revision 3 adds part scruple storage.
/// Revision 4 preserves whole-meal mixtures and records reserve digestion.
/// Revision 5 preserves birth provisioning and records completed tissue returns.
/// Revision 6 introduces producer synthesis and explicit founder tissue recipes.
pub const TROPHIC_GRAMMAR_REVISION: u32 = 6;

/// How long a candidate is grown before its flow record is read. (P4b)
///
/// **One brood interval at the ecology's reference body** —
/// `rates::GESTATION_BASE`, the same number `gestation_for_mass` is quoted
/// against. It is that reading and not a shorter one because a revision only
/// ever shows up in *descendants*: a window with no birth of the line in it
/// scores the candidate and the status quo identically, and a window with a
/// birth but no life after it prices only what the newborn paid to grow the
/// organ. Measured, not guessed — at one judgement window
/// ([`WARN_AFTER_TICKS`](crate::flow::WARN_AFTER_TICKS), 60 ticks) every line
/// in the probe scored the gland *worse* by about a tenth of a percent, which
/// is its development cost and nothing else; the sign turns between 240 and
/// 600, which is where the organ has been carried long enough to be worth
/// carrying. One brood interval is the shortest span with a name that reaches
/// past that, and it costs a boundary two runs of this length per candidate
/// per line.
pub const DEFAULT_SCORE_TICKS: u64 = crate::organism::ecology::GESTATION_BASE as u64;

/// What deterministic condition ends an epoch.
///
/// **Three separate rules, not a composition** (playable ecology plan §6,
/// ruled by Mark 2026-09-01). [`Timed`](Self::Timed) was built first and
/// [`PlayerTriggered`](Self::PlayerTriggered) is built as of DT3;
/// [`Gated`](Self::Gated) is still named-only data, so that the vocabulary,
/// the serialized shape and the digest are settled before it arrives, and a
/// world holding it simply never ends an epoch — which [`Self::built`] says
/// out loud rather than leaving a caller to infer from silence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum EpochRule {
    /// The epoch ends when a fixed tick budget is spent. **Built first.**
    Timed { ticks: u64 },
    /// The epoch ends when named world conditions are all met. **Not built**;
    /// it needs the conditions named first.
    Gated,
    /// The epoch ends on demand, and on nothing else. **Built by DT3, and
    /// never play**: `Intent::EndEpoch` is a dev tool, and a world holding this
    /// rule runs one unbounded epoch until somebody asks for the boundary.
    PlayerTriggered,
}

impl Default for EpochRule {
    fn default() -> Self {
        Self::Timed {
            ticks: DEFAULT_EPOCH_TICKS,
        }
    }
}

impl EpochRule {
    /// Whether this rule is implemented. `false` for the one named-only rule.
    pub fn built(self) -> bool {
        matches!(self, Self::Timed { .. } | Self::PlayerTriggered)
    }

    /// Whether `elapsed` ticks of the current epoch spend its budget.
    ///
    /// A rule that ends on something other than a tick count answers `false`
    /// at every tick, which is the honest behaviour: the epoch does not end
    /// rather than ending on a guess. For [`Self::PlayerTriggered`] that is
    /// the rule itself, not a gap in it — the only thing that ends its epoch
    /// is the demand.
    pub fn spent(self, elapsed: u64) -> bool {
        match self {
            Self::Timed { ticks } => ticks > 0 && elapsed >= ticks,
            Self::Gated | Self::PlayerTriggered => false,
        }
    }

    /// Whether this rule lets `Intent::EndEpoch` close the epoch now. (DT3)
    ///
    /// **Timed accepts the demand as an early end, and that is the ruling.**
    /// The alternative — admit it only under [`Self::PlayerTriggered`] — was
    /// rejected for two reasons. First, closing a Timed epoch early has always
    /// been something a driver could ask for, back when a manual door existed
    /// beside this one (deleted in DT4; see `world::adapt`'s module docs): a
    /// dev key that could not do what that door did would be a weaker tool
    /// guarded by a stricter rule, and the two would then disagree about what
    /// a boundary is. Second, refusing would mean the dev tool only worked in
    /// a world founded under a rule the game does not ship, so the one thing
    /// it exists to exercise — the boundary path PE3a built, in the world play
    /// actually runs — could never be reached with it. Ending early restarts
    /// the budget from this tick, because bodies change between epochs rather
    /// than during them.
    ///
    /// [`Self::Gated`] refuses. It has no condition behind it yet, and a
    /// demand standing in for conditions nobody has named would make it
    /// indistinguishable from [`Self::PlayerTriggered`] — two rules the
    /// playable ecology plan §6 deliberately keeps apart.
    pub fn admits_demand(self) -> bool {
        match self {
            Self::Timed { .. } | Self::PlayerTriggered => true,
            Self::Gated => false,
        }
    }

    fn bytes(self) -> Vec<u8> {
        match self {
            Self::Timed { ticks } => {
                let mut bytes = vec![0u8];
                bytes.extend_from_slice(&ticks.to_le_bytes());
                bytes
            },
            Self::Gated => vec![1],
            Self::PlayerTriggered => vec![2],
        }
    }
}

/// A content address for one complete admitted ruleset.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct RulesetDigest(pub u64);

impl RulesetDigest {
    /// The form a receipt, a panel or a diagnostic prints.
    pub fn hex(self) -> String {
        format!("{:016x}", self.0)
    }
}

/// The immutable record of the rules a world realized.
///
/// Material and field definitions, environmental schedules and the generator
/// version join these as their own gates land (playable ecology plan §2); each
/// arrives here rather than as a second place a rule can live.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct WorldRules {
    /// The process ruleset this world admitted.
    pub processes: RulesetDigest,
    /// What ends an epoch here. (PE3)
    #[serde(default)]
    pub epoch: EpochRule,
    /// Ticks a candidate is grown for before its flow record is read. (P4b)
    ///
    /// Rule-bearing, and that is why it is here rather than a constant in the
    /// scorer: it decides what an unplayed line commits, and what an unplayed
    /// line commits is the world.
    #[serde(default = "default_score_ticks")]
    pub score_ticks: u64,
    /// The trophic grammar that decides which body can admit which food.
    #[serde(default)]
    pub trophic_grammar: u32,
}

fn default_score_ticks() -> u64 {
    DEFAULT_SCORE_TICKS
}

/// **Written out rather than derived**, because a derived one would give this
/// build's defaults for the epoch rule and a zero-length scoring window — and a
/// zero-length window scores every candidate identically, which is a rule
/// nobody chose arriving through a `..Default::default()`.
impl Default for WorldRules {
    fn default() -> Self {
        Self {
            processes: RulesetDigest::default(),
            epoch: EpochRule::default(),
            score_ticks: DEFAULT_SCORE_TICKS,
            trophic_grammar: 0,
        }
    }
}

impl WorldRules {
    /// The rules a world founded on this build's own definitions realizes.
    pub fn native() -> Self {
        Self::of(Registry::native())
    }

    /// The rules an admitted registry amounts to, under this build's defaults
    /// for everything a registry does not decide.
    pub fn of(registry: &Registry) -> Self {
        Self {
            processes: registry.digest(),
            epoch: EpochRule::default(),
            score_ticks: DEFAULT_SCORE_TICKS,
            trophic_grammar: TROPHIC_GRAMMAR_REVISION,
        }
    }

    /// The same rules under a different epoch rule.
    pub fn ending(self, epoch: EpochRule) -> Self {
        Self { epoch, ..self }
    }

    /// The same rules under a different scoring window.
    pub fn scoring_over(self, score_ticks: u64) -> Self {
        Self {
            score_ticks,
            ..self
        }
    }

    /// This whole record's identity: every component folded, in a fixed order.
    ///
    /// What a refusal names when two worlds disagree about their rules and it
    /// is not the ruleset they disagree about. It is not stored — a world
    /// carries the components and this reads them — so nothing can hold a
    /// digest that has fallen behind what it is a digest of.
    pub fn digest(self) -> u64 {
        let mut bytes = self.processes.0.to_le_bytes().to_vec();
        bytes.extend_from_slice(&self.epoch.bytes());
        bytes.extend_from_slice(&self.score_ticks.to_le_bytes());
        bytes.extend_from_slice(&self.trophic_grammar.to_le_bytes());
        crate::snapshot::hash_bytes(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ruleset_digest_is_stable_across_calls() {
        assert_eq!(WorldRules::native(), WorldRules::native());
        assert_eq!(
            WorldRules::native().processes,
            Registry::native().digest(),
            "a world's rules are the registry's digest and nothing else"
        );
    }

    #[test]
    fn declaration_order_is_not_rule_bearing() {
        // The property a pack depends on: a manifest may list its files in any
        // order, and the ruleset it lowers to is the same ruleset.
        let mut reversed: Vec<_> = Registry::native().all().cloned().collect();
        reversed.reverse();
        let admitted = Registry::admit(reversed).expect("no collision");
        assert_eq!(admitted, *Registry::native(), "canonical order is restored");
        assert_eq!(admitted.digest(), Registry::native().digest());
    }

    /// The epoch rule is folded into the record's identity, so two worlds on
    /// different budgets cannot agree about their rules. (PE3)
    #[test]
    fn the_epoch_rule_is_digested() {
        let native = WorldRules::native();
        assert_eq!(native.epoch, EpochRule::Timed { ticks: 1_000 });
        assert_eq!(native.digest(), WorldRules::native().digest());

        let brisk = native.ending(EpochRule::Timed { ticks: 500 });
        assert_ne!(brisk.digest(), native.digest(), "a budget is rule-bearing");
        assert_eq!(
            brisk.processes, native.processes,
            "and it moved nothing about the biology"
        );

        let quick = native.scoring_over(10);
        assert_ne!(quick.digest(), native.digest(), "so is the score window");
    }

    #[test]
    fn the_trophic_grammar_is_digested() {
        let native = WorldRules::native();
        assert_eq!(native.trophic_grammar, TROPHIC_GRAMMAR_REVISION);
        let pre_ports = WorldRules {
            trophic_grammar: 0,
            ..native
        };
        assert_ne!(pre_ports.digest(), native.digest());
    }

    /// Only Timed ends an epoch on the clock; Gated is still named-only data.
    #[test]
    fn only_the_timed_rule_ends_an_epoch_on_its_own() {
        let timed = EpochRule::Timed { ticks: 3 };
        assert!(timed.built());
        assert!(!timed.spent(2), "not before the budget is spent");
        assert!(timed.spent(3), "and exactly when it is");

        for other in [EpochRule::Gated, EpochRule::PlayerTriggered] {
            assert!(
                !other.spent(u64::MAX),
                "a rule that does not end on a tick count never spends a budget"
            );
        }
        assert!(
            EpochRule::PlayerTriggered.built(),
            "DT3 built the demand behind it"
        );
        assert!(!EpochRule::Gated.built(), "and left this one named-only");
    }

    /// Which rules admit `Intent::EndEpoch`, and the reasoning is on
    /// [`EpochRule::admits_demand`]. (DT3)
    #[test]
    fn the_demand_is_admitted_by_timed_and_player_triggered_and_refused_by_gated() {
        assert!(
            EpochRule::Timed { ticks: 1_000 }.admits_demand(),
            "an early end, restarting the budget from that tick"
        );
        assert!(
            EpochRule::PlayerTriggered.admits_demand(),
            "and it is the only thing that ends this one at all"
        );
        assert!(
            !EpochRule::Gated.admits_demand(),
            "a demand must not stand in for conditions nobody has named"
        );
    }

    /// The demand is not rule-bearing on its own: the three rules still digest
    /// differently, which is what keeps two worlds from agreeing wrongly.
    #[test]
    fn the_three_epoch_rules_still_have_three_identities() {
        let native = WorldRules::native();
        let gated = native.ending(EpochRule::Gated);
        let demanded = native.ending(EpochRule::PlayerTriggered);
        assert_ne!(gated.digest(), demanded.digest());
        assert_ne!(native.digest(), demanded.digest());
    }

    #[test]
    fn a_repeated_qualified_id_is_refused() {
        let mut defs: Vec<_> = Registry::native().all().cloned().collect();
        let clash = defs[0].clone();
        defs.push(clash.clone());
        assert_eq!(Registry::admit(defs), Err(clash.id));
    }
}
