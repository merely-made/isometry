// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P1: one organism model, and control as a recorded pointer.
//!
//! Before this the played critter was a `BodyDocument`, a position, and an
//! energy budget living on `World`, beside a vector of scalar organisms that
//! had none of those things. Anatomy could not constrain an unplayed creature,
//! prey had no parts to lose, and switching lineage would have meant rebuilding
//! state.
//!
//! The claim these tests pin is that **the played critter is an ordinary
//! organism that happens to be pointed at**, that nothing in the rules can tell
//! the difference, and that the pointer moves only through a recorded intent.

use mesocosm_core::{
    Intent, OrganismId, Outcome, Placement, Rejection, World, restore, snapshot, state_hash,
};

/// The nearest organism that is not the player and that the frontier lets
/// the player inhabit. Eligibility binds control now (`World::eligibility`),
/// so a fixture that grabbed the nearest body regardless was reaching above
/// the frontier.
fn prey(world: &World) -> OrganismId {
    let here = world.position().expect("somebody is embodied");
    world
        .organisms
        .iter()
        .filter(|o| Some(o.id) != world.controlled_id() && o.is_alive())
        .filter(|o| world.eligibility(o.id).is_ok())
        .map(|o| (o.id, o.position))
        .min_by_key(|(_, at)| (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0))
        .expect("the fixture scatters eligible organisms")
        .0
}

#[test]
fn the_player_is_an_ordinary_organism() {
    let world = World::new(4_242, 24);
    let me = world.controlled().expect("somebody is being played");

    assert!(
        world.organisms.iter().any(|o| o.id == me.id),
        "in the roster like anything else"
    );
    assert!(!me.body().is_empty(), "with a body");
    assert!(me.energy_mg > 0, "and a budget");
}

#[test]
fn every_organism_has_a_body_now() {
    // The thing that unblocks damage, branch transfer, and phenotype-granted
    // action: there is no scalar organism left to special-case.
    let world = World::new(11, 24);
    for organism in &world.organisms {
        assert!(!organism.body().is_empty(), "{:?} has anatomy", organism.id);
        let preview = world
            .lineages()
            .get(organism.species)
            .unwrap()
            .realize(
                organism.development_seed,
                organism.biomass_mg(),
                world.development_palette(),
            )
            .unwrap();
        assert_eq!(
            *organism.body(),
            preview,
            "founding and preview share one developer"
        );
        assert_eq!(
            organism.half_extent(),
            organism
                .body()
                .part(organism.body().root)
                .unwrap()
                .half_extent,
            "and its shape is read off that anatomy rather than stored beside it"
        );
    }
}

#[test]
fn control_moves_only_through_a_recorded_intent() {
    // Ordered intents are the only mutation path. A control change made
    // outside that path would replay every fact about a run except who was
    // living it, and lineage switching is gameplay.
    let mut world = World::new(4_242, 24);
    let other = prey(&world);

    let roster_before = world.organisms.clone();
    let mine = world.body().unwrap().clone();

    let outcome = world.apply(Intent::TakeControl { organism: other });

    assert_eq!(outcome, Outcome::Inhabited { organism: other });
    assert_eq!(world.controlled_id(), Some(other));
    assert_ne!(
        world.body().unwrap(),
        &mine,
        "the played body is somebody else's now"
    );
    // The ecology ran a tick, so the roster is not frozen; what matters is
    // that nothing was rebuilt or copied for the sake of control.
    assert!(roster_before.len().abs_diff(world.organisms.len()) < 5);
}

#[test]
fn a_control_change_replays() {
    // The hole this closes: a trace containing a lineage switch must reproduce
    // who was inhabited, not merely what happened to the world.
    let trace = [
        Intent::Move { delta: [1, 0, 0] },
        Intent::Idle,
        Intent::Move { delta: [0, 0, 1] },
    ];

    let mut straight = World::new(4_242, 24);
    let other = prey(&straight);
    straight.apply(Intent::TakeControl { organism: other });
    straight.apply_all(&trace);

    let mut forked = World::new(4_242, 24);
    forked.apply(Intent::TakeControl { organism: other });
    forked.apply_all(&trace[..1]);
    let mut resumed = restore(&snapshot(&forked).unwrap()).unwrap();
    resumed.apply_all(&trace[1..]);

    assert_eq!(state_hash(&straight), state_hash(&resumed));
    assert_eq!(
        resumed.controlled_id(),
        Some(other),
        "and control survived the snapshot"
    );
}

