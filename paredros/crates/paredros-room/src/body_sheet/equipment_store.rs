// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit immutable saves for the equipment probe, not a game-wide save service.

use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_BYTES: u64 = 16 * 1024 * 1024;
static NEXT: AtomicU64 = AtomicU64::new(0);

/// Publish a new snapshot. Existing saves are never replaced.
pub fn save_equipment(directory: &Path, bytes: &[u8]) -> Result<PathBuf, String> {
    if bytes.len() as u64 > MAX_BYTES {
        return Err("save exceeds 16 MiB limit".into());
    }
    fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    let name = format!(
        "equipment-{nanos:032}-{}-{:016}.bin",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    );
    let path = directory.join(name);
    let pending = path.with_extension("pending");
    let result = (|| -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        // A hard link publishes only a completed file and refuses overwrite on
        // every supported platform. A crash can leave a harmless pending link.
        fs::hard_link(&pending, &path)?;
        Ok(())
    })();
    if result.is_ok() {
        let _ = fs::remove_file(&pending);
    }
    result.map_err(|e| e.to_string())?;
    Ok(path)
}

/// Read the newest published save. Never silently fall back past a broken save.
pub fn load_equipment(directory: &Path) -> Result<(PathBuf, Vec<u8>), String> {
    let mut candidates = Vec::new();
    for entry in fs::read_dir(directory).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.starts_with("equipment-") && name.ends_with(".bin") {
            candidates.push(entry.path());
        }
    }
    candidates.sort();
    let path = candidates
        .pop()
        .ok_or("no equipment saves in this directory")?;
    let file = fs::File::open(&path).map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err("save exceeds 16 MiB limit".into());
    }
    Ok((path, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::body_sheet::EquipmentSession;

    #[test]
    fn saves_are_immutable_and_pending_files_are_not_loaded() {
        let directory = std::env::temp_dir().join(format!(
            "paredros-equipment-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let first = save_equipment(&directory, b"first").unwrap();
        let second = save_equipment(&directory, b"second").unwrap();
        assert_ne!(first, second);
        assert_eq!(fs::read(&first).unwrap(), b"first");
        let pending = directory.join("equipment-zzzz.pending");
        fs::write(&pending, b"incomplete").unwrap();
        assert_eq!(
            load_equipment(&directory).unwrap(),
            (second.clone(), b"second".to_vec())
        );
        for path in [first, second, pending] {
            fs::remove_file(path).unwrap();
        }
        fs::remove_dir(directory).unwrap();
    }

    #[test]
    fn disk_roundtrip_restores_attachment_into_a_fresh_session() {
        let directory = std::env::temp_dir().join(format!(
            "paredros-equipment-roundtrip-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut session = EquipmentSession::new().unwrap();
        session.attach(session.items()[0].id, mesocosm_core::PartId(1));
        let path = save_equipment(&directory, &session.save_bytes().unwrap()).unwrap();
        let (_, bytes) = load_equipment(&directory).unwrap();
        let mut fresh = EquipmentSession::new().unwrap();
        fresh.load_bytes(&bytes).unwrap();
        assert_eq!(fresh.game(), session.game());
        fs::remove_file(path).unwrap();
        fs::remove_dir(directory).unwrap();
    }
}
