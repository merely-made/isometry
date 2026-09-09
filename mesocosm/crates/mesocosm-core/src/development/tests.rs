// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::axis::{Tagma, catalogue};

fn palette() -> PartPalette {
    PartPalette {
        mass: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(1),
            half_extent: [2, 2, 2],
        }),
        limb: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(2),
            half_extent: [4, 1, 1],
        }),
        plate: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(3),
            half_extent: [4, 4, 1],
        }),
        sensor: RoleShapes::only(PartTemplate {
            volume: VolumeRef::from_tag(4),
            half_extent: [1, 1, 1],
        }),
    }
}

fn template(tag: u8, half_extent: [i32; 3]) -> PartTemplate {
    PartTemplate {
        volume: VolumeRef::from_tag(tag),
        half_extent,
    }
}

fn exact_soma(recipe: &Recipe) -> Soma {
    Soma {
        segments: recipe.tagmata.iter().map(|tagma| tagma.segments).collect(),
        absent: Vec::new(),
    }
}

#[test]
fn appendage_roles_become_actual_parts() {
    let recipe = Recipe::of(vec![
        Tagma::new(1, Appendage::Feeler),
        Tagma::new(1, Appendage::Mouth),
        Tagma::new(1, Appendage::Limb),
        Tagma::new(1, Appendage::Plate),
    ]);
    let body = develop_body(
        SpeciesId(7),
        &recipe,
        &exact_soma(&recipe),
        1_000,
        palette(),
    )
    .unwrap();
    let roles: Vec<_> = body
        .living()
        .map(|part| classify(part.half_extent))
        .collect();

    assert_eq!(
        roles.iter().filter(|role| **role == Role::Sensor).count(),
        2
    );
    assert_eq!(roles.iter().filter(|role| **role == Role::Limb).count(), 2);
    assert_eq!(roles.iter().filter(|role| **role == Role::Plate).count(), 1);
    assert_eq!(roles.iter().filter(|role| **role == Role::Mass).count(), 5);
}

#[test]
fn a_snake_and_tetrapod_now_have_different_part_graphs() {
    let tetrapod = catalogue::tetrapod(6);
    let snake = catalogue::snake(16);
    let legs = develop_body(
        SpeciesId(1),
        &tetrapod,
        &exact_soma(&tetrapod),
        4_000,
        palette(),
    )
    .unwrap();
    let legless =
        develop_body(SpeciesId(2), &snake, &exact_soma(&snake), 4_000, palette()).unwrap();

    assert!(
        legs.living()
            .any(|part| classify(part.half_extent) == Role::Limb)
    );
    assert!(
        !legless
            .living()
            .any(|part| classify(part.half_extent) == Role::Limb)
    );
    assert!(
        legless.living().count() > legs.living().count(),
        "length remains anatomy"
    );
}

#[test]
fn developmental_absence_removes_the_appendages_not_the_segment() {
    let recipe = catalogue::centipede(4);
    let complete = exact_soma(&recipe);
    let mut absent = complete.clone();
    absent.absent.push((1, 2));
    let whole = develop_body(SpeciesId(1), &recipe, &complete, 1_000, palette()).unwrap();
    let varied = develop_body(SpeciesId(1), &recipe, &absent, 1_000, palette()).unwrap();

    assert_eq!(
        whole.len() - varied.len(),
        2,
        "one bilateral appendage pair is absent"
    );
    let whole_mass = whole
        .living()
        .filter(|p| classify(p.half_extent) == Role::Mass)
        .count();
    let varied_mass = varied
        .living()
        .filter(|p| classify(p.half_extent) == Role::Mass)
        .count();
    assert_eq!(whole_mass, varied_mass, "the axial segment still developed");
}

#[test]
fn development_conserves_mass_and_is_deterministic() {
    let recipe = catalogue::insect();
    let soma = Soma::develop(&recipe, 19);
    let a = develop_body(SpeciesId(3), &recipe, &soma, 4_003, palette()).unwrap();
    let b = develop_body(SpeciesId(3), &recipe, &soma, 4_003, palette()).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.total_mass_mg(), 4_003);
    assert!(a.living().all(|part| part.mass_mg > 0));
}

