// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A readable projection of the actual body-space part bounds.
//!
//! This is deliberately a diagram, rather than a voxel mesh: each box carries
//! a stable part address and is separated when its projected footprint would
//! hide another part.

use mesocosm_core::{Origin, PartId};
use netrender::{Scene, ScenePath};
use paredros_world::{PartRow, SourceQuery};

use super::{Focus, LifeSheet, SheetView};

pub(super) const LEFT: f32 = 22.;
pub(super) const TOP: f32 = 202.;
pub(super) const RIGHT: f32 = 342.;
pub(super) const BOTTOM: f32 = 650.;

#[derive(Clone, Copy, Debug)]
pub(super) struct PartMark {
    pub id: PartId,
    rect: [f32; 4],
    anchor: [f32; 2],
    displaced: bool,
}

pub(super) fn marks(parts: &[PartRow]) -> Vec<PartMark> {
    // Isometric body axes: four radial limbs remain four separate directions.
    // Project every corner, since two opposing corners do not bound a tilted box.
    let project = |p: [i32; 3]| {
        [
            (p[0] as f32 - p[2] as f32) * 0.866,
            (p[0] as f32 + p[2] as f32) * 0.5 - p[1] as f32,
        ]
    };
    let raw: Vec<_> = parts
        .iter()
        .map(|part| {
            part.bounds.map(|bounds| {
                let mut rect = [
                    f32::INFINITY,
                    f32::INFINITY,
                    f32::NEG_INFINITY,
                    f32::NEG_INFINITY,
                ];
                for mask in 0..8 {
                    let point = std::array::from_fn(|axis| {
                        if mask & (1 << axis) == 0 {
                            bounds.min[axis]
                        } else {
                            bounds.max[axis]
                        }
                    });
                    let p = project(point);
                    rect[0] = rect[0].min(p[0]);
                    rect[1] = rect[1].min(p[1]);
                    rect[2] = rect[2].max(p[0]);
                    rect[3] = rect[3].max(p[1]);
                }
                rect
            })
        })
        .collect();
    let extent = raw.iter().flatten().fold(
        [
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ],
        |mut out, r| {
            out[0] = out[0].min(r[0]);
            out[1] = out[1].min(r[1]);
            out[2] = out[2].max(r[2]);
            out[3] = out[3].max(r[3]);
            out
        },
    );
    let mid = if raw.iter().any(Option::is_some) {
        center(extent)
    } else {
        [0., 0.]
    };
    let scale =
        (230. / (extent[2] - extent[0]).max(1.)).min(270. / (extent[3] - extent[1]).max(1.));
    let origin = [(LEFT + RIGHT) * 0.5, 421.];
    let mut out = Vec::with_capacity(parts.len());
    for (index, part) in parts.iter().enumerate() {
        let Some(raw) = raw[index] else {
            let y = TOP + 18. + index as f32 * 28.;
            out.push(PartMark {
                id: part.id,
                rect: [LEFT + 12., y, LEFT + 116., y + 20.],
                anchor: [LEFT + 12., y],
                displaced: true,
            });
            continue;
        };
        let c = center(raw);
        let anchor = [
            origin[0] + (c[0] - mid[0]) * scale * 1.35,
            origin[1] + (c[1] - mid[1]) * scale * 1.35,
        ];
        let half = [
            ((raw[2] - raw[0]) * scale * 0.29).max(15.),
            ((raw[3] - raw[1]) * scale * 0.29).max(15.),
        ];
        let make_rect = |p: [f32; 2]| {
            [
                p[0] - half[0],
                p[1] - half[1],
                p[0] + half[0],
                p[1] + half[1],
            ]
        };
        let mut rect = make_rect(anchor);
        let mut displaced = false;
        // Keep the nearest free spot instead of sorting overlaps into a sidebar.
        'search: for ring in 0..24 {
            for step in 0..16 {
                let angle = step as f32 * std::f32::consts::TAU / 16.;
                let candidate = make_rect([
                    anchor[0] + angle.cos() * ring as f32 * 7.,
                    anchor[1] + angle.sin() * ring as f32 * 7.,
                ]);
                if candidate[0] < LEFT + 4.
                    || candidate[2] > RIGHT - 4.
                    || candidate[1] < TOP + 18.
                    || candidate[3] > BOTTOM - 40.
                {
                    continue;
                }
                let padded = [
                    candidate[0] - 4.,
                    candidate[1] - 4.,
                    candidate[2] + 4.,
                    candidate[3] + 4.,
                ];
                if out
                    .iter()
                    .all(|other: &PartMark| !overlaps(other.rect, padded))
                {
                    rect = candidate;
                    displaced = ring > 0;
                    break 'search;
                }
            }
        }
        out.push(PartMark {
            id: part.id,
            rect,
            anchor,
            displaced,
        });
    }
    out
}

