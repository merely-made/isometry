// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Completed metabolic returns use the lot actually paid by the body.

use super::*;
use crate::matter::Material;

fn complete_return(
    soil: &mut Soil,
    records: &mut Records<'_>,
    at: [i32; 3],
    subject: Subject,
    source: Account,
    process: Process,
    stock: Stock,
) {
    let mg = u64::try_from(stock.total()).expect("a body payment fits its scalar debt");
    let flow = FlowEvent::returned(process, subject, source, mg).mineralized(stock);
    let output = flow
        .composition
        .expect("completed returns carry composition")
        .output;
    soil.deposit_stock(soil.column_at(at), output)
        .expect("conserved returns fit the finite world's soil");
    records.flow(at, flow);
}

pub(in crate::organism::ecology) fn pay_upkeep(
    organism: &mut Organism,
    soil: &mut Soil,
    records: &mut Records<'_>,
) {
    let subject = Subject::of(organism);
    let rent = organism.pay_upkeep();
    for (source, stock) in [
        (
            Account::Reserve,
            Stock::single(Material::Untyped, rent.reserve_mg),
        ),
        (Account::Substance, rent.substance_stock),
    ] {
        complete_return(
            soil,
            records,
            organism.position,
            subject,
            source,
            Process::Upkeep,
            stock,
        );
    }
}

pub(in crate::organism::ecology) fn pay_travel(
    organism: &mut Organism,
    soil: &mut Soil,
    records: &mut Records<'_>,
    from: [i32; 3],
    distance: u64,
) {
    let subject = Subject::of(organism);
    let stock = organism.phenotype.spend_stock(distance.max(1));
    complete_return(
        soil,
        records,
        from,
        subject,
        Account::Substance,
        Process::Travel,
        stock,
    );
}

/// Death retains nis. Only the cadence's removed lot completes decay.
pub(in crate::organism::ecology) fn decay(
    organism: &mut Organism,
    soil: &mut Soil,
    records: &mut Records<'_>,
) {
    let returning = u64::from(
        organism
            .age
            .is_multiple_of(crate::organism::ecology::CARRION_DECAY_TICKS),
    );
    let subject = Subject::of(organism);
    let stock = organism.phenotype.spend_stock(returning);
    complete_return(
        soil,
        records,
        organism.position,
        subject,
        Account::Substance,
        Process::Decay,
        stock,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body::{SpeciesId, VolumeRef};
    use crate::flow::{Conversion, Ledger};
    use crate::organism::{Kingdom, OrganismId};

    #[test]
    fn death_keeps_nis_and_only_the_decay_dose_is_mineralized() {
        let mut organism = Organism::founding(
            OrganismId(1),
            SpeciesId(1),
            Kingdom::Producer,
            VolumeRef::from_tag(1),
            [2, 2, 2],
            [0, 0, 0],
            100,
        );
        organism
            .phenotype
            .replace_part_stock(organism.body().root, Stock::from_amounts([9, 30, 30, 30]))
            .unwrap();
        organism.energy_mg = 7;
        let before = organism.phenotype.total_stock().unwrap();
        let mut soil = Soil::seeded(2, 0);
        let mut events = Vec::new();
        let mut flows = Ledger::default();
        flows.open(0);
        super::super::perish(
            &mut organism,
            &mut soil,
            &mut Records::new(0, None, &mut events, &mut flows),
        );
        assert_eq!(organism.phenotype.total_stock().unwrap(), before);
        assert_eq!(soil.total_stock(), Stock::single(Material::Untyped, 7));

        organism.age = 1;
        decay(
            &mut organism,
            &mut soil,
            &mut Records::new(1, None, &mut events, &mut flows),
        );
        assert_eq!(organism.phenotype.total_stock().unwrap(), before);
        organism.age = crate::organism::ecology::CARRION_DECAY_TICKS;
        decay(
            &mut organism,
            &mut soil,
            &mut Records::new(4, None, &mut events, &mut flows),
        );
        let paid = before
            .checked_sub(organism.phenotype.total_stock().unwrap())
            .unwrap();
        assert_eq!(paid.total(), 1);
        assert_eq!(paid.amount(Material::Untyped), 0);
        assert_eq!(soil.total_stock(), Stock::single(Material::Untyped, 8));
        let flow = flows
            .records()
            .iter()
            .find(|f| f.record.process == Process::Decay)
            .unwrap()
            .record;
        let composition = flow.composition.unwrap();
        assert_eq!(composition.input, paid);
        assert_eq!(composition.output, Stock::single(Material::Untyped, 1));
        assert_eq!(composition.conversion, Some(Conversion::Mineralization));
    }
}
