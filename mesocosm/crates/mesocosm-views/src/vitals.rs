// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The in-epoch vitals surface: what the played critter has, and what the
//! world just said back — a refusal, or what the body did with a meal.
//!
//! The cambium lane (ruled 2026-08-29), and Mesocosm's first consumer of it.
//! Words and numbers are expected here; the painted lane keeps its own
//! discipline and grows no lettering. The first real playtest ran with energy,
//! refusals and death all unsurfaced, which is what this exists against.
//!
//! Host-agnostic, like the paint leaves beside it: this crate says what the
//! panel *is*, and the host says where its raster lands.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};
use mesocosm_core::{
    Crossing, Discovery, Gland, Graft, Ineligible, Observation, Outcome, Refusal, Rejection, Trend,
    Unrevised, World,
};

/// A view in the vitals tree. Inert: nothing here takes a click, because
/// during an epoch you act on the world, not on a panel.
pub type VitalsChild = Box<dyn AnyView<Vitals, (), GenetCtx, GenetElement>>;

/// The energy bar's full width, in logical pixels. The fill is measured
/// against this rather than against a percentage so the bar reads the same
/// whatever box the panel ends up in.
const BAR_WIDTH: f32 = 168.0;

/// What the panel shows. A **reading**, taken fresh from the world each time:
/// nothing here is stored in the world, enters the trace, or reaches the state
/// hash. The host owns how long a notice stays up, because that is a clock,
/// not a fact.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Vitals {
    /// The played critter's budget. `None` once control is lost, which is the
    /// persistent death state: the panel says so and empties.
    pub energy_mg: Option<u64>,
    /// Where the budget sits against the most it has held this session,
    /// clamped to `0..=1`. Presentation only — the world has no capacity, so
    /// there is no other honest denominator.
    pub fullness: f32,
    /// The most recent thing the world said back, in plain words, while the
    /// host is still showing it: a refusal, or — since TD4 took the meal's
    /// destination out of the player's hands — what the body did with a meal.
    /// The state that decided is the number right above it.
    pub notice: Option<&'static str>,
    /// The first ecology reading: maturation against mortality, and the window
    /// it covers. **A fact with its window on it**, never a bare ratio.
    pub replacement: Option<String>,
    /// The one warning PE0 ships, when the support path has actually run short.
    /// It says what moved and over how long; it never presents a collapse
    /// percentage, and it never repeats the population instrument's test
    /// verdicts, which are not player language.
    pub warning: Option<String>,
    /// PD2's process, when this body has one. Absent for every body a world
    /// founds, because nothing grows a gland.
    pub gland: Option<GlandWords>,
    /// What this line has most recently come to: what it is, the route it came
    /// by, and the evidence that carried it. (PE2)
    pub discovery: Option<DiscoveryWords>,
    /// The branch this body most recently took off something else, and on what
    /// terms. (P3)
    pub graft: Option<GraftWords>,
    /// The last evidence a condition was offered and did not take.
    ///
    /// **Evidence that unlocked nothing is still evidence.** A meal that fed
    /// you and taught you nothing is the ordinary case, and a panel that only
    /// ever spoke on a discovery would leave a player unable to tell "that
    /// taught me nothing" from "the game did not notice."
    pub observation: Option<String>,
}

/// The three things a discovery is owed: what it is, how it was come by, and
/// what the evidence was.
///
/// Separate rows because they answer different questions and a player rereads
/// them at different times: what you now have, why you have it, and — since a
/// candidate is availability rather than expression — where it can go.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryWords {
    /// The condition's name, plainly.
    pub what: String,
    /// The route, and the evidence that came down it.
    pub route: String,
    /// What it grants, and on what.
    pub grants: String,
}

/// The two sentences a transferred branch is owed: where it came from, and
/// what it is doing here. (P3)
///
/// Provenance is the first of them because it is the thing a graft has that
/// growing does not: this tissue was somebody. The second is the verdict made
/// legible — a carried branch that arrived native works, a carried branch over
/// a cross-domain edge is on you and doing nothing, and a regrown one is doing
/// whatever your own rules make of that shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraftWords {
    /// Which parts came, off which part of which line.
    pub taken: String,
    /// The crossing, the verdict, and what that leaves the branch doing.
    pub terms: String,
}

