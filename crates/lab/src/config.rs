//! Config loading for the `lab` binary.
//!
//! Order of precedence (highest wins):
//!   1. CLI flags / process environment variables
//!   2. `~/.lab/.env` (loaded via `dotenvy`)
//!   3. `config.toml` (searched: `./` → `~/.lab/` → `~/.config/lab/`)
//!   4. Built-in defaults
//!
//! URLs and secrets belong in `.env`. Everything else (logging, CORS,
//! MCP transport, admin flags, per-service preferences) belongs in
//! `config.toml` but can still be overridden via env vars.
//!
//! Multi-instance services follow the `S_<LABEL>_URL` pattern: a service
//! like `unraid` reads `UNRAID_URL` as the default instance and
//! `UNRAID_NODE2_URL` as an additional instance labeled `node2`.

use std::{
    collections::BTreeMap,
    collections::HashMap,
    io::Write as _,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use lab_apis::extract::types::ServiceCreds;
use lab_auth::config as auth_config;
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
    /// Logging preferences (overridden by `LAB_LOG` / `LAB_LOG_FORMAT` env vars).
    #[serde(default)]
    pub log: LogPreferences,
    /// HTTP API preferences.
    #[serde(default)]
    pub api: ApiPreferences,
    /// Web UI preferences.
    #[serde(default)]
    pub web: WebPreferences,
    /// OAuth callback relay preferences.
    #[serde(default)]
    pub oauth: OauthPreferences,
    /// Admin tool settings.
    #[serde(default)]
    pub admin: AdminPreferences,
    /// Per-service preference overrides.
    #[serde(default)]
    pub services: ServicePreferences,
    /// HTTP auth mode preferences.
    #[serde(default)]
    pub auth: Option<AuthFileConfig>,
    /// Upstream MCP servers to proxy through the gateway.
    #[serde(default)]
    pub upstream: Vec<UpstreamConfig>,
    /// Virtual MCP servers backed by canonically configured Lab services.
    #[serde(default)]
    pub virtual_servers: Vec<VirtualServerConfig>,
}

/// Configuration for a single upstream MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpstreamConfig {
    /// Human-readable name for this upstream (used as tool-name prefix).
    pub name: String,
    /// URL of the upstream MCP server (must be `http://` or `https://`).
    /// For stdio upstreams, omit `url` and use `command`/`args` fields instead.
    #[serde(default)]
    pub url: Option<String>,
    /// Name of an env var holding the bearer token (not the token itself).
    #[serde(default)]
    pub bearer_token_env: Option<String>,
    /// Command to run for stdio transport upstreams.
    #[serde(default)]
    pub command: Option<String>,
    /// Arguments to pass to the stdio command.
    #[serde(default)]
    pub args: Vec<String>,
    /// Whether to proxy resources from this upstream (opt-in).
    #[serde(default)]
    pub proxy_resources: bool,
    /// Optional allowlist of tool names/patterns to expose from this upstream.
    #[serde(default)]
    pub expose_tools: Option<Vec<String>>,
}

/// Persisted state for a Lab-backed virtual server shown in the gateway.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualServerConfig {
    pub id: String,
    pub service: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub surfaces: VirtualServerSurfacesConfig,
    #[serde(default)]
    pub mcp_policy: Option<VirtualServerMcpPolicyConfig>,
}

/// Per-surface exposure flags for a virtual server.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualServerSurfacesConfig {
    #[serde(default)]
    pub cli: bool,
    #[serde(default)]
    pub api: bool,
    #[serde(default)]
    pub mcp: bool,
    #[serde(default)]
    pub webui: bool,
}

/// Action-level policy for Lab-backed single-tool MCP services.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualServerMcpPolicyConfig {
    #[serde(default)]
    pub allowed_actions: Vec<String>,
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
    /// Default session keep-alive TTL in seconds for HTTP MCP sessions.
    #[serde(default)]
    pub session_ttl_secs: Option<u64>,
    /// Whether HTTP MCP should use stateful sessions by default.
    #[serde(default)]
    pub stateful: Option<bool>,
    /// Additional allowed hosts for DNS rebinding protection.
    #[serde(default)]
    pub allowed_hosts: Option<Vec<String>>,
}

