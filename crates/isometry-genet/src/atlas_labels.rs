//! Cached font-outline preparation for retained atlas callouts.
//!
//! The shared atlas leaf deliberately paints only paths. This host adapter
//! turns caller-supplied font bytes into closed
//! polygon approximations once per `(text, size, font)` cache entry; motion
//! then translates those paths in the retained leaf without rebuilding DOM.

use std::sync::Arc;

use ab_glyph::{Font, FontArc, OutlineCurve};
use cambium::{GraphCanvasAtlasCallout, GraphCanvasAtlasCompoundOutline, GraphCanvasAtlasPaint};
use sceno::{Footprint, Vec2};
use sprigging::ColorF;

/// Convert simple left-to-right text from caller font bytes into a callout.
/// It uses glyph lookup, advances and kerning, not full Unicode shaping.
///
/// `offset` is measured in leaf pixels from the moving graph node. The caller
/// caches the returned value by its font-resource identity, text and size.
pub fn atlas_label_from_font_bytes<Id>(
    id: Id,
    text: &str,
    font_bytes: Vec<u8>,
    px: f32,
    offset: Vec2,
    color: ColorF,
) -> Result<GraphCanvasAtlasCallout<Id>, String> {
    let font = FontArc::try_from_vec(font_bytes).map_err(|error| error.to_string())?;
    let scale = px.max(1.0) / font.height_unscaled().max(1.0);
    let width = label_width(text, &font, scale);
    Ok(GraphCanvasAtlasCallout {
        id,
        offset,
        outlines: Arc::from([GraphCanvasAtlasPaint {
            anchor: Vec2::new(width / 2.0, -px * 0.35),
            footprint: Footprint::Rect {
                size: sceno::Size2::new(width + 6.0, px + 5.0),
            },
            fill: ColorF::new(0.10, 0.13, 0.18, 0.78),
            stroke: Some(ColorF::new(0.62, 0.68, 0.78, 0.62)),
            stroke_width: 0.75,
        }]),
        compound_outlines: Arc::from(compound_outlines(text, &font, scale, color)),
    })
}

fn label_width(text: &str, font: &FontArc, scale: f32) -> f32 {
    let mut width = 0.0;
    let mut previous = None;
    for character in text.chars() {
        let glyph = font.glyph_id(character);
        if let Some(previous) = previous {
            width += font.kern_unscaled(previous, glyph) * scale;
        }
        width += font.h_advance_unscaled(glyph) * scale;
        previous = Some(glyph);
    }
    width.max(1.0)
}

/// Resolve the platform face that Genet's default `sans-serif` route uses on
/// the native Windows host, then prepare outlines. The cache belongs to App;
/// this function performs no repeated IO when its result is retained there.
pub fn atlas_label_from_default_host_font<Id: Clone>(
    id: Id,
    text: &str,
    px: f32,
    offset: Vec2,
    color: ColorF,
) -> GraphCanvasAtlasCallout<Id> {
    match default_host_font_bytes()
        .and_then(|bytes| atlas_label_from_font_bytes(id.clone(), text, bytes, px, offset, color))
    {
        Ok(callout) => callout,
        Err(_) => fallback_callout(id, text, offset),
    }
}

fn default_host_font_bytes() -> Result<Vec<u8>, String> {
    let windows = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
        .join("Fonts")
        .join("segoeui.ttf");
    let candidates = [
        windows,
        std::path::PathBuf::from("/System/Library/Fonts/Supplemental/Arial.ttf"),
        std::path::PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
    ];
    candidates
        .into_iter()
        .find_map(|path| std::fs::read(&path).ok())
        .ok_or_else(|| "default sans-serif font bytes unavailable".to_owned())
}

/// A visible, deliberately plain fallback plate. It preserves a target's
/// moving presence when the platform font cannot be read; the native button
/// still announces the complete label to assistive technology.
fn fallback_callout<Id>(id: Id, text: &str, offset: Vec2) -> GraphCanvasAtlasCallout<Id> {
    let width = (text.chars().count() as f32 * 6.0 + 8.0).max(16.0);
    GraphCanvasAtlasCallout {
        id,
        offset,
        outlines: Arc::from([GraphCanvasAtlasPaint {
            anchor: Vec2::new(width / 2.0, 5.0),
            footprint: Footprint::Rect {
                size: sceno::Size2::new(width, 10.0),
            },
            fill: ColorF::new(0.12, 0.15, 0.20, 0.88),
            stroke: Some(ColorF::new(0.72, 0.75, 0.82, 0.8)),
            stroke_width: 1.0,
        }]),
        compound_outlines: Arc::from([]),
    }
}

