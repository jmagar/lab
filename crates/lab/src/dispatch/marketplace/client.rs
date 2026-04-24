//! No external HTTP client — `marketplace` is a local-only synthetic service.
//!
//! This file satisfies the required dispatch service layout contract
//! (every migrated service must have `client.rs`). All work is local
//! filesystem I/O plus optional `tokio::process::Command` shell-out to
//! `claude plugin install/uninstall`.

use std::collections::HashSet;
use std::future::Future;
use std::io::{BufReader, Read};
use std::path::Path;
use std::path::PathBuf;
use std::pin::Pin;

use serde::Serialize;
use serde_json::Value;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Abstraction over a WebSocket-backed device RPC channel.
///
/// The concrete impl lives in `api/services/marketplace.rs` and is wired via
/// the fleet sender in `api/state.rs` (lab-zxx5.5). Until that bead lands,
/// all dispatch paths use `NoopDeviceRpcPort`.
pub(super) trait DeviceRpcPort: Send + Sync {
    fn send_rpc(
        &self,
        device_id: &str,
        method: &str,
        params: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send + '_>>;
}

/// Stub used by MCP/CLI dispatch until lab-zxx5.5 wires the fleet WebSocket.
pub(super) struct NoopDeviceRpcPort;

impl DeviceRpcPort for NoopDeviceRpcPort {
    fn send_rpc(
        &self,
        device_id: &str,
        _method: &str,
        _params: Value,
    ) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send + '_>> {
        let device_id = device_id.to_string();
        Box::pin(async move {
            Err(ToolError::Sdk {
                sdk_kind: "not_implemented".into(),
                message: format!(
                    "device RPC to `{device_id}` is not yet wired (pending lab-zxx5.5)"
                ),
            })
        })
    }
}

#[cfg(test)]
static TEST_PLUGINS_ROOT_OVERRIDE: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);
#[cfg(test)]
static TEST_PLUGINS_ROOT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Alias used by `backends/claude.rs`.
pub(super) fn claude_plugins_root() -> Result<PathBuf, ToolError> {
    plugins_root()
}

