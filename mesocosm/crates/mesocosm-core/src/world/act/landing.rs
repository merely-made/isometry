// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl World {
    /// Attempts the routed outcome, returning it and where the meal's mass
    /// went. Mutates the body but never the roster or the ledger.
    pub(super) fn land(
        &mut self,
        eaten: &Organism,
        route: Route,
        growth: Option<crate::growth::Growth>,
    ) -> (Outcome, Landed) {
        let stock = eaten
            .phenotype
            .total_stock()
            .expect("a meal fits its biomass");
        match route {
            Route::Burn => (
                Outcome::Burned {
                    organism: eaten.id,
                    energy_mg: eaten.biomass_mg(),
                },
                Landed {
                    budget_mg: eaten.biomass_mg(),
                    body_mg: 0,
                    body_stock: crate::matter::Stock::EMPTY,
                },
            ),
            Route::Incorporate {
                placement:
                    Placement::Explicit {
                        parent,
                        offset,
                        yaw,
                    },
            } => {
                let provenance = self.taken_from(eaten);
                let attached = self.controlled_phenotype_mut().attach_stock(
                    eaten.volume(),
                    eaten.biomass_mg(),
                    stock,
                    eaten.half_extent(),
                    Attachment {
                        parent,
                        offset,
                        yaw,
                    },
                    provenance,
                );
                match attached {
                    Ok(part) => (
                        Outcome::Incorporated { part },
                        Landed {
                            budget_mg: 0,
                            body_mg: eaten.biomass_mg(),
                            body_stock: stock,
                        },
                    ),
                    Err(_) => (
                        Outcome::Rejected(Rejection::NoSuchParent(parent)),
                        Landed::default(),
                    ),
                }
            },
            Route::Incorporate {
                placement: Placement::Planned,
            } => {
                let growth = growth.expect("resolved above for this route");
                let provenance = self.taken_from(eaten);

                // A mirrored pair splits the mass it came from, so the budget
                // stays honest however symmetric the body becomes.
                let parts = if growth.mirror.is_some() { 2 } else { 1 };
                let each = eaten.biomass_mg() / parts;
                let (first_stock, remainder) = stock.take(each);
                let (second_stock, _) = remainder.take(each);

                let Ok(part) = self.controlled_phenotype_mut().attach_stock(
                    eaten.volume(),
                    each,
                    first_stock,
                    eaten.half_extent(),
                    crate::growth::attachment(&growth),
                    provenance.clone(),
                ) else {
                    return (Outcome::Rejected(Rejection::NoRoom), Landed::default());
                };

                // No energy. Growing is the slow answer, and a meal cannot be
                // both meals.
                let (outcome, body_mg) = match crate::growth::mirror_attachment(&growth) {
                    Some(mirrored) => match self.controlled_phenotype_mut().attach_stock(
                        eaten.volume(),
                        each,
                        second_stock,
                        eaten.half_extent(),
                        mirrored,
                        provenance,
                    ) {
                        Ok(mirror) => (Outcome::IncorporatedPair { part, mirror }, each * 2),
                        Err(_) => (Outcome::Incorporated { part }, each),
                    },
                    None => (Outcome::Incorporated { part }, each),
                };
                let body_stock = if body_mg > each {
                    first_stock
                        .checked_add(second_stock)
                        .expect("meal total fits")
                } else {
                    first_stock
                };
                (
                    outcome,
                    Landed {
                        budget_mg: 0,
                        body_mg,
                        body_stock,
                    },
                )
            },
        }
    }
}
