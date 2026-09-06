// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Snapshots and state hashing.
//!
//! A snapshot is the whole world, captured at once, by one call. That is the
//! property the wing's determinism constraint is for: hand-written per-field
//! capture has a failure mode, a field added later and not added to the
//! snapshot, that whole-state capture cannot have.
//!
//! The state hash is over the snapshot bytes, so two worlds hash equal exactly
//! when they would serialize equal.

use serde::{Serialize, de::DeserializeOwned};

use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnapshotError {
    Encode,
    Decode,
    /// The save ran under a different admitted ruleset than the one offered.
    /// (PD3)
    ///
    /// **A refusal, not a divergence.** Plan §6 requires a stale ruleset to
    /// refuse explicitly rather than continue against whatever biology this
    /// build happens to hold: the bodies in the save cite definition digests,
    /// and a ruleset that does not hold them would resolve `None` on every
    /// site and quietly simulate a body that expresses nothing.
    Ruleset {
        expected: crate::rules::RulesetDigest,
        found: crate::rules::RulesetDigest,
    },
    /// The save ran under different world rules than the ones offered, and the
    /// biology is not what they differ about. (PE3)
    ///
    /// Today that is the epoch rule or the scoring window: a world that ended
    /// its epochs on a different budget, or judged its candidates over a
    /// different run, is a different game, and continuing would silently
    /// re-time it. Named by [`WorldRules::digest`](crate::rules::WorldRules::digest),
    /// which folds every component, so a rule added later is covered without
    /// this variant growing a field.
    Rules {
        expected: u64,
        found: u64,
    },
}

/// Captures the whole world as bytes.
pub fn snapshot(world: &World) -> Result<Vec<u8>, SnapshotError> {
    encode(world)
}

/// Restores a world captured by [`snapshot`].
pub fn restore(bytes: &[u8]) -> Result<World, SnapshotError> {
    decode(bytes)
}

/// Restores a world and checks it against the ruleset the caller is holding.
/// (PD3, widened at PD4)
///
/// The door a save, a replay or a peer comes through. [`restore`] is the raw
/// decode and stays for round trips within one process; this is what anything
/// that could be carrying a different biology must use, because a ruleset
/// mismatch has to be an answer rather than a silent divergence.
///
/// **It takes the definitions, not only their digest** (PD4). A snapshot
/// carries the identity — the set is not serialized, because a world records
/// which biology it ran rather than a second copy of it — so this is where the
/// set comes back: the digest is compared first, and the registry is attached
/// only if it is the one the save ran under. That makes it impossible to
/// restore a world holding definitions it did not admit.
pub fn restore_under(
    bytes: &[u8],
    ruleset: std::sync::Arc<crate::process::Registry>,
) -> Result<World, SnapshotError> {
    let mut world = decode::<World>(bytes)?;
    let offered = crate::rules::WorldRules::of(&ruleset);
    if world.rules().processes != offered.processes {
        return Err(SnapshotError::Ruleset {
            expected: world.rules().processes,
            found: offered.processes,
        });
    }
    // The rest of the record, once the biology agrees. A caller offers a
    // registry and this build's own defaults for everything a registry does not
    // decide, so a save founded on a different epoch budget is refused here
    // rather than quietly re-timed. PE4 generates world laws, and it is this
    // door that grows a parameter for them.
    if world.rules() != offered {
        return Err(SnapshotError::Rules {
            expected: world.rules().digest(),
            found: offered.digest(),
        });
    }
    world.reattach_ruleset(ruleset);
    Ok(world)
}