#[test]
fn control_refuses_an_organism_that_cannot_be_played() {
    let mut world = World::new(3, 8);
    let before = world.controlled_id();

    let absent = OrganismId(9_999);
    assert_eq!(
        world.apply(Intent::TakeControl { organism: absent }),
        Outcome::Rejected(Rejection::Ineligible(
            mesocosm_core::Ineligible::NoSuchOrganism
        ))
    );
    assert_eq!(world.controlled_id(), before, "control did not move");
}

#[test]
fn serialization_does_not_distinguish_the_played_critter() {
    // Law C at the level of the simulation rather than the file format. Two
    // worlds identical except for which organism is pointed at must serialize
    // to the same length and restore identically, because the only difference
    // is one id.
    let mut world = World::new(4_242, 24);
    let other = prey(&world);

    let mut moved = world.clone();
    moved.apply(Intent::TakeControl { organism: other });
    world.apply(Intent::Idle);

    // Drained first, because the two worlds did different things and so carry
    // different pending events. The claim is about the *pointer* costing
    // nothing, not about two action histories weighing the same.
    world.drain_events();
    moved.drain_events();

    // The *organisms* are what must be indistinguishable. Taking control also
    // unlocks a lineage, which is player progress rather than a fact about any
    // creature, so it legitimately costs a little. Law C forbids a marker on
    // the creature, not a record of where the player has been.
    assert_eq!(
        world.organisms.len(),
        moved.organisms.len(),
        "no creature was added or removed by inhabiting one"
    );
    // **Identity, not milligrams.** `apply` always advances a tick, and a held
    // body is the one body the ecology does not walk (TD4) — so the two worlds
    // walked different creatures on that tick. Since TD7 a producer's uptake
    // reads a whole forage neighbourhood rather than the column it stands on,
    // which is enough for that one difference to reach a plant's mouthful and
    // move it a milligram. The simulation is entitled to diverge; what Law C
    // forbids is a *mark* on the creature, so every field of an organism that
    // is not something the tick moves must match, body shapes included.
    let identity = |o: &mesocosm_core::Organism| {
        (
            o.id,
            o.species,
            o.guise,
            o.signal,
            o.venom_mg,
            o.tier,
            o.stage,
            o.development_seed,
            o.life_history_mass_mg,
            o.body().plan.clone(),
            o.body()
                .parts
                .iter()
                .map(|part| {
                    (
                        part.id,
                        part.half_extent,
                        part.volume,
                        part.pivot,
                        part.attachment,
                        part.provenance.clone(),
                        part.severed,
                    )
                })
                .collect::<Vec<_>>(),
        )
    };
    for (mine, theirs) in world.organisms.iter().zip(&moved.organisms) {
        assert_eq!(
            identity(mine),
            identity(theirs),
            "a creature carries something that says whether it is the played one"
        );
    }

    let a = snapshot(&world).unwrap();
    let b = snapshot(&moved).unwrap();
    // A loose sanity bound, and deliberately loose: after the field-by-field
    // check above, the only thing left that can move these lengths is the
    // width of the numbers a diverged tick wrote. A played flag would be a
    // field per creature and the check above would already have caught it.
    assert!(
        a.len().abs_diff(b.len()) < a.len() / 100,
        "inhabiting costs a lineage id, not a representation ({} vs {})",
        a.len(),
        b.len()
    );

    assert_eq!(restore(&a).unwrap(), world);
    assert_eq!(restore(&b).unwrap(), moved);
}