/// File-backed auth preferences merged with environment variables at startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthFileConfig {
    /// `bearer` preserves LAB_MCP_HTTP_TOKEN; `oauth` enables the internal auth server.
    #[serde(default)]
    pub mode: Option<String>,
    /// Public URL used for metadata and Google callback construction.
    #[serde(default)]
    pub public_url: Option<String>,
    /// Optional path override for the SQLite auth store.
    #[serde(default)]
    pub sqlite_path: Option<PathBuf>,
    /// Optional path override for the persisted JWT signing key.
    #[serde(default)]
    pub key_path: Option<PathBuf>,
    /// Bootstrap secret required for dynamic client registration.
    #[serde(default)]
    pub bootstrap_secret: Option<String>,
    /// Additional redirect URI patterns allowed for dynamic client registration.
    #[serde(default)]
    pub allowed_client_redirect_uris: Option<Vec<String>>,
    /// Google OAuth client ID.
    #[serde(default)]
    pub google_client_id: Option<String>,
    /// Google OAuth client secret.
    #[serde(default)]
    pub google_client_secret: Option<String>,
    /// Optional callback path override.
    #[serde(default)]
    pub google_callback_path: Option<String>,
    /// Optional comma-separated scope list.
    #[serde(default)]
    pub google_scopes: Option<Vec<String>>,
    /// Optional access-token lifetime override in seconds.
    #[serde(default)]
    pub access_token_ttl_secs: Option<u64>,
    /// Optional refresh-token lifetime override in seconds.
    #[serde(default)]
    pub refresh_token_ttl_secs: Option<u64>,
    /// Optional authorization-code lifetime override in seconds.
    #[serde(default)]
    pub auth_code_ttl_secs: Option<u64>,
}

/// Resolve auth configuration from config file + environment variables.
///
/// Env vars take precedence over config file values.
pub fn resolve_auth(config: Option<&AuthFileConfig>) -> Result<auth_config::AuthConfig> {
    let mut merged: HashMap<String, String> = HashMap::new();

    if let Some(config) = config {
        insert_if_some(&mut merged, "LAB_AUTH_MODE", config.mode.clone());
        insert_if_some(&mut merged, "LAB_PUBLIC_URL", config.public_url.clone());
        insert_if_some(
            &mut merged,
            "LAB_AUTH_SQLITE_PATH",
            config
                .sqlite_path
                .as_ref()
                .map(|path| path.display().to_string()),
        );
        insert_if_some(
            &mut merged,
            "LAB_AUTH_KEY_PATH",
            config
                .key_path
                .as_ref()
                .map(|path| path.display().to_string()),
        );
        insert_if_some(
            &mut merged,
            "LAB_AUTH_BOOTSTRAP_SECRET",
            config.bootstrap_secret.clone(),
        );
        if let Some(patterns) = config.allowed_client_redirect_uris.as_ref() {
            insert_if_some(
                &mut merged,
                "LAB_AUTH_ALLOWED_REDIRECT_URIS",
                Some(patterns.join(",")),
            );
        }
        insert_if_some(
            &mut merged,
            "LAB_GOOGLE_CLIENT_ID",
            config.google_client_id.clone(),
        );
        insert_if_some(
            &mut merged,
            "LAB_GOOGLE_CLIENT_SECRET",
            config.google_client_secret.clone(),
        );
        insert_if_some(
            &mut merged,
            "LAB_GOOGLE_CALLBACK_PATH",
            config.google_callback_path.clone(),
        );
        if let Some(scopes) = config.google_scopes.as_ref() {
            insert_if_some(&mut merged, "LAB_GOOGLE_SCOPES", Some(scopes.join(",")));
        }
        insert_if_some(
            &mut merged,
            "LAB_AUTH_ACCESS_TOKEN_TTL_SECS",
            config.access_token_ttl_secs.map(|value| value.to_string()),
        );
        insert_if_some(
            &mut merged,
            "LAB_AUTH_REFRESH_TOKEN_TTL_SECS",
            config.refresh_token_ttl_secs.map(|value| value.to_string()),
        );
        insert_if_some(
            &mut merged,
            "LAB_AUTH_CODE_TTL_SECS",
            config.auth_code_ttl_secs.map(|value| value.to_string()),
        );
    }

    for (key, value) in std::env::vars() {
        if key.starts_with("LAB_AUTH_") || key == "LAB_PUBLIC_URL" || key.starts_with("LAB_GOOGLE_")
        {
            merged.insert(key, value);
        }
    }

    auth_config::AuthConfig::from_sources(merged).map_err(anyhow::Error::from)
}

fn insert_if_some(target: &mut HashMap<String, String>, key: &str, value: Option<String>) {
    if let Some(value) = value
        && !value.trim().is_empty()
    {
        target.insert(key.to_string(), value);
    }
}

