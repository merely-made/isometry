// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Immutable, revision-addressed body snapshots admitted for Paredros.
//!
//! An admitted record is evidence about one body revision, not body authority.
//! Injury can make it stale; a later transition must admit or reconcile the
//! replacement revision. Admission deliberately has no UI, equipment, or body
//! mutation powers.

use std::collections::{BTreeMap, BTreeSet};

use mesocosm_core::{BodyDocument, PartId, Yaw};
use paredros_identity::{BodyRevisionId, SubjectId};
use serde::{Deserialize, Serialize};

/// A deliberately small bound: the authored three-lives fixture has seven
/// parts, while 256 keeps validation and every parent walk bounded.
pub const MAX_ANATOMY_PARTS: usize = 256;
/// Per-field coordinate bound for an externally supplied body document.
pub const MAX_ANATOMY_COORDINATE: i32 = 1_000_000;
/// A root-to-leaf pivot may accumulate one bounded offset per admitted part.
pub const MAX_ANATOMY_WORLD_COORDINATE: i32 = 256_000_000;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnatomyRecord {
    pub subject: SubjectId,
    pub revision: BodyRevisionId,
    pub document: BodyDocument,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Anatomies {
    records: BTreeMap<SubjectId, AnatomyRecord>,
}

/// Rejections are intentionally coarse: callers can choose their presentation
/// without depending on a deserialization detail of Mesocosm's body types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnatomyError {
    EmptyDocument,
    TooManyParts,
    MissingRoot,
    RootHasParent,
    NonContiguousPartIds,
    MissingAttachment,
    MissingParent,
    CycleOrDisconnected,
    SeveredRoot,
    LivingChildOfSeveredParent,
    NonPositiveHalfExtent,
    PivotOutsidePart,
    CoordinateOutOfBounds,
    TransformOverflow,
    MassOverflow,
    DuplicateSubject,
    Missing(SubjectId),
    DuplicatePart(PartId),
    MissingPart(PartId),
    RootPart(PartId),
    SeveredPart(PartId),
    StaleRevision {
        known: BodyRevisionId,
        current: BodyRevisionId,
    },
    RevisionNotAdvanced {
        from_revision: BodyRevisionId,
        revision: BodyRevisionId,
    },
}

impl Anatomies {
    pub fn get(&self, subject: SubjectId) -> Option<&AnatomyRecord> {
        self.records.get(&subject)
    }

    /// Adds exactly one validated snapshot for a subject. Validation finishes
    /// before the map changes, so a rejected document cannot leave a partial
    /// record behind.
    pub(crate) fn admit(
        &mut self,
        subject: SubjectId,
        revision: BodyRevisionId,
        document: BodyDocument,
    ) -> Result<(), AnatomyError> {
        if self.records.contains_key(&subject) {
            return Err(AnatomyError::DuplicateSubject);
        }
        validate(&document)?;
        self.records.insert(
            subject,
            AnatomyRecord {
                subject,
                revision,
                document,
            },
        );
        Ok(())
    }

    /// Carries an admitted anatomy forward through a summary-body injury.
    ///
    /// The detailed record is a tombstoned copy of the preceding revision:
    /// this does not manufacture a location, volume, attachment, or origin
    /// for any injury the summary did not identify.
    pub(crate) fn reconcile(
        &mut self,
        subject: SubjectId,
        from_revision: BodyRevisionId,
        revision: BodyRevisionId,
        severed_parts: &[PartId],
    ) -> Result<(), AnatomyError> {
        let stored = self
            .records
            .get(&subject)
            .ok_or(AnatomyError::Missing(subject))?;
        if stored.revision != from_revision {
            return Err(AnatomyError::StaleRevision {
                known: from_revision,
                current: stored.revision,
            });
        }
        if revision <= from_revision {
            return Err(AnatomyError::RevisionNotAdvanced {
                from_revision,
                revision,
            });
        }
        if severed_parts.len() > MAX_ANATOMY_PARTS {
            return Err(AnatomyError::TooManyParts);
        }

        let mut requested = BTreeSet::new();
        for &part in severed_parts {
            if !requested.insert(part) {
                return Err(AnatomyError::DuplicatePart(part));
            }
            let found = stored
                .document
                .part(part)
                .ok_or(AnatomyError::MissingPart(part))?;
            if part == stored.document.root {
                return Err(AnatomyError::RootPart(part));
            }
            if found.severed {
                return Err(AnatomyError::SeveredPart(part));
            }
        }

        let mut replacement = stored.clone();
        for &part in severed_parts {
            replacement.document.sever(part);
        }
        validate(&replacement.document)?;
        replacement.revision = revision;
        self.records.insert(subject, replacement);
        Ok(())
    }
}

