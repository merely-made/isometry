// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! P3: branch transfer.
//!
//! One test per clause of the gate. *Harvest or receive a source subtree,
//! remap its local ids, and preserve its source addresses and parent
//! relations. Done when: the source loses the branch, the recipient gains a
//! functioning or visibly incompatible branch according to the chosen route,
//! severing the graft cascades, and snapshot/replay agree.*
//!
//! The donor is a **carcass**, for PE2's reason and not a new one: a severed
//! part's milligrams have already left the conservation account, so harvesting
//! one would create matter, and live dismemberment is a further gate.

use mesocosm_core::{
    AllocationProposal, Arrangement, Attachment, CellId, Crossing, Domain, Expressed, Intent,
    Kingdom, Organism, OrganismId, Origin, Outcome, PartId, Process, ProcessRef, ProposedSite,
    Provenance, Registry, Rejection, SpeciesId, Stage, Verdict, VolumeRef, World, Yaw,
};

use super::bulk_world;

/// The donor line, and the id its carcass carries.
const DONOR_LINE: SpeciesId = SpeciesId(5);
const DONOR: OrganismId = OrganismId(9_700);

/// A plate wide enough to hold a gland and a limb underneath it: PD2's frond,
/// so the arrangement a branch carries is one the receipts already price.
const FROND: [i32; 3] = [6, 4, 1];

fn gland() -> ProcessRef {
    Registry::native().of_native(Process::Secrete).reference()
}

fn fixing() -> ProcessRef {
    Registry::native().of_native(Process::Fix).reference()
}

/// A world whose played critter is a bulk root, and whose two lines' tissue
/// domains are the ones this test wants.
///
/// Set rather than drawn: the world assigns domains from its own stream, and a
/// test about what a verdict *does* must not be a test about what the stream
/// happened to draw.
fn world_with(verdict: Verdict) -> World {
    let mut world = bulk_world(4_242, 24);
    let mine = world.controlled().expect("embodied").species;
    // Domain 1 into domain 1 is native; 1 into 2 is the favoured edge and
    // needs an adapter; 2 into 1 is that edge reversed, and is refused.
    let (from, into) = match verdict {
        Verdict::Native => (1, 1),
        Verdict::Adapter => (1, 2),
        Verdict::Refused => (2, 1),
    };
    let lineages = world.lineages_mut();
    lineages.found(DONOR_LINE);
    lineages.set_domain(DONOR_LINE, Domain(from));
    lineages.set_domain(mine, Domain(into));
    assert_eq!(
        world.verdict_between(DONOR_LINE, mine),
        verdict,
        "the fixture's premise"
    );
    world
}

/// A carcass in reach carrying a two-part branch: a frond with a limb hanging
/// off it, and a gland arranged on four of the frond's twelve cells.
///
/// The limb is what makes this a *branch* rather than an organ: it has to come
/// with the frond, and it has to arrive still hanging off it.
fn donor(world: &mut World) -> (PartId, PartId) {
    let here = world.position().expect("embodied");
    let at = [here[0] + 1, here[1], here[2]];
    let mut corpse = Organism {
        stage: Stage::Carrion,
        ..Organism::founding(
            DONOR,
            DONOR_LINE,
            Kingdom::Producer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            at,
            1_200,
        )
    };
    let root = corpse.body().root;
    let frond = corpse
        .phenotype
        .attach(
            VolumeRef::from_tag(7),
            400,
            FROND,
            Attachment {
                parent: root,
                offset: [0, 7, 0],
                yaw: Yaw::Zero,
            },
            Provenance::founding(),
        )
        .expect("a frond attaches to the root");
    let tip = corpse
        .phenotype
        .attach(
            VolumeRef::from_tag(9),
            150,
            [7, 1, 1],
            Attachment {
                parent: frond,
                offset: [13, 0, 0],
                yaw: Yaw::Quarter,
            },
            Provenance::founding(),
        )
        .expect("and a limb hangs off the frond");

    // The arrangement the branch will carry: a gland on the top third of the
    // frond, fixing on the rest. Through the one validator, like everything
    // else that moves allocation.
    let capacity = corpse
        .phenotype
        .mosaic(frond)
        .expect("a living part carries a mosaic")
        .capacity();
    let kept: Vec<CellId> = (0..capacity - 4).map(|i| CellId(i as u16)).collect();
    let taken: Vec<CellId> = (capacity - 4..capacity).map(|i| CellId(i as u16)).collect();
    let proposal = AllocationProposal {
        expect: corpse.phenotype.digest(),
        source: Arrangement::Direct,
        parts: vec![frond],
        sites: vec![
            ProposedSite {
                part: frond,
                process: fixing(),
                cells: kept,
            },
            ProposedSite {
                part: frond,
                process: gland(),
                cells: taken,
            },
        ],
    };
    corpse
        .phenotype
        .develop(mesocosm_core::Registry::native(), &proposal)
        .expect("the donor's own arrangement is valid on the donor");
    assert!(corpse.phenotype.expresses_on(frond, gland()));

    world.organisms.push(corpse);
    (frond, tip)
}

