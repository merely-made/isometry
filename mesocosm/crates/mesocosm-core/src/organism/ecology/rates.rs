// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The life-history constants, and the allometry that scales them.
//!
//! Split out of `ecology.rs` on 2026-08-29 when TD4's idle gating pushed that
//! file at the six-hundred-line ceiling — the same split-before-adding move
//! that put `ecology::tests` and `ecology::movement` in their own files. What
//! stayed next door is the tick itself; what moved here is every number the
//! tick reads and the pure arithmetic that shapes them, which is also the
//! retune's written record.

use super::Organism;
use crate::process::FeedingMode;

/// Reference body mass for the allometric rates below.
const REFERENCE_MASS_MG: u64 = 100;
/// The integer fourth root of the reference mass, used to normalize quarter
/// power life-history rates.
const REFERENCE_MASS_QRT: u64 = 3;
/// Reference rates at 100 mg. These are model parameters, not organism facts.
/// TD2 set the tempo (life-history times, 3-4x); TD2c reset the trophic rates
/// against TD2b's walled enclosure and balanced founding, which is a different
/// world from the one TD2 was swept in. Receipt:
/// `Code/testing/mesocosm/td2c_persistence.json`.
///
/// TD2: 3x, stretched with lifespan so the juvenile share of a life is
/// unchanged at the slower tempo.
const MATURITY_BASE: u32 = 270;
/// TD2: 3x, putting a 1,000 mg starter's life at 3,000 ticks — five minutes
/// at the canonical 10 ticks/second rather than 100 seconds.
const LIFESPAN_BASE: u32 = 1800;
/// TD2: 4x against lifespan's 3x, one brood fewer per life. Still the knob
/// that decides boil against breathe — 360 boils, 480 does not.
pub const GESTATION_BASE: u32 = 480;
/// TD2c: 2 -> 5. Balanced founding puts ~22 consumers on ~20 producers, and
/// at 2 the base fixed less than the grazers drew (80 mg/tick against 88), so
/// the whole chain starved. The base has to out-produce its grazing pressure.
const FIXES_BASE_MG: u64 = 5;
/// TD2c: 2 -> 3. A grazer paying upkeep plus movement netted +1 on a fed tick
/// at 2, so it starved below a ~75% prey hit rate; 3 halves the rate it needs.
const GRAZES_BASE_MG: u64 = 3;
/// Scavenger income. Raising this does not rescue decomposers: they starve
/// with carrion in the enclosure but outside the radius they can search
/// (see the TD2c structural finding), so the yield per corpse is not the
/// binding constraint.
const DECAYS_BASE_MG: u64 = 4;
/// The basal cost of being alive.
pub(super) const UPKEEP_BASE_MG: u64 = 1;
/// The allometric share of the body's mass paid as upkeep.
/// TD2: halved rent, taking a 1,000 mg starter's energy budget from 166 ticks
/// of upkeep to 333.
const UPKEEP_SCALE: u64 = 62;
/// Edge of a crowding cell, in voxel units.
/// TD2c: 16 -> 8, undoing TD2's doubling. TD6 kept it: the soil bounds a
/// stand's *mass*, but nothing in a closed cycle bounds its *number*, and a
/// stand answered a finite matter budget by subdividing into ever-smaller
/// plants (620 producers and still rising at the horizon with crowding out).
/// `population_instrument.rs` mirrors this.
pub(super) const CROWD_CELL: i32 = 8;
/// Neighbours a cell supports before its occupants start shading each other
/// out. Beyond this a producer's income falls away and self-thinning begins.
/// TD2c: 2 -> 1, holding the stand's ceiling near 250 -- capacity is
/// `FIXES x COMFORT x UPKEEP_SCALE / 31` per cell.
pub(super) const CROWD_COMFORT: u32 = 1;
/// Voxels in the palette's reference segment (`PartPalette::primitive().mass`
/// is half-extent [2,2,2], so 5x5x5). The unit an adult mass is quoted in.
const REFERENCE_SEGMENT_VOXELS: u64 = 125;
/// The mass one reference segment holds, restated as the unit
/// [`upkeep_for_body`] measures a body's ceiling in — the same
/// `REFERENCE_MASS_MG` above, named for the job so the motility formula reads
/// as "swing per reference segment of body" rather than as arithmetic.
const REFERENCE_SEGMENT_MG: u64 = REFERENCE_MASS_MG;
/// Fraction of a parent's mass an offspring costs, as a divisor.
pub(crate) const OFFSPRING_COST: u64 = 4;
/// Share of the body plan's own adult mass a body must carry before it may
/// breed, in percent. **TD8's reproduction gate**, and the missing half of
/// TD6: growth became determinate but breeding still asked an absolute
/// milligram question that knew nothing about the ceiling the plan implies.
/// Picked against the instrument — see the plan's TD8 Progress entry for the
/// sweep — at the share where a body has to grow up without a big-plan body
/// having to spend a whole life doing it.
const BREEDING_SHARE_PCT: u64 = 33;
/// Mass below which an organism cannot sustain itself.
pub(crate) const STARVATION_MG: u64 = 20;
/// Ticks a corpse takes to return one milligram to the ground it lies on.
/// **TD8: duration, not yield.** Quadrupling `DECAYS_BASE_MG` was measured not
/// to rescue decomposers, so the lever is how long a corpse stands rather than
/// how much a scavenger gets per bite; this is the only number in the carrion
/// arm that is a *rate* rather than a *share*.
pub(crate) const CARRION_DECAY_TICKS: u32 = 4;
/// Ticks of upkeep still held in the budget below which a body without a
/// target starts wandering instead of standing still. TD2d: waiting for the
/// literal last milligram (`energy_mg == 0`) left every kingdom motionless
/// until half dead; this trades a few ticks of margin for a chance to find
/// something before the body starts eating itself. TD5 moved it here from
/// `movement` so the tick's one hunger horizon sits with the tick's numbers.
const HUNGRY_UPKEEP_TICKS: u64 = 8;