fn validate(document: &BodyDocument) -> Result<(), AnatomyError> {
    if document.parts.is_empty() {
        return Err(AnatomyError::EmptyDocument);
    }
    if document.parts.len() > MAX_ANATOMY_PARTS {
        return Err(AnatomyError::TooManyParts);
    }
    if document.part(document.root).is_none() {
        return Err(AnatomyError::MissingRoot);
    }
    for (index, part) in document.parts.iter().enumerate() {
        if part.id != PartId(index as u32) {
            return Err(AnatomyError::NonContiguousPartIds);
        }
    }
    let root = document.part(document.root).expect("root checked above");
    if root.attachment.is_some() {
        return Err(AnatomyError::RootHasParent);
    }
    if root.severed {
        return Err(AnatomyError::SeveredRoot);
    }
    for part in &document.parts {
        if part.id != document.root && part.attachment.is_none() {
            return Err(AnatomyError::MissingAttachment);
        }
        if let Some(attachment) = part.attachment
            && document.part(attachment.parent).is_none()
        {
            return Err(AnatomyError::MissingParent);
        }
    }
    for part in &document.parts {
        let mut cursor = part.id;
        for _ in 0..document.parts.len() {
            if cursor == document.root {
                break;
            }
            cursor = document
                .part(cursor)
                .and_then(|found| found.attachment)
                .map(|attachment| attachment.parent)
                .ok_or(AnatomyError::CycleOrDisconnected)?;
        }
        if cursor != document.root {
            return Err(AnatomyError::CycleOrDisconnected);
        }
    }

    let mut total_mass_mg = 0u64;
    for part in &document.parts {
        total_mass_mg = total_mass_mg
            .checked_add(part.mass_mg)
            .ok_or(AnatomyError::MassOverflow)?;
        if let Some(attachment) = part.attachment
            && !part.severed
            && document
                .part(attachment.parent)
                .is_some_and(|parent| parent.severed)
        {
            return Err(AnatomyError::LivingChildOfSeveredParent);
        }
        for axis in 0..3 {
            let half = part.half_extent[axis];
            if half < 1 {
                return Err(AnatomyError::NonPositiveHalfExtent);
            }
            if half > MAX_ANATOMY_COORDINATE {
                return Err(AnatomyError::CoordinateOutOfBounds);
            }
            let maximum_pivot = half.checked_mul(2).ok_or(AnatomyError::TransformOverflow)?;
            if !(0..=maximum_pivot).contains(&part.pivot[axis]) {
                return Err(AnatomyError::PivotOutsidePart);
            }
        }
        if let Some(attachment) = part.attachment {
            for offset in attachment.offset {
                if offset.unsigned_abs() > MAX_ANATOMY_COORDINATE as u32 {
                    return Err(AnatomyError::CoordinateOutOfBounds);
                }
            }
        }
        checked_world_pivot(document, part.id)?;
    }
    Ok(())
}

fn checked_world_pivot(document: &BodyDocument, id: PartId) -> Result<[i32; 3], AnatomyError> {
    let mut offset = [0; 3];
    let mut cursor = id;
    for _ in 0..document.parts.len() {
        let part = document.part(cursor).ok_or(AnatomyError::MissingParent)?;
        let Some(attachment) = part.attachment else {
            return Ok(offset);
        };
        let rotated = checked_rotate(attachment.yaw, offset)?;
        for axis in 0..3 {
            offset[axis] = rotated[axis]
                .checked_add(attachment.offset[axis])
                .ok_or(AnatomyError::TransformOverflow)?;
            if offset[axis].unsigned_abs() > MAX_ANATOMY_WORLD_COORDINATE as u32 {
                return Err(AnatomyError::CoordinateOutOfBounds);
            }
        }
        cursor = attachment.parent;
    }
    Err(AnatomyError::CycleOrDisconnected)
}