fn corpse_of(world: &World) -> &Organism {
    world
        .organisms
        .iter()
        .find(|o| o.id == DONOR)
        .expect("still on the roster")
}

fn take(world: &mut World, part: PartId, crossing: Crossing) -> Outcome {
    world.apply(Intent::Graft {
        organism: DONOR,
        part,
        crossing,
    })
}

// ---------------------------------------------------------------------------
// The source loses the branch
// ---------------------------------------------------------------------------

#[test]
fn the_source_loses_the_branch_and_the_recipient_gains_every_part_of_it() {
    let mut world = world_with(Verdict::Native);
    let (frond, tip) = donor(&mut world);
    let matter_before = world.total_matter_mg();
    let branch_mg = {
        let body = corpse_of(&world).body();
        body.part(frond).unwrap().mass_mg + body.part(tip).unwrap().mass_mg
    };
    let donor_before = corpse_of(&world).biomass_mg();
    let mine_before = world.controlled().unwrap().biomass_mg();
    let parts_before = world.body().unwrap().living().count();

    let outcome = take(&mut world, frond, Crossing::Carry);
    let Outcome::Grafted {
        root,
        parts,
        from,
        from_part,
        mass_mg,
        ..
    } = outcome
    else {
        panic!("{outcome:?}");
    };
    assert_eq!((from, from_part), (DONOR, frond));
    assert_eq!(parts, 2, "the frond and what hung off it");
    assert_eq!(mass_mg, branch_mg);

    // The source lost it, and lost the whole of it.
    let after = corpse_of(&world);
    assert!(!after.body().is_living(frond), "the frond is gone");
    assert!(!after.body().is_living(tip), "and so is what hung off it");
    assert_eq!(after.biomass_mg(), donor_before - branch_mg);

    // The recipient gained it, and gained exactly it.
    let mine = world.body().unwrap();
    assert_eq!(mine.living().count(), parts_before + 2);
    assert!(mine.is_living(root));
    assert_eq!(
        world.controlled().unwrap().biomass_mg(),
        mine_before + branch_mg
    );
    assert_eq!(
        world.total_matter_mg(),
        matter_before,
        "and the enclosure's matter did not move"
    );
}

