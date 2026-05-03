//! Shared path-safety helpers for dispatch modules that operate on the local
//! filesystem.
//!
//! # Contents
//!
//! - `reject_symlink` — consolidated from `dispatch/marketplace/stash_meta.rs`
//!   where it was a private function.  Stash dispatch (and future modules that
//!   walk user-supplied paths) import from here instead of duplicating the
//!   logic.
//! - `reject_system_path` — rejects paths inside well-known system directories
//!   that stash must never read from or write to.  Used by both
//!   `stash::import` (source validation) and `stash::service` (deploy target
//!   and deploy destination validation).
//!
//! # Intentionally omitted
//!
//! - `reject_path_traversal` lives in `dispatch/helpers.rs` and is already
//!   public there; callers import it from `helpers` directly.
//! - `ensure_target_within_write_root`: the only existing implementation lives
//!   in `node/install.rs` and is async + anyhow-based (not `ToolError`-based).
//!   Wave-3 `stash/store.rs` will add a synchronous version when it has
//!   concrete callers.

use std::path::{Path, PathBuf};

use crate::dispatch::error::ToolError;

// ── System-path denylist ──────────────────────────────────────────────────────

/// Paths that stash must never read from or write to, regardless of operator
/// configuration.  Checked after canonicalization so that symlinks and `..`
/// traversals cannot bypass the list.
///
/// Extended beyond the minimal FHS list to include common container / k8s
/// mount roots where the running process may have write access.
pub const SYSTEM_PATH_DENYLIST: &[&str] = &[
    // Core FHS system directories
    "/etc",
    "/usr",
    "/bin",
    "/sbin",
    "/lib",
    "/lib64",
    "/boot",
    "/dev",
    "/proc",
    "/sys",
    // Variable / runtime data — often writable, always sensitive
    "/var",
    "/run",
    // Privileged user homes
    "/root",
    // Sensitive but user-writable
    "/tmp",
    // User home trees — a broad block; legitimate stash paths live under
    // specific subdirectories, not at the tree root
    "/home",
    // Optional / mounted software
    "/opt",
    "/srv",
    // Common container / k8s mount roots (not in standard FHS)
    "/app",
    "/workspace",
    "/data",
    "/config",
    "/mnt",
    "/media",
    "/storage",
];

/// Reject a path that falls inside a known system directory.
///
/// `canonical` must be the result of `std::fs::canonicalize` (or a parent
/// canonicalization + filename rejoin) — it must not be a lexically-only
/// normalized path, because symlinks could otherwise bypass the check.
///
/// Returns `Ok(())` when the path is safe.
/// Returns `Err(ToolError::Sdk { sdk_kind: "path_traversal" })` when the path
/// is inside a system directory.
pub fn reject_system_path(canonical: &Path, original: &Path) -> Result<(), ToolError> {
    let canonical_str = canonical.to_string_lossy();
    for &system in SYSTEM_PATH_DENYLIST {
        if canonical_str == system || canonical_str.starts_with(&format!("{system}/")) {
            return Err(ToolError::Sdk {
                sdk_kind: "path_traversal".into(),
                message: format!(
                    "path `{}` resolves to a system directory (`{}`) and is not allowed",
                    original.display(),
                    system
                ),
            });
        }
    }
    Ok(())
}

