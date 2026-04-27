//! Revision save and list operations for the stash service.
//!
//! A revision is an immutable snapshot of a component's workspace content.
//! The content digest is SHA-256 of all file contents concatenated in
//! sorted-by-relative-path order.

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use lab_apis::stash::types::{StashRevision, StashWorkspaceShape};

use crate::dispatch::error::ToolError;
use crate::dispatch::stash::store::StashStore;

// ── Walk helpers ──────────────────────────────────────────────────────────────

/// Walk `dir` and collect `(relative_path, absolute_path)` for every regular
/// file, sorted by relative path for deterministic digest computation.
///
/// Rejects symlinks. Returns `symlink_rejected` error on encounter.
fn walk_files_sorted(dir: &Path) -> Result<Vec<(PathBuf, PathBuf)>, ToolError> {
    let mut entries: Vec<(PathBuf, PathBuf)> = Vec::new();
    collect_files(dir, dir, &mut entries)?;
    // Sort by relative path for deterministic digest.
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(entries)
}

fn collect_files(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), ToolError> {
    let read_dir = std::fs::read_dir(dir).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("read_dir `{}`: {e}", dir.display()),
    })?;
    for entry in read_dir {
        let entry = entry.map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("read_dir entry: {e}"),
        })?;
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path).map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("symlink_metadata `{}`: {e}", path.display()),
        })?;
        if meta.file_type().is_symlink() {
            return Err(ToolError::Sdk {
                sdk_kind: "symlink_rejected".into(),
                message: format!(
                    "symlink at `{}` rejected during revision save",
                    path.display()
                ),
            });
        }
        if meta.is_dir() {
            collect_files(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| ToolError::Sdk {
                    sdk_kind: "internal_error".into(),
                    message: format!("strip_prefix failed: {e}"),
                })?
                .to_path_buf();
            out.push((rel, path));
        }
    }
    Ok(())
}

// ── Content digest ────────────────────────────────────────────────────────────

/// Hash a single file and return its contents.
///
/// Used inside `spawn_blocking` tasks.
fn read_file_bytes(path: &Path) -> Result<Vec<u8>, ToolError> {
    std::fs::read(path).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("read `{}`: {e}", path.display()),
    })
}

/// Compute SHA-256 of all file contents in sorted order.
///
/// Concatenates raw bytes of each file (sorted by relative path) then hashes
/// the concatenation. Returns lowercase hex string.
fn compute_digest(files: &[(PathBuf, PathBuf)]) -> Result<String, ToolError> {
    let mut hasher = Sha256::new();
    for (_rel, abs) in files {
        let bytes = read_file_bytes(abs)?;
        hasher.update(&bytes);
    }
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

// ── Copy helpers ──────────────────────────────────────────────────────────────

/// Copy a file, creating parent directories as needed.
fn copy_file_to(src: &Path, dst: &Path) -> Result<(), ToolError> {
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("create_dir_all `{}`: {e}", parent.display()),
        })?;
    }
    std::fs::copy(src, dst).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("copy `{}` → `{}`: {e}", src.display(), dst.display()),
    })?;
    Ok(())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Snapshot the current workspace of `component_id` into a new revision.
///
/// - Generates a fresh revision ID (ULID).
/// - Walks the workspace, rejects symlinks.
/// - Computes SHA-256 content digest over all file contents sorted by path.
/// - Copies all workspace files to `revisions/<rev_id>/files/`.
/// - Writes `revisions/<rev_id>/meta.json`.
/// - Updates `head_revision_id` on the component record under an advisory lock.
///
/// # Errors
/// Returns `ToolError::Sdk` with `sdk_kind`:
/// - `not_found` — component does not exist
/// - `symlink_rejected` — symlink found during workspace walk
/// - `internal_error` — I/O failures
pub async fn save_revision(
    store: &StashStore,
    component_id: &str,
    label: Option<&str>,
) -> Result<StashRevision, ToolError> {
    let component_id = component_id.to_string();
    let label = label.map(str::to_string);
    let store = store.clone();

    tokio::task::spawn_blocking(move || {
        save_revision_blocking(&store, &component_id, label.as_deref())
    })
    .await
    .map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("spawn_blocking panicked: {e}"),
    })?
}