pub(super) fn plugins_root() -> Result<PathBuf, ToolError> {
    #[cfg(test)]
    if let Some(path) = TEST_PLUGINS_ROOT_OVERRIDE.lock().unwrap().clone() {
        return Ok(path);
    }

    let home = env_non_empty("HOME").ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "HOME env var not set".into(),
    })?;
    Ok(PathBuf::from(home).join(".claude").join("plugins"))
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DeployResult {
    pub ok: bool,
    pub changed: Vec<String>,
    pub skipped: Vec<String>,
    pub removed: Vec<String>,
    pub failed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DeployPreviewResult {
    pub changed: Vec<String>,
    pub skipped: Vec<String>,
    pub removed: Vec<String>,
    pub entries: Vec<DeployPreviewEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct DeployPreviewEntry {
    pub path: String,
    pub status: &'static str,
    #[serde(rename = "beforeContent", skip_serializing_if = "Option::is_none")]
    pub before_content: Option<String>,
    #[serde(rename = "afterContent", skip_serializing_if = "Option::is_none")]
    pub after_content: Option<String>,
}

#[derive(Debug, Default)]
struct SyncPreview {
    changed: Vec<String>,
    skipped: Vec<String>,
    removed: Vec<String>,
    entries: Vec<DeployPreviewEntry>,
}

pub(super) fn sync_workspace_to_target(
    workspace: &Path,
    target: &Path,
) -> Result<DeployResult, ToolError> {
    let mut changed = Vec::new();
    let mut skipped = Vec::new();
    let mut removed = Vec::new();
    let mut failed = Vec::new();
    sync_tree_to_target(
        workspace,
        target,
        workspace,
        &mut changed,
        &mut skipped,
        &mut removed,
        &mut failed,
    )?;
    Ok(DeployResult {
        ok: failed.is_empty(),
        changed,
        skipped,
        removed,
        failed,
        target: Some(target.to_string_lossy().into_owned()),
    })
}

pub(super) fn preview_workspace_sync(
    workspace: &Path,
    target: &Path,
) -> Result<DeployPreviewResult, ToolError> {
    let preview = preview_tree_sync(workspace, target, workspace)?;
    Ok(DeployPreviewResult {
        changed: preview.changed,
        skipped: preview.skipped,
        removed: preview.removed,
        entries: preview.entries,
        target: Some(target.to_string_lossy().into_owned()),
    })
}

fn sync_tree_to_target(
    workspace: &Path,
    target: &Path,
    current: &Path,
    changed: &mut Vec<String>,
    skipped: &mut Vec<String>,
    removed: &mut Vec<String>,
    failed: &mut Vec<String>,
) -> Result<(), ToolError> {
    let current_rel = current.strip_prefix(workspace).unwrap_or(current);
    let current_target = if current_rel.as_os_str().is_empty() {
        target.to_path_buf()
    } else {
        target.join(current_rel)
    };

    std::fs::create_dir_all(&current_target).map_err(io_internal)?;
    let rd = std::fs::read_dir(current).map_err(io_internal)?;
    let mut seen_names = HashSet::new();
    for entry in rd.flatten() {
        let source = entry.path();
        let file_name = entry.file_name();
        seen_names.insert(file_name.clone());
        let rel = source
            .strip_prefix(workspace)
            .unwrap_or(&source)
            .to_string_lossy()
            .into_owned();
        let dest = current_target.join(&file_name);
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(error) => {
                tracing::warn!(
                    service = "marketplace",
                    event = "sync.file_type_failed",
                    path = %source.display(),
                    error = %error,
                    "could not determine file type during sync; marking failed"
                );
                failed.push(rel);
                continue;
            }
        };
        if ft.is_symlink() {
            tracing::warn!(
                service = "marketplace",
                event = "sync.skipped",
                path = %source.display(),
                "skipping symlink during sync"
            );
            continue;
        }
        if ft.is_dir() {
            std::fs::create_dir_all(&dest).map_err(io_internal)?;
            sync_tree_to_target(workspace, target, &source, changed, skipped, removed, failed)?;
            continue;
        }

        if files_match(&source, &dest)? {
            skipped.push(rel);
            continue;
        }

        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(io_internal)?;
        }
        match std::fs::copy(&source, &dest) {
            Ok(_) => changed.push(rel),
            Err(_) => failed.push(rel),
        }
    }

    let target_rd = std::fs::read_dir(&current_target).map_err(io_internal)?;
    for entry in target_rd.flatten() {
        let file_name = entry.file_name();
        if seen_names.contains(&file_name) {
            continue;
        }
        let stale_path = entry.path();
        let stale_rel = stale_path
            .strip_prefix(target)
            .unwrap_or(&stale_path)
            .to_string_lossy()
            .into_owned();
        let removal = if stale_path.is_dir() {
            std::fs::remove_dir_all(&stale_path)
        } else {
            std::fs::remove_file(&stale_path)
        };
        match removal {
            Ok(()) => removed.push(stale_rel),
            Err(_) => failed.push(stale_rel),
        }
    }

    Ok(())
}