#[test]
fn the_ecology_reads_a_recorded_focus_and_never_the_controller() {
    // This test's earlier phrasing ("forty ticks ran identically regardless
    // of who was inhabited") predates tiers. Focus-tiered dispersal makes
    // the world legitimately follow the played body's *position*. The law
    // survives in its real form: that position is recorded state (control
    // moves only through intents), so replay owns every divergence, the
    // controller's *identity* never enters a rule, and tier membership is
    // per-organism recorded state that a snapshot preserves.
    let mut world = World::new(77, 24);
    for _ in 0..30 {
        world.apply(Intent::Idle);
    }

    // Tier membership must be explained by hops from the recorded focus,
    // never by who the controller is. The hysteresis band (1..3 hops) may
    // hold either tier; outside it the tier is forced.
    use mesocosm_core::places::{Tier, TierLine};
    let focus = world.position().expect("embodied");
    let focus_place = world.places().at(focus).expect("focus is somewhere");
    let line = TierLine::default();
    for organism in world.organisms.iter().filter(|o| o.is_alive()) {
        let place = world.places().at(organism.position).expect("somewhere");
        let hops = world.places().hops(place, focus_place).expect("connected");
        match organism.tier {
            Tier::Near => assert!(
                hops < line.demote_hops,
                "{:?} is Near at {hops} hops",
                organism.id
            ),
            Tier::Far => assert!(
                hops > line.promote_hops,
                "{:?} is Far at {hops} hops",
                organism.id
            ),
        }
    }
    // And demotion is reachable in this very graph: a corner-to-corner
    // journey exceeds the band, so distance alone sends an agent Far.
    // And demotion is reachable in this very graph. Grown links are
    // better-connected than the old lattice (corners may join directly),
    // so the receipt finds a maximally distant pair rather than assuming
    // which corners are far apart.
    let (mut widest, mut pair) = (0, None);
    for a in world.places().all() {
        for b in world.places().all() {
            let hops = world.places().hops(a.id, b.id).unwrap();
            if hops > widest {
                widest = hops;
                pair = Some((a.centre, b.centre));
            }
        }
    }
    assert!(
        widest >= line.demote_hops,
        "the grown enclosure's diameter ({widest}) cannot reach the far tier"
    );
    let (near_c, far_c) = pair.unwrap();
    assert_eq!(
        line.tick(
            world.places(),
            Tier::Near,
            [far_c[0], 0, far_c[1]],
            [near_c[0], 0, near_c[1]],
        ),
        Tier::Far,
        "distance alone sends an agent Far"
    );

    let resumed = restore(&snapshot(&world).unwrap()).unwrap();
    assert_eq!(
        world.organisms, resumed.organisms,
        "tier membership is recorded state, not a per-run view"
    );
    assert_eq!(state_hash(&world), state_hash(&resumed));
}

#[test]
fn a_critter_cannot_eat_itself() {
    // Only expressible since P1 put the player in the roster. This forbids
    // targeting yourself as *prey*; consuming one of your own parts during
    // starvation or metamorphosis is a different, part-addressed operation and
    // is not ruled out here.
    let mut world = World::new(5, 12);
    let me = world.controlled_id().unwrap();

    assert_eq!(
        world.apply(Intent::Metabolize {
            organism: me,
            placement: Placement::Planned
        }),
        Outcome::Rejected(Rejection::Itself)
    );
    assert_eq!(
        world.apply(Intent::Metabolize {
            organism: me,
            placement: Placement::Planned,
        }),
        Outcome::Rejected(Rejection::Itself)
    );
}

#[test]
fn dying_ends_control_rather_than_the_world() {
    // Killed through the ecology, not by removing the row. Natural death makes
    // an organism carrion, which lingers until it is spent, so an earlier cut
    // that only checked for the id kept a decomposing critter walking around.
    let mut world = World::new(13, 16);
    let me = world.controlled_id().unwrap();

    // Starve it. Drain the budget and the body, then let upkeep finish the
    // job; nothing intervenes on its behalf.
    {
        let me_now = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
        me_now.energy_mg = 0;
        let all = me_now.biomass_mg();
        me_now.spend_mass(all.saturating_sub(1));
    }

    let mut ticks = 0;
    while world.is_embodied() && ticks < 500 {
        world.apply(Intent::Idle);
        ticks += 1;
    }

    assert!(
        !world.is_embodied(),
        "the played critter died like anything else"
    );
    assert_eq!(
        world.control_lost(),
        Some(me),
        "and the world says whose body was lost"
    );
    assert_eq!(
        world.controlled_id(),
        None,
        "the pointer was released, not left stale"
    );
}

