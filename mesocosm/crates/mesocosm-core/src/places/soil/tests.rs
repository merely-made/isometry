// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

use crate::world::ENCLOSURE;

const SIDE: usize = (2 * ENCLOSURE + 1) as usize;

#[test]
fn roots_rank_available_nutrients_and_leave_pending_nis() {
    let mut soil = Soil::seeded(1, 0);
    let centre = soil.column_at([0, 0, 0]);
    let rich = soil.column_at([1, 0, 0]);
    let pending = Stock::single(crate::matter::Material::Producer, 1000);
    soil.deposit_stock(centre, pending).unwrap();
    soil.deposit(rich, 5);
    assert_eq!(soil.draw_richest_within(centre, 1, 100), 5);
    assert_eq!(soil.stock(centre), pending);
    assert_eq!(soil.total_mg(), 1000);
    assert_eq!(u128::from(soil.total_mg()), soil.total_stock().total());
}

#[test]
fn the_store_covers_the_enclosure_one_column_per_voxel() {
    let soil = Soil::seeded(ENCLOSURE, 100);
    assert_eq!(soil.side() as usize, SIDE);
    assert_eq!(soil.columns(), SIDE * SIDE);
    assert_eq!(soil.total_mg(), (SIDE * SIDE * 100) as u64);

    let mut seen = std::collections::BTreeSet::new();
    for z in -ENCLOSURE..=ENCLOSURE {
        for x in -ENCLOSURE..=ENCLOSURE {
            assert!(seen.insert(soil.column_at([x, 7, z])), "({x},{z}) collided");
        }
    }
    assert_eq!(seen.len(), soil.columns());
}

#[test]
fn height_is_not_part_of_a_column() {
    let soil = Soil::seeded(ENCLOSURE, 1);
    assert_eq!(soil.column_at([3, -4, -7]), soil.column_at([3, 22, -7]));
}

#[test]
fn a_position_past_the_wall_clamps_rather_than_dropping_matter() {
    let mut soil = Soil::seeded(ENCLOSURE, 0);
    let outside = soil.column_at([900, 0, -900]);
    soil.deposit(outside, 40);
    assert_eq!(soil.total_mg(), 40);
    assert_eq!(outside, soil.column_at([ENCLOSURE, 0, -ENCLOSURE]));
}

#[test]
fn a_draw_never_takes_more_than_the_column_holds() {
    let mut soil = Soil::seeded(2, 30);
    let column = soil.column_at([0, 0, 0]);
    assert_eq!(soil.draw(column, 12), 12);
    assert_eq!(soil.matter_mg(column), 18);
    assert_eq!(soil.draw(column, 1_000), 18);
    assert_eq!(soil.draw(column, 1_000), 0);
}

#[test]
fn scalar_draw_leaves_nis_in_the_ground_and_keeps_the_cached_total_exact() {
    let mut soil = Soil::seeded(0, 10);
    let column = soil.column_at([0, 0, 0]);
    let nis = Stock::from_amounts([0, 3, 5, 7]);
    soil.deposit_stock(column, nis).unwrap();

    assert_eq!(soil.matter_mg(column), 10);
    assert_eq!(soil.draw(column, 20), 10);
    assert_eq!(soil.stock(column), nis);
    assert_eq!(soil.total_stock(), nis);
    assert_eq!(
        soil.total_mg(),
        u64::try_from(soil.total_stock().total()).unwrap()
    );
}

#[test]
fn typed_draw_preserves_a_deterministic_mixture_and_keeps_the_cached_total_exact() {
    let mut soil = Soil::seeded(0, 0);
    let column = soil.column_at([0, 0, 0]);
    soil.deposit_stock(column, Stock::from_amounts([1, 1, 1, 1]))
        .unwrap();

    assert_eq!(soil.draw_stock(column, 3).amounts(), [1, 1, 1, 0]);
    assert_eq!(soil.stock(column).amounts(), [0, 0, 0, 1]);
    assert_eq!(
        soil.total_mg(),
        u64::try_from(soil.total_stock().total()).unwrap()
    );
}

#[test]
fn typed_deposit_rejects_overflow_without_changing_the_column() {
    let mut soil = Soil::seeded(0, u64::MAX - 2);
    let column = soil.column_at([0, 0, 0]);
    let before = soil.stock(column);

    assert_eq!(
        soil.deposit_stock(column, Stock::single(crate::matter::Material::Producer, 3)),
        Err(SoilError::TotalOverflow { column })
    );
    assert_eq!(soil.stock(column), before);
}

