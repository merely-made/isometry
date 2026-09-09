// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Two-process native persistence receipt; does not simulate mouse input.

use super::{App, EquipmentCommand, Live};
use std::path::Path;

pub(super) fn prepare(app: &mut App) {
    // Require explicit receipt storage, protecting ordinary player saves.
    assert!(std::env::var_os("PAREDROS_EQUIPMENT_SAVES").is_some());
    app.comparison = false;
    assert!(
        app.equipment
            .game()
            .attachments(app.equipment.subject())
            .is_empty()
    );
    match std::env::var("PAREDROS_EQUIPMENT_PERSIST_SMOKE")
        .unwrap()
        .as_str()
    {
        "save" => {
            app.equipment_view.part = 1;
            app.equipment_command(EquipmentCommand::Attach);
            app.equipment_command(EquipmentCommand::Save);
            assert!(
                app.equipment.status().starts_with("Saved "),
                "{}",
                app.equipment.status()
            );
        },
        "load" => {
            app.equipment_command(EquipmentCommand::Load);
            assert!(
                app.equipment.status().starts_with("Loaded "),
                "{}",
                app.equipment.status()
            );
            app.equipment_view.part = 1;
        },
        other => panic!("unknown persistence smoke mode {other}"),
    }
    let attachments = app.equipment.game().attachments(app.equipment.subject());
    assert_eq!(attachments.len(), 1);
    assert_eq!(attachments[0].part.0, 1);
    assert!(attachments[0].current);
    println!("Persistence receipt: {}", app.equipment.status());
}

pub(super) fn capture(live: &Live, master: &wgpu::Texture, root: &Path) {
    let frame = live.composer.capture(master);
    assert!(!frame.is_trivial());
    let mode = std::env::var("PAREDROS_EQUIPMENT_PERSIST_SMOKE").unwrap();
    let path = root.join(format!("persistence_{mode}.png"));
    frame.write_png(&path).expect("persistence capture");
    println!("Persistence capture: {}", path.display());
}
