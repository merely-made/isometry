//! Native atlas integration through the real graph-canvas dispatch path.

use super::*;

#[test]
fn watchtower_region_paints_atlas_and_discloses_known_fields() {
    let (mut harness, _) = watchtower();
    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    harness.update(|ui| ui.commit_generation_preview());
    harness.after_dispatch();
    harness.update(|ui| ui.open_overmap());
    harness.relayout();

    let atlas = isometry_views::overmap_atlas(harness.state()).expect("region atlas selected");
    assert_eq!(atlas.bounds.size.w, 11.0);
    assert_eq!(atlas.bounds.size.h, 9.0);
    assert!(
        atlas.paint.len() >= 99,
        "the complete retained region is painted"
    );
    assert_eq!(
        atlas.fields.len(),
        2,
        "only tower and known region are disclosed"
    );
    assert!(
        atlas
            .fields
            .iter()
            .any(|field| field.id == "watchtower:ruined-watchtower"
                && field.anchor.x == 5.5
                && field.anchor.y == 1.5)
    );
    assert!(
        atlas
            .fields
            .iter()
            .any(|field| field.id == "watchtower:forest-region"
                && field.anchor.x == 5.5
                && field.anchor.y == 4.5)
    );
    assert!(
        harness
            .resolve(&Selector::class("graph-canvas-atlas-swatch"))
            .is_some()
    );
}

#[test]
fn atlas_field_drag_captures_from_canvas_and_does_not_travel() {
    let (mut harness, _) = watchtower();
    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    harness.update(|ui| ui.commit_generation_preview());
    harness.after_dispatch();
    harness.update(|ui| ui.open_overmap());
    harness.relayout();

    let before_place = harness.state().world.party_at("dm").map(str::to_owned);
    let field_node = harness.with_dom(|dom| {
        genet_probe::matching(dom, &Selector::class("graph-canvas-atlas-field"))
            .into_iter()
            .next()
            .expect("a keyboard-semantic atlas field exists")
    });
    let (x, y, width, height) = harness.painted_rect(field_node).expect("field bounds");
    harness.press_at(x + width / 2.0, y + height / 2.0);
    assert!(
        harness.pointer_capture().is_some(),
        "atlas canvas captures the geographic field"
    );
    harness.move_to(x + width / 2.0 + 24.0, y + height / 2.0 + 12.0);
    harness.drain_pointer();
    harness.release_at(x + width / 2.0 + 24.0, y + height / 2.0 + 12.0);
    assert_eq!(
        harness.state().world.party_at("dm").map(str::to_owned),
        before_place
    );
    assert!(harness.state().overmap_travel_request.is_none());
    assert_ne!(
        harness
            .state()
            .overmap_motion
            .offset("watchtower:ruined-watchtower"),
        (0.0, 0.0)
    );
}

#[test]
fn atlas_reduced_motion_keeps_dispatch_path_idle_after_release() {
    let (mut harness, _) = watchtower();
    harness.update(|ui| ui.start_generator("watchtower"));
    harness.after_dispatch();
    harness.update(|ui| {
        ui.commit_generation_preview();
        ui.overmap_reduced_motion = true;
    });
    harness.after_dispatch();
    harness.update(|ui| ui.open_overmap());
    harness.relayout();
    let before_place = harness.state().world.party_at("dm").map(str::to_owned);
    let field_node = harness.with_dom(|dom| {
        genet_probe::matching(dom, &Selector::class("graph-canvas-atlas-field"))
            .into_iter()
            .next()
            .expect("atlas field")
    });
    let (x, y, width, height) = harness.painted_rect(field_node).expect("field bounds");
    harness.press_at(x + width / 2.0, y + height / 2.0);
    harness.move_to(x + width / 2.0 + 20.0, y + height / 2.0 + 10.0);
    harness.drain_pointer();
    harness.release_at(x + width / 2.0 + 20.0, y + height / 2.0 + 10.0);
    harness.after_dispatch();
    harness.relayout();
    assert_eq!(
        harness.state().world.party_at("dm").map(str::to_owned),
        before_place
    );
    assert!(
        !harness.prepare_frame(),
        "reduced motion settles in one host frame"
    );
    assert_eq!(
        harness
            .state()
            .overmap_motion
            .offset("watchtower:ruined-watchtower"),
        (0.0, 0.0)
    );
    assert!(harness.pointer_capture().is_none());
}
