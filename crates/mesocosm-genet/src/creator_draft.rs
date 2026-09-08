// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Local generator criteria, independent of an entered world's saved trace.

use mesocosm_core::world::generation::Request;
use std::{io::Write, path::Path};

pub fn load(path: &Path) -> Result<Option<Request>, String> {
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(why) => return Err(why.to_string()),
    };
    let request: Request = serde_json::from_slice(&bytes).map_err(|why| why.to_string())?;
    request.validate().map_err(|why| format!("{why:?}"))?;
    Ok(Some(request))
}

pub fn save(path: &Path, request: &Request) -> Result<(), String> {
    request.validate().map_err(|why| format!("{why:?}"))?;
    let bytes = serde_json::to_vec_pretty(request).map_err(|why| why.to_string())?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|why| why.to_string())?;
    pending.write_all(&bytes).map_err(|why| why.to_string())?;
    pending
        .as_file()
        .sync_all()
        .map_err(|why| why.to_string())?;
    pending.persist(path).map_err(|why| why.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refused_search_survives_reopen_and_invalid_save_preserves_old_draft() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("criteria.json");
        assert_eq!(load(&path).unwrap(), None);
        let mut request = Request::default();
        request.criteria.max_parts = 1;
        save(&path, &request).unwrap();
        assert_eq!(load(&path).unwrap(), Some(request.clone()));
        let bytes = std::fs::read(&path).unwrap();
        request.version = u32::MAX;
        assert!(save(&path, &request).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
        request.version = mesocosm_core::world::generation::VERSION;
        request.variation = 3;
        save(&path, &request).unwrap();
        assert_eq!(load(&path).unwrap(), Some(request));
        std::fs::write(&path, "not a draft").unwrap();
        assert!(load(&path).is_err());
    }
}