#[test]
fn every_transferred_part_names_the_part_it_came_off_and_keeps_its_joint() {
    // Remapping and provenance, which are the two halves of "the same branch,
    // somewhere else": fresh local ids, the internal parent relation rewritten
    // onto them, the joint preserved exactly, and each part's source address
    // naming **that** part rather than the branch root or the donor's root.
    let mut world = world_with(Verdict::Native);
    // A limb on the recipient first, so its next free id is not the donor's
    // and "remapped" is a claim with a difference in it.
    super::grow_a_limb(&mut world);
    let (frond, tip) = donor(&mut world);
    let joint = corpse_of(&world)
        .body()
        .part(tip)
        .unwrap()
        .attachment
        .expect("the limb has a joint");

    let Outcome::Grafted { root, .. } = take(&mut world, frond, Crossing::Carry) else {
        panic!("the graft was refused");
    };
    let body = world.body().unwrap();
    let arrived: Vec<PartId> = body.descendants(root);
    assert_eq!(arrived.len(), 2);

    let source_of = |part: PartId| match body.part(part).unwrap().provenance.origin {
        Origin::Incorporated {
            from_species,
            from_part,
        } => (from_species, from_part),
        Origin::Founding => panic!("a transferred part is not founding tissue"),
    };
    assert_eq!(source_of(arrived[0]), (DONOR_LINE, frond));
    assert_eq!(
        source_of(arrived[1]),
        (DONOR_LINE, tip),
        "the limb names itself, not the branch root"
    );
    assert!(
        arrived[0] != frond && arrived[1] != tip,
        "the ids are this body's own, freshly allocated: {arrived:?}"
    );

    let landed = body.part(arrived[1]).unwrap().attachment.unwrap();
    assert_eq!(
        landed.parent, arrived[0],
        "the internal parent relation was remapped onto the new ids"
    );
    assert_eq!(
        (landed.offset, landed.yaw),
        (joint.offset, joint.yaw),
        "and the joint itself came across untouched"
    );
}

// ---------------------------------------------------------------------------
// Functioning, or visibly incompatible, according to the route
// ---------------------------------------------------------------------------

#[test]
fn a_native_carry_lands_a_functioning_branch() {
    // The donor's arrangement, cell for cell, doing on this body what it did on
    // that one. The gland is the proof because it is the one process a played
    // reading already prices: a body with the branch stings, and the same body
    // without it does not.
    let mut world = world_with(Verdict::Native);
    let (frond, _) = donor(&mut world);
    let donor_cells: Vec<(ProcessRef, Vec<CellId>)> = corpse_of(&world)
        .phenotype
        .mosaic(frond)
        .unwrap()
        .sites()
        .iter()
        .map(|site| (site.process, site.cells.clone()))
        .collect();
    assert_eq!(world.controlled().unwrap().phenotype.secretory_mg(), 0);

    let Outcome::Grafted { root, verdict, .. } = take(&mut world, frond, Crossing::Carry) else {
        panic!("a native carry was refused");
    };
    assert_eq!(verdict, Verdict::Native);

    let phenotype = world.phenotype().unwrap();
    assert!(
        phenotype.expresses_on(root, gland()),
        "the gland arrived with the tissue it was on"
    );
    assert!(
        phenotype.secretory_mg() > 0,
        "and this body stings now, which it could not before"
    );
    let landed: Vec<(ProcessRef, Vec<CellId>)> = phenotype
        .mosaic(root)
        .unwrap()
        .sites()
        .iter()
        .map(|site| (site.process, site.cells.clone()))
        .collect();
    assert_eq!(
        landed, donor_cells,
        "cell for cell, which is what carrying an arrangement means"
    );
    // The sites are the graft's, not this body's geometry talking: the
    // development that placed them says which revision they arrived on.
    assert!(
        phenotype
            .mosaic(root)
            .unwrap()
            .sites()
            .iter()
            .all(|site| matches!(site.cause, Expressed::Arranged { .. })),
        "a carried arrangement was arranged, and the record says so"
    );
    assert!(phenotype.conserves());
}

