// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Producer uptake converts untyped soil into producer tissue at the accepted flow.

use super::*;

/// Records the accepted producer routing without adding a stored metabolic
/// compartment. One synthesis entry covers the accepted draw; reserve and spill
/// then debit that same flow through their completed routes.
pub(in crate::organism::ecology) fn record_synthesis(
    records: &mut Records<'_>,
    at: [i32; 3],
    subject: Subject,
    landed: &Landed,
) {
    let synthesized = landed
        .body_stock
        .checked_add(landed.reserve_stock)
        .and_then(|stock| stock.checked_add(landed.spilled_stock))
        .expect("accepted producer routing stays within its bounded stock");
    record_synthesized_substance(records, at, subject, synthesized);
    let reserve_mg = stock_mg(landed.reserve_stock);
    records.flow(
        at,
        FlowEvent::between(
            Process::Uptake,
            subject,
            Account::Substance,
            subject,
            Account::Reserve,
            reserve_mg,
        )
        .digested(landed.reserve_stock),
    );
    let spilled_mg = stock_mg(landed.spilled_stock);
    records.flow(
        at,
        FlowEvent::returned(Process::Spill, subject, Account::Substance, spilled_mg)
            .with_stock(landed.spilled_stock),
    );
}

fn record_synthesized_substance(
    records: &mut Records<'_>,
    at: [i32; 3],
    subject: Subject,
    stock: Stock,
) {
    let mg = stock_mg(stock);
    records.flow(
        at,
        FlowEvent::uptake(subject, Account::Substance, mg).synthesized(stock),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{SpeciesId, VolumeRef};
    use crate::development::PartPalette;
    use crate::flow::Ledger;
    use crate::matter::{Material, Stock};
    use crate::organism::{Kingdom, Organism, OrganismId, step};
    use crate::places::Soil;
    use crate::rng::Rng;
    use crate::species::Lineages;

    fn subject() -> Subject {
        Subject {
            organism: OrganismId(1),
            lineage: SpeciesId(2),
            kingdom: Kingdom::Producer,
        }
    }

    #[test]
    fn producer_reserve_is_synthesized_then_digested() {
        let mut ledger = Ledger::default();
        ledger.open(0);
        let mut events = Vec::new();
        let mut records = Records::new(0, None, &mut events, &mut ledger);
        let landed = Landed {
            reserve_stock: Stock::single(Material::Producer, 7),
            reserve_mg: 7,
            ..Landed::default()
        };

        record_synthesis(&mut records, [0, 0, 0], subject(), &landed);

        let flows: Vec<_> = ledger
            .records()
            .iter()
            .map(|record| record.record)
            .collect();
        assert_eq!(flows.len(), 2);
        let synthesis = flows[0].composition.expect("producer tissue is converted");
        assert_eq!(
            synthesis.conversion,
            Some(crate::flow::Conversion::Synthesis)
        );
        assert_eq!(synthesis.input, Stock::single(Material::Untyped, 7));
        assert_eq!(synthesis.output, landed.reserve_stock);
        let digestion = flows[1].composition.expect("reserve intake is converted");
        assert_eq!(
            digestion.conversion,
            Some(crate::flow::Conversion::Digestion)
        );
        assert_eq!(digestion.input, landed.reserve_stock);
        assert_eq!(digestion.output, Stock::single(Material::Untyped, 7));
    }

    #[test]
    fn producer_tick_synthesizes_untyped_soil_before_digestion() {
        let mut producer = Organism::founding(
            OrganismId(1),
            SpeciesId(2),
            Kingdom::Producer,
            VolumeRef::from_tag(1),
            [5, 5, 5],
            [0, 0, 0],
            100,
        );
        producer.energy_mg = 0;
        let mut organisms = vec![producer];
        let mut lineages = Lineages::new();
        lineages.found(SpeciesId(2));
        let mut soil = Soil::seeded(4, 1_000);
        let mut events = Vec::new();
        let mut ledger = Ledger::default();
        ledger.open(0);

        step(
            &mut organisms,
            &mut 2,
            &mut Rng::from_seed(1),
            &mut Records::new(0, None, &mut events, &mut ledger),
            &lineages,
            PartPalette::primitive(),
            &mut soil,
        );

        let flows: Vec<_> = ledger
            .records()
            .iter()
            .map(|record| record.record)
            .collect();
        let synthesis = flows
            .iter()
            .find(|flow| {
                flow.source == Account::Soil
                    && flow.destination == Account::Substance
                    && flow
                        .composition
                        .and_then(|composition| composition.conversion)
                        == Some(crate::flow::Conversion::Synthesis)
            })
            .expect("the producer converts its accepted soil draw");
        let composition = synthesis.composition.expect("synthesis has composition");
        assert_eq!(
            composition.input.amount(Material::Untyped),
            synthesis.amount_mg
        );
        assert_eq!(
            composition.output.amount(Material::Producer),
            synthesis.amount_mg
        );
        assert!(flows.iter().any(|flow| {
            flow.source == Account::Substance
                && flow.destination == Account::Reserve
                && flow
                    .composition
                    .and_then(|composition| composition.conversion)
                    == Some(crate::flow::Conversion::Digestion)
        }));
    }

    #[test]
    fn producer_does_not_draw_pending_typed_soil() {
        let mut producer = Organism::founding(
            OrganismId(1),
            SpeciesId(2),
            Kingdom::Producer,
            VolumeRef::from_tag(1),
            [5, 5, 5],
            [0, 0, 0],
            100,
        );
        producer.energy_mg = 0;
        let mut organisms = vec![producer];
        let mut lineages = Lineages::new();
        lineages.found(SpeciesId(2));
        let mut soil = Soil::seeded(0, 0);
        let column = soil.column_at([0, 0, 0]);
        let pending = Stock::from_amounts([0, 17, 19, 23]);
        soil.deposit_stock(column, pending).unwrap();
        let mut events = Vec::new();
        let mut ledger = Ledger::default();
        ledger.open(0);

        step(
            &mut organisms,
            &mut 2,
            &mut Rng::from_seed(1),
            &mut Records::new(0, None, &mut events, &mut ledger),
            &lineages,
            PartPalette::primitive(),
            &mut soil,
        );

        assert_eq!(soil.stock(column).amounts()[1..], pending.amounts()[1..]);
        // Rent returns untyped nutrients before this tick's roots draw. That
        // lot may be synthesized; none of the pending typed stock may be.
        let mut returned = 0;
        let mut synthesized = 0;
        for envelope in ledger.records() {
            let flow = envelope.record;
            if flow.process == Process::Upkeep && flow.destination == Account::Soil {
                returned += flow.amount_mg;
            }
            if flow
                .composition
                .is_some_and(|c| c.conversion == Some(crate::flow::Conversion::Synthesis))
            {
                let input = flow.composition.unwrap().input;
                assert_eq!(input, Stock::single(Material::Untyped, flow.amount_mg));
                synthesized += flow.amount_mg;
            }
        }
        assert!(synthesized > 0);
        assert_eq!(synthesized, returned);
    }
}