/// Round-trips any core value. Used by the body-document tests and by hosts
/// that carry parts of the world without carrying all of it.
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, SnapshotError> {
    postcard::to_allocvec(value).map_err(|_| SnapshotError::Encode)
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, SnapshotError> {
    postcard::from_bytes(bytes).map_err(|_| SnapshotError::Decode)
}

/// FNV-1a over the snapshot bytes. Chosen for being a few integer operations
/// with no platform-dependent behaviour; this is an equality witness for
/// replay, not a cryptographic digest.
pub fn state_hash(world: &World) -> u64 {
    let bytes = snapshot(world).expect("a world is always encodable");
    hash_bytes(&bytes)
}

pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::Yaw;
    use crate::organism::OrganismId;
    use crate::world::{Intent, Placement, World};

    #[test]
    fn snapshot_round_trips() {
        let world = World::new(31, 10);
        let bytes = snapshot(&world).unwrap();
        let restored = restore(&bytes).unwrap();
        assert_eq!(world, restored);
    }

    #[test]
    fn an_omnivores_two_ports_survive_save_and_restore() {
        let world = World::new(31, 60);
        let before = world
            .organisms
            .iter()
            .find(|organism| organism.feeding_mode() == crate::process::FeedingMode::Omnivore)
            .expect("the roster has one")
            .id;
        let restored = restore(&snapshot(&world).unwrap()).unwrap();
        let omnivore = restored
            .organisms
            .iter()
            .find(|organism| organism.id == before)
            .unwrap();
        assert_eq!(
            omnivore.feeding_mode(),
            crate::process::FeedingMode::Omnivore
        );
        assert!(omnivore.admits(crate::process::NisKind::Producer, false));
        assert!(omnivore.admits(crate::process::NisKind::Consumer, false));
    }

    #[test]
    fn a_snapshot_names_the_ruleset_the_world_ran_under() {
        // PD3: `WorldRules` is world state, so it survives the round trip and
        // is inside the hash — two worlds under different biologies cannot
        // agree about a state hash even when everything else about them does.
        let world = World::new(31, 10);
        assert_eq!(world.rules(), crate::rules::WorldRules::native());
        let restored = restore(&snapshot(&world).unwrap()).unwrap();
        assert_eq!(restored.rules(), world.rules());
    }

    #[test]
    fn a_restore_under_a_different_ruleset_is_refused_by_name() {
        // Plan §6, missing packs, at the world scale: refused with both
        // digests rather than continued against whatever this build holds.
        let world = World::new(31, 10);
        let bytes = snapshot(&world).unwrap();
        let restored = restore_under(&bytes, world.admitted()).expect("the same ruleset restores");
        assert_eq!(
            restored.ruleset(),
            world.ruleset(),
            "the checked door hands the set back, not only the digest"
        );

        // A real ruleset that differs, rather than a digest with no
        // definitions behind it: PD4 attaches the set, so the offered thing
        // has to be one.
        let mut defs: Vec<_> = world.ruleset().all().cloned().collect();
        defs.retain(|def| def.id.name != "secrete");
        let other = std::sync::Arc::new(crate::process::Registry::admit(defs).unwrap());
        let found = crate::rules::WorldRules::of(&other).processes;
        assert_eq!(
            restore_under(&bytes, other),
            Err(SnapshotError::Ruleset {
                expected: world.rules().processes,
                found,
            })
        );
    }

    /// The same refusal one component along. A world that ended its epochs on
    /// a different budget is a different game, so restoring it against this
    /// build's rule is refused by name rather than quietly re-timed. (PE3)
    #[test]
    fn a_restore_under_a_different_epoch_rule_is_refused_by_name() {
        let native = crate::rules::WorldRules::native();
        let brisk = World::new(31, 10)
            .with_rules(native.ending(crate::rules::EpochRule::Timed { ticks: 250 }));
        assert_ne!(brisk.rules().digest(), native.digest());
        assert_eq!(
            brisk.rules().processes,
            native.processes,
            "the biology is the same biology"
        );

        let bytes = snapshot(&brisk).unwrap();
        assert_eq!(
            restore_under(&bytes, brisk.admitted()),
            Err(SnapshotError::Rules {
                expected: brisk.rules().digest(),
                found: native.digest(),
            }),
            "and the ruleset refusal is not the one that fires"
        );

        let ordinary = World::new(31, 10);
        assert!(
            restore_under(&snapshot(&ordinary).unwrap(), ordinary.admitted()).is_ok(),
            "a world on this build's own rule restores"
        );
    }

    #[test]
    fn identical_worlds_hash_equal() {
        assert_eq!(state_hash(&World::new(5, 8)), state_hash(&World::new(5, 8)));
    }

    #[test]
    fn divergent_worlds_hash_differently() {
        let mut a = World::new(5, 8);
        let b = World::new(5, 8);
        a.apply(Intent::Move { delta: [1, 0, 0] });
        assert_ne!(state_hash(&a), state_hash(&b));
    }

    #[test]
    fn restoring_mid_run_continues_identically() {
        let trace = [
            Intent::Move { delta: [1, 0, 1] },
            Intent::Idle,
            Intent::Deposit { mass_mg: 50 },
            Intent::Move { delta: [-2, 0, 0] },
        ];

        let mut straight = World::new(77, 12);
        straight.apply_all(&trace);

        let mut forked = World::new(77, 12);
        forked.apply_all(&trace[..2]);
        let bytes = snapshot(&forked).unwrap();
        let mut resumed = restore(&bytes).unwrap();
        resumed.apply_all(&trace[2..]);

        assert_eq!(state_hash(&straight), state_hash(&resumed));
    }

    #[test]
    fn rejected_intents_are_part_of_the_recorded_state() {
        let mut a = World::new(13, 6);
        let mut b = World::new(13, 6);
        a.apply(Intent::Metabolize {
            organism: OrganismId(9999),
            placement: Placement::Explicit {
                parent: a.body().unwrap().root,
                offset: [0, 0, 0],
                yaw: Yaw::Zero,
            },
        });
        // A different refusal, and the same nothing: both advanced one tick and
        // neither changed anything else.
        b.apply(Intent::Metabolize {
            organism: OrganismId(8888),
            placement: Placement::Planned,
        });
        assert_eq!(state_hash(&a), state_hash(&b));

        // Doing nothing, however, is now something: TD4's idle run is world
        // state, so a tick spent idling and a tick spent being refused are no
        // longer the same tick.
        let mut idled = World::new(13, 6);
        idled.apply(Intent::Idle);
        assert_ne!(state_hash(&a), state_hash(&idled));
    }
}
