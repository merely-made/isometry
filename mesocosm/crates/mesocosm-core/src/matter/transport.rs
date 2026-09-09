// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Candidate column transport for TG2. One synchronous pass, exact per channel.
//! The caller owns the columns; this kernel retains neither fields nor history.

use super::{Material, Stock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportError {
    InvalidShape,
    ZeroDivisor,
    Overflow { column: usize, material: Material },
}

/// Apply Soil's west/east/north/south split independently to each channel.
/// Remainders use that fixed order. Invalid input or overflow leaves every
/// column untouched. `divisor` is supplied by the owning world's soil rule.
pub fn percolate(columns: &mut [Stock], side: usize, divisor: u64) -> Result<(), TransportError> {
    if side == 0 || side.checked_mul(side) != Some(columns.len()) {
        return Err(TransportError::InvalidShape);
    }
    if divisor == 0 {
        return Err(TransportError::ZeroDivisor);
    }
    if side == 1 {
        return Ok(());
    }
    let mut next: Vec<[u64; 4]> = columns
        .iter()
        .map(|stock| stock.amounts().map(|mg| mg - mg / divisor))
        .collect();
    for (index, stock) in columns.iter().enumerate() {
        let x = index % side;
        let z = index / side;
        let neighbours = [
            x.checked_sub(1).map(|nx| z * side + nx),
            (x + 1 < side).then_some(index + 1),
            z.checked_sub(1).map(|nz| nz * side + x),
            (z + 1 < side).then_some(index + side),
        ];
        let count = neighbours.iter().flatten().count() as u64;
        for material in Material::ALL {
            let out = stock.amount(material) / divisor;
            let share = out / count;
            let extra = out % count;
            for (rank, &neighbour) in neighbours.iter().flatten().enumerate() {
                let slot = &mut next[neighbour][material.index()];
                *slot = slot
                    .checked_add(share + u64::from((rank as u64) < extra))
                    .ok_or(TransportError::Overflow {
                        column: neighbour,
                        material,
                    })?;
            }
        }
    }
    for (column, amounts) in columns.iter_mut().zip(next) {
        *column = Stock::from_amounts(amounts);
    }
    Ok(())
}

#[cfg(test)]
mod scalar_reference;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_transport_matches_four_independent_scalar_soils() {
        use super::scalar_reference::ScalarSoil as Soil;
        let extent = 2;
        let side = 5;
        let mut columns = vec![Stock::EMPTY; side * side];
        let mut scalar = std::array::from_fn::<_, 4, _>(|_| Soil::seeded(extent, 0));
        for (index, stock) in columns.iter_mut().enumerate() {
            let amounts = [
                index as u64 * 17 + 1,
                index as u64 * 3,
                71 - index as u64,
                9,
            ];
            *stock = Stock::from_amounts(amounts);
            let at = [
                (index % side) as i32 - extent,
                0,
                (index / side) as i32 - extent,
            ];
            for material in Material::ALL {
                let soil = &mut scalar[material.index()];
                soil.deposit(soil.column_at(at), amounts[material.index()]);
            }
        }
        let totals = Material::ALL.map(|m| {
            columns
                .iter()
                .map(|s| u128::from(s.amount(m)))
                .sum::<u128>()
        });
        for _ in 0..100 {
            percolate(&mut columns, side, 8).unwrap();
            for soil in &mut scalar {
                soil.percolate();
            }
            for (index, stock) in columns.iter().enumerate() {
                let at = [
                    (index % side) as i32 - extent,
                    0,
                    (index / side) as i32 - extent,
                ];
                for material in Material::ALL {
                    let soil = &scalar[material.index()];
                    assert_eq!(stock.amount(material), soil.matter_mg(soil.column_at(at)));
                }
            }
            assert_eq!(
                totals,
                Material::ALL.map(|m| columns
                    .iter()
                    .map(|s| u128::from(s.amount(m)))
                    .sum::<u128>())
            );
        }
    }

    #[test]
    fn failures_are_atomic_and_single_column_has_nowhere_to_shed() {
        let mut columns = vec![Stock::single(Material::Consumer, u64::MAX); 9];
        let original = columns.clone();
        assert_eq!(
            percolate(&mut columns, 2, 8),
            Err(TransportError::InvalidShape)
        );
        assert_eq!(
            percolate(&mut columns, 3, 0),
            Err(TransportError::ZeroDivisor)
        );
        assert!(matches!(
            percolate(&mut columns, 3, 1),
            Err(TransportError::Overflow { .. })
        ));
        assert_eq!(columns, original);
        let mut single = [Stock::from_amounts([u64::MAX; 4])];
        let before = single;
        percolate(&mut single, 1, 1).unwrap();
        assert_eq!(single, before);
    }
}
