//! Public types for the extract module.

use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::error::ExtractError;

/// A path to scan for service configs. Either a local filesystem path or a
/// remote `host:/absolute/path` URI resolved through `~/.ssh/config`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Uri {
    /// `/home/jmagar/appdata` or `~/appdata`
    Local(PathBuf),
    /// `squirts:/mnt/appdata` — host is an ssh alias.
    Ssh {
        /// SSH host alias (resolved via `~/.ssh/config`).
        host: String,
        /// Absolute path on the remote.
        path: PathBuf,
    },
}

impl Uri {
    /// Returns the appdata-root path component, regardless of transport.
    #[must_use]
    pub fn path(&self) -> &PathBuf {
        match self {
            Self::Local(p) | Self::Ssh { path: p, .. } => p,
        }
    }
}

impl FromStr for Uri {
    type Err = ExtractError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Local path: starts with `/` or `~`
        if let Some(rest) = s.strip_prefix('~') {
            let home = std::env::var("HOME").map_err(|_| ExtractError::InvalidUri {
                input: s.to_owned(),
                reason: "$HOME not set",
            })?;
            return Ok(Self::Local(PathBuf::from(format!("{home}{rest}"))));
        }
        if s.starts_with('/') {
            return Ok(Self::Local(PathBuf::from(s)));
        }
        // SSH form: `host:/abs/path`
        let (host, path) = s.split_once(":/").ok_or(ExtractError::InvalidUri {
            input: s.to_owned(),
            reason: "expected '/abs/path', '~/path', or 'host:/abs/path'",
        })?;
        if host.is_empty() {
            return Err(ExtractError::InvalidUri {
                input: s.to_owned(),
                reason: "ssh host is empty",
            });
        }
        Ok(Self::Ssh {
            host: host.to_owned(),
            path: PathBuf::from(format!("/{path}")),
        })
    }
}

/// Credentials extracted from one app's config file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceCreds {
    /// Lowercase service name (`radarr`, `sonarr`, ...).
    pub service: String,
    /// Base URL inferred from `BindAddress`/`Port`/`UrlBase`/`EnableSsl`, if known.
    pub url: Option<String>,
    /// API key, token, or password depending on the service.
    pub secret: Option<String>,
    /// Field name to use when writing to `.env` (e.g. `RADARR_API_KEY`).
    pub env_field: String,
}

/// One non-fatal issue encountered while scanning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractWarning {
    /// Service that produced the warning.
    pub service: String,
    /// What went wrong.
    pub message: String,
}

/// Result of an `ExtractClient::scan()` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractReport {
    /// The URI that was scanned.
    pub uri: Uri,
    /// Service names that were found and parsed successfully.
    pub found: Vec<String>,
    /// Per-service credentials (one entry per discovered app).
    pub creds: Vec<ServiceCreds>,
    /// Non-fatal warnings (corrupt files, missing fields, etc.).
    pub warnings: Vec<ExtractWarning>,
}
