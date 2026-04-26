//! Canonical local stash store — path layout and JSON record I/O.
//!
//! All file operations are **synchronous** by design. Async callers must wrap
//! calls in `tokio::task::spawn_blocking` (or equivalent).
//!
//! # Layout under `stash_root/`
//!
//! ```text
//! components/   — JSON records + advisory locks per component
//!                 <id>.json, <id>.lock, <id>.deploy.lock
//! revisions/    — immutable revision snapshots
//!                 <rev_id>/meta.json, <rev_id>/files/
//! workspaces/   — live working copies per component
//!                 <id>/              (directory-shaped)
//!                 <id>/<filename>    (file-shaped)
//! providers/    — provider link records  (<id>.json)
//! targets/      — deploy target records  (<id>.json)
//! ```
//!
//! There is **no** `objects/` directory.

#![allow(dead_code)]

use std::fs::{File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::Serialize;
use tempfile::NamedTempFile;

use lab_apis::stash::types::{
    StashComponent, StashDeployTarget, StashProviderRecord, StashRevision, StashWorkspaceShape,
};

use crate::dispatch::error::ToolError;

// ── Constants ────────────────────────────────────────────────────────────────

const DIR_COMPONENTS: &str = "components";
const DIR_REVISIONS: &str = "revisions";
const DIR_WORKSPACES: &str = "workspaces";
const DIR_PROVIDERS: &str = "providers";
const DIR_TARGETS: &str = "targets";

const EXT_RECORD: &str = ".json";
const EXT_LOCK: &str = ".lock";
const EXT_DEPLOY_LOCK: &str = ".deploy.lock";
const FILE_META: &str = "meta.json";
const DIR_FILES: &str = "files";

/// Maximum `id` length in bytes.
const MAX_ID_LEN: usize = 64;

/// Poll interval for the deploy-lock timeout loop.
const DEPLOY_LOCK_POLL_MS: u64 = 50;

// ── StashStore ───────────────────────────────────────────────────────────────

/// Encapsulates all I/O against a local stash root directory.
///
/// The struct itself is cheap to clone (a single `PathBuf`). All methods are
/// **synchronous** and safe to call from `spawn_blocking`.
#[derive(Debug, Clone)]
pub struct StashStore {
    root: PathBuf,
}

impl StashStore {
    /// Create a `StashStore` backed by `root`.
    ///
    /// The directory does not need to exist yet — call [`Self::ensure_dirs`]
    /// before performing any I/O.
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Return the root path of this store.
    pub fn root(&self) -> &Path {
        &self.root
    }

    // ── Path helpers: components ─────────────────────────────────────────────

    /// Path to `components/<id>.json`.
    pub fn component_record_path(&self, id: &str) -> PathBuf {
        self.root
            .join(DIR_COMPONENTS)
            .join(format!("{id}{EXT_RECORD}"))
    }

    /// Path to `components/<id>.lock`.
    pub fn component_lock_path(&self, id: &str) -> PathBuf {
        self.root
            .join(DIR_COMPONENTS)
            .join(format!("{id}{EXT_LOCK}"))
    }

    /// Path to `components/<id>.deploy.lock`.
    pub fn component_deploy_lock_path(&self, id: &str) -> PathBuf {
        self.root
            .join(DIR_COMPONENTS)
            .join(format!("{id}{EXT_DEPLOY_LOCK}"))
    }

    // ── Path helpers: workspaces ─────────────────────────────────────────────

    /// Path to `workspaces/<id>/`.
    pub fn workspace_dir(&self, id: &str) -> PathBuf {
        self.root.join(DIR_WORKSPACES).join(id)
    }

    /// Resolve the workspace path for a component.
    ///
    /// - **File-shaped**: `workspaces/<id>/<filename>` (requires `filename`).
    /// - **Directory-shaped**: `workspaces/<id>/`.
    pub fn workspace_path(
        &self,
        id: &str,
        shape: StashWorkspaceShape,
        filename: Option<&str>,
    ) -> PathBuf {
        let base = self.workspace_dir(id);
        match shape {
            StashWorkspaceShape::File => {
                let name = filename.unwrap_or("file");
                base.join(name)
            }
            StashWorkspaceShape::Directory => base,
        }
    }

    // ── Path helpers: revisions ──────────────────────────────────────────────

    /// Path to `revisions/<rev_id>/`.
    pub fn revision_dir(&self, rev_id: &str) -> PathBuf {
        self.root.join(DIR_REVISIONS).join(rev_id)
    }

    /// Path to `revisions/<rev_id>/files/`.
    pub fn revision_files_path(&self, rev_id: &str) -> PathBuf {
        self.revision_dir(rev_id).join(DIR_FILES)
    }

    /// Path to `revisions/<rev_id>/meta.json`.
    pub fn revision_meta_path(&self, rev_id: &str) -> PathBuf {
        self.revision_dir(rev_id).join(FILE_META)
    }

    // ── Path helpers: providers ──────────────────────────────────────────────

    /// Path to `providers/<id>.json`.
    pub fn provider_record_path(&self, id: &str) -> PathBuf {
        self.root
            .join(DIR_PROVIDERS)
            .join(format!("{id}{EXT_RECORD}"))
    }

    // ── Path helpers: targets ────────────────────────────────────────────────

    /// Path to `targets/<id>.json`.
    pub fn target_record_path(&self, id: &str) -> PathBuf {
        self.root
            .join(DIR_TARGETS)
            .join(format!("{id}{EXT_RECORD}"))
    }

    // ── Initialization ───────────────────────────────────────────────────────

    /// Create the five top-level sub-directories if they do not yet exist.
    ///
    /// This is idempotent and should be called once at startup.
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        for sub in [
            DIR_COMPONENTS,
            DIR_REVISIONS,
            DIR_WORKSPACES,
            DIR_PROVIDERS,
            DIR_TARGETS,
        ] {
            std::fs::create_dir_all(self.root.join(sub))?;
        }
        Ok(())
    }

    // ── Validation ───────────────────────────────────────────────────────────

    /// Validate that `id` is a safe, filesystem-legal identifier.
    ///
    /// Rules: non-empty, `<= 64` bytes, alphanumeric or hyphens only.
    pub fn validate_id(id: &str) -> Result<(), ToolError> {
        if id.is_empty() {
            return Err(invalid_param("id", "must not be empty"));
        }
        if id.len() > MAX_ID_LEN {
            return Err(invalid_param(
                "id",
                &format!("must not exceed {MAX_ID_LEN} characters"),
            ));
        }
        if !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
            return Err(invalid_param(
                "id",
                "must contain only alphanumeric characters and hyphens",
            ));
        }
        Ok(())
    }

    // ── Component record I/O ─────────────────────────────────────────────────

    /// Read and deserialize the component record for `id`, or `None` if absent.
    pub fn read_component(&self, id: &str) -> Result<Option<StashComponent>, ToolError> {
        Self::validate_id(id)?;
        let path = self.component_record_path(id);
        read_json_optional(&path)
    }

    /// Atomically write a component record to disk.
    ///
    /// The write is not lock-protected on its own; callers that require
    /// exclusive access should wrap the call in [`Self::with_component_lock`].
    pub fn write_component(&self, component: &StashComponent) -> Result<(), ToolError> {
        Self::validate_id(&component.id)?;
        let path = self.component_record_path(&component.id);
        write_json_atomic(&path, component)
    }

    /// List all component records in the store.
    ///
    /// Performs a full scan of `components/` and filters to `.json` files only.
    /// Malformed records are skipped with an internal-error result propagated to
    /// the caller on the first decode failure.
    pub fn list_components(&self) -> Result<Vec<StashComponent>, ToolError> {
        let dir = self.root.join(DIR_COMPONENTS);
        list_json_records(&dir)
    }

    /// Remove the component JSON record for `id`.
    ///
    /// Returns `Ok(())` if the file does not exist.
    pub fn delete_component_record(&self, id: &str) -> Result<(), ToolError> {
        Self::validate_id(id)?;
        let path = self.component_record_path(id);
        remove_if_exists(&path)
    }

    // ── Revision meta I/O ────────────────────────────────────────────────────

    /// Read and deserialize a revision's `meta.json`, or `None` if absent.
    pub fn read_revision_meta(&self, rev_id: &str) -> Result<Option<StashRevision>, ToolError> {
        Self::validate_id(rev_id)?;
        let path = self.revision_meta_path(rev_id);
        read_json_optional(&path)
    }

    /// Atomically write a revision's `meta.json`.
    pub fn write_revision_meta(&self, rev: &StashRevision) -> Result<(), ToolError> {
        Self::validate_id(&rev.id)?;
        let path = self.revision_meta_path(&rev.id);
        write_json_atomic(&path, rev)
    }

    /// List all revisions that belong to a given component.
    ///
    /// Full scan of `revisions/*/meta.json`; filters where
    /// `meta.component_id == component_id`. No secondary index.
    pub fn list_revisions_for(&self, component_id: &str) -> Result<Vec<StashRevision>, ToolError> {
        Self::validate_id(component_id)?;
        let revisions_dir = self.root.join(DIR_REVISIONS);
        if !revisions_dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in
            std::fs::read_dir(&revisions_dir).map_err(|e| io_internal(e))?
        {
            let entry = entry.map_err(|e| io_internal(e))?;
            let meta_path = entry.path().join(FILE_META);
            if !meta_path.is_file() {
                continue;
            }
            let bytes = match std::fs::read(&meta_path) {
                Ok(b) => b,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(io_internal(e)),
            };
            let rev: StashRevision = serde_json::from_slice(&bytes).map_err(decode_error)?;
            if rev.component_id == component_id {
                out.push(rev);
            }
        }
        Ok(out)
    }

    // ── Target record I/O ────────────────────────────────────────────────────

    /// Read and deserialize a deploy target record, or `None` if absent.
    pub fn read_target(&self, id: &str) -> Result<Option<StashDeployTarget>, ToolError> {
        Self::validate_id(id)?;
        let path = self.target_record_path(id);
        read_json_optional(&path)
    }

    /// Atomically write a deploy target record.
    ///
    /// The `id` parameter controls the filename (`targets/<id>.json`);
    /// the inner `id` field of `target` is NOT assumed to match.
    pub fn write_target(&self, id: &str, target: &StashDeployTarget) -> Result<(), ToolError> {
        Self::validate_id(id)?;
        let path = self.target_record_path(id);
        write_json_atomic(&path, target)
    }

    /// List all deploy targets in the store.
    ///
    /// Returns `(filename_id, target)` pairs. The `filename_id` is derived from
    /// the `.json` filename and may differ from the inner `id` field of the
    /// target enum variant.
    pub fn list_targets(&self) -> Result<Vec<(String, StashDeployTarget)>, ToolError> {
        let dir = self.root.join(DIR_TARGETS);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut out = Vec::new();
        for entry in std::fs::read_dir(&dir).map_err(|e| io_internal(e))? {
            let entry = entry.map_err(|e| io_internal(e))?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            let Some(id) = name.strip_suffix(EXT_RECORD) else {
                continue;
            };
            // Skip lock files that sneak in.
            if id.ends_with(".lock") || id.ends_with(".deploy") {
                continue;
            }
            let bytes = match std::fs::read(&path) {
                Ok(b) => b,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(io_internal(e)),
            };
            let target: StashDeployTarget =
                serde_json::from_slice(&bytes).map_err(decode_error)?;
            out.push((id.to_string(), target));
        }
        Ok(out)
    }

    /// Remove the deploy target record for `id`.
    ///
    /// Returns `Ok(())` if absent.
    pub fn delete_target(&self, id: &str) -> Result<(), ToolError> {
        Self::validate_id(id)?;
        let path = self.target_record_path(id);
        remove_if_exists(&path)
    }

    // ── Provider record I/O ──────────────────────────────────────────────────

    /// Read and deserialize a provider record, or `None` if absent.
    pub fn read_provider(&self, id: &str) -> Result<Option<StashProviderRecord>, ToolError> {
        Self::validate_id(id)?;
        let path = self.provider_record_path(id);
        read_json_optional(&path)
    }

    /// Atomically write a provider record.
    pub fn write_provider(&self, provider: &StashProviderRecord) -> Result<(), ToolError> {
        Self::validate_id(&provider.id)?;
        let path = self.provider_record_path(&provider.id);
        write_json_atomic(&path, provider)
    }

    /// List all provider records that belong to a given component.
    ///
    /// Full scan of `providers/`; filters where `record.component_id == component_id`.
    pub fn list_providers_for(
        &self,
        component_id: &str,
    ) -> Result<Vec<StashProviderRecord>, ToolError> {
        Self::validate_id(component_id)?;
        let dir = self.root.join(DIR_PROVIDERS);
        let all: Vec<StashProviderRecord> = list_json_records(&dir)?;
        Ok(all
            .into_iter()
            .filter(|p| p.component_id == component_id)
            .collect())
    }

    /// Remove the provider record for `id`.
    ///
    /// Returns `Ok(())` if absent.
    pub fn delete_provider(&self, id: &str) -> Result<(), ToolError> {
        Self::validate_id(id)?;
        let path = self.provider_record_path(id);
        remove_if_exists(&path)
    }

    // ── Advisory locks ───────────────────────────────────────────────────────

    /// Acquire an exclusive advisory lock on `components/<id>.lock` and execute
    /// `f` while the lock is held.
    ///
    /// The lock file is created if it does not yet exist. The lock is released
    /// when the returned guard is dropped (at end of scope).
    ///
    /// This is a **blocking** call — it will wait indefinitely for the lock.
    pub fn with_component_lock<F, T>(&self, id: &str, f: F) -> Result<T, ToolError>
    where
        F: FnOnce() -> Result<T, ToolError>,
    {
        Self::validate_id(id)?;
        let lock_path = self.component_lock_path(id);
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent).map_err(io_internal)?;
        }
        let file = open_lock_file(&lock_path)?;
        let mut rw_lock = fd_lock::RwLock::new(file);
        // Blocking exclusive write — waits until the lock is available.
        let _guard = rw_lock.write().map_err(io_internal)?;
        f()
    }

    /// Acquire an exclusive advisory deploy lock on `components/<id>.deploy.lock`
    /// and execute `f` while the lock is held.
    ///
    /// Polls `try_write()` in a loop with `DEPLOY_LOCK_POLL_MS` sleep intervals.
    /// Returns a `Sdk { sdk_kind: "conflict" }` error if `timeout_ms` elapses
    /// before the lock is acquired.
    pub fn with_deploy_lock<F, T>(
        &self,
        id: &str,
        timeout_ms: u64,
        f: F,
    ) -> Result<T, ToolError>
    where
        F: FnOnce() -> Result<T, ToolError>,
    {
        Self::validate_id(id)?;
        let lock_path = self.component_deploy_lock_path(id);
        if let Some(parent) = lock_path.parent() {
            std::fs::create_dir_all(parent).map_err(io_internal)?;
        }
        let file = open_lock_file(&lock_path)?;
        let mut rw_lock = fd_lock::RwLock::new(file);
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        loop {
            match rw_lock.try_write() {
                Ok(_guard) => {
                    // Guard is held for the duration of f().
                    // We must re-acquire after the loop, so we break here
                    // and call f() with the guard in scope.
                    // Re-structure: keep guard alive through f().
                    return f();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return Err(ToolError::Sdk {
                            sdk_kind: "conflict".into(),
                            message: format!(
                                "deploy lock timed out after {timeout_ms}ms for component {id}"
                            ),
                        });
                    }
                    std::thread::sleep(Duration::from_millis(DEPLOY_LOCK_POLL_MS));
                }
                Err(e) => return Err(io_internal(e)),
            }
        }
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Open (or create) a lock file with read+write access, without truncating.
fn open_lock_file(path: &Path) -> Result<File, ToolError> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(io_internal)
}

