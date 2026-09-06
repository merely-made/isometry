// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use netrender::Scene;
use netrender_text::parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, Layout, LayoutContext, StyleProperty,
    fontique,
};
use std::sync::Arc;

pub(super) struct Text {
    font: FontContext,
    layouts: LayoutContext<[f32; 4]>,
    family: String,
}
impl Text {
    pub(super) fn new() -> Result<Self, String> {
        let custom = std::env::var("PAREDROS_FONT").ok();
        let paths = custom.iter().map(String::as_str).chain([
            r"C:\Windows\Fonts\segoeui.ttf",
            r"C:\Windows\Fonts\arial.ttf",
            "/System/Library/Fonts/Helvetica.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ]);
        for path in paths {
            let Ok(bytes) = std::fs::read(path) else {
                continue;
            };
            let mut font = FontContext::new();
            let Some((id, _)) = font
                .collection
                .register_fonts(fontique::Blob::new(Arc::new(bytes)), None)
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
        Err("Body sheet needs a readable local font.".into())
    }
    pub(super) fn label(
        &mut self,
        scene: &mut Scene,
        text: &str,
        xy: [f32; 2],
        size: f32,
        color: [f32; 4],
        width: f32,
    ) {
        let mut builder = self.layouts.ranged_builder(&mut self.font, text, 1., true);
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::Brush(color));
        builder.push_default(StyleProperty::FontFamily(FontFamily::named(&self.family)));
        let mut layout: Layout<[f32; 4]> = builder.build(text);
        layout.break_all_lines(Some(width));
        layout.align(Alignment::Start, AlignmentOptions::default());
        netrender_text::push_layout(scene, &layout, xy);
    }
}
