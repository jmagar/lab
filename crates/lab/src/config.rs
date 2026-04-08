//! Config loading for the `lab` binary.
//!
//! Order of precedence (highest wins):
//!   1. Process environment variables
//!   2. `~/.lab/.env` (loaded via `dotenvy`)
//!   3. `~/.config/lab/config.toml` (preferences, not secrets)
//!
//! Multi-instance services follow the `S_<LABEL>_URL` pattern: a service
//! like `unraid` reads `UNRAID_URL` as the default instance and
//! `UNRAID_NODE2_URL` as an additional instance labeled `node2`.

use std::{collections::HashMap, path::PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Fully-resolved `lab` configuration, assembled from env + TOML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LabConfig {
    /// Default output format for CLI commands that print tables.
    #[serde(default)]
    pub output: OutputPreferences,
    /// MCP server defaults.
    #[serde(default)]
    pub mcp: McpPreferences,
}

/// Table/json formatting defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputPreferences {
    /// Default format: `human` or `json`. Honored unless `--json` overrides.
    #[serde(default)]
    pub format: Option<String>,
}

/// MCP server defaults.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpPreferences {
    /// Default transport (`stdio` or `http`).
    #[serde(default)]
    pub transport: Option<String>,
    /// Default bind address for the HTTP transport.
    #[serde(default)]
    pub host: Option<String>,
    /// Default port for the HTTP transport.
    #[serde(default)]
    pub port: Option<u16>,
}

/// Load `.env` + `config.toml` from the standard locations.
///
/// Returns `Ok(LabConfig)` if loading succeeded, even if no files were
/// present. Returns `Err` only for parse failures — missing files are
/// not errors.
pub fn load() -> Result<LabConfig> {
    if let Some(env_path) = dotenv_path() {
        if env_path.exists() {
            dotenvy::from_path(&env_path)
                .with_context(|| format!("failed to load {}", env_path.display()))?;
        }
    }

    let cfg = if let Some(path) = toml_path() {
        if path.exists() {
            let raw = std::fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            toml::from_str::<LabConfig>(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?
        } else {
            LabConfig::default()
        }
    } else {
        LabConfig::default()
    };

    Ok(cfg)
}

/// Standard location for the `.env` file: `$HOME/.lab/.env`.
fn dotenv_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".lab").join(".env"))
}

/// Standard location for the TOML config: `$HOME/.config/lab/config.toml`.
fn toml_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join("lab")
            .join("config.toml")
    })
}

/// Parse multi-instance env vars for a given service prefix.
///
/// Returns a map from instance label (`"default"` or `"<label>"`) to the
/// set of `(suffix, value)` pairs. Example: for prefix `UNRAID`, env vars
/// `UNRAID_URL`, `UNRAID_API_KEY`, `UNRAID_NODE2_URL`, `UNRAID_NODE2_API_KEY`
/// yield two entries keyed `"default"` and `"node2"`.
#[must_use]
pub fn scan_instances(prefix: &str) -> HashMap<String, HashMap<String, String>> {
    let mut out: HashMap<String, HashMap<String, String>> = HashMap::new();
    let known_suffixes = ["URL", "API_KEY", "TOKEN", "USERNAME", "PASSWORD"];

    for (key, value) in std::env::vars() {
        let Some(rest) = key.strip_prefix(&format!("{prefix}_")) else {
            continue;
        };

        for suffix in &known_suffixes {
            if rest == *suffix {
                out.entry("default".to_string())
                    .or_default()
                    .insert((*suffix).to_string(), value.clone());
                break;
            }
            if let Some(label) = rest.strip_suffix(&format!("_{suffix}")) {
                if !label.is_empty() {
                    out.entry(label.to_ascii_lowercase())
                        .or_default()
                        .insert((*suffix).to_string(), value.clone());
                    break;
                }
            }
        }
    }

    out
}
