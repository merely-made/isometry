// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The volumes and placement rules the host draws with.
//!
//! Legacy cuboid fixtures for existing recordings and historical probes.
//! New host sessions admit a generated content pack before founding; see
//! `app/content.rs`. The core carries addresses and biological envelopes.

use mesocosm_core::{
    Crossing, Founding, Intent, OrganismId, PartId, Placement, Role, Verdict, VolumeRef, World,
    world::organism_extent,
};
use mesocosm_mesh::{Volume, VolumeMap};

pub fn volumes() -> VolumeMap {
    let mut map = VolumeMap::new();

    // Every template the world can develop a part from, sized from the
    // palette itself. Enumerating roles rather than hardcoding tags is what
    // keeps this table from silently falling behind the core: when the
    // palette grew a sensor, a literal list quietly stopped resolving and
    // `mesh_body` failed with `MissingVolume`.
    // **Every admitted shape, not every role** (DC4). A role holds up to
    // `PALETTE_SHAPES` templates now and the shipping founding fills most of
    // them, so taking each role's default resolved four of twelve and
    // `mesh_body` failed with `MissingVolume` on the first archetype body.
    // Read off the founding's own palette, so the table cannot fall behind it.
    let palette = Founding::default().palette();
    for role in [Role::Mass, Role::Limb, Role::Plate, Role::Sensor] {
        for template in palette.shapes(role).admitted() {
            let size = template.half_extent.map(|half| (half * 2).max(1) as u32);
            // `from_tag` puts the tag in byte zero; the material is only a
            // palette index for the placeholder look.
            map.insert(template.volume, Volume::solid(size, template.volume.0[0]));
        }
    }

    for tag in 16..24u8 {
        // A volume is exactly the extent the core placed with, or the picture
        // and the physics would disagree about the same part.
        let half = organism_extent(tag);
        let size = [
            (half[0] * 2).max(1) as u32,
            (half[1] * 2).max(1) as u32,
            (half[2] * 2).max(1) as u32,
        ];
        map.insert(VolumeRef::from_tag(tag), Volume::solid(size, tag));
    }
    map.insert(VolumeRef::from_tag(64), Volume::solid([1, 1, 1], 5));
    map
}

/// VB1's cuboid content resolved from the actual founded bodies. This is still
/// placeholder geometry; it does not regenerate a different default palette
/// when replay founded the world with another admitted set of part extents.
pub fn volumes_for(world: &World) -> VolumeMap {
    let mut map = VolumeMap::new();
    for organism in &world.organisms {
        for part in organism.body().living() {
            let size = part.half_extent.map(|half| (half * 2).max(1) as u32);
            map.insert(part.volume, Volume::solid(size, part.volume.0[0].max(1)));
        }
    }
    map
}

/// Eating, the default way: the body plan decides where the part goes.
///
/// `Placement::Explicit` still exists for an editor, but automatic and
/// symmetric is the resting state — and since TD4 the host has nothing else to
/// say about a meal at all. Whether it burns or builds is the body's, read off
/// a budget the player can see.
pub fn metabolize(
    _world: &World,
    organism: OrganismId,
    _volumes: &VolumeMap,
    placement: Placement,
) -> Intent {
    Intent::Metabolize {
        organism,
        placement,
    }
}

/// One step toward the nearest thing worth eating, or `None` when there is
/// nothing left or it is already in reach.
pub fn toward_prey(world: &World) -> Option<[i32; 3]> {
    let here = world.position()?;
    let eater = world.controlled()?;
    let at = world
        .organisms
        .iter()
        .filter(|m| Some(m.id) != world.controlled_id() && m.biomass_mg() > 0)
        .filter(|m| eater.admits(m.kingdom().nis_kind(), !m.is_alive()))
        .min_by_key(|m| {
            (
                !m.is_alive(),
                (0..3)
                    .map(|a| (m.position[a] - here[a]).abs())
                    .max()
                    .unwrap_or(0),
                m.id,
            )
        })?
        .position;

    let step = [0, 1, 2].map(|a| (at[a] - here[a]).signum());
    if step == [0, 0, 0] { None } else { Some(step) }
}