#[test]
fn a_palette_cannot_lie_about_a_role() {
    let recipe = catalogue::insect();
    let soma = exact_soma(&recipe);
    let mut wrong = palette();
    wrong.limb.default.half_extent = [2, 2, 2];
    assert_eq!(
        develop_body(SpeciesId(1), &recipe, &soma, 2_000, wrong),
        Err(DevelopmentError::WrongRole {
            expected: Role::Limb,
            actual: Role::Mass
        })
    );
}

// The classifier reads every admitted shape, not just the role's default: a
// second Limb shape that is actually a block is the same lie as a first one.
#[test]
fn a_palette_cannot_lie_about_a_role_in_a_later_slot() {
    let recipe = catalogue::insect();
    let soma = exact_soma(&recipe);
    let mut wrong = palette();
    wrong.mass = wrong.mass.and(template(9, [4, 1, 1]));
    assert_eq!(
        develop_body(SpeciesId(1), &recipe, &soma, 2_000, wrong),
        Err(DevelopmentError::WrongRole {
            expected: Role::Mass,
            actual: Role::Limb
        })
    );
}

// §2.3 of the default creatures plan, asserted rather than commented. TD7's
// "no anatomy can price itself past ~7x" is the primitive limb's own build
// price, so a palette that thins a limb below a 3x3 cross-section raises that
// ceiling to 20x or 60x without moving a constant. `[3,1,0]` is the plan's
// worked example: 21 voxels, a 16 mg ceiling, span 3, price 18.75.
#[test]
fn a_palette_cannot_thin_a_limb_past_the_td7_bound() {
    let recipe = catalogue::insect();
    let soma = exact_soma(&recipe);
    let mut thin = palette();
    thin.limb.default.half_extent = [3, 1, 0];
    assert_eq!(
        develop_body(SpeciesId(1), &recipe, &soma, 2_000, thin),
        Err(DevelopmentError::Overpriced {
            role: Role::Limb,
            half_extent: [3, 1, 0]
        })
    );
    // And the primitive shape, exactly on the bound, still develops.
    assert!(develop_body(SpeciesId(1), &recipe, &soma, 2_000, palette()).is_ok());
}

// The same guard on the sensing side. TD11's "no anatomy may see the
// enclosure, 46 voxels" is `8 * (1 + 4.76)`, and 4.76 is the primitive
// sensor's build price. `[1,1,0]` is 9 voxels, a 7 mg ceiling, span 1, price
// 14.29 — a body that would see three times as far for the same anatomy.
#[test]
fn a_palette_cannot_thin_a_sensor_past_the_td11_bound() {
    let recipe = Recipe::of(vec![Tagma::new(2, Appendage::Feeler)]);
    let soma = exact_soma(&recipe);
    let mut thin = palette();
    thin.sensor.default.half_extent = [1, 1, 0];
    assert_eq!(
        develop_body(SpeciesId(1), &recipe, &soma, 2_000, thin),
        Err(DevelopmentError::Overpriced {
            role: Role::Sensor,
            half_extent: [1, 1, 0]
        })
    );
    assert!(develop_body(SpeciesId(1), &recipe, &soma, 2_000, palette()).is_ok());
}

// The two bounds are the primitive palette's own numbers. Written as
// constants so the guard does not move when the palette does, and asserted
// here so they cannot silently disagree with it either.
#[test]
fn the_price_bounds_are_the_primitive_palettes_own() {
    let primitive = PartPalette::primitive();
    for (role, bound) in [
        (Role::Limb, LIMB_PRICE_BOUND),
        (Role::Sensor, SENSOR_PRICE_BOUND),
    ] {
        let half_extent = primitive.template(role).half_extent;
        assert_eq!(
            (
                span_voxels(half_extent),
                crate::organism::ecology::part_ceiling_mg(half_extent),
            ),
            bound,
            "{role:?}'s bound drifted from the shape it was read off"
        );
        assert!(!overpriced(role, half_extent));
    }
    assert!(primitive.validate().is_ok());
}