/// Load `.env` + `config.toml` from the standard locations.
///
/// These map to `LAB_LOG` and `LAB_LOG_FORMAT` env vars but live in TOML so
/// operators don't need to clutter `.env` with non-secret preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogPreferences {
    /// Tracing filter directive (e.g. `"lab=info,lab_apis=warn"`).
    /// Overridden by `LAB_LOG` env var.
    #[serde(default)]
    pub filter: Option<String>,
    /// Log format: `"text"` (default) or `"json"`.
    /// Overridden by `LAB_LOG_FORMAT` env var.
    #[serde(default)]
    pub format: Option<String>,
}

/// HTTP API preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ApiPreferences {
    /// Additional CORS origins (comma-separated string or TOML array).
    /// Loopback origins are always included.
    /// Overridden by `LAB_CORS_ORIGINS` env var.
    #[serde(default)]
    pub cors_origins: Vec<String>,
}

/// Web UI preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WebPreferences {
    /// Path to the exported Labby assets directory served by `lab serve`.
    #[serde(default)]
    pub assets_dir: Option<PathBuf>,
    /// Disable `/v1/*` auth for the hosted web UI. Intended only for trusted reverse-proxy setups.
    #[serde(default)]
    pub disable_auth: Option<bool>,
}

/// OAuth local relay preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OauthPreferences {
    /// Named callback relay targets.
    #[serde(default)]
    pub machines: BTreeMap<String, OauthMachineConfig>,
}

/// A named OAuth callback relay target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OauthMachineConfig {
    /// Full callback target base URL.
    pub target_url: String,
    /// Optional operator-facing description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional preferred callback port for the browser-local listener.
    #[serde(default)]
    pub default_port: Option<u16>,
}

/// Admin tool settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminPreferences {
    /// Enable the `lab_admin` MCP tool. Default: `false`.
    /// Overridden by `LAB_ADMIN_ENABLED=1` env var.
    #[serde(default)]
    pub enabled: bool,
}

/// Per-service preference overrides (non-secret values only).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServicePreferences {
    /// Tailscale preferences.
    #[serde(default)]
    pub tailscale: TailscalePreferences,
}

/// Tailscale non-secret preferences.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TailscalePreferences {
    /// Tailnet name. Overridden by `TAILSCALE_TAILNET` env var.
    /// Default: `"-"` (auto-detect).
    #[serde(default)]
    pub tailnet: Option<String>,
}

/// Load `config.toml` only — no `.env`, no side effects beyond file reads.
///
/// Called early in `main()` before tracing is initialized so that `[log]`
/// preferences can feed into `init_tracing()`. Safe to call before any
/// other subsystem.
///
/// Config TOML resolution (first found wins):
///   1. `./config.toml` (repo/CWD override)
///   2. `~/.lab/config.toml` (user-level, colocated with `.env`)
///   3. `~/.config/lab/config.toml` (XDG-style fallback)
pub fn load_toml(candidates: &[PathBuf]) -> Result<LabConfig> {
    for path in candidates {
        match std::fs::read_to_string(path) {
            Ok(raw) => {
                return toml::from_str::<LabConfig>(&raw)
                    .with_context(|| format!("failed to parse {}", path.display()));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                return Err(
                    anyhow::Error::new(e).context(format!("failed to read {}", path.display()))
                );
            }
        }
    }
    Ok(LabConfig::default())
}

/// Load `.env` files into the process environment.
///
/// Called after `load_toml()` and tracing init. Env vars loaded here
/// override config.toml values at the point of use (each consumer checks
/// env first, then falls back to config).
pub fn load_dotenv() -> Result<()> {
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

    Ok(())
}

/// Load `.env` + `config.toml` in a single call (convenience for tests).
#[allow(dead_code)]
pub fn load() -> Result<LabConfig> {
    let cfg = load_toml(&toml_candidates())?;
    load_dotenv()?;
    Ok(cfg)
}

/// Candidate paths for `config.toml`, ordered by priority (highest first).
pub fn toml_candidates() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("config.toml")];
    if let Some(home) = home_dir() {
        paths.push(home.join(".lab").join("config.toml"));
        paths.push(home.join(".config").join("lab").join("config.toml"));
    }
    paths
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