/// The smallest branch worth taking off a carcass within reach. (P3)
///
/// A **branch**, not an organ: a living non-root part with something still
/// hanging off it, so the transfer the demo records is the one the gate is
/// about. The *smallest* such subtree, because a body that has been eating has
/// little room left and the whole of a segmented corpse below its root is a
/// branch by the letter and a second creature by the look of it. Ties break on
/// ids, so the script is determinate rather than dependent on iteration order.
pub fn branch_donor(world: &World) -> Option<(OrganismId, PartId, [i32; 3])> {
    let mut best: Option<(usize, OrganismId, PartId, [i32; 3])> = None;
    for carcass in world
        .organisms
        .iter()
        .filter(|o| !o.is_alive() && world.in_reach(o.position))
    {
        let body = carcass.body();
        for part in body.living().map(|part| part.id) {
            if part == body.root || body.children(part).next().is_none() {
                continue;
            }
            let size = body.descendants(part).len();
            let candidate = (size, carcass.id, part, carcass.position);
            if best.is_none_or(|(found, id, at, _)| (size, carcass.id, part) < (found, id, at)) {
                best = Some(candidate);
            }
        }
    }
    best.map(|(_, id, part, at)| (id, part, at))
}

/// The crossing to take with that donor's tissue: carry it when this world's
/// affinity permits, and regrow it here when it does not.
///
/// The verdict is the world's and the choice is the player's, which is exactly
/// what a script has to model to be a fair demo of the verb.
pub fn crossing_for(world: &World, donor: OrganismId) -> Crossing {
    match world.verdict_for(donor) {
        Some(Verdict::Refused) | None => Crossing::Regrow,
        Some(_) => Crossing::Carry,
    }
}