#[test]
fn typed_deposit_rejects_global_overflow_without_changing_either_column() {
    let mut soil = Soil::seeded(1, 0);
    let first = soil.column_at([-1, 0, -1]);
    let second = soil.column_at([0, 0, -1]);
    soil.deposit_stock(
        first,
        Stock::single(crate::matter::Material::Untyped, u64::MAX),
    )
    .unwrap();

    assert_eq!(
        soil.deposit_stock(second, Stock::single(crate::matter::Material::Producer, 1)),
        Err(SoilError::GlobalTotalOverflow)
    );
    assert_eq!(soil.stock(first).amounts(), [u64::MAX, 0, 0, 0]);
    assert_eq!(soil.stock(second), Stock::EMPTY);
    assert_eq!(soil.total_mg(), u64::MAX);
}

#[test]
fn percolation_carries_each_typed_channel_without_loss() {
    let mut soil = Soil::seeded(1, 0);
    let columns: Vec<_> = soil.columns_within(soil.column_at([0, 0, 0]), 1).collect();
    for (index, column) in columns.into_iter().enumerate() {
        soil.deposit_stock(
            column,
            Stock::from_amounts([index as u64, 2 * index as u64, 3, 5]),
        )
        .unwrap();
    }
    let before = soil.total_stock();

    soil.percolate().unwrap();

    assert_eq!(soil.total_stock(), before);
    assert_eq!(soil.total_mg(), u64::try_from(before.total()).unwrap());
}

#[test]
fn drawing_and_depositing_move_matter_without_making_it() {
    let mut soil = Soil::seeded(4, 50);
    let before = soil.total_mg();
    let from = soil.column_at([-3, 0, 2]);
    let to = soil.column_at([1, 0, -4]);
    let moved = soil.draw(from, 40);
    soil.deposit(to, moved);
    assert_eq!(soil.total_mg(), before);
}

#[test]
fn a_radius_reads_a_real_neighbourhood_at_the_shipping_size() {
    let soil = Soil::seeded(ENCLOSURE, 0);
    let middle = soil.column_at([0, 0, 0]);
    assert_eq!(soil.columns_within(middle, 3).count(), 49);
    assert_eq!(soil.columns_within(middle, 0).count(), 1);
    let corner = soil.column_at([-ENCLOSURE, 0, -ENCLOSURE]);
    assert_eq!(soil.columns_within(corner, 3).count(), 16);
}

#[test]
fn a_root_reaches_the_richest_column_it_can_search_and_takes_only_its_income() {
    let mut soil = Soil::seeded(ENCLOSURE, 0);
    let standing = soil.column_at([0, 0, 0]);
    let rich = soil.column_at([2, 0, -3]);
    soil.deposit(rich, 500);

    assert_eq!(soil.draw_richest_within(standing, FORAGE_RADIUS, 20), 20);
    assert_eq!(soil.matter_mg(rich), 480);
    assert_eq!(soil.total_mg(), 480);

    let far = soil.column_at([12, 0, 12]);
    soil.deposit(far, 900);
    assert_eq!(
        soil.draw_richest_within(standing, FORAGE_RADIUS, 1_000),
        480
    );
    assert_eq!(soil.matter_mg(far), 900);
}

#[test]
fn a_forage_read_is_deterministic_when_every_column_is_equal() {
    let mut a = Soil::seeded(ENCLOSURE, 100);
    let mut b = a.clone();
    let middle = a.column_at([0, 0, 0]);
    a.draw_richest_within(middle, FORAGE_RADIUS, 7);
    b.draw_richest_within(middle, FORAGE_RADIUS, 7);
    assert_eq!(a, b);
}

#[test]
fn a_store_round_trips() {
    let mut soil = Soil::seeded(ENCLOSURE, 7);
    let column = soil.column_at([5, 0, -5]);
    soil.deposit(column, 99);
    soil.deposit_stock(column, Stock::from_amounts([0, 3, 5, 8]))
        .unwrap();
    let bytes = crate::snapshot::encode(&soil).unwrap();
    assert_eq!(crate::snapshot::decode::<Soil>(&bytes).unwrap(), soil);
}

#[test]
fn deserialization_rejects_a_non_square_or_overfull_soil() {
    let malformed = serde_json::json!({
        "extent": 1,
        "matter_mg": [[0, 0, 0, 0]],
    });
    assert!(serde_json::from_value::<Soil>(malformed).is_err());

    let overfull = serde_json::json!({
        "extent": 0,
        "matter_mg": [[18446744073709551615_u64, 1, 0, 0]],
    });
    assert!(serde_json::from_value::<Soil>(overfull).is_err());

    let globally_overfull = serde_json::json!({
        "extent": 1,
        "matter_mg": [
            [18446744073709551615_u64, 0, 0, 0], [1, 0, 0, 0],
            [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0],
            [0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0],
            [0, 0, 0, 0],
        ],
    });
    assert!(serde_json::from_value::<Soil>(globally_overfull).is_err());
}
