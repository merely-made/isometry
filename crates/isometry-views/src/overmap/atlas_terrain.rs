//! Cached cartographic marks derived from the region's terrain vocabulary.

use super::AtlasTerrainCell;
use cambium::GraphCanvasAtlasPaint;
use sceno::{Footprint, Size2, Vec2};
use sprigging::ColorF;

pub(super) fn terrain_paint(cells: &[AtlasTerrainCell]) -> Vec<GraphCanvasAtlasPaint> {
    let mut paint = Vec::with_capacity(cells.len() * 3);
    for cell in cells {
        let anchor = Vec2::new(cell.col as f32 + 0.5, cell.row as f32 + 0.5);
        let color = match cell.kind.as_str() {
            "water" => rgba(0.26, 0.48, 0.57, 1.0),
            "stone" => rgba(0.59, 0.58, 0.48, 1.0),
            "grass" => rgba(0.65, 0.69, 0.43, 1.0),
            "road" => rgba(0.74, 0.64, 0.43, 1.0),
            "forest-floor" | "forest" => rgba(0.39, 0.50, 0.32, 1.0),
            _ => rgba(0.55, 0.59, 0.43, 1.0),
        };
        paint.push(GraphCanvasAtlasPaint {
            anchor,
            footprint: Footprint::Rect {
                size: Size2::new(1.0, 1.0),
            },
            fill: color,
            stroke: None,
            stroke_width: 0.0,
        });
        if matches!(cell.kind.as_str(), "forest-floor" | "forest") {
            // Map symbols describe a forest cell, not individual tactical trees.
            // Fixed cell arithmetic keeps regenerated marks deterministic.
            let jitter = ((cell.col * 17 + cell.row * 31) % 7) as f32 * 0.015;
            for (dx, dy) in [(-0.20, 0.10), (0.20, -0.06)] {
                paint.push(GraphCanvasAtlasPaint {
                    anchor: Vec2::new(anchor.x + dx + jitter, anchor.y + dy),
                    footprint: Footprint::Polygon {
                        points: vec![
                            Vec2::new(0.0, -0.36),
                            Vec2::new(0.22, 0.08),
                            Vec2::new(0.10, 0.06),
                            Vec2::new(0.27, 0.28),
                            Vec2::new(-0.27, 0.28),
                            Vec2::new(-0.10, 0.06),
                            Vec2::new(-0.22, 0.08),
                        ],
                    },
                    fill: rgba(0.22, 0.36 + jitter, 0.25, 1.0),
                    stroke: Some(rgba(0.16, 0.28, 0.19, 0.6)),
                    stroke_width: 0.018,
                });
            }
        } else if cell.kind == "water" {
            paint.push(GraphCanvasAtlasPaint {
                anchor: Vec2::new(anchor.x, anchor.y + 0.12),
                footprint: Footprint::Rect {
                    size: Size2::new(0.50, 0.025),
                },
                fill: rgba(0.64, 0.78, 0.76, 0.5),
                stroke: None,
                stroke_width: 0.0,
            });
        }
    }
    paint
}

fn rgba(r: f32, g: f32, b: f32, a: f32) -> ColorF {
    ColorF { r, g, b, a }
}
