// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! PD4, the refusals half: every boundary, named. (Split from
//! `authored_gland.rs` at the 600-line ceiling.)
//!
//! Done-conditions 4 and 5 — unknown id, invalid part, excessive output,
//! exhausted fuel, and the structural claim that Lua has no world mutation
//! path. The body plan, the shipped script and the declared contexts are the
//! parent suite's, so a refusal is stated against exactly the situation the
//! accepted case was.

use std::sync::Arc;

use mesocosm_core::{BodyPhenotype, Intent, Organism, Process, Registry, Stage, VolumeRef, World};
use mesocosm_phenotype::express::{
    Entropy, Expression, Policy, Proposal, Refused, Request, Runner, lower,
};

use super::{
    RICH_GROUND, body_plan, context, endure, frond_on, gland, hunger, packed, shipped_root,
};

// ---------------------------------------------------------------------------
// 4. Every refusal names its boundary
// ---------------------------------------------------------------------------

/// Runs a one-off script against the shipped body plan.
fn propose(source: &str, policy: Policy) -> Result<Proposal, Refused> {
    let (phenotype, _) = body_plan();
    let mut runner = Runner::load(source, policy)?;
    runner.propose(&context(&phenotype, RICH_GROUND), &Entropy::from_seed(1))
}

#[test]
fn an_unknown_id_refuses_cleanly() {
    // Never substituted with the nearest local definition (plan §6): a world
    // that does not hold `reef:filter` says so.
    let registry = Arc::new(packed());
    let (phenotype, part) = body_plan();
    let proposal = propose(
        &format!(
            r#"function express(request, entropy)
                 return {{ sites = {{ {{ part = {}, process = "reef:filter", cells = 2 }} }} }}
               end"#,
            part.0
        ),
        Policy::default(),
    )
    .expect("the script itself is fine");
    assert_eq!(
        lower(&registry, &phenotype, &proposal),
        Err(Refused::UnknownProcess {
            id: "reef:filter".to_owned()
        })
    );
    assert!(
        lower(&registry, &phenotype, &proposal)
            .unwrap_err()
            .words()
            .contains("reef:filter"),
        "and it says which id — a real part, so the id is the only thing wrong"
    );
}

#[test]
fn an_invalid_part_refuses_cleanly() {
    let registry = Arc::new(packed());
    let (phenotype, _) = body_plan();
    let proposal = propose(
        r#"function express(request, entropy)
             return { sites = { { part = 99, process = "mesocosm:secrete", cells = 1 } } }
           end"#,
        Policy::default(),
    )
    .expect("the script itself is fine");
    assert_eq!(
        lower(&registry, &phenotype, &proposal),
        Err(Refused::UnknownPart {
            part: mesocosm_core::PartId(99)
        })
    );
}

#[test]
fn excessive_output_refuses_cleanly() {
    // A proposal larger than the host will carry. The policy is host policy: a
    // script cannot raise it by asking.
    let policy = Policy {
        max_output_bytes: 64,
        ..Policy::default()
    };
    let refusal = propose(
        r#"function express(request, entropy)
             local sites = {}
             for i = 1, 12 do
               sites[i] = { part = 1, process = "mesocosm:secrete", cells = 1 }
             end
             return { sites = sites }
           end"#,
        policy,
    )
    .unwrap_err();
    match refusal {
        Refused::Output { bytes, limit } => {
            assert_eq!(limit, 64);
            assert!(bytes > limit, "and it says how much was offered");
        },
        other => panic!("{other:?}"),
    }
}

#[test]
fn an_overlong_collection_refuses_cleanly() {
    let policy = Policy {
        max_entries: 4,
        ..Policy::default()
    };
    let refusal = propose(
        r#"function express(request, entropy)
             local sites = {}
             for i = 1, 9 do
               sites[i] = { part = 1, process = "mesocosm:secrete", cells = 1 }
             end
             return { sites = sites }
           end"#,
        policy,
    )
    .unwrap_err();
    match refusal {
        Refused::Collection { entries, limit } => {
            assert_eq!(limit, 4);
            assert!(entries > limit);
        },
        other => panic!("{other:?}"),
    }
}

#[test]
fn exhausted_fuel_refuses_cleanly() {
    // A script that never finishes is a refusal with a number on it, not a
    // hung host.
    assert_eq!(
        propose(
            "function express(request, entropy) while true do end end",
            Policy::default(),
        ),
        Err(Refused::Fuel {
            limit: Policy::default().fuel
        })
    );
}