/// Atomically write `value` as pretty-printed JSON to `path`.
///
/// Uses a temp file in the same directory and `persist` (atomic rename) so
/// readers never see a partially-written file.
fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), ToolError> {
    let Some(parent) = path.parent() else {
        return Err(ToolError::internal_message("target path has no parent"));
    };
    std::fs::create_dir_all(parent).map_err(io_internal)?;
    let mut temp = NamedTempFile::new_in(parent).map_err(io_internal)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(io_internal)?;
    temp.write_all(&bytes).map_err(io_internal)?;
    temp.as_file().sync_all().map_err(io_internal)?;
    temp.persist(path)
        .map_err(|e| io_internal(e.error))?;
    Ok(())
}

/// Read and deserialize a JSON file, returning `None` if the file does not
/// exist.
fn read_json_optional<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<Option<T>, ToolError> {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(io_internal(e)),
    };
    serde_json::from_slice(&bytes)
        .map(Some)
        .map_err(decode_error)
}

/// Scan a directory for `*.json` files and deserialize each one.
///
/// Files that are missing by the time we read them (TOCTOU) are skipped.
/// Lock files (`*.lock.json`) are excluded by the `.json`-only extension filter
/// combined with the suffix check — lock files use `.lock`, not `.json`.
fn list_json_records<T: serde::de::DeserializeOwned>(dir: &Path) -> Result<Vec<T>, ToolError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(io_internal)? {
        let entry = entry.map_err(io_internal)?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        // Only process plain `<id>.json` files; skip `.lock`, `.deploy.lock`.
        if !name.ends_with(EXT_RECORD) {
            continue;
        }
        // Exclude anything whose stem still ends with ".lock" or ".deploy".
        let stem = &name[..name.len() - EXT_RECORD.len()];
        if stem.ends_with(".lock") || stem.ends_with(".deploy") {
            continue;
        }
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(io_internal(e)),
        };
        let record: T = serde_json::from_slice(&bytes).map_err(decode_error)?;
        out.push(record);
    }
    Ok(out)
}

