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
    pub const fn path(&self) -> &PathBuf {
        match self {
            Self::Local(p) | Self::Ssh { path: p, .. } => p,
        }
    }
}

/// The requested extract scan mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", content = "uri", rename_all = "snake_case")]
pub enum ScanTarget {
    /// Scan a concrete local or remote appdata root.
    Targeted(Uri),
    /// Scan the local SSH inventory for supported services.
    Fleet,
}

impl ScanTarget {
    /// Returns the targeted URI when this scan operates on a concrete root.
    #[must_use]
    pub const fn uri(&self) -> Option<&Uri> {
        match self {
            Self::Targeted(uri) => Some(uri),
            Self::Fleet => None,
        }
    }
}

impl From<Uri> for ScanTarget {
    fn from(value: Uri) -> Self {
        Self::Targeted(value)
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
        let (host, path) = s.split_once(":/").ok_or_else(|| ExtractError::InvalidUri {
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

/// Container/runtime provenance for a discovered service or warning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProvenance {
    /// Runtime object that produced the discovery, usually a container name.
    pub container_name: String,
    /// Runtime image name when available.
    pub image: Option<String>,
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
    /// SSH host or local source that produced this discovery.
    pub source_host: Option<String>,
    /// Host identity used to probe the endpoint.
    pub probe_host: Option<String>,
    /// Runtime/container provenance for this discovery.
    pub runtime: Option<RuntimeProvenance>,
    /// Whether the URL was verified as reachable.
    pub url_verified: bool,
}

/// One non-fatal issue encountered while scanning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractWarning {
    /// Service that produced the warning.
    pub service: String,
    /// Host associated with the warning when available.
    pub host: Option<String>,
    /// Runtime/container provenance for the warning.
    pub runtime: Option<RuntimeProvenance>,
    /// What went wrong.
    pub message: String,
}

/// Result of an `ExtractClient::scan()` call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractReport {
    /// The scan target that was requested.
    pub target: ScanTarget,
    /// Legacy targeted URI field preserved for compatibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<Uri>,
    /// Service names that were found and parsed successfully.
    pub found: Vec<String>,
    /// Per-service credentials (one entry per discovered app).
    pub creds: Vec<ServiceCreds>,
    /// Non-fatal warnings (corrupt files, missing fields, etc.).
    pub warnings: Vec<ExtractWarning>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targeted_report_serializes_target_metadata() {
        let report = ExtractReport {
            target: ScanTarget::Targeted("/mnt/appdata".parse().expect("uri")),
            uri: Some("/mnt/appdata".parse().expect("uri")),
            found: vec!["radarr".to_owned()],
            creds: vec![],
            warnings: vec![],
        };

        let value = serde_json::to_value(&report).expect("serialize report");

        assert_eq!(value["target"]["mode"], "targeted");
        assert_eq!(value["target"]["uri"], "/mnt/appdata");
        assert_eq!(value["uri"], "/mnt/appdata");
    }

    #[test]
    fn fleet_report_serializes_provenance_and_verification() {
        let report = ExtractReport {
            target: ScanTarget::Fleet,
            uri: None,
            found: vec!["radarr".to_owned()],
            creds: vec![ServiceCreds {
                service: "radarr".to_owned(),
                url: Some("http://100.64.0.12:7878".to_owned()),
                secret: None,
                env_field: "RADARR_API_KEY".to_owned(),
                source_host: Some("media-node".to_owned()),
                probe_host: Some("100.64.0.12".to_owned()),
                runtime: Some(RuntimeProvenance {
                    container_name: "radarr".to_owned(),
                    image: Some("lscr.io/linuxserver/radarr:latest".to_owned()),
                }),
                url_verified: true,
            }],
            warnings: vec![ExtractWarning {
                service: "plex".to_owned(),
                host: Some("media-node".to_owned()),
                runtime: Some(RuntimeProvenance {
                    container_name: "plex".to_owned(),
                    image: Some("plexinc/pms-docker:latest".to_owned()),
                }),
                message: "config root could not be resolved".to_owned(),
            }],
        };

        let value = serde_json::to_value(&report).expect("serialize report");

        assert_eq!(value["target"]["mode"], "fleet");
        assert!(value.get("uri").is_none());
        assert_eq!(value["creds"][0]["source_host"], "media-node");
        assert_eq!(value["creds"][0]["probe_host"], "100.64.0.12");
        assert_eq!(value["creds"][0]["runtime"]["container_name"], "radarr");
        assert_eq!(value["creds"][0]["url_verified"], true);
        assert_eq!(value["warnings"][0]["host"], "media-node");
        assert_eq!(value["warnings"][0]["runtime"]["container_name"], "plex");
    }
}
