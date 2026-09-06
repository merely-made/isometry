// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Trophic admission regressions that exercise the ecology tick.

use super::*;
use crate::flow::Process as FlowProcess;
use crate::organism::Kingdom;

#[test]
fn a_producer_without_an_active_fix_port_draws_no_soil() {
    let mut producer = organism(Kingdom::Producer, 300);
    let frond = producer
        .phenotype
        .body()
        .canopy_parts()
        .next()
        .expect("the producer fixture has a fixing frond");
    producer.phenotype.sever(frond);
    assert_eq!(
        producer.feeding_mode(),
        crate::process::FeedingMode::Producer
    );
    assert!(!producer.phenotype.canopy());

    let mut world = vec![producer];
    let mut sink = Sink::default();
    let mut rng = Rng::from_seed(6);
    let mut next = 1;
    let lines = registry(&world);
    step(
        &mut world,
        &mut next,
        &mut rng,
        &mut sink.stream(),
        &lines,
        PartPalette::primitive(),
        &mut soil(),
    );

    assert!(
        sink.flows
            .records()
            .iter()
            .all(|record| record.record.process != FlowProcess::Uptake),
        "a producer reading with no active Fix allocation drew from soil"
    );
}
