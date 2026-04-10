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
use serde::{Deserialize, Serialize, Serializer};

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
    // Load ~/.lab/.env first (user-level secrets).
    if let Some(env_path) = dotenv_path()
        && env_path.exists()
    {
        dotenvy::from_path(&env_path)
            .with_context(|| format!("failed to load {}", env_path.display()))?;
    }

    // Also load .env from the current working directory (dev convenience).
    // Does not override vars already set by the user-level file.
    let cwd_env = std::path::Path::new(".env");
    if cwd_env.exists()
        && let Err(e) = dotenvy::from_path(cwd_env) {
            tracing::warn!(path = ".env", error = %e, "failed to load local .env (skipping)");
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

/// Cross-platform home directory.
///
/// Checks `HOME` (Unix) then `USERPROFILE` (Windows). No external crate needed.
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

/// Standard location for the `.env` file: `~/.lab/.env`.
fn dotenv_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".lab").join(".env"))
}

/// Standard location for the TOML config: `~/.config/lab/config.toml`.
fn toml_path() -> Option<PathBuf> {
    home_dir().map(|home| home.join(".config").join("lab").join("config.toml"))
}

/// A string value that redacts itself in `Debug` and `Display` output.
///
/// Use for secret env values (`API_KEY`, `TOKEN`, `PASSWORD`) so they
/// never leak through `Debug`-printing config structs or tracing fields.
#[allow(dead_code)]
#[derive(Clone, Deserialize, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    #[must_use]
    pub const fn new(value: String) -> Self {
        Self(value)
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl std::fmt::Display for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

impl Serialize for Secret {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("***REDACTED***")
    }
}

/// Value from an instance env var — either plain text or a secret.
///
/// Always constructed programmatically via [`scan_instances_from`]; never
/// deserialized from JSON. `Deserialize` is intentionally omitted — `Secret`
/// serializes as `"***REDACTED***"` (a plain string), so an `#[serde(untagged)]`
/// impl would silently pick `Plain` for every value, bypassing redaction.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize)]
pub enum InstanceValue {
    Plain(String),
    Redacted(Secret),
}

impl InstanceValue {
    #[must_use]
    #[allow(dead_code)]
    pub fn expose(&self) -> &str {
        match self {
            Self::Plain(s) => s,
            Self::Redacted(s) => s.expose(),
        }
    }
}

/// Suffixes that carry secret values and must be wrapped in [`Secret`].
#[allow(dead_code)]
const SECRET_SUFFIXES: &[&str] = &["API_KEY", "TOKEN", "PASSWORD"];

/// Parse multi-instance env vars for a given service prefix.
///
/// Returns a map from instance label (`"default"` or `"<label>"`) to the
/// set of `(suffix, value)` pairs. Example: for prefix `UNRAID`, env vars
/// `UNRAID_URL`, `UNRAID_API_KEY`, `UNRAID_NODE2_URL`, `UNRAID_NODE2_API_KEY`
/// yield two entries keyed `"default"` and `"node2"`.
///
/// Suffixes are matched longest-first to avoid collisions when a label
/// contains a shorter suffix as a substring.
#[must_use]
#[allow(dead_code)]
pub fn scan_instances(prefix: &str) -> HashMap<String, HashMap<String, InstanceValue>> {
    scan_instances_from(prefix, std::env::vars())
}

/// Inner implementation testable without mutating process env.
fn scan_instances_from(
    prefix: &str,
    vars: impl Iterator<Item = (String, String)>,
) -> HashMap<String, HashMap<String, InstanceValue>> {
    let mut out: HashMap<String, HashMap<String, InstanceValue>> = HashMap::new();

    let mut known_suffixes = ["URL", "API_KEY", "TOKEN", "USERNAME", "PASSWORD"];
    known_suffixes.sort_by_key(|s| std::cmp::Reverse(s.len()));

    let prefix_under = format!("{prefix}_");

    for (key, value) in vars {
        let Some(rest) = key.strip_prefix(&prefix_under) else {
            continue;
        };

        for suffix in &known_suffixes {
            let wrap = |v: String| {
                if SECRET_SUFFIXES.contains(suffix) {
                    InstanceValue::Redacted(Secret::new(v))
                } else {
                    InstanceValue::Plain(v)
                }
            };

            if rest == *suffix {
                out.entry("default".to_string())
                    .or_default()
                    .insert((*suffix).to_string(), wrap(value.clone()));
                break;
            }
            if let Some(label) = rest.strip_suffix(&format!("_{suffix}"))
                && !label.is_empty()
            {
                out.entry(label.to_ascii_lowercase())
                    .or_default()
                    .insert((*suffix).to_string(), wrap(value.clone()));
                break;
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Iterator<Item = (String, String)> + 'a {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
    }

    #[test]
    fn secret_debug_redacts() {
        let s = Secret::new("hunter2".into());
        assert_eq!(format!("{s:?}"), "[REDACTED]");
        assert_eq!(format!("{s}"), "[REDACTED]");
        assert_eq!(s.expose(), "hunter2");
    }

    #[test]
    fn secret_serialize_emits_placeholder_not_plaintext() {
        let s = Secret::new("super-secret-api-key".into());
        let json = serde_json::to_string(&s).expect("serialize must not fail");
        assert_eq!(json, "\"***REDACTED***\"", "Secret must serialize to placeholder");
        assert!(
            !json.contains("super-secret-api-key"),
            "Secret must never emit plaintext through serde"
        );
    }

    #[test]
    fn suffix_collision_longest_wins() {
        let env = [("S_NODE_API_KEY_URL", "http://example.com")];
        let result = scan_instances_from("S", vars(&env));
        let inst = result
            .get("node_api_key")
            .expect("should find instance node_api_key");
        assert_eq!(
            inst.get("URL").expect("should have URL").expose(),
            "http://example.com"
        );
    }

    #[test]
    fn default_instance_parsed() {
        let env = [
            ("SVC_URL", "http://localhost"),
            ("SVC_API_KEY", "secret123"),
        ];
        let result = scan_instances_from("SVC", vars(&env));
        let def = result.get("default").expect("should find default");
        assert_eq!(def.get("URL").expect("URL").expose(), "http://localhost");
        assert_eq!(def.get("API_KEY").expect("API_KEY").expose(), "secret123");
        assert!(format!("{:?}", def.get("API_KEY").unwrap()).contains("[REDACTED]"));
    }

    #[test]
    fn named_instance_parsed() {
        let env = [
            ("UNRAID_NODE2_URL", "http://node2"),
            ("UNRAID_NODE2_TOKEN", "tok"),
        ];
        let result = scan_instances_from("UNRAID", vars(&env));
        let inst = result.get("node2").expect("should find node2");
        assert_eq!(inst.get("URL").expect("URL").expose(), "http://node2");
        assert_eq!(inst.get("TOKEN").expect("TOKEN").expose(), "tok");
        assert!(format!("{:?}", inst.get("TOKEN").unwrap()).contains("[REDACTED]"));
    }

    #[test]
    fn unrelated_vars_ignored() {
        let env = [
            ("SVC_URL", "http://localhost"),
            ("OTHER_URL", "http://other"),
        ];
        let result = scan_instances_from("SVC", vars(&env));
        assert_eq!(result.len(), 1);
        assert!(result.contains_key("default"));
    }

    #[test]
    fn username_is_plain_not_secret() {
        let env = [("SVC_USERNAME", "admin")];
        let result = scan_instances_from("SVC", vars(&env));
        let def = result.get("default").expect("should find default");
        assert!(!format!("{:?}", def.get("USERNAME").unwrap()).contains("[REDACTED]"));
    }
}
