// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Presentation helpers for the authored contact fixture.

use mesocosm_render::geometry::Vertex;
use netrender::Scene;
use netrender_text::parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, Layout, LayoutContext, StyleProperty,
    fontique,
};
use renderling::glam::Vec3;
use std::sync::Arc;

pub fn cuboid(out: &mut Vec<Vertex>, min: [f32; 3], max: [f32; 3], color: [f32; 3]) {
    let [x, y, z] = min;
    let [xx, yy, zz] = max;
    let faces = [
        ([[x, y, z], [x, y, zz], [x, yy, zz], [x, yy, z]], 0.68),
        ([[xx, y, zz], [xx, y, z], [xx, yy, z], [xx, yy, zz]], 0.82),
        ([[x, y, zz], [x, y, z], [xx, y, z], [xx, y, zz]], 0.48),
        ([[x, yy, z], [x, yy, zz], [xx, yy, zz], [xx, yy, z]], 1.0),
        ([[xx, y, z], [x, y, z], [x, yy, z], [xx, yy, z]], 0.78),
        ([[x, y, zz], [xx, y, zz], [xx, yy, zz], [x, yy, zz]], 0.90),
    ];
    for (points, shade) in faces {
        for i in [0, 1, 2, 0, 2, 3] {
            out.push(Vertex {
                position: points[i],
                color: color.map(|c| c * shade),
            });
        }
    }
}

pub fn line(out: &mut Vec<Vertex>, a: [f32; 3], b: [f32; 3], color: [f32; 3]) {
    let a = Vec3::from(a);
    let b = Vec3::from(b);
    let steps = ((a.distance(b) / 0.10).ceil() as usize).clamp(1, 100);
    for i in 0..=steps {
        let p = a.lerp(b, i as f32 / steps as f32);
        cuboid(
            out,
            (p - Vec3::splat(0.035)).to_array(),
            (p + Vec3::splat(0.035)).to_array(),
            color,
        );
    }
}

pub struct Text {
    font: FontContext,
    layouts: LayoutContext<[f32; 4]>,
    family: String,
}

impl Text {
    pub fn new() -> Result<Self, String> {
        let custom = std::env::var("PAREDROS_FONT").ok();
        let paths = custom.iter().map(String::as_str).chain([
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\arial.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ]);
        for path in paths {
            let Ok(bytes) = std::fs::read(path) else {
                continue;
            };
            let mut font = FontContext::new();
            let blob = fontique::Blob::new(Arc::new(bytes));
            let Some((id, _)) = font
                .collection
                .register_fonts(blob, None)
                .into_iter()
                .next()
            else {
                continue;
            };
            let Some(family) = font.collection.family_name(id).map(str::to_owned) else {
                continue;
            };
            return Ok(Self {
                font,
                layouts: LayoutContext::new(),
                family,
            });
        }
        Err("Crossing needs a readable font. Set PAREDROS_FONT to a local TTF/OTF file.".into())
    }

    pub fn label(
        &mut self,
        scene: &mut Scene,
        text: &str,
        xy: [f32; 2],
        size: f32,
        color: [f32; 4],
    ) {
        let mut builder = self.layouts.ranged_builder(&mut self.font, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::Brush(color));
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(&self.family)));
        let mut layout: Layout<[f32; 4]> = builder.build(text);
        layout.break_all_lines(Some(1228.0));
        layout.align(Alignment::Start, AlignmentOptions::default());
        netrender_text::push_layout(scene, &layout, xy);
    }
}