fn save_revision_blocking(
    store: &StashStore,
    component_id: &str,
    label: Option<&str>,
) -> Result<StashRevision, ToolError> {
    // Load component.
    let component = store
        .read_component(component_id)?
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("component `{component_id}` not found"),
        })?;

    // Collect workspace files.
    let workspace_dir = store.workspace_dir(component_id);

    let (file_entries, file_count, unix_mode) = if component.workspace_shape
        == StashWorkspaceShape::File
    {
        // Single-file workspace: the workspace dir contains exactly one file.
        let filename = component
            .workspace_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("file");
        let ws_file = store.workspace_path(component_id, StashWorkspaceShape::File, Some(filename));
        // Validate no symlink.
        let meta = std::fs::symlink_metadata(&ws_file).map_err(|e| ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("symlink_metadata `{}`: {e}", ws_file.display()),
        })?;
        if meta.file_type().is_symlink() {
            return Err(ToolError::Sdk {
                sdk_kind: "symlink_rejected".into(),
                message: format!("symlink at `{}`", ws_file.display()),
            });
        }
        let rel = PathBuf::from(filename);
        let entries = vec![(rel, ws_file)];
        let mode = component.unix_mode;
        (entries, 1_usize, mode)
    } else {
        // Directory-shaped: walk workspace dir.
        let entries = walk_files_sorted(&workspace_dir)?;
        let count = entries.len();
        (entries, count, None)
    };

    // Compute content digest (all file bytes concatenated in sorted order).
    let content_digest = compute_digest(&file_entries)?;

    // Generate revision ID.
    let rev_id = ulid::Ulid::new().to_string().to_lowercase();

    // Copy files to revision snapshot directory.
    let files_dst = store.revision_files_path(&rev_id);
    for (rel, abs_src) in &file_entries {
        let dst = files_dst.join(rel);
        copy_file_to(abs_src, &dst)?;
    }

    // Write meta.json and update head_revision_id under component lock.
    let now = jiff::Timestamp::now().to_string();
    let revision = StashRevision {
        id: rev_id.clone(),
        component_id: component_id.to_string(),
        label: label.map(str::to_string),
        content_digest,
        created_at: now,
        file_count,
        unix_mode,
    };

    store.write_revision_meta(&revision)?;

    // Update component head_revision_id under lock.
    store.with_component_lock(component_id, || {
        let mut comp = store
            .read_component(component_id)?
            .ok_or_else(|| ToolError::Sdk {
                sdk_kind: "not_found".into(),
                message: format!("component `{component_id}` not found during head update"),
            })?;
        comp.head_revision_id = Some(rev_id.clone());
        comp.updated_at = jiff::Timestamp::now().to_string();
        store.write_component(&comp)
    })?;

    Ok(revision)
}

