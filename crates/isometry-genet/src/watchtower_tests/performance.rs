//! Opt-in timing receipt through the real pointer and dispatch paths.

use super::*;
use std::time::Instant;

#[test]
#[ignore = "diagnostic timings; run explicitly with --ignored --nocapture"]
fn profile_overmap_interaction() {
    let (mut harness, _) = watchtower();
    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    harness.update(|ui| ui.commit_generation_preview());
    harness.after_dispatch();
    harness.update(|ui| {
        ui.drag_move_token(TokenId(1), (1, 7));
        ui.travel(TokenId(1));
        ui.open_overmap();
    });
    harness.relayout();
    let (x, y) = harness
        .resolve(&Selector::class("graph-canvas-atlas-field"))
        .expect("a visible atlas field");
    harness.press_at(x, y);
    assert!(
        harness.pointer_capture().is_some(),
        "real graph drag started"
    );
    let mut pointer = Vec::new();
    let mut dispatch = Vec::new();
    let mut layout = Vec::new();
    let mut style_resolve = Vec::new();
    let mut layout_with_text = Vec::new();
    let mut content_extent = Vec::new();
    for step in 1..=30 {
        let start = Instant::now();
        harness.move_to(x + step as f32, y + 12.0);
        harness.drain_pointer();
        pointer.push(start.elapsed().as_secs_f64() * 1000.0);
        let start = Instant::now();
        harness.after_dispatch();
        dispatch.push(start.elapsed().as_secs_f64() * 1000.0);
        let start = Instant::now();
        harness.relayout();
        layout.push(start.elapsed().as_secs_f64() * 1000.0);
        let profile = harness.relayout_profile();
        style_resolve.push(profile.style_resolve_us as f64 / 1000.0);
        layout_with_text.push(profile.layout_with_text_us as f64 / 1000.0);
        content_extent.push(profile.content_extent_us as f64 / 1000.0);
    }
    harness.release_at(x + 30.0, y + 12.0);
    assert_ne!(
        harness
            .state()
            .overmap_motion
            .offset("watchtower:ruined-watchtower"),
        (0.0, 0.0),
        "a released atlas field has a visual motion offset"
    );
    for (name, mut samples) in [
        ("pointer", pointer),
        ("dispatch", dispatch),
        ("layout", layout),
        ("style-resolve", style_resolve),
        ("layout-text", layout_with_text),
        ("content-extent", content_extent),
    ] {
        samples.sort_by(f64::total_cmp);
        eprintln!(
            "[isometry-perf] {name} n={} median_ms={:.3} p95_ms={:.3}",
            samples.len(),
            samples[15],
            samples[28]
        );
    }
    let start = Instant::now();
    for _ in 0..100 {
        std::hint::black_box(isometry_views::overmap_swatch(harness.state()));
    }
    eprintln!(
        "[isometry-perf] swatch mean_ms={:.3}",
        start.elapsed().as_secs_f64() * 10.0
    );
}
