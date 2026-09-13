// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{Account, Carrier, Composition, FlowEvent, Process, Subject};

impl FlowEvent {
    /// Matter routed between accounts within one body is not new income.
    pub fn is_internal(&self) -> bool {
        self.source.is_body()
            && self.destination.is_body()
            && matches!((self.from, self.to), (Some(from), Some(to)) if from.organism == to.organism)
    }

    /// Out of the ground and into a body.
    pub fn uptake(to: Subject, into: Account, amount_mg: u64) -> Self {
        Self {
            process: Process::Uptake,
            carrier: Carrier::Matter,
            source: Account::Soil,
            destination: into,
            amount_mg,
            composition: Some(Composition::untyped(amount_mg)),
            from: None,
            to: Some(to),
        }
    }

    /// Out of a body and back into the ground.
    pub fn returned(process: Process, from: Subject, out_of: Account, amount_mg: u64) -> Self {
        Self {
            process,
            carrier: Carrier::Matter,
            source: out_of,
            destination: Account::Soil,
            amount_mg,
            composition: (out_of == Account::Reserve).then(|| Composition::untyped(amount_mg)),
            from: Some(from),
            to: None,
        }
    }

    /// Out of the dev source and into the ground. (DT3)
    ///
    /// Names no [`Subject`] on either end, exactly as [`Self::uptake`] names
    /// none on the soil end: neither account belongs to a body, so a
    /// reconciliation over bodies passes this by and the soil's own claim is
    /// the whole of it.
    pub fn placed(amount_mg: u64) -> Self {
        Self {
            process: Process::Place,
            carrier: Carrier::Matter,
            source: Account::Dev,
            destination: Account::Soil,
            amount_mg,
            composition: Some(Composition::untyped(amount_mg)),
            from: None,
            to: None,
        }
    }

    /// Between two bodies: a meal, or a parent provisioning a child.
    pub fn between(
        process: Process,
        from: Subject,
        out_of: Account,
        to: Subject,
        into: Account,
        amount_mg: u64,
    ) -> Self {
        Self {
            process,
            carrier: Carrier::Matter,
            source: out_of,
            destination: into,
            amount_mg,
            composition: (out_of == Account::Reserve).then(|| Composition::untyped(amount_mg)),
            from: Some(from),
            to: Some(to),
        }
    }

    /// The signed effect of this flow on one account, in milligrams.
    ///
    /// A transfer between two of the same account nets to nothing, which is what
    /// makes soil-to-soil transport honest to leave unrecorded.
    pub fn net_on(&self, account: Account) -> i128 {
        let into = i128::from(self.destination == account);
        let out = i128::from(self.source == account);
        (into - out) * i128::from(self.amount_mg)
    }
}