/// List all revisions for a component, in no particular order.
///
/// No lock is acquired; concurrent reads are safe.
///
/// # Errors
/// - `not_found` — component does not exist
pub fn list_revisions(
    store: &StashStore,
    component_id: &str,
) -> Result<Vec<StashRevision>, ToolError> {
    // Verify component exists.
    store
        .read_component(component_id)?
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("component `{component_id}` not found"),
        })?;

    store.list_revisions_for(component_id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use lab_apis::stash::types::{StashComponent, StashComponentKind, StashWorkspaceShape};
    use tempfile::tempdir;

    fn make_store() -> (StashStore, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let store = StashStore::new(dir.path().to_path_buf());
        store.ensure_dirs().expect("ensure_dirs");
        (store, dir)
    }

    fn write_dir_component(store: &StashStore, id: &str) -> StashComponent {
        let ws_dir = store.workspace_dir(id);
        std::fs::create_dir_all(&ws_dir).unwrap();
        std::fs::write(ws_dir.join("main.ts"), b"export const x = 1;").unwrap();
        std::fs::write(ws_dir.join("SKILL.md"), b"# Skill").unwrap();

        let comp = StashComponent {
            id: id.to_string(),
            kind: StashComponentKind::Skill,
            name: "test-skill".to_string(),
            label: None,
            head_revision_id: None,
            origin: None,
            workspace_root: ws_dir.clone(),
            workspace_shape: StashWorkspaceShape::Directory,
            unix_mode: None,
            created_at: "2026-04-26T12:00:00Z".to_string(),
            updated_at: "2026-04-26T12:00:00Z".to_string(),
        };
        store.write_component(&comp).unwrap();
        comp
    }

    fn write_file_component(store: &StashStore, id: &str) -> StashComponent {
        let ws_dir = store.workspace_dir(id);
        std::fs::create_dir_all(&ws_dir).unwrap();
        let ws_file = ws_dir.join("settings.json");
        std::fs::write(&ws_file, b"{\"key\": \"value\"}").unwrap();

        let comp = StashComponent {
            id: id.to_string(),
            kind: StashComponentKind::Settings,
            name: "test-settings".to_string(),
            label: None,
            head_revision_id: None,
            origin: None,
            workspace_root: ws_file.clone(),
            workspace_shape: StashWorkspaceShape::File,
            unix_mode: None,
            created_at: "2026-04-26T12:00:00Z".to_string(),
            updated_at: "2026-04-26T12:00:00Z".to_string(),
        };
        store.write_component(&comp).unwrap();
        comp
    }

    #[test]
    fn save_revision_dir_component() {
        let (store, _dir) = make_store();
        write_dir_component(&store, "comp-01");

        let rev = save_revision_blocking(&store, "comp-01", Some("v1")).unwrap();
        assert_eq!(rev.component_id, "comp-01");
        assert_eq!(rev.file_count, 2);
        assert!(!rev.content_digest.is_empty());
        assert_eq!(rev.label.as_deref(), Some("v1"));

        // head_revision_id should be updated.
        let comp = store.read_component("comp-01").unwrap().unwrap();
        assert_eq!(comp.head_revision_id.as_deref(), Some(rev.id.as_str()));

        // Revision files should exist.
        let files_dir = store.revision_files_path(&rev.id);
        assert!(files_dir.join("main.ts").exists());
        assert!(files_dir.join("SKILL.md").exists());
    }

    #[test]
    fn save_revision_file_component() {
        let (store, _dir) = make_store();
        write_file_component(&store, "comp-02");

        let rev = save_revision_blocking(&store, "comp-02", None).unwrap();
        assert_eq!(rev.file_count, 1);
        assert!(!rev.content_digest.is_empty());

        // Snapshot file should exist.
        let files_dir = store.revision_files_path(&rev.id);
        assert!(files_dir.join("settings.json").exists());
    }

    #[test]
    fn save_revision_not_found_error() {
        let (store, _dir) = make_store();
        let err = save_revision_blocking(&store, "nonexistent", None).unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn list_revisions_returns_saved_revisions() {
        let (store, _dir) = make_store();
        write_dir_component(&store, "comp-03");

        save_revision_blocking(&store, "comp-03", Some("r1")).unwrap();
        save_revision_blocking(&store, "comp-03", Some("r2")).unwrap();

        let revs = list_revisions(&store, "comp-03").unwrap();
        assert_eq!(revs.len(), 2);
    }

    #[test]
    fn list_revisions_not_found_error() {
        let (store, _dir) = make_store();
        let err = list_revisions(&store, "nonexistent").unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[test]
    fn digest_is_deterministic() {
        let (store, _dir) = make_store();
        write_dir_component(&store, "comp-04");

        let rev1 = save_revision_blocking(&store, "comp-04", None).unwrap();
        let rev2 = save_revision_blocking(&store, "comp-04", None).unwrap();
        assert_eq!(rev1.content_digest, rev2.content_digest);
    }
}