#[test]
fn a_carcass_cannot_be_played() {
    // The specific defect: carrion is still in the roster, so an id check
    // alone reports it as embodied.
    let mut world = World::new(21, 16);
    let me = world.controlled_id().unwrap();
    world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .stage = mesocosm_core::Stage::Carrion;

    assert!(
        world.organisms.iter().any(|o| o.id == me),
        "the row is still there"
    );
    assert!(!world.is_embodied(), "and it is not somebody you can be");
    assert_eq!(
        world.apply(Intent::Move { delta: [1, 0, 0] }),
        Outcome::Rejected(Rejection::Disembodied),
        "a corpse does not walk"
    );
}

#[test]
fn a_world_can_outlive_whoever_was_in_it() {
    // Nobody home is a state, not a crash, and it is the seam where
    // witnessing, adaptation, and choosing another critter happen.
    let mut world = World::new(13, 16);
    let me = world.controlled_id().unwrap();
    world.organisms.retain(|o| o.id != me);
    world.apply(Intent::Idle);

    assert!(!world.is_embodied());
    assert_eq!(world.controlled(), None);
    assert_eq!(world.body(), None, "no ghost body at the origin");
    assert_eq!(world.position(), None);
    assert_eq!(world.energy_mg(), None);

    assert_eq!(
        world.apply(Intent::Move { delta: [1, 0, 0] }),
        Outcome::Rejected(Rejection::Disembodied),
        "acting refuses rather than panicking"
    );
    assert_eq!(
        world.apply(Intent::Idle),
        Outcome::Idled,
        "and time still passes"
    );
}

#[test]
fn a_disembodied_world_can_be_inhabited_again() {
    // What disembodiment is *for*: it is a seam, not a dead end.
    let mut world = World::new(29, 24);
    let me = world.controlled_id().unwrap();
    let heir = prey(&world);

    // Hold it long enough to earn a frontier, then lose the body entirely.
    world.apply(Intent::Idle);
    let earned = world.frontier();
    assert!(earned > 0, "holding a body earns a frontier");

    world.organisms.retain(|o| o.id != me);
    world.apply(Intent::Idle);
    assert!(!world.is_embodied());
    assert_eq!(world.frontier(), earned, "losing a body does not unearn it");

    // Something simpler than what was reached is a legitimate heir.
    let simpler = world
        .organisms
        .iter()
        .filter(|o| o.is_alive() && o.complexity() < earned)
        .map(|o| o.id)
        .min()
        .unwrap_or(heir);

    assert_eq!(
        world.apply(Intent::TakeControl { organism: simpler }),
        Outcome::Inhabited { organism: simpler },
        "a world nobody is in can be entered again"
    );
    assert!(world.is_embodied());
}

#[test]
fn replay_holds_within_the_new_schema() {
    // The determinism guarantee, restated honestly: a build replays its own
    // traces. The migration moved hashes because the schema changed, and that
    // is a version boundary rather than a broken promise.
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
    let mut resumed = restore(&snapshot(&forked).unwrap()).unwrap();
    resumed.apply_all(&trace[2..]);

    assert_eq!(state_hash(&straight), state_hash(&resumed));
}