/// Whether a body's reserve has fallen low enough to search rather than wait.
/// The shared predicate lives on [`Organism`]; the horizon is this file's, and
/// **both** the wander and the dispersal bonus ask through it — TD2d moved the
/// wander off literal `energy_mg == 0` and left the bonus behind, which TD5
/// closes.
pub(crate) fn is_hungry(organism: &Organism) -> bool {
    organism.budget_below(HUNGRY_UPKEEP_TICKS)
}

/// The adult mass one part can hold: its own voxel volume, priced so that a
/// reference segment holds the ecology's own reference mass.
///
/// **No new authored number.** The two it reads are already here — the
/// allometry's `REFERENCE_MASS_MG` and the palette's own segment shape — which
/// is what makes this the body plan's ceiling rather than a second knob.
/// Floored at one milligram: development already gives every structural part
/// at least that, so a ceiling below it would make a legal body illegal.
pub(crate) fn part_ceiling_mg(half_extent: [i32; 3]) -> u64 {
    let voxels: u64 = half_extent
        .iter()
        .map(|h| 2 * u64::from(h.unsigned_abs()) + 1)
        .product();
    (voxels * REFERENCE_MASS_MG / REFERENCE_SEGMENT_VOXELS).max(1)
}

/// The mass a body must carry before it may breed: a share of the adult mass
/// its own body plan implies. (TD8)
///
/// **Life history's own answer, and no new kind of number** — the ceiling is
/// TD6's `Organism::mass_ceiling_mg`, derived per part from the plan's voxel
/// volume, so a big-plan body has to grow up and a small-plan body does not
/// wait on a stranger's yardstick. It replaces an absolute 80 mg floor that a
/// 3,500 mg plan cleared at 2% of adult size.
pub(crate) fn breeding_mass_mg(ceiling_mg: u64) -> u64 {
    ceiling_mg * BREEDING_SHARE_PCT / 100
}