pub fn config_toml_path() -> Option<PathBuf> {
    toml_candidates()
        .into_iter()
        .find(|path| path.exists())
        .or_else(|| home_dir().map(|home| home.join(".config").join("lab").join("config.toml")))
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
pub fn write_env(path: &Path, new_creds: &[ServiceCreds], force: bool) -> Result<Vec<String>> {
    // Read the existing file (empty if absent).
    let existing_raw = if path.exists() {
        std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?
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
        if !trimmed.is_empty()
            && !trimmed.starts_with('#')
            && let Some((k, _)) = trimmed.split_once('=')
        {
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
            writeln!(file, "{line}").with_context(|| format!("write {}", tmp_path.display()))?;
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
            l.split_once('=').map(|(k, v)| {
                let trimmed = v.trim();
                // Strip surrounding double quotes so that quoted values
                // written by write_env() compare equal to the raw secret.
                let unquoted = trimmed
                    .strip_prefix('"')
                    .and_then(|s| s.strip_suffix('"'))
                    .map_or_else(
                        || trimmed.to_owned(),
                        // Unescape sequences that write_env() would have escaped.
                        |inner| inner.replace(r#"\""#, "\"").replace(r"\\", r"\"),
                    );
                (k.trim().to_owned(), unquoted)
            })
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
            && existing.get(&cred.env_field).map(String::as_str) != Some(secret.as_str())
        {
            return false;
        }
    }
    true
}

/// Quote a value that contains shell-significant characters.
fn quote_env_value(v: &str) -> String {
    let needs_quotes = v
        .chars()
        .any(|c| matches!(c, ' ' | '\t' | '#' | '$' | '\\' | '"' | '\'' | '`'));
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
    fn resolve_auth_reads_ttls_from_config_toml_fields() {
        let cfg = AuthFileConfig {
            mode: Some("oauth".to_string()),
            public_url: Some("https://lab.example.com".to_string()),
            sqlite_path: None,
            key_path: None,
            bootstrap_secret: Some("bootstrap".to_string()),
            allowed_client_redirect_uris: Some(vec![
                "https://callback.tootie.tv/callback/*".to_string(),
            ]),
            google_client_id: Some("client-id".to_string()),
            google_client_secret: Some("client-secret".to_string()),
            google_callback_path: Some("/auth/google/callback".to_string()),
            google_scopes: Some(vec!["openid".to_string(), "email".to_string()]),
            access_token_ttl_secs: Some(120),
            refresh_token_ttl_secs: Some(3600),
            auth_code_ttl_secs: Some(45),
        };

        let resolved = resolve_auth(Some(&cfg)).expect("auth config should resolve");
        assert_eq!(resolved.access_token_ttl.as_secs(), 120);
        assert_eq!(resolved.refresh_token_ttl.as_secs(), 3600);
        assert_eq!(resolved.auth_code_ttl.as_secs(), 45);
        assert_eq!(
            resolved.allowed_client_redirect_uris,
            vec!["https://callback.tootie.tv/callback/*".to_string()]
        );
    }

    #[test]
    fn oauth_machine_config_deserializes() {
        let cfg = toml::from_str::<LabConfig>(
            r#"
[oauth.machines.dookie]
target_url = "http://100.88.16.79:38935/callback/dookie"
description = "Dookie Claude callback target"
default_port = 38935
"#,
        )
        .expect("oauth machine config should parse");

        assert_eq!(
            cfg.oauth.machines["dookie"].target_url,
            "http://100.88.16.79:38935/callback/dookie"
        );
        assert_eq!(
            cfg.oauth.machines["dookie"].description.as_deref(),
            Some("Dookie Claude callback target")
        );
        assert_eq!(cfg.oauth.machines["dookie"].default_port, Some(38935));
    }

    #[test]
    fn oauth_machine_defaults_keep_partial_configs_valid() {
        let cfg = toml::from_str::<LabConfig>(
            r#"
[web]
assets_dir = "/tmp/labby"
"#,
        )
        .expect("config without oauth section should still parse");

        assert!(cfg.oauth.machines.is_empty());
        assert_eq!(cfg.web.assets_dir, Some(PathBuf::from("/tmp/labby")));
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

    #[test]
    fn env_is_up_to_date_handles_quoted_values() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".env");
        let cred = ServiceCreds {
            service: "svc".to_owned(),
            url: None,
            secret: Some("has space".to_owned()),
            env_field: "SVC_KEY".to_owned(),
        };
        // write_env quotes values with spaces
        write_env(&path, &[cred.clone()], false).unwrap();
        // env_is_up_to_date must strip quotes before comparing
        assert!(
            env_is_up_to_date(&path, &[cred]),
            "quoted value in .env should match raw secret"
        );
    }
}