pub fn reachable(world: &World) -> Option<OrganismId> {
    let eater = world.controlled()?;
    world
        .organisms
        .iter()
        // Never offer the critter itself. Since P1 the played organism is in
        // this vector like everything else, so it is a candidate unless it is
        // filtered out.
        .filter(|m| Some(m.id) != world.controlled_id() && m.biomass_mg() > 0)
        .filter(|m| eater.admits(m.kingdom().nis_kind(), !m.is_alive()))
        // Anatomy decides how far this critter can touch, not a constant. A
        // stubby one reaches about three voxels; a limbed one reaches further.
        .filter(|m| world.in_reach(m.position))
        .min_by_key(|m| (!m.is_alive(), m.id))
        .map(|m| m.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_mesh::{VolumeSource, flatten};

    #[test]
    fn food_controls_skip_targets_the_controlled_ports_do_not_admit() {
        use mesocosm_core::{IntakePort, Kingdom, NisKind, Process, Registry};
        let mut world = World::new(7, 80);
        let me = world.controlled_id().unwrap();
        let plant = world
            .organisms
            .iter()
            .find(|o| o.id != me && o.kingdom() == Kingdom::Producer)
            .unwrap()
            .id;
        let prey = world
            .organisms
            .iter()
            .find(|o| o.id != me && o.kingdom() == Kingdom::Consumer)
            .unwrap()
            .id;
        world
            .organisms
            .retain(|o| [me, plant, prey].contains(&o.id));
        let eater = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
        let parts: Vec<_> = eater.body().living().map(|p| p.id).collect();
        for part in parts {
            eater.phenotype.declare_port(part, IntakePort::none());
        }
        let root = eater.body().root;
        eater.phenotype.declare_port(
            root,
            IntakePort::live(NisKind::Consumer)
                .supported_by(Registry::native().of_native(Process::Intake).reference()),
        );
        let at = eater.position;
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == plant)
            .unwrap()
            .position = [at[0] + 1, at[1], at[2]];
        world
            .organisms
            .iter_mut()
            .find(|o| o.id == prey)
            .unwrap()
            .position = [at[0], at[1], at[2] + 2];
        let hash = mesocosm_core::state_hash(&world);
        assert_eq!(reachable(&world), Some(prey));
        assert_eq!(toward_prey(&world), Some([0, 0, 1]));
        assert_eq!(mesocosm_core::state_hash(&world), hash);

        world
            .organisms
            .iter_mut()
            .find(|o| o.id == prey)
            .unwrap()
            .stage = mesocosm_core::Stage::Carrion;
        assert_eq!(reachable(&world), None);
        let eater = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
        eater.phenotype.declare_port(
            root,
            IntakePort::deadstock()
                .supported_by(Registry::native().of_native(Process::Intake).reference()),
        );
        assert_eq!(reachable(&world), Some(prey));
        assert_eq!(toward_prey(&world), Some([0, 0, 1]));
    }

    #[test]
    fn the_fixture_resolves_every_volume_a_world_mints() {
        let volumes = volumes();
        let world = World::new(7, 40);
        assert!(mesocosm_mesh::mesh_body(world.body().unwrap(), &volumes).is_ok());
        for organism in &world.organisms {
            assert!(volumes.volume(organism.volume()).is_some());
        }
    }

    /// The regression this policy exists for: a well-fed critter used to
    /// collapse into a pile because placement cycled six faces and the seventh
    /// part landed on the first.
    #[test]
    fn parts_never_land_on_top_of_each_other() {
        let volumes = volumes();
        let mut world = World::new(2024, 80);
        // **A compact body to grow from** (DC4). The claim is about where
        // *incorporation* puts a part, and the played critter now founds from
        // an archetype whose chained segments already sit flush against one
        // another — a developed body shares one voxel plane per joint, which
        // `flatten` cannot represent and this assertion would read as an
        // overlap. So the fixture starts from a single root, as it effectively
        // did when worldgen happened to draw one.
        {
            let me = world.controlled_id().expect("embodied");
            let organism = world.organisms.iter_mut().find(|o| o.id == me).unwrap();
            let (species, position) = (organism.species, organism.position);
            *organism = mesocosm_core::Organism {
                stage: mesocosm_core::Stage::Mature,
                ..mesocosm_core::Organism::founding(
                    me,
                    species,
                    mesocosm_core::Kingdom::Decomposer,
                    VolumeRef::from_tag(1),
                    [2, 2, 2],
                    position,
                    1_500,
                )
            };
            // Placement is the claim here. Give this compact test body a
            // declared live-food port instead of relying on decomposers
            // historically being able to metabolize every living target.
            let port = mesocosm_core::IntakePort::live(mesocosm_core::NisKind::Producer)
                .with_live(mesocosm_core::NisKind::Consumer)
                .with_live(mesocosm_core::NisKind::Decomposer)
                .supported_by(
                    mesocosm_core::Registry::native()
                        .of_native(mesocosm_core::Process::Intake)
                        .reference(),
                );
            let root = organism.body().root;
            assert!(organism.phenotype.declare_port(root, port));
        }

        // Walk to prey rather than assuming it is adjacent: reach is anatomy
        // now, and a starting critter touches very little.
        //
        // Stop the moment the body is past one face cycle, which is the whole
        // of the claim. Since TD4 a held critter goes where it is steered and
        // nowhere else, so this fixture's one-voxel steering can strand it —
        // and a fixed meal quota would then be a starvation test wearing a
        // placement test's name.
        let mut grown = None;
        for _ in 0..800 {
            let Some(body) = world.body() else { break };
            if body.len() > 6 {
                grown = Some(body.clone());
                break;
            }
            if let Some(target) = reachable(&world) {
                world.apply(metabolize(&world, target, &volumes, Placement::Planned));
                continue;
            }
            let Some(here) = world.position() else { break };
            let Some(at) = world
                .organisms
                .iter()
                .filter(|m| Some(m.id) != world.controlled_id() && m.is_alive())
                .map(|m| m.position)
                .min_by_key(|at: &[i32; 3]| {
                    (0..3).map(|a| (at[a] - here[a]).abs()).max().unwrap_or(0)
                })
            else {
                break;
            };
            world.apply(Intent::Move {
                delta: [0, 1, 2].map(|a| (at[a] - here[a]).signum()),
            });
        }

        let body = grown.expect("the fixture must out-eat one face cycle");

        // Every solid voxel is accounted for exactly once: nothing overlaps.
        let flat = flatten(&body, &volumes).unwrap();
        let expected: usize = body
            .parts
            .iter()
            .filter_map(|p| volumes.volume(p.volume))
            .map(|v| v.solid_count())
            .sum();
        assert_eq!(
            flat.volume.solid_count(),
            expected,
            "overlapping parts would lose voxels to overwriting"
        );
    }

    #[test]
    fn growth_is_deterministic() {
        let volumes = volumes();
        let grow = || {
            let mut world = World::new(99, 60);
            for _ in 0..8 {
                let Some(target) = reachable(&world) else {
                    break;
                };
                world.apply(metabolize(&world, target, &volumes, Placement::Planned));
            }
            world.body().unwrap().clone()
        };
        assert_eq!(grow(), grow());
    }
}
