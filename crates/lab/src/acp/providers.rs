//! Installed ACP provider metadata shared by marketplace install and chat runtime.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::dispatch::error::ToolError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpProviderEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub distribution: String,
    pub command: String,
    pub installed_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

pub fn providers_path() -> Result<PathBuf, ToolError> {
    let env_path = crate::config::dotenv_path().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "cannot determine ~/.lab path".to_string(),
    })?;
    let dir = env_path
        .parent()
        .ok_or_else(|| ToolError::internal_message("dotenv path has no parent"))?;
    Ok(dir.join("acp-providers.json"))
}

pub fn read_providers() -> Result<Vec<AcpProviderEntry>, ToolError> {
    let path = providers_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let bytes = std::fs::read(&path)
        .map_err(|e| ToolError::internal_message(format!("read {}: {e}", path.display())))?;
    if bytes.is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_slice(&bytes).map_err(|e| ToolError::Sdk {
        sdk_kind: "decode_error".to_string(),
        message: format!("parse {}: {e}", path.display()),
    })
}

pub fn write_providers(entries: &[AcpProviderEntry]) -> Result<(), ToolError> {
    use std::io::Write;

    let path = providers_path()?;
    let dir = path
        .parent()
        .ok_or_else(|| ToolError::internal_message("providers path has no parent"))?;
    std::fs::create_dir_all(dir)
        .map_err(|e| ToolError::internal_message(format!("create {}: {e}", dir.display())))?;
    let mut tmp = tempfile::NamedTempFile::new_in(dir)
        .map_err(|e| ToolError::internal_message(format!("temp file: {e}")))?;
    let body = serde_json::to_vec_pretty(entries)
        .map_err(|e| ToolError::internal_message(format!("serialize providers: {e}")))?;
    tmp.write_all(&body)
        .map_err(|e| ToolError::internal_message(format!("write temp: {e}")))?;
    tmp.flush()
        .map_err(|e| ToolError::internal_message(format!("flush temp: {e}")))?;
    if let Ok(meta) = std::fs::symlink_metadata(&path)
        && meta.file_type().is_symlink()
    {
        return Err(ToolError::Sdk {
            sdk_kind: "invalid_param".to_string(),
            message: format!(
                "refusing to overwrite symlink at {} (acp-providers.json must be a regular file)",
                path.display()
            ),
        });
    }
    tmp.persist(&path)
        .map_err(|e| ToolError::internal_message(format!("persist {}: {e}", path.display())))?;
    Ok(())
}

pub fn upsert_provider(entry: &AcpProviderEntry) -> Result<(), ToolError> {
    let mut entries = read_providers()?;
    if let Some(existing) = entries.iter_mut().find(|e| e.id == entry.id) {
        *existing = entry.clone();
    } else {
        entries.push(entry.clone());
    }
    write_providers(&entries)
}

pub fn remove_provider(id: &str) -> Result<bool, ToolError> {
    let mut entries = read_providers()?;
    let before = entries.len();
    entries.retain(|e| e.id != id);
    let removed = entries.len() != before;
    if removed {
        write_providers(&entries)?;
    }
    Ok(removed)
}
