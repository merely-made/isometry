// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! TG2a accounting and transport instrument; does not run the live World.
use std::{hint::black_box, time::Instant};

use mesocosm_core::{
    OrganismId, PartId,
    matter::{
        Material, Stock,
        receipt::{Address, Book, Conversion, Receipt, reconcile, replay},
        transport,
    },
};
#[path = "../src/matter/transport/scalar_reference.rs"]
mod scalar_reference;
use scalar_reference::ScalarSoil as Soil;
use serde_json::json;

fn main() {
    let soil = Address::Soil([0, 0, 0]);
    let donor = Address::Part(OrganismId(1), PartId(0));
    let graft = Address::Part(OrganismId(2), PartId(0));
    let reserve = Address::Reserve(OrganismId(2));
    let before = Book::from([(soil, Stock::single(Material::Untyped, 101))]);
    let receipts = vec![
        Receipt::Conversion {
            kind: Conversion::Synthesis,
            from: soil,
            to: donor,
            input: Stock::single(Material::Untyped, 31),
            output: Stock::single(Material::Producer, 31),
        },
        Receipt::Transfer {
            from: donor,
            to: graft,
            stock: Stock::single(Material::Producer, 17),
        },
        Receipt::Conversion {
            kind: Conversion::Digestion,
            from: graft,
            to: reserve,
            input: Stock::single(Material::Producer, 7),
            output: Stock::single(Material::Untyped, 7),
        },
        Receipt::Conversion {
            kind: Conversion::Mineralization,
            from: donor,
            to: soil,
            input: Stock::single(Material::Producer, 14),
            output: Stock::single(Material::Untyped, 14),
        },
    ];
    // Independent expected balances, not the replay result used as its own oracle.
    let after = Book::from([
        (soil, Stock::single(Material::Untyped, 84)),
        (donor, Stock::EMPTY),
        (graft, Stock::single(Material::Producer, 10)),
        (reserve, Stock::single(Material::Untyped, 7)),
    ]);
    reconcile(&before, &after, &receipts).unwrap();
    let bytes = postcard::to_allocvec(&(before.clone(), receipts.clone())).unwrap();
    let (decoded, decoded_receipts): (Book, Vec<Receipt>) = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(replay(&decoded, &decoded_receipts).unwrap(), after);
    let mut broken = after.clone();
    broken.insert(graft, Stock::single(Material::Consumer, 10));
    let scalar = |book: &Book| book.values().map(|stock| stock.total()).sum::<u128>();
    assert_eq!(scalar(&before), scalar(&broken));
    let broken_error = reconcile(&before, &broken, &receipts).unwrap_err();
    let (taken, remainder) = Stock::from_amounts([1, 1, 1, 1]).take(3);
    assert_eq!(taken.amounts(), [1, 1, 1, 0]);
    assert_eq!(remainder.amounts(), [0, 0, 0, 1]);

    let result = json!({
        "scope": "TG2a prototype accounts and column kernel; live World integration and full-tick budget remain open",
        "optimized": !cfg!(debug_assertions),
        "conservation": {"total_mg": scalar(&after) as u64, "typed_reconciles": true,
            "serialized_replay_exact": true, "graft_retains_producer_mg": 10,
            "broken_control_scalar_passes": true, "broken_control_typed_error": format!("{broken_error:?}")},
        "stock_bytes_per_column": std::mem::size_of::<Stock>(),
        "scalar_bytes_per_column": std::mem::size_of::<u64>(),
        "transport": benchmark(),
    });
    let rendered = serde_json::to_string_pretty(&result).unwrap();
    if let Some(path) = std::env::args().nth(1) {
        std::fs::write(path, &rendered).unwrap();
    }
    println!("{rendered}");
}

fn benchmark() -> serde_json::Value {
    const EXTENT: i32 = 64;
    const SIDE: usize = 129;
    const TICKS: usize = 100;
    const SAMPLES: usize = 9;
    let mut scalar_seed = Soil::seeded(EXTENT, 0);
    let mut typed_seed = vec![Stock::EMPTY; SIDE * SIDE];
    for (index, stock) in typed_seed.iter_mut().enumerate() {
        let mg = 10_000 + (index as u64 * 7919) % 100_003;
        *stock = Stock::from_amounts([mg, mg / 2, mg / 3, mg / 7]);
        let at = [
            (index % SIDE) as i32 - EXTENT,
            0,
            (index / SIDE) as i32 - EXTENT,
        ];
        scalar_seed.deposit(scalar_seed.column_at(at), mg);
    }
    let mut scalar_ms = Vec::new();
    let mut typed_ms = Vec::new();
    // First pair warms both paths; alternate order to reduce order bias.
    for sample in 0..=SAMPLES {
        let mut scalar = scalar_seed.clone();
        let mut typed = typed_seed.clone();
        let mut run_scalar = || {
            let started = Instant::now();
            for _ in 0..TICKS {
                black_box(&mut scalar).percolate();
            }
            started.elapsed().as_secs_f64() * 1000.0 / TICKS as f64
        };
        let mut run_typed = || {
            let started = Instant::now();
            for _ in 0..TICKS {
                transport::percolate(black_box(&mut typed), SIDE, 8).unwrap();
            }
            started.elapsed().as_secs_f64() * 1000.0 / TICKS as f64
        };
        let (s, t) = if sample % 2 == 0 {
            (run_scalar(), run_typed())
        } else {
            let t = run_typed();
            (run_scalar(), t)
        };
        for (index, stock) in typed.iter().enumerate() {
            let at = [
                (index % SIDE) as i32 - EXTENT,
                0,
                (index / SIDE) as i32 - EXTENT,
            ];
            assert_eq!(
                stock.amount(Material::Untyped),
                scalar.matter_mg(scalar.column_at(at))
            );
        }
        for material in Material::ALL {
            assert_eq!(
                typed
                    .iter()
                    .map(|s| u128::from(s.amount(material)))
                    .sum::<u128>(),
                typed_seed
                    .iter()
                    .map(|s| u128::from(s.amount(material)))
                    .sum::<u128>()
            );
        }
        if sample > 0 {
            scalar_ms.push(s);
            typed_ms.push(t);
        }
    }
    scalar_ms.sort_by(f64::total_cmp);
    typed_ms.sort_by(f64::total_cmp);
    json!({"columns": SIDE * SIDE, "ticks_per_sample": TICKS, "samples": SAMPLES,
        "scalar_one_channel_median_ms": scalar_ms[SAMPLES / 2],
        "typed_four_channel_median_ms": typed_ms[SAMPLES / 2],
        "scalar_one_channel_sample_ms": scalar_ms, "typed_four_channel_sample_ms": typed_ms,
        "untyped_matches_incumbent": true, "all_channel_totals_exact": true})
}