/// A branch transfer in plain words.
pub fn graft_words(graft: &Graft, expressing: bool) -> GraftWords {
    let taken = format!(
        "{} part{} from part {} of line {}",
        graft.parts.len(),
        if graft.parts.len() == 1 { "" } else { "s" },
        graft.donor_part.0,
        graft.donor_line.0,
    );
    // What the branch is *doing* is read off the body rather than inferred from
    // the verdict, because they are two different facts and a panel that
    // guessed the second from the first would be describing the table instead
    // of the creature.
    let doing = if expressing {
        "working"
    } else {
        "doing nothing yet"
    };
    GraftWords {
        taken,
        terms: format!(
            "{} on part {} — {}, {}",
            graft.crossing.name(),
            graft.root.0,
            graft.verdict.name(),
            doing
        ),
    }
}

/// The three sentences a gland is owed, one per question a player asks of it:
/// where is it, is it working, and what is it costing me.
///
/// Separate rows rather than one paragraph because they change on different
/// clocks: the tissue moves only at a development, the sting turns on and off
/// as the body walks, and the rent is the same every tick until one of the
/// other two changes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlandWords {
    /// Where the tissue is, or where it was.
    pub tissue: String,
    /// What a bite of this body costs right now, and why when it costs
    /// nothing.
    pub sting: String,
    /// What carrying it costs per tick.
    pub rent: String,
}

/// The gland's three readings, in plain words.
///
/// **States, not numbers with adjectives.** A dry gland says what the ground
/// holds and what the gland holds, because the difference is the thing a
/// player can act on — walk to better ground, or enrich this one.
pub fn gland_words(gland: &Gland) -> GlandWords {
    let tissue = match (gland.sites.first(), gland.lost.first()) {
        (Some((part, cells)), _) => format!("{cells} cells of part {}", part.0),
        // Severing took the tissue and the consequence together; the branch
        // can still say what it used to do.
        (None, Some(part)) => format!("gone with part {}", part.0),
        (None, None) => "none".to_string(),
    };
    let sting = if gland.sites.is_empty() {
        "nothing left to sting with".to_string()
    } else if gland.charged {
        format!("{} mg a bite", gland.potency_mg)
    } else {
        format!(
            "dry: this ground holds {} mg, the gland needs {}",
            gland.ground_mg, gland.potency_mg
        )
    };
    GlandWords {
        tissue,
        sting,
        rent: format!("{} mg a tick", gland.rent_mg),
    }
}

/// A discovery in plain words. (PE2)
///
/// **Evidence and route, not only the thing unlocked** — the plan's §1 asks
/// for exactly that, because a player who cannot see what taught them is back
/// in a diet tree whether or not the code is.
pub fn discovery_words(discovery: &Discovery) -> DiscoveryWords {
    let what = condition_word(discovery.condition);
    let grants = format!(
        "{} on {}{}",
        process_word(discovery.candidate.process),
        site_word(discovery.candidate.site),
        // Inheritance is a separate fact from expression, so it is a separate
        // clause: this body may develop it, and its descendants may be born
        // with the shape to.
        if discovery.candidate.word.is_some() {
            ", and the word for one"
        } else {
            ""
        }
    );
    DiscoveryWords {
        what,
        route: format!("{}: {}", discovery.route.name(), discovery.evidence.words()),
        grants,
    }
}

/// The last evidence offered, and what became of it.
///
/// It names the condition that refused it and why, because "this is not a
/// question that one asks" and "not enough of it" are different facts and only
/// the second is worth trying harder at.
///
/// `None` when a condition took it: the three discovery rows above the panel's
/// evidence line have already said what it was and what it bought, and saying
/// it twice is how a panel stops being read.
pub fn observation_words(observation: &Observation) -> Option<String> {
    if observation.matched.is_some() {
        return None;
    }
    Some(match observation.missed.first() {
        Some((condition, miss)) => format!(
            "{} — {}: {}",
            observation.evidence.words(),
            condition_word(*condition),
            miss.words()
        ),
        None => observation.evidence.words(),
    })
}

/// A condition's name, plainly and the same way everywhere it appears.
///
/// The namespace is dropped and the hyphens opened out, because two rows of one
/// panel naming the same condition two ways reads as two conditions. `None`
/// from the registry is the missing-ruleset diagnostic and is said rather than
/// papered over with a similar local name.
pub(crate) fn condition_word(condition: mesocosm_core::ConditionId) -> String {
    mesocosm_core::discovery::name_of(condition)
        .map(|name| name.trim_start_matches("mesocosm:").replace('-', " "))
        .unwrap_or_else(|| "a condition this world does not hold".to_string())
}

fn process_word(process: mesocosm_core::ProcessRef) -> String {
    mesocosm_core::Registry::native()
        .resolve(process)
        .map(|def| def.id.name.clone())
        .unwrap_or_else(|| "an unknown process".to_string())
}