#[test]
fn reproduction_does_not_manufacture_body_mass() {
    // A parent used to pay a quarter of its scalar mass while its offspring
    // received a clone of its whole anatomy, so a forty-part parent produced a
    // forty-part child out of nothing. A newborn now realizes its lineage's
    // recipe with exactly what the parent paid; if that cannot keep every part
    // positive-mass, the birth waits.
    let mut world = World::new(4_242, 40);

    let mut checked = 0;
    for _ in 0..600 {
        world.apply(Intent::Idle);
        for newborn in world.organisms.iter().filter(|o| o.age == 0) {
            assert_eq!(
                newborn.body().total_mass_mg(),
                newborn.biomass_mg(),
                "{:?} was born with anatomy it did not pay for",
                newborn.id
            );
            let expected = world
                .lineages()
                .get(newborn.species)
                .unwrap()
                .realize(
                    newborn.development_seed,
                    newborn.biomass_mg(),
                    world.development_palette(),
                )
                .unwrap();
            assert_eq!(*newborn.body(), expected, "birth and founder preview agree");
            assert!(
                newborn.body().len() > 1,
                "the recipe reached live offspring"
            );
            checked += 1;
        }
        if checked > 3 {
            break;
        }
    }

    assert!(checked > 0, "the fixture actually reproduced");
}

#[test]
fn you_may_step_down_but_not_across() {
    // The complexity frontier, finally binding at the point control moves.
    // It was ruled long ago and lived in `epoch::can_switch_to`, which nothing
    // outside its own tests ever called, so control could take anything alive
    // however elaborate. That module was deleted on 2026-09-04; this is where
    // the rule binds now.
    let world = World::new(4_242, 40);
    let frontier = world.frontier();

    let simpler = world
        .organisms
        .iter()
        .find(|o| {
            o.is_alive() && world.intricacy(o) < frontier && Some(o.id) != world.controlled_id()
        })
        .map(|o| o.id)
        .expect("something in the world is simpler than the player");

    assert!(
        world.is_eligible(simpler),
        "stepping down into a simpler niche is the point"
    );

    // Something more elaborate than anything earned is refused, and says why.
    let mut world = World::new(4_242, 40);
    let grand = OrganismId(9_500);
    world.organisms.push(mesocosm_core::Organism {
        stage: mesocosm_core::Stage::Mature,
        ..mesocosm_core::Organism::founding(
            grand,
            mesocosm_core::SpeciesId(99),
            mesocosm_core::Kingdom::Consumer,
            mesocosm_core::VolumeRef::from_tag(3),
            [4, 4, 4],
            world.position().unwrap(),
            50_000,
        )
    });

    assert!(
        matches!(
            world.eligibility(grand),
            Err(mesocosm_core::Ineligible::AboveTheFrontier { .. })
        ),
        "an unearned peer is refused"
    );
    assert!(matches!(
        world.apply(Intent::TakeControl { organism: grand }),
        Outcome::Rejected(Rejection::Ineligible(
            mesocosm_core::Ineligible::AboveTheFrontier { .. }
        ))
    ));
    let _ = simpler;
}

#[test]
fn a_line_you_have_lived_is_always_yours_to_return_to() {
    // The frontier gates reaching outward, not going home. Otherwise growing a
    // body would lock you out of the line you grew it in.
    let mut world = World::new(4_242, 40);
    let mine = world.controlled().unwrap().species;

    for _ in 0..40 {
        world.apply(Intent::Idle);
    }

    let kin = world
        .organisms
        .iter()
        .find(|o| o.species == mine && o.is_alive() && Some(o.id) != world.controlled_id())
        .map(|o| o.id);

    if let Some(kin) = kin {
        assert!(
            world.is_eligible(kin),
            "your own kind is never above your frontier"
        );
    }
    assert!(world.unlocked().any(|s| s == mine));
}

#[test]
fn the_frontier_only_goes_up() {
    // What you reach, you keep. An earlier cut read the frontier from living
    // organisms, so a lineage dying out collapsed it to zero and left the
    // world permanently uninhabitable, which contradicts disembodiment being a
    // seam rather than a dead end.
    let mut world = World::new(4_242, 40);
    world.apply(Intent::Idle);
    let earned = world.frontier();
    assert!(earned > 0);

    let me = world.controlled_id().unwrap();
    world.organisms.retain(|o| o.id != me);
    for _ in 0..20 {
        world.apply(Intent::Idle);
    }

    assert!(!world.is_embodied(), "the body is gone");
    assert_eq!(
        world.frontier(),
        earned,
        "and the standing it earned is not"
    );
}

