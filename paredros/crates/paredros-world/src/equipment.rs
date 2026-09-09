// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Derived attachment readings. An attachment is not a grant of an ability.

use crate::{GameState, ItemId, ItemLocation};
use mesocosm_core::PartId;
use paredros_identity::{BodyRevisionId, SubjectId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AttachmentView {
    pub item: ItemId,
    pub part: PartId,
    pub anatomy_revision: Option<BodyRevisionId>,
    /// False for stale anatomy, lost targets, or an unavailable subject.
    pub current: bool,
}

impl GameState {
    /// Inspect owned attachment facts without changing them or repairing staleness.
    pub fn attachments(&self, subject: SubjectId) -> Vec<AttachmentView> {
        let historical = self.anatomies().get(subject);
        let current = self.current_anatomy(subject).ok();
        self.items()
            .carried_by(subject)
            .filter_map(|item| {
                let ItemLocation::Attached { part, .. } = item.location else {
                    return None;
                };
                Some(AttachmentView {
                    item: item.id,
                    part,
                    anatomy_revision: historical.map(|record| record.revision),
                    current: current.is_some_and(|record| {
                        record.document.part(part).is_some_and(|part| !part.severed)
                    }),
                })
            })
            .collect()
    }
}