pub(super) fn hit(parts: &[PartRow], point: [f32; 2]) -> Option<usize> {
    marks(parts)
        .iter()
        .position(|mark| inside(mark.rect, point))
}

pub(super) fn draw(
    scene: &mut Scene,
    life: &LifeSheet,
    view: &SheetView,
    text: &mut super::Text,
    ink: [f32; 4],
) {
    let marks = marks(&life.sheet.parts);
    for mark in &marks {
        let Some(part) = life.sheet.parts.iter().find(|part| part.id == mark.id) else {
            continue;
        };
        let Some(parent) = part.parent else { continue };
        let Some(parent_mark) = marks.iter().find(|other| other.id == parent) else {
            continue;
        };
        line(
            scene,
            center(parent_mark.rect),
            center(mark.rect),
            [0.20, 0.42, 0.46, 0.82],
            1.5,
        );
    }
    for mark in &marks {
        let Some(part) = life.sheet.parts.iter().find(|part| part.id == mark.id) else {
            continue;
        };
        if mark.displaced {
            line(
                scene,
                mark.anchor,
                center(mark.rect),
                [0.42, 0.55, 0.58, 1.],
                1.,
            );
        }
        let selected = view.focus == Focus::Part
            && life
                .sheet
                .parts
                .get(view.part)
                .is_some_and(|item| item.id == part.id);
        let sourced = view.focus == Focus::Action
            && life
                .sheet
                .actions
                .get(view.action)
                .is_some_and(|action| action.sources.contains(&SourceQuery::Part(part.id)));
        let color = if part.severed {
            [0.28, 0.12, 0.14, 0.52]
        } else if matches!(&part.provenance.origin, Origin::Incorporated { .. }) {
            [0.23, 0.16, 0.38, 1.]
        } else {
            [0.10, 0.25, 0.30, 1.]
        };
        scene.push_rect(
            mark.rect[0],
            mark.rect[1],
            mark.rect[2],
            mark.rect[3],
            color,
        );
        if selected || sourced {
            stroke_rect(
                scene,
                mark.rect,
                if selected {
                    [0.42, 0.95, 0.93, 1.]
                } else {
                    [0.72, 0.86, 0.88, 1.]
                },
                if selected { 3. } else { 1.5 },
            );
        }
        if part.severed {
            cross(scene, mark.rect, [1., 0.48, 0.43, 1.]);
        }
        if matches!(&part.provenance.origin, Origin::Incorporated { .. }) {
            diamond(
                scene,
                [mark.rect[2] - 9., mark.rect[1] + 9.],
                [0.96, 0.73, 0.39, 1.],
            );
        }
        text.label(
            scene,
            &format!("#{}", part.id.0),
            [mark.rect[0] + 4., mark.rect[1] + 7.],
            12.,
            ink,
            (mark.rect[2] - mark.rect[0] - 8.).max(20.),
        );
    }
    text.label(
        scene,
        "cross: severed   diamond: incorporated",
        [LEFT + 5., BOTTOM - 13.],
        11.,
        [0.96, 0.73, 0.55, 1.],
        312.,
    );
    text.label(
        scene,
        "Exploded body plan / relative sizes / L: named list",
        [LEFT + 5., BOTTOM + 3.],
        11.,
        [0.62, 0.73, 0.76, 1.],
        312.,
    );
}

