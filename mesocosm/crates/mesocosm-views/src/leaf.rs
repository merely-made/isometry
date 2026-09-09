// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The minimap paint leaf: a solved hulls scene, realized.
//!
//! A sprigging [`Leaf`], so any cambium host can splice it into a paint list
//! and any test can read its commands back without a GPU. It paints what it is
//! handed and computes nothing: cells come solved from `scenomise`, tints come
//! resolved from the adapter, and the leaf's only arithmetic is mapping scene
//! units onto its own pixels.

use sceno::{Footprint, Scene, Vec2};
use sprigging::{ColorF, Leaf, PaintCx, Path, Size, SizeHint, solid_stroke};

/// How much of a cell's tint survives as fill. The boundary keeps full
/// strength; the interior stays translucent so a backdrop reads through,
/// which is what lets the world's own image sit underneath.
const FILL_ALPHA: f32 = 0.35;

/// The played critter's marker radius, in leaf pixels.
const MARKER_RADIUS: f32 = 4.0;

/// A realized minimap: regions tinted by holder, sites dotted, the player
/// marked.
pub struct MinimapLeaf {
    scene: Scene,
    /// One tint per scene region, adapter-resolved, in region order. `None`
    /// is an unheld region, painted as boundary only.
    tints: Vec<Option<ColorF>>,
    /// The played critter's world x/z, if anyone is embodied.
    player: Option<(f32, f32)>,
    size: Size,
    dirty: bool,
}

impl MinimapLeaf {
    pub fn new(scene: Scene, tints: Vec<Option<ColorF>>, player: Option<(f32, f32)>) -> Self {
        Self {
            scene,
            tints,
            player,
            size: Size {
                width: 160.0,
                height: 160.0,
            },
            dirty: true,
        }
    }

    /// Replaces the projection wholesale. The next paint repaints everything;
    /// diffing a nine-region minimap would cost more than it saves.
    pub fn update(&mut self, scene: Scene, tints: Vec<Option<ColorF>>, player: Option<(f32, f32)>) {
        if self.scene == scene && self.tints == tints && self.player == player {
            return;
        }
        self.scene = scene;
        self.tints = tints;
        self.player = player;
        self.dirty = true;
    }

    /// Adopts a freshly assembled projection, keeping this leaf's paint
    /// retention: identical projections stay settled.
    pub fn refresh_from(&mut self, fresh: MinimapLeaf) {
        self.update(fresh.scene, fresh.tints, fresh.player);
    }

    /// Scene units to leaf pixels: fit the scene bounds into the leaf,
    /// preserving aspect.
    fn to_px(&self, p: Vec2) -> (f32, f32) {
        let bounds = self.scene.bounds;
        let scale = (self.size.width / bounds.size.w.max(1.0))
            .min(self.size.height / bounds.size.h.max(1.0));
        (
            (p.x - bounds.origin.x) * scale,
            (p.y - bounds.origin.y) * scale,
        )
    }

    fn polygon_path(&self, points: &[Vec2]) -> Option<sprigging::PathData> {
        let (first, rest) = points.split_first()?;
        let (x, y) = self.to_px(*first);
        let mut path = Path::new().move_to(x, y);
        for p in rest {
            let (x, y) = self.to_px(*p);
            path = path.line_to(x, y);
        }
        Some(path.close().build())
    }
}

impl Leaf for MinimapLeaf {
    fn measure(&mut self, known: SizeHint, available: SizeHint) -> Size {
        // Square, sized to whatever the host offers, defaulting modest. A
        // minimap earns corner space, not a panel.
        let side = known
            .width
            .or(known.height)
            .or_else(|| {
                available
                    .width
                    .map(|w| w.min(available.height.unwrap_or(w)))
            })
            .unwrap_or(160.0);
        self.size = Size {
            width: side,
            height: side,
        };
        self.size
    }

    fn paint(&mut self, cx: &mut PaintCx<'_>) {
        self.size = cx.size();

        for (index, region) in self.scene.regions.iter().enumerate() {
            let Some(Footprint::Polygon { points }) = &region.contour else {
                continue;
            };
            let Some(path) = self.polygon_path(points) else {
                continue;
            };
            let tint = self.tints.get(index).copied().flatten();
            let fill = tint.map(|t| ColorF::new(t.r, t.g, t.b, t.a * FILL_ALPHA));
            let edge = tint.unwrap_or(ColorF::new(0.5, 0.5, 0.5, 1.0));
            cx.path(path, fill, Some(solid_stroke(edge, 1.0)));
        }

        // Sites, over the cells they own.
        for item in &self.scene.items {
            if !item.visible {
                continue;
            }
            let (x, y) = self.to_px(Vec2::new(
                item.transform.translate.x,
                item.transform.translate.y,
            ));
            cx.fill_path(Path::circle(x, y, 1.5), ColorF::new(0.85, 0.85, 0.85, 0.9));
        }

        // The player, over everything: the one mark that answers "where am I".
        if let Some((wx, wz)) = self.player {
            let (x, y) = self.to_px(Vec2::new(wx, wz));
            cx.fill_path(
                Path::circle(x, y, MARKER_RADIUS),
                ColorF::new(1.0, 1.0, 1.0, 1.0),
            );
        }

        self.dirty = false;
    }

    fn paint_dirty(&self) -> bool {
        self.dirty
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::minimap::minimap_leaf;
    use mesocosm_core::World;
    use sprigging::PaintCmd;

    fn leaf() -> MinimapLeaf {
        minimap_leaf(&World::new(4_242, 40))
    }

    fn painted(leaf: &mut MinimapLeaf) -> Vec<PaintCmd> {
        let mut cmds = Vec::new();
        let mut cx = PaintCx::new(
            &mut cmds,
            Size {
                width: 160.0,
                height: 160.0,
            },
        );
        leaf.paint(&mut cx);
        cmds
    }

    #[test]
    fn a_minimap_paints_its_regions_sites_and_player() {
        let mut leaf = leaf();
        let region_paths = leaf.scene.regions.len();
        let site_paths = leaf.scene.items.iter().filter(|item| item.visible).count();
        assert!(leaf.player.is_some(), "the paint fixture is embodied");
        let cmds = painted(&mut leaf);

        let paths = cmds
            .iter()
            .filter(|cmd| matches!(cmd, PaintCmd::DrawPath(_)))
            .count();
        assert_eq!(
            paths,
            region_paths + site_paths + 1,
            "got {paths} path draws for {region_paths} regions, {site_paths} sites, and one player"
        );
    }

    #[test]
    fn painting_settles_and_updating_dirties() {
        let mut leaf = leaf();
        assert!(leaf.paint_dirty(), "a new leaf wants a first paint");
        painted(&mut leaf);
        assert!(!leaf.paint_dirty(), "painted is settled");

        let same = (leaf.scene.clone(), leaf.tints.clone(), leaf.player);
        leaf.update(same.0, same.1, same.2);
        assert!(
            !leaf.paint_dirty(),
            "an identical projection repaints nothing"
        );

        leaf.update(leaf.scene.clone(), leaf.tints.clone(), Some((99.0, 99.0)));
        assert!(leaf.paint_dirty(), "a moved player repaints");
    }

    #[test]
    fn the_painted_map_is_deterministic() {
        // PaintCmd carries no PartialEq, so the streams compare as debug text,
        // which is exact enough for identical inputs.
        let mut a = leaf();
        let mut b = leaf();
        assert_eq!(
            format!("{:?}", painted(&mut a)),
            format!("{:?}", painted(&mut b))
        );
    }
}
