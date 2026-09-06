// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The dev lane, hosted. (DT1, DT2)
//!
//! Same arrangement as [`crate::vitals`] and [`crate::succession`]:
//! [`mesocosm_views::dev_root`] diffed into a `ScriptedDom`, Livery resolving
//! the sheet and laying the tree out, the paint list lowered through
//! `paint_list_render`, [`crate::chrome`] rasterizing and blending it over the
//! frame. No text rendering lives here or anywhere in this repo; words are
//! Livery's and the sentences are `mesocosm-views`'.
//!
//! Live only while `--dev` is set — [`crate::app::Host`] gates every call
//! into this module on that flag and never builds a [`Dev`] reading
//! otherwise, so an ordinary build pays for an idle raster and nothing else.
//!
//! # Placement is a workbench tile's, not a hand-rolled corner
//!
//! Ruled 2026-09-02, dev tools plan §4.2: the dev lane's placement is
//! whatever `workbench`'s split-and-tab tree gives, not the fixed-margin
//! `placement()` every other lane in this crate hand-rolls. The tree DT1
//! builds holds exactly one tile, so [`rect_of`] always hands that tile the
//! whole of the reserved dock — but it is the honest walk of the tree's own
//! splits and shares, not a shortcut that happens to agree with one for a
//! single tile: a `Split` divides the dock by each child's fraction exactly
//! as `workbench::TileBranch::fraction` documents it, and only a `Stack`
//! bottoms out in a rect.
//!
//! What is genuinely missing from the stack, found doing this: `workbench`
//! (the `genet` repo, `components/workbench`) is a host-owned tree and
//! reducer only — there is no genet-side *surface* that renders a `TileTree`
//! to pixels, draws its tab strip, or turns a drag into a `TileEvent`. DT1
//! never needs one, because its tree never holds more than the one tile
//! nothing ever drags. A second dev tile (DT2's follow-and-inspect panel,
//! most likely) is what would make the split arithmetic below do real work
//! for the first time; drawing an actual tab strip a player could click, or a
//! divider they could drag, needs that missing surface piece and is out of
//! this phase's reach for exactly that reason — reported here rather than
//! built, per the plan's stop rule against a new stack widget.
//!
//! **DT2 stayed inside the one tile for exactly that reason**, and it is what
//! moved the dock. DT1's dock was the top-left corner, which is 196 pixels tall
//! before it reaches the vitals panel below it; an inspector with a dozen rows
//! does not fit there and covering the vitals panel would not be an
//! improvement. The dock is now the right column under the minimap — the other
//! region no lane claims — and the tile still takes it through [`rect_of`]
//! rather than a hardcoded rectangle. The corner was never a ruling: §4.2 rules
//! that the placement is the tree's, and this is the same tree.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use cambium::GenetAppRunner;
use genet_livery::{
    Device, InteractionStates, LiveryLayout, StylePlane, StyleSet, TextSystem, ViewportSizes,
    emit_paint_list_with_text_system_scrolled_with_images, layout_with_text_system, resolve_styles,
};
use genet_scripted_dom::{NodeId, ScriptedDom};
use mesocosm_views::{Dev, DevChild};
use paint_list_api::{DeviceIntSize, PaintList as _};
use workbench::{ContentSource, SplitAxis, Tile, TileId, TileTree};

use crate::chrome::{Chrome, Raster};

/// The panel's box, in pixels, and the raster's size with it.
///
/// Sized for DT2's inspector rather than DT1's four rows: the tile carries the
/// time section, nine fixed follow rows and up to
/// [`MAX_PART_ROWS`](mesocosm_views::dev::MAX_PART_ROWS) part rows, at the
/// smaller type the sheet sets. The width matches the vitals panel's so the
/// two chrome surfaces read as one system.
const WIDTH: u32 = 300;
const HEIGHT: u32 = 332;

/// Distance from the frame's corner, matching the other lanes' own margin.
const MARGIN: f32 = 12.0;

/// The one tile DT1's tree ever holds.
const DEV_TILE: TileId = TileId(1);

type Runner = GenetAppRunner<Dev, fn(&Dev) -> DevChild, DevChild>;

pub struct DevChrome {
    runner: Runner,
    raster: Raster,
    /// Livery's retained text shaping, reused across frames for the reason
    /// every other lane reuses one: building a font context per frame is
    /// what makes a panel expensive.
    text: TextSystem,
    style_set: StyleSet,
    device: Device,
    generation: u64,
    /// The reading the raster currently holds. A frame whose dev state is
    /// unchanged pays a comparison and no raster — which is most frames while
    /// paused, and is why the inspector's dozen rows cost nothing to leave up.
    shown: Option<Dev>,
    /// The workbench tree the lane's one tile lives in. See the module docs.
    tree: TileTree,
}