fn checked_rotate(yaw: Yaw, [x, y, z]: [i32; 3]) -> Result<[i32; 3], AnatomyError> {
    let neg = |value: i32| value.checked_neg().ok_or(AnatomyError::TransformOverflow);
    match yaw {
        Yaw::Zero => Ok([x, y, z]),
        Yaw::Quarter => Ok([z, y, neg(x)?]),
        Yaw::Half => Ok([neg(x)?, y, neg(z)?]),
        Yaw::ThreeQuarter => Ok([neg(z)?, y, x]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::three_lives::wetland_body;
    use mesocosm_core::{Attachment, SpeciesId, VolumeRef};

    const SUBJECT: SubjectId = SubjectId(4);

    #[test]
    fn admits_the_existing_severable_fixture_once() {
        let mut anatomies = Anatomies::default();
        anatomies
            .admit(SUBJECT, BodyRevisionId(2), wetland_body())
            .expect("fixture is an admissible immutable snapshot");
        let record = anatomies.get(SUBJECT).expect("stored record");
        assert_eq!(record.revision, BodyRevisionId(2));
        assert_eq!(record.document.parts.len(), 7);
        assert_eq!(
            anatomies.admit(SUBJECT, BodyRevisionId(3), wetland_body()),
            Err(AnatomyError::DuplicateSubject)
        );
        assert_eq!(anatomies.get(SUBJECT).unwrap().revision, BodyRevisionId(2));
    }

    #[test]
    fn rejects_malformed_graphs_before_storing_them() {
        let mut empty = wetland_body();
        empty.parts.clear();
        assert_rejected(empty, AnatomyError::EmptyDocument);

        let mut no_parent = wetland_body();
        no_parent.parts[1].attachment = None;
        assert_rejected(no_parent, AnatomyError::MissingAttachment);

        let mut missing_parent = wetland_body();
        missing_parent.parts[1].attachment.as_mut().unwrap().parent = PartId(99);
        assert_rejected(missing_parent, AnatomyError::MissingParent);

        let mut cycle = wetland_body();
        cycle.parts[1].attachment.as_mut().unwrap().parent = PartId(2);
        cycle.parts[2].attachment.as_mut().unwrap().parent = PartId(1);
        assert_rejected(cycle, AnatomyError::CycleOrDisconnected);
    }

    #[test]
    fn rejects_bad_geometry_without_calling_core_transforms() {
        let mut flat = wetland_body();
        flat.parts[0].half_extent[0] = 0;
        assert_rejected(flat, AnatomyError::NonPositiveHalfExtent);

        let mut pivot = wetland_body();
        pivot.parts[0].pivot[0] = 5;
        assert_rejected(pivot, AnatomyError::PivotOutsidePart);

        let mut huge_offset = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1, [1, 1, 1]);
        huge_offset
            .attach(
                VolumeRef::from_tag(2),
                1,
                [1, 1, 1],
                Attachment {
                    parent: PartId(0),
                    offset: [i32::MAX, 0, 0],
                    yaw: Yaw::Zero,
                },
                mesocosm_core::Provenance::founding(),
            )
            .unwrap();
        assert_rejected(huge_offset, AnatomyError::CoordinateOutOfBounds);
    }

    #[test]
    fn rejects_overflowing_mass_and_incoherent_tombstones() {
        let mut overflow = wetland_body();
        overflow.parts[0].mass_mg = u64::MAX;
        overflow.parts[1].mass_mg = 1;
        assert_rejected(overflow, AnatomyError::MassOverflow);

        let mut severed_root = wetland_body();
        severed_root.parts[0].severed = true;
        assert_rejected(severed_root, AnatomyError::SeveredRoot);

        let mut living_child = wetland_body();
        living_child.parts[1].severed = true;
        living_child.parts[2].attachment.as_mut().unwrap().parent = PartId(1);
        assert_rejected(living_child, AnatomyError::LivingChildOfSeveredParent);
    }

    #[test]
    fn reconciliation_preserves_identity_and_provenance_except_tombstones() {
        let mut anatomies = admitted(BodyRevisionId(2), wetland_body());
        let mut expected = wetland_body();
        expected.sever(PartId(1));

        anatomies
            .reconcile(SUBJECT, BodyRevisionId(2), BodyRevisionId(3), &[PartId(1)])
            .expect("current detailed body reconciles the known loss");

        let record = anatomies.get(SUBJECT).unwrap();
        assert_eq!(record.subject, SUBJECT);
        assert_eq!(record.revision, BodyRevisionId(3));
        assert_eq!(record.document, expected);
        assert_eq!(
            record.document.part(PartId(1)).unwrap().provenance,
            mesocosm_core::Provenance::founding()
        );
    }

    #[test]
    fn reconciliation_rejects_stale_and_nonadvancing_revisions_atomically() {
        let mut anatomies = admitted(BodyRevisionId(2), wetland_body());
        let before = anatomies.clone();
        assert_eq!(
            anatomies.reconcile(SUBJECT, BodyRevisionId(1), BodyRevisionId(3), &[]),
            Err(AnatomyError::StaleRevision {
                known: BodyRevisionId(1),
                current: BodyRevisionId(2),
            })
        );
        assert_eq!(anatomies, before);
        assert_eq!(
            anatomies.reconcile(SUBJECT, BodyRevisionId(2), BodyRevisionId(2), &[]),
            Err(AnatomyError::RevisionNotAdvanced {
                from_revision: BodyRevisionId(2),
                revision: BodyRevisionId(2),
            })
        );
        assert_eq!(anatomies, before);
    }

    #[test]
    fn reconciliation_rejects_bad_requested_parts_atomically() {
        for (document, requested, expected) in [
            (
                wetland_body(),
                vec![PartId(1), PartId(1)],
                AnatomyError::DuplicatePart(PartId(1)),
            ),
            (
                wetland_body(),
                vec![PartId(99)],
                AnatomyError::MissingPart(PartId(99)),
            ),
            (
                wetland_body(),
                vec![PartId(0)],
                AnatomyError::RootPart(PartId(0)),
            ),
            (
                {
                    let mut document = wetland_body();
                    document.parts[1].severed = true;
                    document
                },
                vec![PartId(1)],
                AnatomyError::SeveredPart(PartId(1)),
            ),
        ] {
            let mut anatomies = admitted(BodyRevisionId(2), document);
            let before = anatomies.clone();
            assert_eq!(
                anatomies.reconcile(SUBJECT, BodyRevisionId(2), BodyRevisionId(3), &requested),
                Err(expected)
            );
            assert_eq!(anatomies, before, "rejection leaves the record intact");
        }
    }

    #[test]
    fn empty_reconciliation_explicitly_refreshes_without_inventing_a_loss() {
        let document = wetland_body();
        let mut anatomies = admitted(BodyRevisionId(2), document.clone());
        anatomies
            .reconcile(SUBJECT, BodyRevisionId(2), BodyRevisionId(3), &[])
            .expect("an injury may not identify a detailed part");
        let record = anatomies.get(SUBJECT).unwrap();
        assert_eq!(record.revision, BodyRevisionId(3));
        assert_eq!(record.document, document);
    }

    #[test]
    fn ancestor_and_descendant_requests_are_deterministic() {
        let mut document = wetland_body();
        let child = document
            .attach(
                VolumeRef::from_tag(9),
                100,
                [1, 1, 1],
                Attachment {
                    parent: PartId(1),
                    offset: [2, 0, 0],
                    yaw: Yaw::Zero,
                },
                mesocosm_core::Provenance::founding(),
            )
            .unwrap();
        let mut anatomies = admitted(BodyRevisionId(2), document);
        anatomies
            .reconcile(
                SUBJECT,
                BodyRevisionId(2),
                BodyRevisionId(3),
                &[PartId(1), child],
            )
            .expect("the descendant is valid against the original document");
        let document = &anatomies.get(SUBJECT).unwrap().document;
        assert!(document.part(PartId(1)).unwrap().severed);
        assert!(document.part(child).unwrap().severed);
    }

    fn admitted(revision: BodyRevisionId, document: BodyDocument) -> Anatomies {
        let mut anatomies = Anatomies::default();
        anatomies.admit(SUBJECT, revision, document).unwrap();
        anatomies
    }

    fn assert_rejected(document: BodyDocument, expected: AnatomyError) {
        let mut anatomies = Anatomies::default();
        assert_eq!(
            anatomies.admit(SUBJECT, BodyRevisionId(1), document),
            Err(expected)
        );
        assert!(anatomies.get(SUBJECT).is_none(), "rejection is atomic");
    }
}
