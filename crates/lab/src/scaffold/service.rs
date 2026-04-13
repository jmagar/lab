//! Types and validation for service scaffolding.

use std::fmt;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result alias used by scaffold helpers.
pub type Result<T> = std::result::Result<T, ScaffoldError>;

/// Scaffold target kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum ScaffoldKind {
    /// HTTP service with CLI, MCP, and API adapters.
    #[default]
    Http,
    /// Non-HTTP service that still participates in the onboarding contract.
    #[value(name = "non-http")]
    NonHttp,
}

/// Input configuration for service scaffolding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldConfig {
    pub service: String,
    #[serde(default)]
    pub kind: ScaffoldKind,
    pub repo_root: PathBuf,
}

/// One file change produced by the scaffold plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileOp {
    pub path: PathBuf,
    pub content: String,
}

/// Result produced by the scaffold command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScaffoldResult {
    pub service: String,
    pub kind: ScaffoldKind,
    pub dry_run: bool,
    pub planned: Vec<FileOp>,
    pub created: Vec<PathBuf>,
    pub modified: Vec<PathBuf>,
}

/// Errors produced by scaffold planning and execution.
#[derive(Debug, Error)]
pub enum ScaffoldError {
    #[error("invalid service name `{name}`: {message}")]
    InvalidServiceName { name: String, message: String },

    #[error("scaffold target `{path}` escapes repository root `{base}`")]
    InvalidTarget { path: PathBuf, base: PathBuf },

    #[error("io error while writing `{path}`")]
    Io { path: PathBuf, source: std::io::Error },

    #[error("invalid TOML while patching: {message}")]
    Toml { message: String },
}

impl ScaffoldError {
    pub fn io(path: PathBuf, source: std::io::Error) -> Self {
        Self::Io { path, source }
    }
}

impl fmt::Display for ScaffoldKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http => write!(f, "http"),
            Self::NonHttp => write!(f, "non-http"),
        }
    }
}

/// Validate a service name before any path construction or subprocess use.
pub fn validate_service_name(name: &str) -> Result<String> {
    let valid = matches!(name.as_bytes(), [first, rest @ ..]
        if first.is_ascii_lowercase()
            && !rest.is_empty()
            && rest.len() <= 63
            && rest.iter().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_'));
    if !valid {
        return Err(ScaffoldError::InvalidServiceName {
            name: name.to_string(),
            message: "must match ^[a-z][a-z0-9_]{1,63}$".into(),
        });
    }
    Ok(name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_service_name_accepts_valid_names() {
        assert!(validate_service_name("ab").is_ok());
        assert!(validate_service_name("my_service").is_ok());
        assert!(validate_service_name("foo123").is_ok());
    }

    #[test]
    fn validate_service_name_rejects_single_char() {
        assert!(validate_service_name("a").is_err());
    }

    #[test]
    fn validate_service_name_rejects_invalid() {
        assert!(validate_service_name("").is_err());
        assert!(validate_service_name("1foo").is_err()); // starts with digit
        assert!(validate_service_name("../escape").is_err()); // path traversal
        assert!(validate_service_name("MyService").is_err()); // uppercase
        assert!(validate_service_name("a-b").is_err()); // hyphen
    }
}

/// Validate that a scaffold target stays under the provided base directory.
pub fn validate_scaffold_target(name: &str, base: &Path) -> Result<PathBuf> {
    let base_canon = base
        .canonicalize()
        .map_err(|_| ScaffoldError::InvalidTarget {
            path: base.to_path_buf(),
            base: base.to_path_buf(),
        })?;
    let target = base.join(name);
    let parent = target.parent().unwrap_or(base);
    let parent_canon = parent.canonicalize().map_err(|_| ScaffoldError::InvalidTarget {
        path: target.clone(),
        base: base_canon.clone(),
    })?;
    if !parent_canon.starts_with(&base_canon) {
        return Err(ScaffoldError::InvalidTarget {
            path: target,
            base: base_canon,
        });
    }
    Ok(target)
}
