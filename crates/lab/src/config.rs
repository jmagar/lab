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

use std::{
    collections::HashMap,
    io::Write as _,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use lab_apis::extract::types::ServiceCreds;
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
    /// OAuth 2.1 resource server configuration (optional).
    #[serde(default)]
    pub oauth: Option<OAuthConfig>,
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

/// OAuth 2.1 resource server configuration.
///
/// Lab acts as a resource server (RFC 9728) — it validates tokens,
/// it does not issue them. Populated from `[oauth]` in `config.toml`
/// and/or `LAB_OAUTH_*` env vars.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OIDC issuer URL (must be HTTPS). Used for JWKS discovery.
    pub issuer: String,
    /// Expected `aud` claim (RFC 8707).
    pub audience: String,
    /// Optional `azp` claim validation.
    #[serde(default)]
    pub client_id: Option<String>,
    /// Public URL of this lab instance (for metadata + `allowed_hosts`).
    #[serde(default)]
    pub resource_url: Option<String>,
}

/// Resolve `OAuthConfig` from config file + environment variables.
///
/// Env vars take precedence over config file values.
#[allow(dead_code)] // Used starting in Phase 1.4 (JWT validation middleware)
pub fn resolve_oauth(config: Option<&OAuthConfig>) -> Option<OAuthConfig> {
    let issuer = std::env::var("LAB_OAUTH_ISSUER")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| config.map(|c| c.issuer.clone()));

    let audience = std::env::var("LAB_OAUTH_AUDIENCE")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| config.map(|c| c.audience.clone()));

    let issuer = issuer?;
    let audience = audience?;

    // Security: reject non-HTTPS issuers.
    if !issuer.starts_with("https://") {
        tracing::error!(
            issuer,
            "LAB_OAUTH_ISSUER must use HTTPS — refusing to start with HTTP issuer"
        );
        return None;
    }

    let client_id = std::env::var("LAB_OAUTH_CLIENT_ID")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| config.and_then(|c| c.client_id.clone()));

    let resource_url = std::env::var("LAB_RESOURCE_URL")
        .ok()
        .filter(|v| !v.is_empty())
        .or_else(|| config.and_then(|c| c.resource_url.clone()));

    Some(OAuthConfig {
        issuer,
        audience,
        client_id,
        resource_url,
    })
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
    let cwd_env = Path::new(".env");
    if cwd_env.exists()
        && let Err(e) = dotenvy::from_path(cwd_env)
    {
        tracing::debug!(path = ".env", error = %e, "failed to load local .env (skipping)");
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

// ─── .env writer (used by `lab extract --apply`) ─────────────────────────────

/// Copy `path` to `path.bak.<unix-seconds>`. No-op if `path` does not exist.
///
/// Returns the backup path (useful for messaging the user).
///
/// # Errors
/// Returns an error if the copy fails.
pub fn backup_env(path: &Path) -> Result<PathBuf> {
    if !path.exists() {
        // Nothing to back up; return a synthetic path for display only.
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs());
        return Ok(path.with_extension(format!("bak.{ts}")));
    }
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let backup = PathBuf::from(format!("{}.bak.{ts}", path.display()));
    std::fs::copy(path, &backup)
        .with_context(|| format!("backup {} → {}", path.display(), backup.display()))?;
    Ok(backup)
}