/// Integer approximation of `mass^0.75`. It is monotonic, deterministic, and
/// uses no floating point in the authority boundary.
pub(crate) fn three_quarter_power(mass_mg: u64) -> u64 {
    let mass = mass_mg.max(1) as u128;
    integer_sqrt(mass * integer_sqrt(mass)) as u64
}

/// Integer approximation of `mass^0.25` for life-history tempo.
pub(crate) fn quarter_power(mass_mg: u64) -> u64 {
    integer_sqrt(integer_sqrt(mass_mg.max(1) as u128)) as u64
}

fn integer_sqrt(value: u128) -> u128 {
    let mut low = 0u128;
    let mut high = value.saturating_add(1);
    while high - low > 1 {
        let middle = low + (high - low) / 2;
        if middle <= value / middle.max(1) {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

fn quarter_rate(base: u32, mass_mg: u64) -> u32 {
    let q = quarter_power(mass_mg).max(1);
    (u64::from(base) * q / REFERENCE_MASS_QRT).max(1) as u32
}

fn allometric_rate(base: u64, mass_mg: u64) -> u64 {
    let reference = three_quarter_power(REFERENCE_MASS_MG).max(1);
    (base * three_quarter_power(mass_mg) / reference).max(1)
}

pub(crate) fn maturity_for_mass(mass_mg: u64) -> u32 {
    quarter_rate(MATURITY_BASE, mass_mg)
}

pub(crate) fn lifespan_for_mass(mass_mg: u64) -> u32 {
    quarter_rate(LIFESPAN_BASE, mass_mg)
}

pub(crate) fn gestation_for_mass(mass_mg: u64) -> u32 {
    quarter_rate(GESTATION_BASE, mass_mg)
}

pub(crate) fn producer_income_for_mass(mass_mg: u64) -> u64 {
    allometric_rate(FIXES_BASE_MG, mass_mg)
}

/// The one number both sides of the ledger read: **the actuator swing this
/// body carries per reference segment of body it carries it on**, kept as an
/// exact numerator over the plan's own adult mass so a caller can apply it in
/// a single division.
///
/// TD7 derived it and charged rent by it; TD9 pays income by it. Splitting it
/// out is what makes the symmetry a fact of the code rather than a claim in a
/// comment — there is one build multiple, and both halves of a body's ledger
/// are the same allometric base times it.
fn build_multiple(actuator_span: u32, ceiling_mg: u64) -> (u64, u64) {
    // A body with every part severed has no ceiling to normalize against; it
    // also has no actuators, so the multiple is exactly one.
    let ceiling = ceiling_mg.max(1);
    (
        ceiling + u64::from(actuator_span) * REFERENCE_SEGMENT_MG,
        ceiling,
    )
}

/// An allometric rate scaled by [`build_multiple`], in one division so the
/// result is integer-exact and a sessile body's answer is bit-identical to the
/// plain [`allometric_rate`] it replaces.
fn build_scaled_rate(base: u64, mass_mg: u64, actuator_span: u32, ceiling_mg: u64) -> u64 {
    let reference = three_quarter_power(REFERENCE_MASS_MG).max(1);
    let (priced, ceiling) = build_multiple(actuator_span, ceiling_mg);
    (base * three_quarter_power(mass_mg) * priced / (reference * ceiling)).max(1)
}

/// The mouthful a grazer or a predator reaches for. (TD9)
///
/// **The bite scales with build.** Same shape as [`upkeep_for_body`], same
/// three body-plan numbers, same multiple:
///
/// ```text
/// bite = GRAZES_BASE_MG * m^0.75 * (ceiling + span * REFERENCE_SEGMENT_MG)
///                       / (m_ref^0.75 * ceiling)
/// ```
///
/// TD7 made rent read a body's build and left income reading its mass alone,
/// so a body that paid a motility surcharge earned no return on it — measured
/// in TD8 as consumers clearing TD2c's ~75% prey hit-rate bar and starving
/// anyway on 5-11 mg mouthfuls against a rent that had risen from 1.5-1.8 to
/// 2.3-6.4 mg/tick. The machinery a body built to feed with is the same
/// contractile machinery it pays for; this is what makes limbs a strategy
/// rather than a tax.
///
/// **No new authored constant.** The base is TD2c's `GRAZES_BASE_MG`, untouched
/// — the sweep to 12 is on record as not reaching this — and the multiple is
/// TD7's, read off the same `actuator_span` and `mass_ceiling_mg`.
///
/// A sessile body reads span 0, the multiple is `ceiling / ceiling`, and the
/// rate is **exactly** what it was before TD9, to the milligram: the division
/// is one floor over a fraction that reduces, not two roundings. A test
/// asserts it, mirroring TD7's own symmetry check on rent.
pub(crate) fn feeding_rate_for_body(mass_mg: u64, actuator_span: u32, ceiling_mg: u64) -> u64 {
    build_scaled_rate(GRAZES_BASE_MG, mass_mg, actuator_span, ceiling_mg)
}

/// The mouthful a scavenger draws off a corpse, by the same rule. (TD9)
///
/// A decomposer that grew something to tear with should tear more off, for the
/// same reason a predator should. This is not the yield lever TD6 and TD7
/// measured out and TD8 ruled against: `DECAYS_BASE_MG` is unchanged, and what
/// moves is the same build multiple every other body reads.
pub(crate) fn decay_rate_for_body(mass_mg: u64, actuator_span: u32, ceiling_mg: u64) -> u64 {
    build_scaled_rate(DECAYS_BASE_MG, mass_mg, actuator_span, ceiling_mg)
}

/// The near tier's search horizon, read off the body's sense organs. (TD11)
///
/// **Sight reads the body**, by exactly the arithmetic rent and the bite read —
/// [`build_multiple`], handed a *sensory* span instead of a contractile one:
///
/// ```text
/// sight = base * (ceiling + sensor_span * REFERENCE_SEGMENT_MG) / ceiling
/// ```
///
/// It answers TD10's sixth finding: bodies foraged at eight and bit at fifty.
///
/// **No new authored constant, and the old eight survives as the reference.**
/// `base` is the caller's existing `NEAR_SIGHT_RANGE`; the multiple is the one
/// every other body-derived rate here already reads. A body with no sense organ
/// reads span 0 and a `ceiling / ceiling` multiple, so its horizon is
/// **exactly** the eight it had before — eight is the floor, not a subtraction.
/// Normalized against the plan's adult mass for TD7's reason, so the term is
/// scale-free and reads **build** rather than size.
///
/// **Bounded by construction**, like rent's multiple: the palette's sensor is
/// half-extent `[1, 1, 1]`, so it swings 1 against a 21 mg ceiling and a
/// reference segment's 100 mg. A body of nothing but sense organs reads
/// `121 / 21` and tops out at 46 voxels; no anatomy can see the enclosure.
///
/// Integer-exact and monotonic: one division. The caller does **not** clamp by
/// reach any more — that clamp was the conflation TD10 named, and `sight_range`
/// documents its removal.
pub(crate) fn sight_for_body(base: i32, sensor_span: u32, ceiling_mg: u64) -> i32 {
    let (priced, ceiling) = build_multiple(sensor_span, ceiling_mg);
    (u64::from(base.max(0).unsigned_abs()) * priced / ceiling).min(i32::MAX as u64) as i32
}

/// Rent priced by how a body lives, not only by what it weighs. (TD7)
///
/// **Derived, not tuned — three body-plan numbers and no new constant.**
///
/// ```text
/// rent = UPKEEP_BASE_MG + m^0.75 * (ceiling + span * REFERENCE_SEGMENT_MG)
///                       / (UPKEEP_SCALE * ceiling)
/// ```
///
/// The mass term is unchanged. The motile multiple is
/// `span * REFERENCE_SEGMENT_MG / ceiling`: **the actuator swing this body
/// carries per reference segment of body it carries it on**, where `span` is
/// `Organism::actuator_span` (each contractile part's longest half-extent,
/// summed) and `ceiling` is `Organism::mass_ceiling_mg` (the adult mass the
/// plan describes, itself a sum of per-part voxel volumes).
///
/// Why this is the honest normalizer: the swing has to be measured against
/// *something*, or a long body pays for being long rather than for moving.
/// Dividing by the plan's own adult mass makes the term scale-free — both
/// halves grow with the body — so it reads a body's **build**, not its size.
///
/// The consequences are all readings, not choices. A sessile plan reads span
/// 0 and pays exactly the mass rent it paid before, to the milligram. A
/// palette limb is half-extent `[4,1,1]`, so it swings 4 and holds a 64 mg
/// ceiling while an axial segment holds 100 mg; the seeded consumers and
/// decomposers this world founds land at roughly 2 to 4 swing per reference
/// segment, so they pay 3x to 5x a plant's rent for the same mass. And the
/// multiple is **bounded by construction**: a body made of nothing but limbs
/// reads `4 * 100 / 64` = 6.25, so no anatomy can price itself past ~7x.
///
/// Integer-exact and monotonic: one division, so conservation holds and the
/// rent a body owes is the rent it pays.
///
/// TD9 lifted the multiple out into [`build_multiple`] so income could read the
/// same one. The arithmetic here is unchanged.
///
/// **PD2 adds a third term, in the same shape and the same unit.** A gland is
/// machinery a body carries, exactly as an actuator is, so it is priced the
/// same way: `secretory_mg` is the toxin the body's allocation holds, added
/// into the numerator the actuator's swing is already added to, and normalized
/// against the same adult mass. It is already a milligram quantity — see
/// [`BodyPhenotype::secretory_mg`](crate::phenotype::BodyPhenotype::secretory_mg),
/// which prices allocated cells at what a cell of that part's tissue is worth
/// — so no conversion constant appears here and none was invented.
///
/// **A body with no gland reads zero and pays exactly what it paid before, to
/// the milligram**, the way a sessile body reads span 0. Nothing seeds a
/// gland, so that is every body a world founds.
pub(crate) fn upkeep_for_body(
    mass_mg: u64,
    actuator_span: u32,
    ceiling_mg: u64,
    secretory_mg: u64,
) -> u64 {
    let (priced, ceiling) = build_multiple(actuator_span, ceiling_mg);
    UPKEEP_BASE_MG
        + three_quarter_power(mass_mg) * (priced + secretory_mg) / (UPKEEP_SCALE * ceiling)
}

/// One graph step's dispersal budget. Contractile geometry gives larger bodies
/// more options, while hunger makes leaving an exhausted place worthwhile.
///
/// **No actuator, no travel** (TD8). This read `locomotion()`, which floors the
/// span at one for the drive selector's arithmetic, so a body that drew no
/// `Limb` tagma at all still got a step — a grazer moving at a plant's rent.
/// It reads `actuator_span` now: zero parts that contract, zero budget. Every
/// limbed body's budget is unchanged, because `locomotion` and `actuator_span`
/// differ only at zero.
pub(crate) fn dispersal_for(organism: &Organism) -> u32 {
    match organism.actuator_span() {
        0 => 0,
        span => (span / 4).max(1) + u32::from(is_hungry(organism)),
    }
}

/// Whether this body may go anywhere at all this tick.
///
/// **No actuator, no travel** (TD8) — **and producers creep** (TD9). A body
/// that carries nothing contractile is sessile, which is the rule that
/// withdrew the free lunch; a producer is the one exception, because spreading
/// is part of how a producer makes its living rather than something it does
/// with limbs.
///
/// The exception is written against the **feeding mode**, not against the
/// absence of limbs, and that is the whole of the care here: an unlimbed
/// *consumer* still reads false and stays exactly as sessile as TD8 left it.
/// The budget the exception buys is deliberately the smallest one in the file —
/// see `movement::disperse`, where a creeping body gets one grounded voxel and
/// never a place-graph hop.
pub(crate) fn travels(organism: &Organism) -> bool {
    organism.actuator_span() > 0 || organism.feeding_mode() == FeedingMode::Producer
}

#[cfg(test)]
mod tests;