fn preview_tree_sync(workspace: &Path, target: &Path, current: &Path) -> Result<SyncPreview, ToolError> {
    let current_rel = current.strip_prefix(workspace).unwrap_or(current);
    let current_target = if current_rel.as_os_str().is_empty() {
        target.to_path_buf()
    } else {
        target.join(current_rel)
    };

    let mut preview = SyncPreview::default();
    let rd = std::fs::read_dir(current).map_err(io_internal)?;
    let mut seen_names = HashSet::new();
    for entry in rd.flatten() {
        let source = entry.path();
        let file_name = entry.file_name();
        seen_names.insert(file_name.clone());
        let rel = source
            .strip_prefix(workspace)
            .unwrap_or(&source)
            .to_string_lossy()
            .into_owned();
        let dest = current_target.join(&file_name);
        let ft = match entry.file_type() {
            Ok(ft) => ft,
            Err(error) => {
                tracing::warn!(
                    service = "marketplace",
                    event = "preview.file_type_failed",
                    path = %source.display(),
                    error = %error,
                    rel = %rel,
                    "could not determine file type during preview; entry will be absent from preview"
                );
                continue;
            }
        };
        if ft.is_symlink() {
            tracing::warn!(
                service = "marketplace",
                event = "preview.skipped",
                path = %source.display(),
                "skipping symlink during preview"
            );
            continue;
        }
        if ft.is_dir() {
            let nested = preview_tree_sync(workspace, target, &source)?;
            preview.changed.extend(nested.changed);
            preview.skipped.extend(nested.skipped);
            preview.removed.extend(nested.removed);
            preview.entries.extend(nested.entries);
            continue;
        }

        if files_match(&source, &dest)? {
            preview.skipped.push(rel);
            continue;
        }

        preview.changed.push(rel.clone());
        preview.entries.push(DeployPreviewEntry {
            path: rel,
            status: "changed",
            before_content: read_text_if_present(&dest),
            after_content: read_text_if_present(&source),
        });
    }

    if let Ok(target_rd) = std::fs::read_dir(&current_target) {
        for entry in target_rd.flatten() {
            let file_name = entry.file_name();
            if seen_names.contains(&file_name) {
                continue;
            }
            let stale_path = entry.path();
            let stale_rel = stale_path
                .strip_prefix(target)
                .unwrap_or(&stale_path)
                .to_string_lossy()
                .into_owned();
            preview.removed.push(stale_rel);
            preview.entries.push(DeployPreviewEntry {
                path: stale_path
                    .strip_prefix(target)
                    .unwrap_or(&stale_path)
                    .to_string_lossy()
                    .into_owned(),
                status: "removed",
                before_content: read_text_if_present(&stale_path),
                after_content: None,
            });
        }
    }

    Ok(preview)
}

fn files_match(source: &Path, dest: &Path) -> Result<bool, ToolError> {
    let source_meta = std::fs::metadata(source).map_err(io_internal)?;
    let Ok(dest_meta) = std::fs::metadata(dest) else {
        return Ok(false);
    };
    if source_meta.len() != dest_meta.len() {
        return Ok(false);
    }
    let source_file = std::fs::File::open(source).map_err(io_internal)?;
    let dest_file = std::fs::File::open(dest).map_err(io_internal)?;
    let mut source_reader = BufReader::new(source_file);
    let mut dest_reader = BufReader::new(dest_file);
    let mut source_buf = [0_u8; 8192];
    let mut dest_buf = [0_u8; 8192];

    loop {
        let source_read = source_reader.read(&mut source_buf).map_err(io_internal)?;
        let dest_read = dest_reader.read(&mut dest_buf).map_err(io_internal)?;
        if source_read != dest_read {
            return Ok(false);
        }
        if source_read == 0 {
            return Ok(true);
        }
        if source_buf[..source_read] != dest_buf[..dest_read] {
            return Ok(false);
        }
    }
}

fn read_text_if_present(path: &Path) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

pub(crate) use super::dispatch::walk_artifacts;

/// Cross-platform home directory (checks `HOME` then `USERPROFILE`).
pub(crate) fn home_dir() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
}

/// Path to the Codex TOML config file (`~/.codex/config.toml`).
pub(crate) fn codex_config_path() -> Result<std::path::PathBuf, ToolError> {
    home_dir()
        .map(|h| h.join(".codex").join("config.toml"))
        .ok_or_else(|| io_internal("cannot determine home directory"))
}

/// Root of the Codex cache directory (`~/.codex/cache/`).
pub(crate) fn codex_cache_root() -> Option<std::path::PathBuf> {
    home_dir().map(|h| h.join(".codex").join("cache"))
}

pub(crate) fn io_internal(error: impl std::fmt::Display) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: error.to_string(),
    }
}

#[cfg(test)]
pub(super) fn with_test_plugins_root<T>(home: &Path, run: impl FnOnce() -> T) -> T {
    let _guard = TEST_PLUGINS_ROOT_LOCK.lock().unwrap();
    let plugins_root = home.join(".claude").join("plugins");
    let previous = {
        let mut slot = TEST_PLUGINS_ROOT_OVERRIDE.lock().unwrap();
        std::mem::replace(&mut *slot, Some(plugins_root))
    };
    let result = run();
    let mut slot = TEST_PLUGINS_ROOT_OVERRIDE.lock().unwrap();
    *slot = previous;
    result
}