#[test]
fn a_cross_domain_carry_lands_a_visibly_incompatible_branch() {
    // The second verdict. The branch is *on* the body — it weighs what it
    // weighed, it is drawn, its parts are named — and its cut boundary does not
    // speak this body's language, so it expresses nothing until an adapter is
    // grown on it. That is the difference between a graft that failed and one
    // that is incompatible, and a player can see both.
    let mut world = world_with(Verdict::Adapter);
    let (frond, _) = donor(&mut world);
    let branch_mg = corpse_of(&world).body().part(frond).unwrap().mass_mg;
    let mine_before = world.controlled().unwrap().biomass_mg();

    let Outcome::Grafted { root, verdict, .. } = take(&mut world, frond, Crossing::Carry) else {
        panic!("a favoured cross-domain carry was refused");
    };
    assert_eq!(verdict, Verdict::Adapter);

    let phenotype = world.phenotype().unwrap();
    assert!(
        world.body().unwrap().is_living(root),
        "the branch is on the body"
    );
    assert!(
        world.controlled().unwrap().biomass_mg() > mine_before + branch_mg - 1,
        "carrying its full weight"
    );
    assert!(
        !phenotype.expresses_on(root, gland()) && !phenotype.expresses_on(root, fixing()),
        "and doing nothing at all with it"
    );
    let explained = phenotype.explain(root).expect("a living part explains");
    assert!(explained.living);
    assert_eq!(
        explained.free, explained.capacity,
        "every cell of it is free, which is what needing an adapter looks like"
    );
    assert!(explained.sites.is_empty());
    assert_eq!(phenotype.secretory_mg(), 0, "the gland did not come across");

    // And it is repairable: an adapter is an ordinary development on ordinary
    // free tissue, stated to the one validator (PD3 deleted the door that
    // carried a hand-drawn arrangement).
    let capacity = explained.capacity;
    let proposal = AllocationProposal {
        expect: world.phenotype().unwrap().digest(),
        source: Arrangement::Direct,
        parts: vec![root],
        sites: vec![ProposedSite {
            part: root,
            process: gland(),
            cells: (0..capacity).map(|i| CellId(i as u16)).collect(),
        }],
    };
    super::develop_played(&mut world, &proposal).expect("the branch can be made to work");
    assert!(world.phenotype().unwrap().secretory_mg() > 0);
}

#[test]
fn a_disfavoured_carry_is_refused_and_regrowth_is_the_route_that_remains() {
    // The third verdict, and the wing contract's rule for it: an incompatible
    // carry is refused or redirected to regrowth, never silently rewritten. So
    // the refusal names the boundary, and the other crossing still lands.
    let mut world = world_with(Verdict::Refused);
    let (frond, _) = donor(&mut world);
    let before = mesocosm_core::state_hash(&world);

    let refused = take(&mut world, frond, Crossing::Carry);
    assert_eq!(
        refused,
        Outcome::Rejected(Rejection::Incompatible {
            from: Domain(2),
            into: Domain(1),
        }),
        "the refusal names which tissue would not go into which"
    );
    assert!(
        corpse_of(&world).body().is_living(frond),
        "and the corpse still has its branch"
    );

    let outcome = take(&mut world, frond, Crossing::Regrow);
    let Outcome::Grafted { root, verdict, .. } = outcome else {
        panic!("regrowth is supposed to be feasible: {outcome:?}");
    };
    assert_eq!(
        verdict,
        Verdict::Refused,
        "the table did not change its mind"
    );
    let phenotype = world.phenotype().unwrap();
    assert!(
        phenotype.expresses_on(root, fixing()),
        "a regrown plate does what this body's rules make of a plate"
    );
    assert!(
        !phenotype.expresses_on(root, gland()),
        "and not what the donor had arranged on it: regrowing is not carrying"
    );
    // Regrowing preserves identity and provenance and realizes the phenotype
    // under the destination's rules — the wing contract's own words for this
    // route, and the half of it a rebuilt allocation must not quietly drop.
    assert_eq!(
        phenotype
            .body()
            .part(root)
            .map(|part| &part.provenance.origin),
        Some(&Origin::Incorporated {
            from_species: DONOR_LINE,
            from_part: frond,
        })
    );
    assert_ne!(
        mesocosm_core::state_hash(&world),
        before,
        "the refused carry and the landed regrowth are different worlds"
    );
}

// ---------------------------------------------------------------------------
// Severing the graft cascades
// ---------------------------------------------------------------------------