fn site_word(site: mesocosm_core::Role) -> &'static str {
    match site {
        mesocosm_core::Role::Mass => "bulk",
        mesocosm_core::Role::Limb => "a limb",
        mesocosm_core::Role::Plate => "a plate",
        mesocosm_core::Role::Sensor => "a sensor",
    }
}

impl Vitals {
    pub fn is_dead(&self) -> bool {
        self.energy_mg.is_none()
    }
}

/// Reads the vitals out of a world. `high_water` is the host's running
/// maximum, passed in because the bar's scale is a session fact rather than a
/// world one; `trend` is the driver's bounded windows, passed in for the same
/// reason — a world cannot say what happened, only what is.
pub fn vitals_of(
    world: &World,
    high_water: u64,
    notice: Option<&'static str>,
    trend: Option<&Trend>,
) -> Vitals {
    let mut vitals = reading(
        world.controlled().map(|critter| critter.energy_mg),
        high_water,
        notice,
        trend,
    );
    vitals.gland = world.gland().as_ref().map(gland_words);
    // The most recent one. A running list of everything a line ever came to is
    // a journal, and a journal is not a vitals panel.
    vitals.discovery = world.discoveries().last().map(discovery_words);
    vitals.observation = world.last_observation().and_then(observation_words);
    // The most recent branch, and whether it is expressing anything today. Read
    // off the phenotype, so an incompatible branch that has since been given an
    // adapter stops calling itself idle.
    vitals.graft = world.carried_branch().map(|graft| {
        let expressing = world.phenotype().is_some_and(|phenotype| {
            graft.parts.iter().any(|part| {
                phenotype
                    .explain(*part)
                    .is_some_and(|read| read.living && !read.sites.is_empty())
            })
        });
        graft_words(graft, expressing)
    });
    vitals
}

/// The reading itself, off the world. Separated so the disembodied case is
/// testable: nothing public drops control, and the world it happens in is a
/// thousand ticks away.
fn reading(
    energy_mg: Option<u64>,
    high_water: u64,
    notice: Option<&'static str>,
    trend: Option<&Trend>,
) -> Vitals {
    let fullness = match (energy_mg, high_water) {
        (Some(energy), top) if top > 0 => (energy as f32 / top as f32).clamp(0.0, 1.0),
        _ => 0.0,
    };
    Vitals {
        energy_mg,
        fullness,
        // A dead critter refuses nothing and eats nothing; the death state
        // stands alone.
        notice: notice.filter(|_| energy_mg.is_some()),
        // The enclosure keeps happening whether or not anyone is in it, so
        // these two survive losing a body.
        replacement: trend.map(replacement_words),
        warning: trend.and_then(warning_words),
        // Filled by `vitals_of`, which has the world the ground is in.
        gland: None,
        discovery: None,
        graft: None,
        observation: None,
    }
}

/// Maturation against mortality, and the window it covers.
///
/// Counts rather than a ratio, because a ratio hides how much evidence it is
/// made of: three deaths in two hundred ticks and three hundred are the same
/// number and not the same fact.
pub fn replacement_words(trend: &Trend) -> String {
    format!(
        "{} matured, {} died in {} ticks",
        trend.matured, trend.died, trend.replacement_ticks
    )
}

/// The warning, when the support path has run short long enough to say so.
///
/// **What moved, over what window.** Never a collapse percentage, and never one
/// of the population instrument's verdicts: those classify a test run, and
/// turning one into player language is a separate interaction ruling.
///
/// The two numbers sit side by side rather than one inside the other. Grazing is
/// one of several ways a stand loses matter, and it can exceed the net loss when
/// the survivors are still growing; saying "of which" would be a causal claim
/// the record does not make.
pub fn warning_words(trend: &Trend) -> Option<String> {
    trend.warns().then(|| {
        format!(
            "the stand has been shrinking for {} ticks: {} mg lost over the last {}; mouths took {} mg in the same window",
            trend.shortfall_ticks,
            trend.stand_change_mg.unsigned_abs(),
            trend.stand_ticks,
            trend.grazed_mg
        )
    })
}