impl DevChrome {
    pub fn new(chrome: &Chrome) -> Self {
        let dom = Rc::new(RefCell::new(ScriptedDom::new()));
        let runner = Runner::new(
            dom,
            mesocosm_views::dev_root as fn(&Dev) -> DevChild,
            Dev::default(),
        );
        Self {
            runner,
            raster: Raster::new(chrome.device(), "dev", WIDTH, HEIGHT),
            text: TextSystem::new(),
            style_set: StyleSet::cambium(&[mesocosm_views::dev_css()]),
            device: Device::screen(WIDTH as f32, HEIGHT as f32),
            generation: 0,
            shown: None,
            tree: TileTree::single(Tile {
                id: DEV_TILE,
                title: "dev".into(),
                // The open lane: this contract does not need to know what a
                // dev strip is, only that a host does.
                content: ContentSource::Open {
                    kind: "mesocosm.dev".into(),
                    id: "dev".into(),
                },
                accent: None,
            }),
        }
    }

    /// Takes the frame's reading and rasterizes it if it changed.
    pub fn refresh(&mut self, chrome: &Chrome, dev: &Dev) {
        if self.shown.as_ref() == Some(dev) {
            return;
        }
        self.runner.update(|state| *state = dev.clone());
        self.shown = Some(dev.clone());
        self.raster_panel(chrome);
    }

    /// Style, lay out, paint, lower, rasterize. The whole cambium frame.
    fn raster_panel(&mut self, chrome: &Chrome) {
        self.generation = self.generation.saturating_add(1);
        let dom = self.runner.dom();
        let dom = dom.borrow();
        let width = WIDTH as f32;
        let height = HEIGHT as f32;

        let styles = resolve_styles(
            &*dom,
            &self.style_set,
            &self.device,
            &InteractionStates::default(),
        );
        let Ok((styles, fragments)) = layout_with_text_system(
            &*dom,
            &styles,
            width,
            height,
            ViewportSizes::uniform(width, height),
            &mut self.text,
            &HashMap::new(),
        ) else {
            // A panel that will not lay out is not worth taking the game down
            // for; the frame simply keeps the last raster.
            eprintln!("dev: the panel did not lay out");
            return;
        };
        let list = self.emit(&dom, &styles, &fragments);
        let translated = paint_list_render::translate_paint_cmd_stream(
            list.viewport(),
            list.commands(),
            list.fonts(),
            list.images(),
        );
        chrome.raster(&self.raster, &translated.scene);
    }

    fn emit(
        &mut self,
        dom: &ScriptedDom,
        styles: &StylePlane<NodeId>,
        fragments: &LiveryLayout<NodeId>,
    ) -> genet_livery::LiveryPaintList {
        emit_paint_list_with_text_system_scrolled_with_images(
            dom,
            styles,
            fragments,
            DeviceIntSize::new(WIDTH as i32, HEIGHT as i32),
            self.generation,
            &mut self.text,
            &HashMap::new(),
            &HashMap::new(),
        )
    }

    /// The dev tile's rect, read off the workbench tree rather than a
    /// hardcoded corner.
    ///
    /// The dock is the right column under the minimap — the region none of the
    /// other lanes claims once DT2's inspector is too tall for the top-left
    /// corner: the minimap sits above it, the vitals panel is bottom left, and
    /// the checkpoint and board sit centre when either stands. Clamped so a
    /// window shorter than the dock still puts the tile on screen, exactly as
    /// the vitals panel clamps its own.
    fn placement(&self, frame: (u32, u32)) -> (f32, f32, f32, f32) {
        let x = frame.0 as f32 - WIDTH as f32 - MARGIN;
        // Under the minimap, but never past the bottom of a short window: a
        // tile drawn off the frame is worse than one crowding the map above.
        let under_the_minimap = MARGIN + crate::hud::SIDE as f32 + MARGIN;
        let lowest = (frame.1 as f32 - HEIGHT as f32 - MARGIN).max(0.0);
        let dock = (
            x.max(0.0),
            under_the_minimap.min(lowest),
            WIDTH as f32,
            HEIGHT as f32,
        );
        rect_of(&self.tree, DEV_TILE, dock).unwrap_or(dock)
    }

