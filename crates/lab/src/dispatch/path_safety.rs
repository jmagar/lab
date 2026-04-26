//! Shared path-safety helpers for dispatch modules that operate on the local
//! filesystem.
//!
//! # Contents
//!
//! - `reject_symlink` — consolidated from `dispatch/marketplace/stash_meta.rs`
//!   where it was a private function.  Stash dispatch (and future modules that
//!   walk user-supplied paths) import from here instead of duplicating the
//!   logic.
//!
//! # Intentionally omitted
//!
//! - `reject_path_traversal` lives in `dispatch/helpers.rs` and is already
//!   public there; callers import it from `helpers` directly.
//! - `ensure_target_within_write_root`: the only existing implementation lives
//!   in `node/install.rs` and is async + anyhow-based (not `ToolError`-based).
//!   Wave-3 `stash/store.rs` will add a synchronous version when it has
//!   concrete callers.

use std::path::Path;

use crate::dispatch::error::ToolError;

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
        return Err(ToolError::internal_message(
            "refusing to operate on symlinked path",
        ));
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
        assert_eq!(err.kind(), "internal_error");
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
