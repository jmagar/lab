//! Client construction helpers for the `unraid` dispatch layer.

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use lab_apis::core::Auth;
use lab_apis::unraid::UnraidClient;

use crate::config::scan_instances;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Module-level cache of named `Unraid` clients, built once on first access.
static NAMED_CLIENTS: OnceLock<HashMap<String, Arc<UnraidClient>>> = OnceLock::new();

/// All labels discovered by `scan_instances("UNRAID")`, built once alongside
/// `NAMED_CLIENTS`. A label present here but absent from `NAMED_CLIENTS`
/// means the instance was discovered but its env vars are broken/incomplete.
static ALL_LABELS: OnceLock<Vec<String>> = OnceLock::new();

/// Return (or lazily build) the map of named `Unraid` clients.
///
/// The map is keyed by label (e.g. `"default"`, `"node2"`) and built once
/// by scanning env vars at first call. Subsequent calls are lock-free reads.
fn named_clients() -> &'static HashMap<String, Arc<UnraidClient>> {
    NAMED_CLIENTS.get_or_init(|| {
        let mut map = HashMap::new();
        let mut all: Vec<String> = Vec::new();
        for (label, _vars) in scan_instances("UNRAID") {
            all.push(label.clone());
            let (url_key, key_key) = if label == "default" {
                ("UNRAID_URL".to_string(), "UNRAID_API_KEY".to_string())
            } else {
                let upper = label.to_ascii_uppercase();
                (
                    format!("UNRAID_{upper}_URL"),
                    format!("UNRAID_{upper}_API_KEY"),
                )
            };
            if let (Some(raw_url), Some(key)) = (env_non_empty(&url_key), env_non_empty(&key_key)) {
                let url = raw_url
                    .trim_end_matches('/')
                    .trim_end_matches("graphql")
                    .trim_end_matches('/')
                    .to_string();
                if let Ok(client) = UnraidClient::new(
                    &url,
                    Auth::ApiKey {
                        // Unraid's GraphQL API requires "X-API-Key" (all-caps KEY) per its
                        // API spec — intentional deviation from the dispatch template which
                        // uses "X-Api-Key". HTTP headers are case-insensitive on the wire,
                        // but Unraid's server validates the exact name.
                        header: "X-API-Key".into(),
                        key,
                    },
                ) {
                    map.insert(label, Arc::new(client));
                }
            }
        }
        // Populate the labels cache as a side-effect of the first init.
        drop(ALL_LABELS.set(all));
        map
    })
}

/// Return all instance labels discovered in the environment (including those
/// with broken or incomplete configuration).
fn all_labels() -> &'static Vec<String> {
    // Trigger named_clients() init which also populates ALL_LABELS.
    let _ = named_clients();
    ALL_LABELS.get_or_init(Vec::new)
}

/// Build an `UnraidClient` from the default-instance environment variables.
///
/// Reads `UNRAID_URL` and `UNRAID_API_KEY`. Returns `None` if either is
/// absent or empty. Called by `AppState::ServiceClients::from_env()` at
/// startup — must be pure (no side effects, no logging).
pub fn client_from_env() -> Option<UnraidClient> {
    let raw_url = env_non_empty("UNRAID_URL")?;
    // Normalise: strip trailing /graphql if present so the URL stored in env
    // can be either the bare host (https://host:31337) or the full endpoint
    // (https://host:31337/graphql) — the client always appends /graphql itself.
    let url = raw_url
        .trim_end_matches('/')
        .trim_end_matches("graphql")
        .trim_end_matches('/')
        .to_string();
    let key = env_non_empty("UNRAID_API_KEY")?;
    UnraidClient::new(
        &url,
        Auth::ApiKey {
            // Unraid's GraphQL API requires "X-API-Key" (all-caps KEY) per its
            // API spec — intentional deviation from the dispatch template which
            // uses "X-Api-Key". HTTP headers are case-insensitive on the wire,
            // but Unraid's server validates the exact name.
            header: "X-API-Key".into(),
            key,
        },
    )
    .ok()
}

/// Resolve an `Unraid` client by optional instance label.
///
/// - `None` — use the default instance (equivalent to `Some("default")`).
/// - `Some(label)` — look up the named instance in the module-level cache.
///
/// Returns an `Arc`-wrapped client from the module-level cache — no new
/// `reqwest::Client` is created after the first call.
///
/// # Errors
///
/// - `InvalidParam` — `label` is empty.
/// - `Sdk { sdk_kind: "internal_error" }` — the label was discovered in the
///   environment but its URL or API key is missing or invalid.
/// - `UnknownInstance` — the label was never found in the environment at all.
pub fn client_from_instance(label: Option<&str>) -> Result<Arc<UnraidClient>, ToolError> {
    let label = label.unwrap_or("default");
    let label = label.trim();
    if label.is_empty() {
        return Err(ToolError::InvalidParam {
            message: "parameter `instance` must not be empty".to_string(),
            param: "instance".to_string(),
        });
    }
    let label = label.to_ascii_lowercase();
    let clients = named_clients();
    if let Some(client) = clients.get(&label) {
        return Ok(client.clone());
    }

    // Check whether the label was at least discovered in the environment.
    // If so, the config is broken rather than the label being unknown.
    let labels = all_labels();
    if labels.iter().any(|l| l == &label) {
        let upper = if label == "default" {
            String::new()
        } else {
            format!("_{}", label.to_ascii_uppercase())
        };
        let url_var = format!("UNRAID{upper}_URL");
        let key_var = format!("UNRAID{upper}_API_KEY");
        return Err(ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: format!(
                "instance `{label}` is configured but `{url_var}` or `{key_var}` is missing or invalid"
            ),
        });
    }

    let mut valid: Vec<String> = clients.keys().cloned().collect();
    valid.sort();
    Err(ToolError::UnknownInstance {
        message: format!("unknown instance `{label}`"),
        valid,
    })
}

/// Return a client or a structured `internal_error` if not configured.
///
/// Used by MCP and CLI dispatch where `AppState` is not available.
pub fn require_client() -> Result<UnraidClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNRAID_URL or UNRAID_API_KEY not configured".to_string(),
    })
}