/// Merge `new_creds` into the `.env` file at `path` following the 8-rule algorithm.
///
/// Rule summary (full spec in `crates/lab-apis/src/extract/CLAUDE.md`):
/// 1. Backup is the caller's responsibility — call [`backup_env`] before this.
/// 2. Atomic write: `path.tmp` → rename.
/// 3. Existing key order and comments are preserved.
/// 4. Comments (`#`) and blank lines pass through unchanged.
/// 5. Dedupe: one entry per key.
/// 6. Conflicts (key exists, different value): skip-and-warn unless `force=true`.
/// 7. Values containing whitespace or shell metacharacters are double-quoted.
/// 8. Idempotence: caller must check before invoking (this fn always writes).
///
/// Returns a `Vec<String>` of warnings for skipped conflicts.
///
/// # Errors
/// Returns an error if the tmp file cannot be written or renamed.
pub fn write_env(
    path: &Path,
    new_creds: &[ServiceCreds],
    force: bool,
) -> Result<Vec<String>> {
    // Read the existing file (empty if absent).
    let existing_raw = if path.exists() {
        std::fs::read_to_string(path)
            .with_context(|| format!("read {}", path.display()))?
    } else {
        String::new()
    };
    let existing_lines: Vec<&str> = existing_raw.lines().collect();

    // Build map of existing key → value from non-comment lines.
    let mut existing: HashMap<String, String> = HashMap::new();
    for line in &existing_lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = trimmed.split_once('=') {
            existing.insert(k.trim().to_owned(), v.trim().to_owned());
        }
    }

    // Collect all (key, value) pairs to write from new_creds.
    let mut to_write: Vec<(String, String)> = Vec::new();
    for cred in new_creds {
        let svc_upper = cred.service.to_uppercase();
        if let Some(url) = &cred.url {
            to_write.push((format!("{svc_upper}_URL"), url.clone()));
        }
        if let Some(secret) = &cred.secret {
            to_write.push((cred.env_field.clone(), secret.clone()));
        }
    }

    // Process each pair: classify as NEW, SAME, or CONFLICT.
    let mut conflicts: Vec<String> = Vec::new();
    // Track keys that are overrides (force=true conflicts).
    let mut override_keys: HashMap<String, String> = HashMap::new();
    // Track keys that are genuinely new.
    let mut new_keys: Vec<(String, String)> = Vec::new();

    for (key, value) in &to_write {
        match existing.get(key) {
            None => new_keys.push((key.clone(), value.clone())),
            Some(existing_val) if existing_val == value => {
                // Idempotent — already present with same value, skip.
            }
            Some(existing_val) => {
                if force {
                    override_keys.insert(key.clone(), value.clone());
                } else {
                    conflicts.push(format!(
                        "CONFLICT: {key} already set to {existing_val:?}; skipping (use --force to overwrite)"
                    ));
                }
            }
        }
    }

    // Build the new file: start with existing lines, applying overrides in-place.
    let mut out_lines: Vec<String> = Vec::new();
    for line in &existing_lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#')
            && let Some((k, _)) = trimmed.split_once('=') {
                let key = k.trim();
                if let Some(new_val) = override_keys.get(key) {
                    out_lines.push(format!("{}={}", key, quote_env_value(new_val)));
                    continue;
                }
            }
        out_lines.push((*line).to_owned());
    }

    // Append new keys at the end.
    if !new_keys.is_empty() {
        if !out_lines.last().is_none_or(|l| l.trim().is_empty()) {
            out_lines.push(String::new()); // blank separator
        }
        for (key, value) in &new_keys {
            out_lines.push(format!("{}={}", key, quote_env_value(value)));
        }
    }

    // Atomic write: write to .tmp, sync, rename.
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create dir {}", parent.display()))?;
    }

    let tmp_path = PathBuf::from(format!("{}.tmp", path.display()));
    {
        let mut file = std::fs::File::create(&tmp_path)
            .with_context(|| format!("create {}", tmp_path.display()))?;
        for line in &out_lines {
            writeln!(file, "{line}")
                .with_context(|| format!("write {}", tmp_path.display()))?;
        }
        file.sync_all()
            .with_context(|| format!("sync {}", tmp_path.display()))?;
    }
    std::fs::rename(&tmp_path, path)
        .with_context(|| format!("rename {} → {}", tmp_path.display(), path.display()))?;

    Ok(conflicts)
}