#[test]
fn growing_raises_the_frontier() {
    let mut world = World::new(4_242, 40);
    let before = world.frontier();

    // Since 2026-08-03 the frontier reads the *recipe's* intricacy rather than
    // the body's part count, so bulk does not lift it. Learning does: teach
    // the line a word it did not have and the ceiling rises.
    let mine = world.controlled().unwrap().species;
    {
        let species = world.lineages_mut().get_mut(mine).unwrap();
        assert!(species.recipe.acquire(mesocosm_core::Appendage::Vane));
    }
    world.apply(Intent::Idle);

    assert!(
        world.frontier() > before,
        "the ceiling rose with what the line learned"
    );
}

#[test]
fn speciation_is_an_act_and_the_name_is_the_doing() {
    // Lineages could never split before this: reproduction copied the parent's
    // species verbatim and nothing else ever assigned one, so no new species
    // was ever born in a Mesocosm world.
    let mut world = World::new(4_242, 40);
    let me = world.controlled_id().unwrap();
    let before = world.controlled().unwrap().species;

    let outcome = world.apply(Intent::Speciate {
        name: "the pale kind".into(),
    });

    let Outcome::Speciated {
        species,
        from,
        founder,
    } = outcome
    else {
        panic!("expected a split, got {outcome:?}")
    };
    assert_eq!((from, founder), (before, me));
    assert_ne!(species, before, "a new line, not a rename");

    let forked = world.lineages().get(species).unwrap();
    assert_eq!(forked.name.as_deref(), Some("the pale kind"));
    assert_eq!(
        forked.parent,
        Some(before),
        "and it remembers what it came from"
    );
}

#[test]
fn a_founder_crosses_alone() {
    // Forking takes the creature you are holding and nothing else, which makes
    // it a commitment rather than a free rename. A new line begins with one
    // individual, which is how a founder effect actually works.
    let mut world = World::new(4_242, 40);
    let me = world.controlled_id().unwrap();
    let old = world.controlled().unwrap().species;
    let kin: Vec<OrganismId> = world
        .organisms
        .iter()
        .filter(|o| o.species == old && o.id != me)
        .map(|o| o.id)
        .collect();

    let Outcome::Speciated { species, .. } = world.apply(Intent::Speciate {
        name: "alone".into(),
    }) else {
        panic!("expected a split")
    };

    assert_eq!(
        world.controlled().unwrap().species,
        species,
        "the founder crossed"
    );
    for other in kin {
        let still = world.organisms.iter().find(|o| o.id == other);
        if let Some(still) = still {
            assert_eq!(still.species, old, "its former kin kept the old line");
        }
    }
}

#[test]
fn a_world_begins_with_unnamed_lineages_and_gains_named_ones() {
    // Naming promotes a line out of being a variation, which is the same rule
    // that promotes a critter out of being a statistic.
    let mut world = World::new(4_242, 40);
    assert_eq!(
        world.lineages().named().count(),
        0,
        "nobody was there to name them"
    );
    assert!(world.lineages().len() > 1, "but the world has lineages");

    world.apply(Intent::Speciate {
        name: "named".into(),
    });
    assert_eq!(world.lineages().named().count(), 1);
}

#[test]
fn kinship_becomes_computable() {
    // The axis graft compatibility was ruled to scale with. Before lineages
    // could split, every pair of creatures was either identical or unrelated
    // and the measure said nothing.
    let mut world = World::new(4_242, 40);
    let me = world.controlled_id().unwrap();

    world.apply(Intent::Speciate {
        name: "first".into(),
    });
    let stranger = world
        .organisms
        .iter()
        .find(|o| o.id != me && o.is_alive())
        .map(|o| o.id)
        .expect("the world is populated");

    assert_eq!(
        world.kinship(me, me),
        Some(0),
        "a creature is no distance from itself"
    );
    // A founding lineage and a line forked off a different founder share no
    // ancestor, which is a real answer rather than a large number.
    let apart = world.kinship(me, stranger);
    assert!(apart.is_none() || apart == Some(1), "got {apart:?}");
}