// Mass and Plate detail is free: neither carries a span term, so the guard
// must not price them. A 60-part voxel fern is economically a 22-block stalk.
#[test]
fn mass_and_plate_shapes_are_not_priced() {
    assert!(price_bound(Role::Mass).is_none());
    assert!(price_bound(Role::Plate).is_none());
    assert!(!overpriced(Role::Mass, [1, 1, 0]));
    assert!(!overpriced(Role::Plate, [4, 4, 0]));
}

// Zero is every role's default, and a selector this world does not admit
// falls back to it rather than failing to develop — which is what lets a
// recipe travel to a world with a poorer vocabulary.
#[test]
fn an_unadmitted_shape_selector_falls_back_to_the_default() {
    let mut wide = palette();
    let snout = template(5, [2, 1, 0]);
    wide.mass = wide.mass.and(snout);
    assert_eq!(wide.template_at(Role::Mass, 0), wide.mass.default);
    assert_eq!(wide.template_at(Role::Mass, 1), snout);
    assert_eq!(wide.template_at(Role::Mass, 2), wide.mass.default);
    assert_eq!(wide.template_at(Role::Mass, 255), wide.mass.default);
    assert_eq!(wide.mass.admitted().count(), 2);
}

// The selectors are what make a body more than four shapes, and they reach
// both the segments and the appendages a stretch grows.
#[test]
fn a_tagma_selects_its_own_segment_and_appendage_shapes() {
    let mut wide = palette();
    wide.mass = wide.mass.and(template(5, [2, 1, 0]));
    wide.limb = wide.limb.and(template(6, [3, 1, 1]));
    let recipe = Recipe::of(vec![
        Tagma::new(2, Appendage::Limb),
        Tagma::new(2, Appendage::Limb).with_shapes(1, 1),
    ]);
    let body = develop_body(SpeciesId(1), &recipe, &exact_soma(&recipe), 2_000, wide).unwrap();
    let extents: std::collections::BTreeSet<[i32; 3]> =
        body.living().map(|part| part.half_extent).collect();
    assert!(extents.contains(&[2, 2, 2]), "the first stretch's segments");
    assert!(
        extents.contains(&[2, 1, 0]),
        "the second stretch's segments"
    );
    assert!(extents.contains(&[4, 1, 1]), "the first stretch's limbs");
    assert!(extents.contains(&[3, 1, 1]), "the second stretch's limbs");
}

// The null change DC1 exists to prove, at the unit scale: a recipe that names
// no shape reads the same templates it read when a role had only one.
#[test]
fn a_recipe_that_names_no_shape_is_unmoved_by_a_wider_palette() {
    let recipe = catalogue::tetrapod(6);
    let soma = Soma::develop(&recipe, 11);
    let narrow = develop_body(SpeciesId(1), &recipe, &soma, 4_000, palette()).unwrap();
    let mut wide = palette();
    wide.mass = wide.mass.and(template(5, [2, 1, 0]));
    wide.limb = wide.limb.and(template(6, [3, 1, 1]));
    wide.sensor = wide.sensor.and(template(7, [1, 1, 1]));
    let widened = develop_body(SpeciesId(1), &recipe, &soma, 4_000, wide).unwrap();
    assert_eq!(narrow, widened);
}

#[test]
fn the_expression_floor_is_the_number_of_parts() {
    let recipe = catalogue::centipede(4);
    let soma = exact_soma(&recipe);
    let minimum = minimum_body_mass_mg(&recipe, &soma).unwrap();
    let body = develop_body(SpeciesId(1), &recipe, &soma, u64::from(minimum), palette()).unwrap();

    assert_eq!(minimum as usize, body.len());
    assert!(body.living().all(|part| part.mass_mg == 1));
}