fn compound_outlines(
    text: &str,
    font: &FontArc,
    scale: f32,
    color: ColorF,
) -> Vec<GraphCanvasAtlasCompoundOutline> {
    let mut pen_x = 0.0;
    let mut previous = None;
    let mut output = Vec::new();
    for character in text.chars() {
        let glyph = font.glyph_id(character);
        if let Some(previous) = previous {
            pen_x += font.kern_unscaled(previous, glyph) * scale;
        }
        if let Some(outline) = font.outline(glyph) {
            let contours = outline_polygons(&outline.curves, scale, pen_x);
            if !contours.is_empty() {
                output.push(GraphCanvasAtlasCompoundOutline {
                    contours,
                    fill: color,
                    stroke: None,
                    stroke_width: 0.0,
                });
            }
        }
        pen_x += font.h_advance_unscaled(glyph) * scale;
        previous = Some(glyph);
    }
    output
}

fn outline_polygons(curves: &[OutlineCurve], scale: f32, pen_x: f32) -> Vec<Vec<Vec2>> {
    let mut output = Vec::new();
    let mut current = Vec::new();
    let mut last = None;
    for curve in curves {
        let (start, end, samples) = match *curve {
            OutlineCurve::Line(a, b) => (a, b, vec![b]),
            OutlineCurve::Quad(a, control, b) => (
                a,
                b,
                (1..=6)
                    .map(|i| quad(a, control, b, i as f32 / 6.0))
                    .collect(),
            ),
            OutlineCurve::Cubic(a, c1, c2, b) => (
                a,
                b,
                (1..=8)
                    .map(|i| cubic(a, c1, c2, b, i as f32 / 8.0))
                    .collect(),
            ),
        };
        if last.is_none_or(|previous: ab_glyph::Point| previous != start) {
            if current.len() >= 3 {
                output.push(std::mem::take(&mut current));
            }
            current.push(project(start, scale, pen_x));
        }
        current.extend(
            samples
                .into_iter()
                .map(|point| project(point, scale, pen_x)),
        );
        last = Some(end);
    }
    if current.len() >= 3 {
        output.push(current);
    }
    output
}

fn project(point: ab_glyph::Point, scale: f32, pen_x: f32) -> Vec2 {
    Vec2::new(pen_x + point.x * scale, -point.y * scale)
}

fn quad(a: ab_glyph::Point, c: ab_glyph::Point, b: ab_glyph::Point, t: f32) -> ab_glyph::Point {
    let u = 1.0 - t;
    ab_glyph::point(
        u * u * a.x + 2.0 * u * t * c.x + t * t * b.x,
        u * u * a.y + 2.0 * u * t * c.y + t * t * b.y,
    )
}

fn cubic(
    a: ab_glyph::Point,
    c1: ab_glyph::Point,
    c2: ab_glyph::Point,
    b: ab_glyph::Point,
    t: f32,
) -> ab_glyph::Point {
    let u = 1.0 - t;
    ab_glyph::point(
        u.powi(3) * a.x + 3.0 * u * u * t * c1.x + 3.0 * u * t * t * c2.x + t.powi(3) * b.x,
        u.powi(3) * a.y + 3.0 * u * u * t * c1.y + 3.0 * u * t * t * c2.y + t.powi(3) * b.y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_outer_and_counter_contours_as_one_compound_glyph() {
        let p = |x, y| ab_glyph::point(x, y);
        let curves = vec![
            OutlineCurve::Line(p(0.0, 0.0), p(10.0, 0.0)),
            OutlineCurve::Line(p(10.0, 0.0), p(10.0, 10.0)),
            OutlineCurve::Line(p(10.0, 10.0), p(0.0, 10.0)),
            OutlineCurve::Line(p(0.0, 10.0), p(0.0, 0.0)),
            OutlineCurve::Line(p(3.0, 3.0), p(3.0, 7.0)),
            OutlineCurve::Line(p(3.0, 7.0), p(7.0, 7.0)),
            OutlineCurve::Line(p(7.0, 7.0), p(7.0, 3.0)),
            OutlineCurve::Line(p(7.0, 3.0), p(3.0, 3.0)),
        ];
        let contours = outline_polygons(&curves, 1.0, 0.0);
        assert_eq!(
            contours.len(),
            2,
            "outer and counter stay distinct contours"
        );
    }
}