/// The plain words for a rejection. Short because they are read in motion, and
/// stated as what happened rather than as an error code.
pub fn refusal_words(rejection: &Rejection) -> &'static str {
    match rejection {
        Rejection::InsufficientMass => "not enough energy",
        Rejection::OutOfReach(_) => "out of reach",
        Rejection::Disembodied => "no body",
        Rejection::Itself => "that is you",
        Rejection::Inedible(_) => "your intake organs cannot accept it",
        Rejection::NoRoom => "no room for it",
        Rejection::NoSuchOrganism(_) => "nothing there",
        Rejection::NoSuchParent(_) => "nowhere to attach",
        // PE2's part-level meal. Each says what would make it possible: wait,
        // pick another organ, or find one that still has something in it.
        Rejection::NoSuchPart(_) => "it has no such part",
        Rejection::StillLiving(_) => "that one is still using it",
        Rejection::NothingLeft(_) => "nothing left of that part",
        // P3's branch transfer. Each says what would make it possible: take a
        // branch instead of the whole thing, or regrow what cannot be carried.
        Rejection::WholeBody(_) => "that is the whole of it",
        Rejection::Incompatible { .. } => "that tissue will not go in you",
        Rejection::Ineligible(Ineligible::NotAlive) => "that one is dead",
        Rejection::Ineligible(Ineligible::AboveTheFrontier { .. }) => "beyond you",
        Rejection::Ineligible(Ineligible::NoSuchOrganism) => "nothing there",
        // PD3's bounded door. A hand asks for what its line came to, so both
        // refusals are about availability rather than about arrangement: it
        // has not come to that, or it has and this body is the wrong shape.
        Rejection::Undiscovered(_) => "your line has not come to that",
        Rejection::Nowhere(_) => "nowhere on you to put it",
        // The development refusals a hand can actually produce get their own
        // words; the rest say that it did not develop. The exact boundary is
        // in the outcome either way, because PD1b made the refusal order part
        // of the contract and a receipt has to be able to name it.
        Rejection::Refused(refusal) => match refusal {
            Refusal::SiteMismatch { .. } => "that shape does not do that",
            Refusal::Disconnected(_) => "an organ is one piece of tissue",
            Refusal::Overlap { .. } => "that tissue is taken",
            Refusal::SeveredPart(_) => "that branch is gone",
            Refusal::UnknownProcess(_) => "nothing here knows that process",
            Refusal::Stale { .. } => "the body moved under it",
            _ => "it would not develop",
        },
        // P4's lineage verb. A commit refuses for what the *line* has, not for
        // what the body is: it has not come to that, this world does not hold
        // it, or there is no such line at all.
        Rejection::Unrevised(why) => match why {
            Unrevised::Undiscovered(_) => "your line has not come to that",
            Unrevised::Nothing => "nothing here to pass on",
            Unrevised::NoSuchSpecies(_) => "no such line",
            Unrevised::NotYet => "not at this point in the run",
        },
        // DT3's dev intents. They refuse in the same lane every other verb
        // refuses in, because they are ordinary intents and a dev tool that
        // failed silently would be worse than no tool.
        Rejection::EpochNotOnDemand(_) => "this world's epoch does not end on demand",
        Rejection::NotLiving(_) => "that one is not alive",
        Rejection::OffGrid(_) => "that is outside the enclosure",
        Rejection::OverBound { .. } => "more than one placement may carry",
    }
}

/// The first thing worth saying in a batch of outcomes, in plain words.
///
/// First rather than last: a frame's worth of steps can refuse the same intent
/// several times, and the one that arrived is the one worth saying.
///
/// A landed meal counts. Since TD4 the body decides what a meal becomes, so
/// the player is reading a decision rather than confirming one — which makes
/// the difference between "burned" and "grew" the whole feedback for the verb.
/// Nothing new is built for it: the outcome is already here, and these are the
/// same three words the refusals are.
pub fn notice_in(outcomes: &[Outcome]) -> Option<&'static str> {
    outcomes.iter().find_map(|outcome| match outcome {
        Outcome::Rejected(rejection) => Some(refusal_words(rejection)),
        Outcome::Burned { .. } => Some("burned"),
        Outcome::Incorporated { .. } | Outcome::IncorporatedPair { .. } => Some("grew"),
        // PE2's part-level meal. Not "grew": what happened is that a specific
        // organ came off something and onto you, and the player chose which.
        Outcome::Consumed { .. } => Some("took the organ"),
        // PD2's verb. "Rebuilt" rather than "rearranged": what happened to the
        // body is that an organ now does something else, and the tissue it was
        // made of was paid for again.
        Outcome::Expressed { .. } => Some("rebuilt"),
        // P3's branch transfer. The crossing is the fact worth saying: a
        // carried branch arrived as what it was, a regrown one as what this
        // body makes of it.
        Outcome::Grafted {
            crossing: Crossing::Carry,
            ..
        } => Some("carried the branch"),
        Outcome::Grafted {
            crossing: Crossing::Regrow,
            ..
        } => Some("regrew the branch"),
        // DT3's four, said in the same plain words as everything else — a dev
        // action that landed silently would be indistinguishable from one that
        // was quietly dropped at the keyboard.
        Outcome::EpochEnded { .. } => Some("ended the epoch"),
        Outcome::Bore { .. } => Some("forced a birth"),
        Outcome::Killed { .. } => Some("killed it"),
        Outcome::Placed { .. } => Some("placed matter"),
        _ => None,
    })
}

