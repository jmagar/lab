//! Client / configuration for the workspace filesystem browser service.
//!
//! # Responsibility (Phase 1 scope — lab-f1t2.1)
//!
//! - Resolve the user-configured workspace root from `LAB_WORKSPACE_ROOT`.
//! - Canonicalize it and validate that it exists.
//! - Surface a structured error when the env var is unset so the caller
//!   (HTTP / MCP dispatcher) can return a clean `workspace_not_configured`
//!   envelope rather than a cryptic I/O failure.
//!
//! Later phases (2, 3) add the `GlobSet` deny-list builder here and the
//! thin wrapper that opens files via `openat2(RESOLVE_BENEATH | RESOLVE_NO_SYMLINKS)`.

use std::path::PathBuf;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Structured error for when the fs service cannot serve a request because
/// `LAB_WORKSPACE_ROOT` is unset or invalid.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "workspace_not_configured".to_string(),
        message: "LAB_WORKSPACE_ROOT is not set or does not point at an existing directory"
            .to_string(),
    }
}

/// Resolve the configured workspace root, canonicalize it, and verify it
/// exists and is a directory. Returns `None` when the env var is absent or
/// empty — the caller decides whether that's an error.
///
/// Called once at startup (from `AppState::from_env` in a future wire-up);
/// keep pure — no logging, no side effects.
pub fn resolve_workspace_root_from_env() -> Option<std::io::Result<PathBuf>> {
    let raw = env_non_empty("LAB_WORKSPACE_ROOT")?;
    Some(canonicalize_existing_dir(PathBuf::from(raw)))
}

/// Canonicalize a path and verify it names an existing directory. Fails
/// closed — a relative path, a non-existent path, or a path that resolves
/// to a non-directory all return `Err`.
fn canonicalize_existing_dir(path: PathBuf) -> std::io::Result<PathBuf> {
    if !path.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("LAB_WORKSPACE_ROOT must be absolute; got {}", path.display()),
        ));
    }
    let canonical = std::fs::canonicalize(&path)?;
    let meta = std::fs::metadata(&canonical)?;
    if !meta.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "LAB_WORKSPACE_ROOT is not a directory: {}",
                canonical.display()
            ),
        ));
    }
    Ok(canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn canonicalize_existing_dir_accepts_absolute_existing_dir() {
        let tmp = tempdir().expect("tempdir");
        let resolved = canonicalize_existing_dir(tmp.path().to_path_buf()).expect("ok");
        assert!(resolved.is_absolute());
        assert!(resolved.is_dir());
    }

    #[test]
    fn canonicalize_existing_dir_rejects_relative() {
        let err = canonicalize_existing_dir(PathBuf::from("relative/path")).expect_err("err");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn canonicalize_existing_dir_rejects_nonexistent() {
        let err = canonicalize_existing_dir(PathBuf::from("/nonexistent/lab-f1t2-probe"))
            .expect_err("err");
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }

    #[test]
    fn canonicalize_existing_dir_rejects_file_target() {
        let tmp = tempdir().expect("tempdir");
        let file = tmp.path().join("a-file");
        std::fs::write(&file, b"hi").expect("write");
        let err = canonicalize_existing_dir(file).expect_err("err");
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }
}