fn overlaps(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[2] && a[2] > b[0] && a[1] < b[3] && a[3] > b[1]
}
fn inside(rect: [f32; 4], point: [f32; 2]) -> bool {
    (rect[0]..rect[2]).contains(&point[0]) && (rect[1]..rect[3]).contains(&point[1])
}
fn center(rect: [f32; 4]) -> [f32; 2] {
    [(rect[0] + rect[2]) * 0.5, (rect[1] + rect[3]) * 0.5]
}
fn line(scene: &mut Scene, from: [f32; 2], to: [f32; 2], color: [f32; 4], width: f32) {
    let mut path = ScenePath::new();
    path.move_to(from[0], from[1]).line_to(to[0], to[1]);
    scene.push_shape_stroked(path, color, width);
}
fn stroke_rect(scene: &mut Scene, rect: [f32; 4], color: [f32; 4], width: f32) {
    let mut path = ScenePath::new();
    path.move_to(rect[0], rect[1])
        .line_to(rect[2], rect[1])
        .line_to(rect[2], rect[3])
        .line_to(rect[0], rect[3])
        .close();
    scene.push_shape_stroked(path, color, width);
}
fn cross(scene: &mut Scene, rect: [f32; 4], color: [f32; 4]) {
    line(
        scene,
        [rect[0] + 4., rect[1] + 4.],
        [rect[2] - 4., rect[3] - 4.],
        color,
        2.,
    );
    line(
        scene,
        [rect[2] - 4., rect[1] + 4.],
        [rect[0] + 4., rect[3] - 4.],
        color,
        2.,
    );
}
fn diamond(scene: &mut Scene, at: [f32; 2], color: [f32; 4]) {
    let mut path = ScenePath::new();
    path.move_to(at[0], at[1] - 5.)
        .line_to(at[0] + 5., at[1])
        .line_to(at[0], at[1] + 5.)
        .line_to(at[0] - 5., at[1])
        .close();
    scene.push_shape_filled(path, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mesocosm_core::Aabb;
    use paredros_world::{SubjectSheet, SubjectSheetInput, fixtures::three_lives as fixture};

    #[test]
    fn placed_parts_are_all_hit_testable_even_when_projected_together() {
        let life = &fixture::three_lives()[2];
        let sheet = SubjectSheet::from_input(SubjectSheetInput {
            subject: life.subject,
            revision: life.revision,
            current_revision: life.revision,
            body: &life.body,
            knowledge: &fixture::knowledge(life),
            inputs: &fixture::inputs(life),
            part_names: fixture::PART_NAMES,
            selected_part: None,
        });
        let marks = marks(&sheet.parts);
        assert_eq!(marks.len(), sheet.parts.len());
        let lives = [LifeSheet {
            label: life.name,
            summary: life.ordinary_task,
            sheet: sheet.clone(),
        }];
        for mark in &marks {
            let mut view = SheetView::default();
            view.click(&lives, center(mark.rect));
            assert_eq!(lives[0].sheet.parts[view.part].id, mark.id);
            assert_eq!(
                hit(&sheet.parts, center(mark.rect)),
                Some(
                    sheet
                        .parts
                        .iter()
                        .position(|part| part.id == mark.id)
                        .unwrap()
                )
            );
        }
    }

    #[test]
    fn missing_geometry_keeps_each_address_selectable_without_inventing_a_shape() {
        let life = &fixture::three_lives()[0];
        let mut sheet = SubjectSheet::from_input(SubjectSheetInput {
            subject: life.subject,
            revision: life.revision,
            current_revision: life.revision,
            body: &life.body,
            knowledge: &fixture::knowledge(life),
            inputs: &fixture::inputs(life),
            part_names: fixture::PART_NAMES,
            selected_part: None,
        });
        for part in &mut sheet.parts {
            part.bounds = None;
        }
        let marks = marks(&sheet.parts);
        assert_eq!(marks.len(), sheet.parts.len());
        for mark in &marks {
            assert!(hit(&sheet.parts, center(mark.rect)).is_some());
        }
    }

    #[test]
    fn large_bounds_fit_the_diagram_before_minimum_touch_targets_are_applied() {
        let life = &fixture::three_lives()[0];
        let mut sheet = SubjectSheet::from_input(SubjectSheetInput {
            subject: life.subject,
            revision: life.revision,
            current_revision: life.revision,
            body: &life.body,
            knowledge: &fixture::knowledge(life),
            inputs: &fixture::inputs(life),
            part_names: fixture::PART_NAMES,
            selected_part: None,
        });
        sheet.parts[0].bounds = Some(Aabb {
            min: [-20_000, -20_000, -20_000],
            max: [20_000, 20_000, 20_000],
        });
        for mark in marks(&sheet.parts)
            .into_iter()
            .filter(|mark| !mark.displaced)
        {
            assert!(mark.rect[0] >= LEFT && mark.rect[2] <= RIGHT);
            assert!(mark.rect[1] >= TOP && mark.rect[3] <= BOTTOM);
        }
    }
}