    /// This lane's retained DOM, its dock, and its sheet. See
    /// [`crate::vitals::VitalsChrome::probe`]. (DT4)
    pub fn probe(&self, frame: (u32, u32)) -> (cambium::DomHandle, [f32; 4], &'static str) {
        let (x, y, w, h) = self.placement(frame);
        (self.runner.dom(), [x, y, w, h], mesocosm_views::dev_css())
    }

    pub fn composite(
        &self,
        chrome: &Chrome,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        chrome.draw(
            encoder,
            target,
            self.raster.sample_view(),
            self.placement(frame),
            frame,
        );
    }

    /// The same, into a capture frame's offscreen format.
    pub fn capture_composite(
        &self,
        chrome: &Chrome,
        format: wgpu::TextureFormat,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        frame: (u32, u32),
    ) {
        chrome.draw_as(
            format,
            encoder,
            target,
            self.raster.sample_view(),
            self.placement(frame),
            frame,
        );
    }
}

/// Resolves `id`'s on-screen rect within `dock`, walking the tree the way
/// `workbench` itself describes it: a split divides its rect along its axis
/// by each child's fractional share ([`workbench::TileBranch::fraction`]),
/// and a stack's own rect is whatever it was handed — the tab strip that
/// would choose among several tiles is chrome this workspace does not draw
/// yet (see the module docs), and DT1 never has more than the one tile to
/// draw it for. `None` if `id` is not in the tree.
fn rect_of(
    tree: &TileTree,
    id: TileId,
    dock: (f32, f32, f32, f32),
) -> Option<(f32, f32, f32, f32)> {
    match tree {
        TileTree::Stack(stack) => stack.tabs.iter().any(|tab| tab.id == id).then_some(dock),
        TileTree::Split { axis, children } => {
            let (x, y, w, h) = dock;
            let mut offset = 0.0f32;
            for branch in children {
                let child_dock = match axis {
                    SplitAxis::Row => (x + offset * w, y, branch.fraction * w, h),
                    SplitAxis::Column => (x, y + offset * h, w, branch.fraction * h),
                };
                if let Some(rect) = rect_of(&branch.tree, id, child_dock) {
                    return Some(rect);
                }
                offset += branch.fraction;
            }
            None
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tile(id: TileId) -> Tile {
        Tile {
            id,
            title: "t".into(),
            content: ContentSource::Open {
                kind: "x".into(),
                id: "y".into(),
            },
            accent: None,
        }
    }

    /// The exact shape DT1 ever builds: one tile, one stack, the whole dock.
    #[test]
    fn a_single_tile_stack_takes_the_whole_dock() {
        let tree = TileTree::single(tile(DEV_TILE));
        let dock = (10.0, 20.0, 200.0, 128.0);
        assert_eq!(rect_of(&tree, DEV_TILE, dock), Some(dock));
    }

    /// The split arithmetic DT1 does not exercise today, proven correct ahead
    /// of the tile that will.
    #[test]
    fn a_row_split_divides_the_dock_by_each_childs_fraction() {
        let left = TileId(1);
        let right = TileId(2);
        let tree = TileTree::split(
            SplitAxis::Row,
            vec![
                workbench::TileBranch::new(0.25, TileTree::single(tile(left))),
                workbench::TileBranch::new(0.75, TileTree::single(tile(right))),
            ],
        );
        let dock = (0.0, 0.0, 400.0, 100.0);
        assert_eq!(rect_of(&tree, left, dock), Some((0.0, 0.0, 100.0, 100.0)));
        assert_eq!(
            rect_of(&tree, right, dock),
            Some((100.0, 0.0, 300.0, 100.0))
        );
    }

    /// A column split divides the other axis instead.
    #[test]
    fn a_column_split_divides_height_instead_of_width() {
        let top = TileId(1);
        let bottom = TileId(2);
        let tree = TileTree::split(
            SplitAxis::Column,
            vec![
                workbench::TileBranch::new(0.5, TileTree::single(tile(top))),
                workbench::TileBranch::new(0.5, TileTree::single(tile(bottom))),
            ],
        );
        let dock = (0.0, 0.0, 100.0, 200.0);
        assert_eq!(rect_of(&tree, top, dock), Some((0.0, 0.0, 100.0, 100.0)));
        assert_eq!(
            rect_of(&tree, bottom, dock),
            Some((0.0, 100.0, 100.0, 100.0))
        );
    }

    /// A tile the tree does not hold resolves to nothing, rather than a
    /// stale or default rect.
    #[test]
    fn an_id_not_in_the_tree_resolves_to_nothing() {
        let tree = TileTree::single(tile(DEV_TILE));
        assert_eq!(rect_of(&tree, TileId(99), (0.0, 0.0, 1.0, 1.0)), None);
    }
}
