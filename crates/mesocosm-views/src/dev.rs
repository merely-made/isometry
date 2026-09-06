// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The dev lane's surface: what the host is doing to time, and what the critter
//! under the camera is. (DT1, DT2)
//!
//! Same posture as [`crate::vitals`] and [`crate::succession`]: a **reading**,
//! taken fresh from facts the host already holds. Nothing here is stored in the
//! world, enters the trace, or reaches the state hash — the dev tools plan's
//! second principle is what makes that true by construction. Pause, step, speed
//! and *follow* are host state over `mesocosm_runtime::Runtime`, never a second
//! authority over the world, so this panel has nothing to say that a replay
//! could ever disagree with.
//!
//! DT1's four facts are the top section. DT2's inspector is the second one, and
//! every line of it is a core query put into words — see [`follow`], where that
//! discipline is spelled out and the two readings DT2 had to add to core are
//! named.

use cambium::{AnyView, DetailRow, DetailSection, GenetCtx, GenetElement, detail_panel, el, text};

pub mod follow;
pub mod part;

pub use follow::{
    Follow, Lost, MAX_DISCOVERY_NAMES, MAX_PART_ROWS, follow_of, lost_of, lost_words, role_word,
};
pub use part::{PartInspection, PartReading, part_of};

pub type DevChild = Box<dyn AnyView<Dev, (), GenetCtx, GenetElement>>;

/// What the dev lane shows. Whether the clock is running, at what multiplier,
/// the world's own tick, how many of those ticks this session took through the
/// step keys rather than the clock — and, since DT2, the critter the camera is
/// following.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Dev {
    /// `false` while the pause key is holding the clock's elapsed time at
    /// zero — see [`crate`] module docs and the host's `advance`.
    pub running: bool,
    /// The clock's speed multiplier, already in the words the ladder shows:
    /// `"1/4"`, `"1/2"`, `"1"`, `"2"`, `"4"`.
    pub speed: &'static str,
    /// The world's own tick count: `Runtime::advance`/`::step`'s returns,
    /// summed. The same number the receipt calls `steps`.
    pub tick: u64,
    /// Ticks taken through the step or step-N keys this session. Tracked by
    /// the host and never by the world: a fact about how this run was
    /// driven, not about what it did — so it is not in the trace either.
    pub manual_steps: u64,
    /// The critter under the camera, read off core. `None` when nobody is
    /// embodied and no follow target stands. (DT2)
    pub follow: Option<Follow>,
    /// A followed critter that stopped being one, kept after follow snapped
    /// back so the death is reported rather than dropped. (DT2)
    pub lost: Option<Lost>,
    pub inspection: Option<PartInspection>,
}

/// The dev panel: the catalog's labelled-facts component, exactly as
/// [`crate::vitals::vitals_root`] uses it for its own top section.
pub fn dev_root(dev: &Dev) -> DevChild {
    let mut sections = vec![DetailSection::new(
        "dev",
        vec![
            DetailRow::new("state", if dev.running { "running" } else { "paused" }),
            DetailRow::new("speed", dev.speed),
            DetailRow::new("tick", dev.tick.to_string()),
            DetailRow::new("stepped", dev.manual_steps.to_string()),
        ],
    )];
    if let Some(inspection) = &dev.inspection {
        sections.push(DetailSection::new(
            "part inspection (dev truth)",
            inspection_rows(inspection),
        ));
    } else if let Some(follow) = &dev.follow {
        sections.push(DetailSection::new("follow", follow_rows(follow)));
    }

    let panel: DevChild = Box::new(detail_panel::<Dev, ()>(&sections));
    let mut children: Vec<DevChild> = vec![panel];
    // Last, and only when it is true — the vitals panel's own rule for a
    // notice. A followed critter's death outlives the follow it ended.
    if let Some(lost) = dev.lost {
        children.push(Box::new(
            el::<_, Dev, ()>("div", text(follow::lost_words(lost))).attr("class", "dev-notice"),
        ));
    }
    Box::new(el::<_, Dev, ()>("div", children).attr("class", "dev"))
}

fn inspection_rows(inspection: &PartInspection) -> Vec<DetailRow> {
    let keys = || DetailRow::new("keys", "I close, J/L part, N/B follow, U clear");
    let Some(reading) = &inspection.reading else {
        return vec![
            DetailRow::new(
                "selection",
                inspection.notice.as_deref().unwrap_or("unknown"),
            ),
            keys(),
        ];
    };
    vec![
        DetailRow::new("organism", reading.organism.clone()),
        DetailRow::new("part", reading.id.clone()),
        DetailRow::new("role", reading.role.clone()),
        DetailRow::new("process", reading.process.clone()),
        DetailRow::new("intake", reading.intake.clone()),
        DetailRow::new("diet", reading.feeding.clone()),
        DetailRow::new("condition", reading.condition.clone()),
        DetailRow::new("discovered", reading.discovery_condition.clone()),
        DetailRow::new("history", reading.history_event.clone()),
        DetailRow::new("lineage", reading.lineage.clone()),
        DetailRow::new("donor", reading.donor.clone()),
        keys(),
    ]
}

/// The inspector's rows, in the order a reader wants them: who and where
/// first, then what it holds and what it is spending, then what its line has,
/// then the body itself.
///
/// The body comes last because it is the part that truncates: everything above
/// it is a fixed number of rows and is therefore never the thing a long body
/// pushes off the tile.
fn follow_rows(follow: &Follow) -> Vec<DetailRow> {
    let mut rows = vec![
        DetailRow::new("id", follow.id.clone()),
        DetailRow::new("species", follow.species.clone()),
        DetailRow::new("at", follow.at.clone()),
        // The two accounts a body holds, in the flow record's own words.
        DetailRow::new("reserve", follow.reserve.clone()),
        DetailRow::new("substance", follow.substance.clone()),
        DetailRow::new("flows", follow.flows.clone()),
        DetailRow::new("window", follow.window.clone()),
        DetailRow::new("revision", follow.revision.clone()),
        DetailRow::new("discovered", follow.discovered.clone()),
        DetailRow::new("parts", follow.parts.clone()),
    ];
    for (key, value) in &follow.part_rows {
        rows.push(DetailRow::new(key.clone(), value.clone()));
    }
    if follow.more_parts > 0 {
        rows.push(DetailRow::new(
            "more",
            format!("+{} parts", follow.more_parts),
        ));
    }
    rows
}

/// The sheet the panel is styled by. Smaller type than the vitals panel's: this
/// is an inspector with a dozen rows in it, and it is read while stopped rather
/// than in motion.
pub fn dev_css() -> &'static str {
    r#"
.dev {
    width: 276px;
    padding: 10px 12px;
    background-color: #10141aee;
    color: #dfe6dd;
    font-family: sans-serif;
    font-size: 12px;
}
.detail-section-title {
    color: #8fa08c;
    font-size: 11px;
    margin-bottom: 3px;
}
.detail-row { margin-bottom: 1px; }
.detail-key { color: #8fa08c; }
.detail-value { color: #eaf2e6; font-weight: bold; margin-left: 6px; }
.dev-notice {
    margin-top: 6px;
    color: #e2a06a;
    font-size: 12px;
}
"#
}

#[cfg(test)]
mod tests;