#[test]
fn a_missing_entrypoint_refuses_cleanly() {
    assert_eq!(
        propose("local unused = 1", Policy::default()),
        Err(Refused::NoEntrypoint)
    );
}

#[test]
fn a_malformed_proposal_refuses_cleanly() {
    match propose(
        "function express(request, entropy) return 7 end",
        Policy::default(),
    ) {
        Err(Refused::Malformed { why }) => assert!(why.contains("table"), "{why}"),
        other => panic!("{other:?}"),
    }
}

#[test]
fn the_validator_still_owns_its_own_boundaries() {
    // **One developmental authority.** A script may ask for a gland on the
    // bulk root; the site requirement is not restated at this door, it is
    // refused where every other proposal source is refused.
    let registry = Arc::new(packed());
    let (phenotype, _) = body_plan();
    let proposal = propose(
        r#"function express(request, entropy)
             return { sites = { { part = 0, process = "mesocosm:secrete", cells = 1 } } }
           end"#,
        Policy::default(),
    )
    .expect("the script itself is fine");
    let allocation = lower(&registry, &phenotype, &proposal).expect("it lowers");
    let mut candidate = phenotype.clone();
    match candidate.develop(&registry, &allocation) {
        Err(mesocosm_core::Refusal::SiteMismatch { part, .. }) => {
            assert_eq!(part, mesocosm_core::PartId(0), "a bulk root is not a plate")
        },
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_stale_ruleset_refuses_at_the_one_validator() {
    // **PD3's residue closed.** The validator resolves against the ruleset the
    // world admitted, so a proposal citing a definition this world does not
    // hold is `UnknownProcess` rather than a definition somebody else's build
    // happened to have. This is only reachable because `develop` takes a
    // `&Registry` now.
    let (phenotype, part) = body_plan();
    let full = Arc::new(packed());
    let proposal = Proposal {
        sites: vec![Expression {
            part: part.0,
            process: gland().qualified(),
            cells: 5,
        }],
    };
    let allocation = lower(&full, &phenotype, &proposal).expect("it lowers under the full ruleset");

    let mut defs: Vec<_> = packed().all().cloned().collect();
    defs.retain(|def| def.id.name != "secrete");
    let without = Arc::new(Registry::admit(defs).expect("no collision"));
    let mut candidate = phenotype.clone();
    assert_eq!(
        candidate.develop(&without, &allocation),
        Err(mesocosm_core::Refusal::UnknownProcess(
            Registry::native().of_native(Process::Secrete).reference()
        )),
        "the ruleset that lost the definition refuses the body that cites it"
    );
    // And the same allocation is fine under the ruleset it was authored for.
    assert!(phenotype.clone().develop(&full, &allocation).is_ok());
}

#[test]
fn a_world_validates_against_the_ruleset_it_admitted() {
    // The world half of the same claim: `World` carries the set, not only the
    // digest, and a world founded on a ruleset without the gland refuses the
    // development its own discovery would have proposed.
    let mut defs: Vec<_> = packed().all().cloned().collect();
    defs.retain(|def| def.id.name != "secrete");
    let without = Arc::new(Registry::admit(defs).expect("no collision"));
    let mut world = World::founded_on(4_242, 24, mesocosm_core::Founding::default(), without)
        .expect("the palette is valid");
    assert_ne!(world.rules(), mesocosm_core::WorldRules::native());
    assert!(world.ruleset().get(&gland()).is_none());

    let me = world.controlled_id().expect("embodied");
    let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
    let (species, position) = (organism.species, organism.position);
    *organism = Organism {
        stage: Stage::Mature,
        ..Organism::founding(
            me,
            species,
            mesocosm_core::Kingdom::Consumer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            position,
            1_500,
        )
    };
    endure(&mut world, mesocosm_core::discovery::HUNGER_TICKS + 1);
    frond_on(&mut world);

    // The condition table is native, so the candidate cites the gland this
    // world did not admit. That is exactly the stale-ruleset case, and it is
    // refused by name rather than substituted.
    assert_eq!(
        world.apply(Intent::Express {
            condition: hunger()
        }),
        mesocosm_core::Outcome::Rejected(mesocosm_core::Rejection::Refused(
            mesocosm_core::Refusal::UnknownProcess(
                Registry::native().of_native(Process::Secrete).reference()
            )
        ))
    );
}

// ---------------------------------------------------------------------------
// 5. Lua has no world mutation path
// ---------------------------------------------------------------------------

#[test]
fn lua_has_no_world_mutation_path() {
    // **Structural, not a review note.** Three claims, each about what the
    // sandbox *contains* rather than about what a script chose to do.
    let mut runner = Runner::load(
        r#"
        function express(request, entropy)
          -- Everything a host might have registered, and everything an ambient
          -- standard library would have brought. An undefined global is nil in
          -- Lua, so this counts what is actually reachable from inside.
          local reachable = 0
          if math.random ~= nil then reachable = reachable + 1 end
          if math.randomseed ~= nil then reachable = reachable + 1 end
          if io ~= nil then reachable = reachable + 1 end
          if os ~= nil then reachable = reachable + 1 end
          if require ~= nil then reachable = reachable + 1 end
          if dofile ~= nil then reachable = reachable + 1 end
          if loadfile ~= nil then reachable = reachable + 1 end
          if load ~= nil then reachable = reachable + 1 end
          if package ~= nil then reachable = reachable + 1 end
          if debug ~= nil then reachable = reachable + 1 end
          return { sites = { { part = reachable, process = "probe", cells = 0 } } }
        end
        "#,
        Policy::default(),
    )
    .expect("the probe loads");
    let (phenotype, _) = body_plan();
    let probe = runner
        .propose(&context(&phenotype, RICH_GROUND), &Entropy::from_seed(1))
        .expect("the probe runs");
    assert_eq!(
        probe.sites[0].part, 0,
        "neither math.random nor math.randomseed survives Runner::load, \
         so a script has no randomness of its own"
    );

    // The API itself: the only two things a caller can hand a runner are a
    // `&Request` (owned, frozen, declared facts) and a `&Entropy` (numbers the
    // host drew). Neither can reach a `World` or a `BodyPhenotype`, and the
    // only thing that comes back is a `Proposal`, which has to go through the
    // one validator to become anything. This assertion is the type signature,
    // written down: if `propose` ever grows a mutable argument, this stops
    // compiling.
    let _: fn(&mut Runner, &Request, &Entropy) -> Result<Proposal, Refused> = Runner::propose;
    let _: fn(
        &Registry,
        &BodyPhenotype,
        &Proposal,
    ) -> Result<mesocosm_core::AllocationProposal, Refused> = lower;
}

#[test]
fn the_script_the_pack_declares_is_the_script_that_runs() {
    // A pack asset is opened through the manifest or not at all. An undeclared
    // sibling is refused by the same rule an undeclared definition is.
    let root = shipped_root();
    let manifest = mesocosm_phenotype::discover(&root).expect("reads");
    assert!(mesocosm_phenotype::asset(&root, &manifest, "expression/gland.lua").is_ok());
    assert_eq!(
        mesocosm_phenotype::asset(&root, &manifest, "expression/other.lua"),
        Err(mesocosm_phenotype::Admission::UndeclaredFile {
            path: "expression/other.lua".to_owned()
        })
    );
    // Declaration is checked *before* the path is resolved, so a walk out of
    // the pack never reaches the filesystem at all: it is refused for not
    // being declared, which is the stronger of the two refusals. The escape
    // check underneath it is receipted at the manifest arm, in
    // `admission.rs::a_path_escape_is_refused`.
    assert_eq!(
        mesocosm_phenotype::asset(&root, &manifest, "../../secrets.lua"),
        Err(mesocosm_phenotype::Admission::UndeclaredFile {
            path: "../../secrets.lua".to_owned()
        })
    );
}

#[test]
fn a_script_cannot_express_what_the_line_has_not_come_to() {
    // The bound the shipped script reads off the request itself, and the one a
    // pack author cannot write around: the candidate list is the host's, and
    // an empty proposal is what a body with no grant is worth. It lowers to a
    // proposal that claims no part, which the one validator answers by name.
    let registry = Arc::new(packed());
    let (phenotype, _) = body_plan();
    let ungranted = Request::frozen(
        &registry,
        &phenotype,
        Vec::new(),
        1_500,
        vec![mesocosm_phenotype::express::Ambient {
            name: "ground_mg".to_owned(),
            value: RICH_GROUND,
        }],
    );
    let mut runner = Runner::load(&super::script(), Policy::default()).expect("loads");
    let proposal = runner
        .propose(&ungranted, &Entropy::from_seed(2))
        .expect("a script that asks for nothing is not a script that failed");
    assert_eq!(proposal, Proposal::default(), "it proposes nothing at all");

    let allocation = lower(&registry, &phenotype, &proposal).expect("nothing lowers to nothing");
    assert_eq!(
        phenotype.clone().develop(&registry, &allocation),
        Err(mesocosm_core::Refusal::NothingProposed)
    );
}
