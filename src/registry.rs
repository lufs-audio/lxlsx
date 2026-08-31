//! Filesystem-as-registry: `processes/<slug>/manifest.json` + generated `index.json`.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Manifest {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub stages: Vec<String>,
}

/// Walk `processes/*/manifest.json` and return the sorted manifests.
///
/// # Errors
/// Returns [`Error::Io`] if the `processes/` dir can't be read, or
/// [`Error::Contract`] if a manifest fails to parse.
pub fn discover(root: &Path) -> Result<Vec<Manifest>, Error> {
    let processes_dir = root.join("processes");
    let mut out: Vec<Manifest> = Vec::new();
    let entries = std::fs::read_dir(&processes_dir).map_err(Error::Io)?;
    for entry in entries {
        let entry = entry.map_err(Error::Io)?;
        if !entry.file_type().is_ok_and(|t| t.is_dir()) {
            continue;
        }
        let manifest_path = entry.path().join("manifest.json");
        if !manifest_path.exists() {
            continue;
        }
        let bytes = std::fs::read_to_string(&manifest_path).map_err(Error::Io)?;
        let manifest: Manifest = serde_json::from_str(&bytes).map_err(|e| {
            Error::Contract(format!("bad manifest {}: {e}", entry.path().display()))
        })?;
        out.push(manifest);
    }
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

/// Regenerate `processes/index.json` from the discovered manifests.
///
/// # Errors
/// Returns any error from [`discover`], or [`Error::Io`] if the index can't be written.
pub fn generate_index(root: &Path) -> Result<PathBuf, Error> {
    let manifests = discover(root)?;
    let index_path = root.join("processes").join("index.json");
    let json = serde_json::to_string_pretty(&serde_json::json!({ "processes": manifests }))
        .map_err(|e| Error::Contract(e.to_string()))?;
    std::fs::write(&index_path, json)?;
    Ok(index_path)
}

/// Return true when `processes/index.json` is consistent with the manifests on disk.
///
/// # Errors
/// Returns any error from [`discover`], or [`Error::Contract`] if the index is malformed.
pub fn index_is_current(root: &Path) -> Result<bool, Error> {
    let manifests = discover(root)?;
    let index_path = root.join("processes").join("index.json");
    let bytes = std::fs::read_to_string(&index_path);
    let Ok(bytes) = bytes else {
        return Ok(false);
    };
    let parsed: serde_json::Value =
        serde_json::from_str(&bytes).map_err(|e| Error::Contract(e.to_string()))?;
    let current: Vec<Manifest> =
        serde_json::from_value(parsed.get("processes").cloned().unwrap_or_default())
            .map_err(|e| Error::Contract(e.to_string()))?;
    Ok(manifests == current)
}