/// Returns true if all (key, value) pairs that would be written by `write_env`
/// are already present in `path` with matching values. Used to implement
/// idempotence: if this returns true, skip backup and write entirely.
pub fn env_is_up_to_date(path: &Path, new_creds: &[ServiceCreds]) -> bool {
    let Ok(raw) = std::fs::read_to_string(path) else {
        return false;
    };
    let existing: HashMap<String, String> = raw
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
        .filter_map(|l| {
            l.split_once('=')
                .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        })
        .collect();

    for cred in new_creds {
        let svc_upper = cred.service.to_uppercase();
        if let Some(url) = &cred.url {
            let key = format!("{svc_upper}_URL");
            if existing.get(&key).map(String::as_str) != Some(url.as_str()) {
                return false;
            }
        }
        if let Some(secret) = &cred.secret
            && existing.get(&cred.env_field).map(String::as_str) != Some(secret.as_str()) {
                return false;
            }
    }
    true
}

/// Quote a value that contains shell-significant characters.
fn quote_env_value(v: &str) -> String {
    let needs_quotes = v.chars().any(|c| matches!(c, ' ' | '\t' | '#' | '$' | '\\' | '"' | '\'' | '`'));
    if needs_quotes {
        let escaped = v.replace('\\', r"\\").replace('"', r#"\""#);
        format!("\"{escaped}\"")
    } else {
        v.to_owned()
    }
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
        assert_eq!(
            json, "\"***REDACTED***\"",
            "Secret must serialize to placeholder"
        );
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

    // ─── write_env / backup_env tests ───────────────────────────────────────

    fn radarr_cred() -> ServiceCreds {
        ServiceCreds {
            service: "radarr".to_owned(),
            url: Some("http://localhost:7878".to_owned()),
            secret: Some("abc123".to_owned()),
            env_field: "RADARR_API_KEY".to_owned(),
        }
    }

    #[test]
    fn write_env_adds_new_keys() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let warnings = write_env(&path, &[radarr_cred()], false).unwrap();
        assert!(warnings.is_empty());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("RADARR_URL=http://localhost:7878"));
        assert!(content.contains("RADARR_API_KEY=abc123"));
    }

    #[test]
    fn write_env_preserves_comments_and_blanks() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(&path, "# my comment\nOTHER=val\n").unwrap();
        write_env(&path, &[radarr_cred()], false).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("# my comment"));
        assert!(content.contains("OTHER=val"));
    }

    #[test]
    fn write_env_conflict_skip_without_force() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(&path, "RADARR_API_KEY=oldvalue\n").unwrap();
        let warnings = write_env(&path, &[radarr_cred()], false).unwrap();
        assert!(!warnings.is_empty());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("oldvalue"));
        assert!(!content.contains("abc123"));
    }

    #[test]
    fn write_env_conflict_overwrite_with_force() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(&path, "RADARR_API_KEY=oldvalue\n").unwrap();
        let warnings = write_env(&path, &[radarr_cred()], true).unwrap();
        assert!(warnings.is_empty());
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("abc123"));
        assert!(!content.contains("oldvalue"));
    }

    #[test]
    fn env_is_up_to_date_returns_true_when_matching() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(
            &path,
            "RADARR_URL=http://localhost:7878\nRADARR_API_KEY=abc123\n",
        )
        .unwrap();
        assert!(env_is_up_to_date(&path, &[radarr_cred()]));
    }

    #[test]
    fn env_is_up_to_date_returns_false_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        std::fs::write(&path, "RADARR_URL=http://localhost:7878\n").unwrap();
        assert!(!env_is_up_to_date(&path, &[radarr_cred()]));
    }

    #[test]
    fn write_env_quotes_value_with_special_chars() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let cred = ServiceCreds {
            service: "svc".to_owned(),
            url: None,
            secret: Some("has space".to_owned()),
            env_field: "SVC_KEY".to_owned(),
        };
        write_env(&path, &[cred], false).unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert!(content.contains("SVC_KEY=\"has space\""));
    }
}