/// Canonicalize `path` and then call [`reject_system_path`].
///
/// Returns `Err(ToolError::Sdk { sdk_kind: "path_traversal" })` when
/// canonicalization fails — the path cannot be verified safe if we cannot
/// resolve it.  This prevents the silent-fallback vulnerability where an
/// unreachable (or permission-denied) path bypasses the denylist.
pub fn canonicalize_and_reject_system_path(path: &Path) -> Result<PathBuf, ToolError> {
    // Canonicalize the path if it exists; otherwise canonicalize the nearest
    // existing ancestor and rejoin the remaining components.
    let canonical = if path.exists() {
        std::fs::canonicalize(path).map_err(|e| ToolError::Sdk {
            sdk_kind: "path_traversal".into(),
            message: format!(
                "cannot verify path `{}` is safe: canonicalize failed: {e}",
                path.display()
            ),
        })?
    } else if let Some(parent) = path.parent() {
        if parent == Path::new("") || !parent.exists() {
            // Cannot canonicalize — fail closed.
            return Err(ToolError::Sdk {
                sdk_kind: "path_traversal".into(),
                message: format!(
                    "cannot verify path `{}` is safe: parent directory does not exist",
                    path.display()
                ),
            });
        }
        let canonical_parent = std::fs::canonicalize(parent).map_err(|e| ToolError::Sdk {
            sdk_kind: "path_traversal".into(),
            message: format!(
                "cannot verify path `{}` is safe: canonicalize parent failed: {e}",
                path.display()
            ),
        })?;
        let file_name = path.file_name().unwrap_or_default();
        canonical_parent.join(file_name)
    } else {
        return Err(ToolError::Sdk {
            sdk_kind: "path_traversal".into(),
            message: format!(
                "cannot verify path `{}` is safe: no parent directory",
                path.display()
            ),
        });
    };

    reject_system_path(&canonical, path)?;
    Ok(canonical)
}

/// Reject a path that exists on disk as a symlink.
///
/// This is a **lstat-based** check — it does not follow the link. Callers that
/// need a post-canonicalize within-root guarantee must perform that check
/// separately (the TOCTOU window between `reject_symlink` and the actual I/O
/// operation is narrow but non-zero; treat it as defence-in-depth, not as the
/// sole guard).
///
/// Returns `ToolError::Sdk { sdk_kind: "not_found" }` when the path does not
/// exist, and `ToolError::internal_message` when the path *is* a symlink.
/// Returns `Ok(())` for regular files and directories.
pub fn reject_symlink(path: &Path) -> Result<(), ToolError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(ToolError::Sdk {
                sdk_kind: "not_found".into(),
                message: "path is missing".into(),
            });
        }
        Err(error) => {
            return Err(ToolError::Sdk {
                sdk_kind: "internal_error".into(),
                message: format!("lstat failed: {error}"),
            });
        }
    };
    if metadata.file_type().is_symlink() {
        return Err(ToolError::Sdk {
            sdk_kind: "symlink_rejected".into(),
            message: format!("refusing to operate on symlinked path `{}`", path.display()),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dispatch::helpers::reject_path_traversal;
    use tempfile::tempdir;

    #[test]
    fn reject_symlink_accepts_regular_file() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("regular.txt");
        std::fs::write(&file, b"hi").unwrap();
        assert!(reject_symlink(&file).is_ok());
    }

    #[test]
    fn reject_symlink_accepts_directory() {
        let dir = tempdir().unwrap();
        assert!(reject_symlink(dir.path()).is_ok());
    }

    #[test]
    fn reject_symlink_rejects_missing_path() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("missing");
        let err = reject_symlink(&missing).unwrap_err();
        assert_eq!(err.kind(), "not_found");
    }

    #[cfg(unix)]
    #[test]
    fn reject_symlink_rejects_symlink() {
        let dir = tempdir().unwrap();
        let target = dir.path().join("target.txt");
        std::fs::write(&target, b"hi").unwrap();
        let link = dir.path().join("link.txt");
        std::os::unix::fs::symlink(&target, &link).unwrap();
        let err = reject_symlink(&link).unwrap_err();
        assert_eq!(err.kind(), "symlink_rejected");
    }

    #[test]
    fn reject_path_traversal_rejects_dotdot() {
        let err = reject_path_traversal("../escape").unwrap_err();
        assert_eq!(err.kind(), "invalid_param");
    }

    #[test]
    fn reject_path_traversal_accepts_relative_normal() {
        assert!(reject_path_traversal("sub/path.txt").is_ok());
    }
}
