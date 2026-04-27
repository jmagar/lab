//! Filesystem storage provider for stash.
//!
//! Copies revision content between the local stash store and another directory
//! on the same host (the "remote root"). This is useful for syncing stash
//! revisions to a NAS mount, a shared directory, or another path on the same
//! machine.
//!
//! # Configuration
//!
//! The provider record's `config` object must contain `"root"`: an absolute
//! path to the remote directory root.

use std::path::{Path, PathBuf};

use lab_apis::stash::types::{StashProviderRecord, StashRevision};

use crate::dispatch::error::ToolError;
use crate::dispatch::stash::provider::StashProvider;
use crate::dispatch::stash::store::StashStore;

/// Filesystem-backed storage provider.
///
/// Stores revision content under `<root>/<component_id>/<rev_id>/`.
pub struct FilesystemProvider {
    /// Absolute path to the remote filesystem root.
    root: PathBuf,
}

// Manual Debug impl — `root` is not a secret but we follow the convention
// of explicit Debug impls for all provider types to make future credential
// fields easier to redact.
impl std::fmt::Debug for FilesystemProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilesystemProvider")
            .field("root", &self.root)
            .finish()
    }
}

impl FilesystemProvider {
    /// Construct a `FilesystemProvider` from a provider record.
    ///
    /// Expects `record.config["root"]` to be a non-empty string.
    pub fn from_record(record: &StashProviderRecord) -> Result<Self, ToolError> {
        let root = record
            .config
            .get("root")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .ok_or_else(|| ToolError::InvalidParam {
                param: "config.root".into(),
                message: "filesystem provider requires config.root (non-empty string path)".into(),
            })?;
        Ok(Self { root })
    }

    /// Return the directory used to store all revisions for a component.
    fn component_remote_dir(&self, component_id: &str) -> PathBuf {
        self.root.join(component_id)
    }

    /// Return the directory for a specific revision.
    fn revision_remote_dir(&self, component_id: &str, rev_id: &str) -> PathBuf {
        self.component_remote_dir(component_id).join(rev_id)
    }
}

impl StashProvider for FilesystemProvider {
    fn kind(&self) -> &'static str {
        "filesystem"
    }

    fn push_revision(
        &self,
        store: &StashStore,
        component_id: &str,
        rev: &StashRevision,
    ) -> Result<(), ToolError> {
        let src = store.revision_files_path(&rev.id);
        let dst = self.revision_remote_dir(component_id, &rev.id);

        std::fs::create_dir_all(&dst).map_err(|e| ToolError::Sdk {
            sdk_kind: "sync_failed".into(),
            message: format!("create remote revision dir `{}`: {e}", dst.display()),
        })?;

        copy_dir_recursive(&src, &dst)
    }

    fn pull_latest(
        &self,
        store: &StashStore,
        component_id: &str,
    ) -> Result<Option<StashRevision>, ToolError> {
        let remote_dir = self.component_remote_dir(component_id);
        if !remote_dir.exists() {
            return Ok(None);
        }

        // Collect remote revision IDs (directory names).
        let remote_ids = list_subdirectory_names(&remote_dir)?;
        if remote_ids.is_empty() {
            return Ok(None);
        }

        // Use the lexicographically last ID (ULIDs sort chronologically).
        let latest_id = remote_ids
            .into_iter()
            .max()
            .expect("non-empty vec has a max");

        let remote_rev_dir = remote_dir.join(&latest_id);

        // Generate a fresh revision ID for the pulled content.
        let new_rev_id = ulid::Ulid::new().to_string().to_lowercase();
        let dst = store.revision_files_path(&new_rev_id);

        std::fs::create_dir_all(&dst).map_err(|e| ToolError::Sdk {
            sdk_kind: "sync_failed".into(),
            message: format!("create local revision dir `{}`: {e}", dst.display()),
        })?;

        copy_dir_recursive(&remote_rev_dir, &dst)?;

        // Count files and compute digest for the new revision.
        let file_count = count_files(&dst)?;
        let now = jiff::Timestamp::now().to_string();

        let rev = StashRevision {
            id: new_rev_id.clone(),
            component_id: component_id.to_string(),
            label: Some(format!("pulled from {latest_id}")),
            content_digest: String::new(), // No re-hash on pull; remote is authoritative.
            created_at: now,
            file_count,
            unix_mode: None,
        };

        store.write_revision_meta(&rev)?;
        Ok(Some(rev))
    }

    fn list_remote(&self, component_id: &str) -> Result<Vec<String>, ToolError> {
        let dir = self.component_remote_dir(component_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        list_subdirectory_names(&dir)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Recursively copy `src/` into `dst/`.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), ToolError> {
    if !src.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(src).map_err(|e| ToolError::Sdk {
        sdk_kind: "sync_failed".into(),
        message: format!("read_dir `{}`: {e}", src.display()),
    })? {
        let entry = entry.map_err(|e| ToolError::Sdk {
            sdk_kind: "sync_failed".into(),
            message: format!("read_dir entry: {e}"),
        })?;
        let src_path = entry.path();
        let rel = entry.file_name();
        let dst_path = dst.join(&rel);

        if src_path.is_dir() {
            std::fs::create_dir_all(&dst_path).map_err(|e| ToolError::Sdk {
                sdk_kind: "sync_failed".into(),
                message: format!("create dir `{}`: {e}", dst_path.display()),
            })?;
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path).map_err(|e| ToolError::Sdk {
                sdk_kind: "sync_failed".into(),
                message: format!(
                    "copy `{}` → `{}`: {e}",
                    src_path.display(),
                    dst_path.display()
                ),
            })?;
        }
    }
    Ok(())
}

/// List names of subdirectories under `dir`.
fn list_subdirectory_names(dir: &Path) -> Result<Vec<String>, ToolError> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| ToolError::Sdk {
        sdk_kind: "sync_failed".into(),
        message: format!("read_dir `{}`: {e}", dir.display()),
    })? {
        let entry = entry.map_err(|e| ToolError::Sdk {
            sdk_kind: "sync_failed".into(),
            message: format!("read_dir entry: {e}"),
        })?;
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                names.push(name.to_string());
            }
        }
    }
    Ok(names)
}

/// Recursively count regular files under `dir`.
fn count_files(dir: &Path) -> Result<usize, ToolError> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in std::fs::read_dir(dir).map_err(|e| ToolError::Sdk {
        sdk_kind: "sync_failed".into(),
        message: format!("read_dir `{}`: {e}", dir.display()),
    })? {
        let entry = entry.map_err(|e| ToolError::Sdk {
            sdk_kind: "sync_failed".into(),
            message: format!("read_dir entry: {e}"),
        })?;
        let path = entry.path();
        if path.is_dir() {
            count += count_files(&path)?;
        } else {
            count += 1;
        }
    }
    Ok(count)
}