#[test]
fn speciating_needs_a_body() {
    let mut world = World::new(13, 16);
    let me = world.controlled_id().unwrap();
    world.organisms.retain(|o| o.id != me);
    world.apply(Intent::Idle);

    assert_eq!(
        world.apply(Intent::Speciate {
            name: "nobody".into()
        }),
        Outcome::Rejected(Rejection::Disembodied),
        "a line with nobody in it cannot be split"
    );
}

#[test]
fn a_split_is_recorded_in_the_founders_own_history() {
    use mesocosm_core::{Event, History};

    let mut world = World::new(4_242, 40);
    let mut history = History::new();
    history.record_all(world.drain_events());

    let me = world.controlled_id().unwrap();
    world.apply(Intent::Speciate {
        name: "recorded".into(),
    });
    history.record_all(world.drain_events());

    let split = history
        .log()
        .entries()
        .iter()
        .any(|e| matches!(e.record, Event::Speciated { founder, .. } if founder == me));
    assert!(split, "the founder's line shows where it forked");
}

#[test]
fn eating_teaches_the_line_a_word() {
    // The acquisition half of kleptoplasty, ruled 2026-08-03: a lineage cannot
    // express an appendage it has never eaten, and a meal that teaches is a
    // different kind of event from a meal that feeds.
    let mut world = World::new(4_242, 40);
    let mine = world.controlled().unwrap().species;

    // Find something whose line grows an appendage ours has not learned.
    let unknown: Vec<_> = world
        .lineages()
        .all()
        .flat_map(|s| {
            s.recipe
                .tagmata
                .iter()
                .map(|t| t.appendage)
                .collect::<Vec<_>>()
        })
        .filter(|a| !a.is_innate())
        .filter(|a| !world.lineages().get(mine).unwrap().recipe.can_express(*a))
        .collect();
    assert!(
        !unknown.is_empty(),
        "the enclosure holds words we do not have"
    );

    // Teach it directly and confirm the rule the world enforces.
    let word = unknown[0];
    assert!(!world.lineages().get(mine).unwrap().recipe.can_express(word));
    assert!(
        world
            .lineages_mut()
            .get_mut(mine)
            .unwrap()
            .recipe
            .acquire(word)
    );
    assert!(world.lineages().get(mine).unwrap().recipe.can_express(word));
    assert!(
        !world
            .lineages_mut()
            .get_mut(mine)
            .unwrap()
            .recipe
            .acquire(word),
        "the second one is just a meal"
    );
}

#[test]
fn a_seeded_world_holds_several_body_plans() {
    // The enclosure used to be one shape at several sizes. Every founding
    // line now draws its own recipe.
    let world = World::new(4_242, 40);
    let shapes: std::collections::BTreeSet<String> = world
        .lineages()
        .all()
        .map(|s| format!("{:?}", s.recipe.tagmata))
        .collect();

    assert!(shapes.len() > 1, "a world of one plan is the old world");
    assert!(
        world.lineages().all().any(|s| s.recipe.appendages() > 1),
        "something out there has appendages"
    );
}

#[test]
fn a_fork_inherits_what_its_parent_learned() {
    let mut world = World::new(4_242, 40);
    let mine = world.controlled().unwrap().species;
    world
        .lineages_mut()
        .get_mut(mine)
        .unwrap()
        .recipe
        .acquire(mesocosm_core::Appendage::Vane);

    let Outcome::Speciated { species, .. } = world.apply(Intent::Speciate {
        name: "heir".into(),
    }) else {
        panic!("the fork happened");
    };
    assert!(
        world
            .lineages()
            .get(species)
            .unwrap()
            .recipe
            .can_express(mesocosm_core::Appendage::Vane),
        "a founder does not forget what its line had learned"
    );
}
