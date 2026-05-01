//! Path resolution helpers for the `setup` dispatch service.
//!
//! Honors `LAB_HOME` for tests; defaults to `~/.lab/` in production.

use std::path::PathBuf;

/// Resolve the lab home directory: `$LAB_HOME` if set, else `$HOME/.lab/`.
#[must_use]
pub fn lab_home() -> PathBuf {
    if let Ok(home) = std::env::var("LAB_HOME")
        && !home.is_empty()
    {
        return PathBuf::from(home);
    }
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join(".lab")
}

#[must_use]
pub fn env_path() -> PathBuf {
    lab_home().join(".env")
}

#[must_use]
pub fn draft_path() -> PathBuf {
    lab_home().join(".env.draft")
}