#[test]
fn severing_the_graft_takes_the_whole_imported_branch_and_still_explains_it() {
    let mut world = world_with(Verdict::Native);
    let (frond, _) = donor(&mut world);
    let Outcome::Grafted { root, .. } = take(&mut world, frond, Crossing::Carry) else {
        panic!("the graft was refused");
    };
    let arrived = world.body().unwrap().descendants(root);
    assert_eq!(arrived.len(), 2);
    let standing = world.body().unwrap().living().count();

    let me = world.controlled_id().unwrap();
    let lost = world
        .organisms
        .iter_mut()
        .find(|o| o.id == me)
        .unwrap()
        .phenotype
        .sever(root);

    assert_eq!(lost, arrived, "the branch left the way it came: whole");
    let phenotype = world.phenotype().unwrap();
    assert_eq!(phenotype.body().living().count(), standing - 2);
    assert!(
        !phenotype.expresses(gland()),
        "the consequence went with it"
    );
    assert_eq!(phenotype.lost_glands(), vec![root]);
    // PD1b's explanation still names what the branch did, which is the sentence
    // a player is owed about a part they no longer have.
    let explained = phenotype
        .explain(root)
        .expect("a severed part still explains");
    assert!(!explained.living);
    assert!(
        explained.sites.iter().any(|site| site.process == gland()),
        "that branch is where the sting was"
    );
    assert!(phenotype.conserves());
}

// ---------------------------------------------------------------------------
// Snapshot and replay agree
// ---------------------------------------------------------------------------

#[test]
fn a_transfer_survives_a_snapshot_and_replays_to_the_same_hash() {
    let mut world = world_with(Verdict::Native);
    let (frond, _) = donor(&mut world);
    let intents = [
        Intent::Graft {
            organism: DONOR,
            part: frond,
            crossing: Crossing::Carry,
        },
        Intent::Idle,
    ];
    let mut twin = world.clone();
    for intent in &intents {
        world.apply(intent.clone());
    }

    let restored =
        mesocosm_core::restore(&mesocosm_core::snapshot(&world).unwrap()).expect("decodes");
    assert_eq!(
        mesocosm_core::state_hash(&restored),
        mesocosm_core::state_hash(&world),
        "the transfer is in the bytes"
    );
    assert_eq!(
        restored.last_graft(),
        world.last_graft(),
        "and so is the transaction that made it"
    );
    let landed = world.last_graft().expect("a transfer happened").clone();
    assert_eq!(landed.recipient, world.controlled_id().expect("embodied"));
    assert!(
        world.carried_branch().is_some(),
        "the body being played is the one carrying it"
    );
    assert_eq!(landed.parts.len(), 2);
    assert_eq!(landed.crossing, Crossing::Carry);
    assert_eq!(landed.verdict, Verdict::Native);

    for intent in &intents {
        twin.apply(intent.clone());
    }
    assert_eq!(
        mesocosm_core::state_hash(&twin),
        mesocosm_core::state_hash(&world),
        "the same intents against the same world reach the same bytes"
    );
}

// ---------------------------------------------------------------------------
// What is not on offer
// ---------------------------------------------------------------------------

#[test]
fn the_whole_of_a_body_is_not_a_branch_and_a_living_one_offers_nothing() {
    let mut world = world_with(Verdict::Native);
    let (frond, _) = donor(&mut world);
    let root = corpse_of(&world).body().root;

    assert_eq!(
        take(&mut world, root, Crossing::Carry),
        Outcome::Rejected(Rejection::WholeBody(root)),
        "a body without a root is not an injured body"
    );
    assert!(matches!(
        take(&mut world, PartId(200), Crossing::Carry),
        Outcome::Rejected(Rejection::NoSuchPart(_))
    ));

    // Live dismemberment is a further gate, and this proof does not open it.
    let living = OrganismId(9_701);
    let mut alive = corpse_of(&world).clone();
    alive.id = living;
    alive.stage = Stage::Mature;
    world.organisms.push(alive);
    assert!(matches!(
        world.apply(Intent::Graft {
            organism: living,
            part: frond,
            crossing: Crossing::Carry,
        }),
        Outcome::Rejected(Rejection::StillLiving(_))
    ));

    // And a branch can only leave once.
    assert!(matches!(
        take(&mut world, frond, Crossing::Carry),
        Outcome::Grafted { .. }
    ));
    assert!(matches!(
        take(&mut world, frond, Crossing::Carry),
        Outcome::Rejected(Rejection::NothingLeft(_))
    ));
}