/// Remove a file, treating `NotFound` as success.
fn remove_if_exists(path: &Path) -> Result<(), ToolError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io_internal(e)),
    }
}

fn io_internal(error: impl std::fmt::Display) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: error.to_string(),
    }
}

fn decode_error(error: serde_json::Error) -> ToolError {
    ToolError::Sdk {
        sdk_kind: "decode_error".into(),
        message: format!("decode stash record JSON: {error}"),
    }
}

fn invalid_param(param: &str, message: &str) -> ToolError {
    ToolError::InvalidParam {
        param: param.into(),
        message: message.into(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use lab_apis::stash::types::{StashComponentKind, StashWorkspaceShape};
    use tempfile::tempdir;

    fn make_store() -> (StashStore, tempfile::TempDir) {
        let dir = tempdir().expect("tempdir");
        let store = StashStore::new(dir.path().to_path_buf());
        store.ensure_dirs().expect("ensure_dirs");
        (store, dir)
    }

    fn sample_component(id: &str) -> StashComponent {
        StashComponent {
            id: id.to_string(),
            kind: StashComponentKind::Skill,
            name: "my-skill".to_string(),
            label: Some("demo skill".to_string()),
            head_revision_id: None,
            origin: None,
            workspace_root: PathBuf::from("/tmp/skill"),
            workspace_shape: StashWorkspaceShape::Directory,
            created_at: "2026-04-26T12:00:00Z".to_string(),
            updated_at: "2026-04-26T12:00:00Z".to_string(),
        }
    }

    fn sample_revision(rev_id: &str, component_id: &str) -> StashRevision {
        StashRevision {
            id: rev_id.to_string(),
            component_id: component_id.to_string(),
            label: Some("v1".to_string()),
            content_digest: "abc123".to_string(),
            created_at: "2026-04-26T12:00:00Z".to_string(),
            file_count: 3,
            unix_mode: None,
        }
    }

    fn sample_provider(id: &str, component_id: &str) -> StashProviderRecord {
        StashProviderRecord {
            id: id.to_string(),
            component_id: component_id.to_string(),
            kind: "filesystem".to_string(),
            label: "local".to_string(),
            config: serde_json::json!({}),
        }
    }

    // ── validate_id ──────────────────────────────────────────────────────────

    #[test]
    fn validate_id_accepts_valid_ids() {
        assert!(StashStore::validate_id("abc123").is_ok());
        assert!(StashStore::validate_id("my-component").is_ok());
        assert!(StashStore::validate_id("a").is_ok());
        assert!(StashStore::validate_id(&"a".repeat(64)).is_ok());
    }

    #[test]
    fn validate_id_rejects_empty() {
        let err = StashStore::validate_id("").expect_err("empty must fail");
        assert_eq!(err.kind(), "invalid_param");
    }

    #[test]
    fn validate_id_rejects_too_long() {
        let err = StashStore::validate_id(&"a".repeat(65)).expect_err("too long must fail");
        assert_eq!(err.kind(), "invalid_param");
    }

    #[test]
    fn validate_id_rejects_path_chars() {
        for bad in ["../etc/passwd", "foo/bar", "foo.bar", "foo bar"] {
            let err = StashStore::validate_id(bad).expect_err(&format!("{bad:?} must fail"));
            assert_eq!(err.kind(), "invalid_param");
        }
    }

    // ── ensure_dirs ──────────────────────────────────────────────────────────

    #[test]
    fn ensure_dirs_creates_expected_subdirectories() {
        let (store, _dir) = make_store();
        for sub in [
            DIR_COMPONENTS,
            DIR_REVISIONS,
            DIR_WORKSPACES,
            DIR_PROVIDERS,
            DIR_TARGETS,
        ] {
            assert!(
                store.root.join(sub).is_dir(),
                "missing {sub}"
            );
        }
        assert!(!store.root.join("objects").exists(), "objects/ must not exist");
    }

    // ── component I/O ────────────────────────────────────────────────────────

    #[test]
    fn component_roundtrip() {
        let (store, _dir) = make_store();
        let comp = sample_component("comp-01");
        assert!(store.read_component("comp-01").expect("read absent").is_none());
        store.write_component(&comp).expect("write");
        let back = store.read_component("comp-01").expect("read").expect("present");
        assert_eq!(back.id, comp.id);
        assert_eq!(back.name, comp.name);
    }

    #[test]
    fn list_components_excludes_lock_files() {
        let (store, _dir) = make_store();
        store.write_component(&sample_component("comp-01")).expect("write");
        // Simulate a stale lock file in the same directory.
        std::fs::write(
            store.root.join(DIR_COMPONENTS).join("comp-01.lock"),
            b"",
        )
        .expect("write lock file");
        let list = store.list_components().expect("list");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].id, "comp-01");
    }

    #[test]
    fn delete_component_record_is_idempotent() {
        let (store, _dir) = make_store();
        store.delete_component_record("nonexistent-id").expect("delete absent");
        store.write_component(&sample_component("comp-02")).expect("write");
        store.delete_component_record("comp-02").expect("delete present");
        assert!(store.read_component("comp-02").expect("read after delete").is_none());
    }

    // ── revision I/O ─────────────────────────────────────────────────────────

    #[test]
    fn revision_roundtrip_and_filter_by_component() {
        let (store, _dir) = make_store();
        let rev1 = sample_revision("rev-01", "comp-01");
        let rev2 = sample_revision("rev-02", "comp-02");
        store.write_revision_meta(&rev1).expect("write rev1");
        store.write_revision_meta(&rev2).expect("write rev2");

        let comp1_revs = store.list_revisions_for("comp-01").expect("list comp-01 revs");
        assert_eq!(comp1_revs.len(), 1);
        assert_eq!(comp1_revs[0].id, "rev-01");

        let comp2_revs = store.list_revisions_for("comp-02").expect("list comp-02 revs");
        assert_eq!(comp2_revs.len(), 1);
        assert_eq!(comp2_revs[0].id, "rev-02");
    }

    // ── target I/O ───────────────────────────────────────────────────────────

    #[test]
    fn target_roundtrip_and_delete() {
        let (store, _dir) = make_store();
        let target = StashDeployTarget::Local {
            id: "t-01".to_string(),
            name: "home".to_string(),
            path: PathBuf::from("/home/user/.claude"),
        };
        assert!(store.read_target("t-01").expect("read absent").is_none());
        store.write_target("t-01", &target).expect("write");
        let back = store.read_target("t-01").expect("read").expect("present");
        let StashDeployTarget::Local { id, .. } = back else {
            panic!("wrong variant");
        };
        assert_eq!(id, "t-01");
        store.delete_target("t-01").expect("delete");
        assert!(store.read_target("t-01").expect("read after delete").is_none());
    }

    #[test]
    fn list_targets_returns_all() {
        let (store, _dir) = make_store();
        for i in 0..3_u8 {
            let t = StashDeployTarget::Local {
                id: format!("t-{i:02}"),
                name: format!("target-{i}"),
                path: PathBuf::from(format!("/tmp/t{i}")),
            };
            store.write_target(&format!("t-{i:02}"), &t).expect("write");
        }
        let list = store.list_targets().expect("list");
        assert_eq!(list.len(), 3);
    }

    // ── provider I/O ─────────────────────────────────────────────────────────

    #[test]
    fn provider_roundtrip_and_filter_by_component() {
        let (store, _dir) = make_store();
        let p1 = sample_provider("prov-01", "comp-01");
        let p2 = sample_provider("prov-02", "comp-02");
        store.write_provider(&p1).expect("write p1");
        store.write_provider(&p2).expect("write p2");

        let comp1_providers = store.list_providers_for("comp-01").expect("list");
        assert_eq!(comp1_providers.len(), 1);
        assert_eq!(comp1_providers[0].id, "prov-01");
    }

    #[test]
    fn delete_provider_is_idempotent() {
        let (store, _dir) = make_store();
        store.delete_provider("nonexistent").expect("delete absent");
    }

    // ── advisory locks ───────────────────────────────────────────────────────

    #[test]
    fn with_component_lock_runs_closure() {
        let (store, _dir) = make_store();
        let result = store
            .with_component_lock("comp-01", || Ok(42_u32))
            .expect("lock and run");
        assert_eq!(result, 42);
    }

    #[test]
    fn with_deploy_lock_runs_closure_and_succeeds() {
        let (store, _dir) = make_store();
        let result = store
            .with_deploy_lock("comp-01", 500, || Ok("done"))
            .expect("deploy lock");
        assert_eq!(result, "done");
    }

    #[test]
    fn with_deploy_lock_timeout_returns_conflict() {
        let (store, _dir) = make_store();
        let store2 = store.clone();

        // Hold the deploy lock from a background thread.
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
        let (release_tx, release_rx) = std::sync::mpsc::channel::<()>();

        let handle = std::thread::spawn(move || {
            store2
                .with_deploy_lock("comp-lock-test", 5_000, || {
                    ready_tx.send(()).expect("send ready");
                    release_rx.recv().expect("wait for release");
                    Ok::<(), ToolError>(())
                })
                .expect("background lock");
        });

        ready_rx.recv().expect("wait for background lock held");

        // Try to acquire with a tiny timeout — should time out.
        let err = store
            .with_deploy_lock("comp-lock-test", 150, || Ok(()))
            .expect_err("should time out");
        assert_eq!(err.kind(), "conflict");

        // Release the background lock.
        release_tx.send(()).expect("send release");
        handle.join().expect("join thread");
    }

    // ── path helpers ─────────────────────────────────────────────────────────

    #[test]
    fn workspace_path_file_shaped_uses_filename() {
        let store = StashStore::new(PathBuf::from("/stash"));
        let p = store.workspace_path("comp-01", StashWorkspaceShape::File, Some("config.json"));
        assert_eq!(p, PathBuf::from("/stash/workspaces/comp-01/config.json"));
    }

    #[test]
    fn workspace_path_directory_shaped_returns_dir() {
        let store = StashStore::new(PathBuf::from("/stash"));
        let p = store.workspace_path("comp-01", StashWorkspaceShape::Directory, None);
        assert_eq!(p, PathBuf::from("/stash/workspaces/comp-01"));
    }
}