/// The vitals panel.
pub fn vitals_root(vitals: &Vitals) -> VitalsChild {
    let mut children: Vec<VitalsChild> = Vec::new();

    // The catalog's labelled-facts component, which is exactly what a vital
    // sign is: an inert key and its value. Not a hand-rolled row.
    let mut rows = match vitals.energy_mg {
        Some(energy) => vec![DetailRow::new("energy", format!("{energy} mg"))],
        None => vec![DetailRow::new("state", "dead")],
    };
    if let Some(replacement) = &vitals.replacement {
        rows.push(DetailRow::new("replacement", replacement.clone()));
    }
    // Only when the body has one, which is never unless somebody built it.
    // A row that is always on screen saying "no gland" would be decoration.
    if let Some(gland) = &vitals.gland {
        rows.push(DetailRow::new("gland", gland.tissue.clone()));
        rows.push(DetailRow::new("sting", gland.sting.clone()));
        rows.push(DetailRow::new("gland rent", gland.rent.clone()));
    }
    // Three rows, and only once there is one: what you came to, how, and what
    // it lets you build. A discovery is availability, so "grants" says where
    // it could go rather than claiming the body already does it.
    if let Some(discovery) = &vitals.discovery {
        rows.push(DetailRow::new("discovered", discovery.what.clone()));
        rows.push(DetailRow::new("by", discovery.route.clone()));
        rows.push(DetailRow::new("grants", discovery.grants.clone()));
    }
    // Two rows, and only once a branch has come across: whose it was, and what
    // it is doing here. Provenance first, because it is the fact a graft has
    // that growing does not.
    if let Some(graft) = &vitals.graft {
        rows.push(DetailRow::new("branch", graft.taken.clone()));
        rows.push(DetailRow::new("terms", graft.terms.clone()));
    }
    if let Some(observation) = &vitals.observation {
        rows.push(DetailRow::new("last evidence", observation.clone()));
    }
    children.push(Box::new(detail_panel::<Vitals, ()>(&[DetailSection::new(
        "vitals", rows,
    )])));

    // The bar is a box, not a drawing: its fill is a width, and the engine
    // paints it. A dead critter has no bar at all.
    if !vitals.is_dead() {
        let fill = el::<_, Vitals, ()>("div", ())
            .attr("class", "vital-bar-fill")
            .attr(
                "style",
                format!("width: {:.0}px", vitals.fullness * BAR_WIDTH),
            );
        children.push(Box::new(
            el::<_, Vitals, ()>("div", Box::new(fill) as VitalsChild).attr("class", "vital-bar"),
        ));
    }

    if let Some(words) = vitals.notice {
        children.push(Box::new(
            el::<_, Vitals, ()>("div", text(words)).attr("class", "vital-notice"),
        ));
    }

    // Last, and only when it is true. A warning that is always on screen is a
    // decoration.
    if let Some(words) = &vitals.warning {
        children.push(Box::new(
            el::<_, Vitals, ()>("div", text(words.clone())).attr("class", "vital-warning"),
        ));
    }

    Box::new(el::<_, Vitals, ()>("div", children).attr("class", "vitals"))
}

/// The sheet the panel is styled by. Dark and translucent so the section
/// reads through it, and plain: this is a status panel, not a flourish.
pub fn vitals_css() -> &'static str {
    r#"
.vitals {
    /* The host's raster is 300 wide and this is a content box, so the twelve
       pixels of padding on each side come out of it. Setting the two equal
       clipped the last character off every line that reached the edge. */
    width: 276px;
    padding: 10px 12px;
    background-color: #10141aee;
    color: #dfe6dd;
    font-family: sans-serif;
    font-size: 14px;
}
.detail-section-title {
    color: #8fa08c;
    font-size: 12px;
    margin-bottom: 4px;
}
.detail-row { margin-bottom: 2px; }
.detail-key { color: #8fa08c; }
.detail-value { color: #eaf2e6; font-weight: bold; margin-left: 8px; }
.vital-bar {
    width: 168px;
    height: 8px;
    margin-top: 6px;
    background-color: #263026;
}
.vital-bar-fill {
    height: 8px;
    background-color: #7fc46a;
}
.vital-notice {
    margin-top: 8px;
    color: #e2a06a;
    font-size: 13px;
}
.vital-warning {
    margin-top: 8px;
    color: #d8776a;
    font-size: 12px;
}
"#
}

#[cfg(test)]
mod tests;
