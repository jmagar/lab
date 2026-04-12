use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use lab_apis::core::Auth;
use lab_apis::unifi::UnifiClient;

use crate::config::scan_instances;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Module-level cache of named `UniFi` clients, built once on first access.
static NAMED_CLIENTS: OnceLock<HashMap<String, Arc<UnifiClient>>> = OnceLock::new();

/// All labels discovered by `scan_instances("UNIFI")`, built once alongside
/// `NAMED_CLIENTS`. A label present here but absent from `NAMED_CLIENTS`
/// means the instance was discovered but its env vars are broken/incomplete.
static ALL_LABELS: OnceLock<Vec<String>> = OnceLock::new();

/// Return (or lazily build) the map of named `UniFi` clients.
///
/// The map is keyed by label (e.g. `"default"`, `"home"`) and built once
/// by scanning env vars at first call. Subsequent calls are lock-free reads.
fn named_clients() -> &'static HashMap<String, Arc<UnifiClient>> {
    NAMED_CLIENTS.get_or_init(|| {
        let mut map = HashMap::new();
        let mut all: Vec<String> = Vec::new();
        for (label, _vars) in scan_instances("UNIFI") {
            all.push(label.clone());
            let (url_key, key_key) = if label == "default" {
                ("UNIFI_URL".to_string(), "UNIFI_API_KEY".to_string())
            } else {
                let upper = label.to_ascii_uppercase();
                (
                    format!("UNIFI_{upper}_URL"),
                    format!("UNIFI_{upper}_API_KEY"),
                )
            };
            if let (Some(url), Some(key)) = (env_non_empty(&url_key), env_non_empty(&key_key)) {
                if let Ok(client) = UnifiClient::new(
                    &url,
                    Auth::ApiKey {
                        header: "X-API-KEY".into(),
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

/// Build a `UniFi` client from the default-instance env vars.
pub fn client_from_env() -> Option<UnifiClient> {
    let url = env_non_empty("UNIFI_URL")?;
    let key = env_non_empty("UNIFI_API_KEY")?;
    UnifiClient::new(
        &url,
        Auth::ApiKey {
            header: "X-API-KEY".into(),
            key,
        },
    )
    .ok()
}

/// Resolve a `UniFi` client by optional instance label.
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
pub fn client_from_instance(label: Option<&str>) -> Result<Arc<UnifiClient>, ToolError> {
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
        let url_var = format!("UNIFI{upper}_URL");
        let key_var = format!("UNIFI{upper}_API_KEY");
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

#[allow(dead_code)]
pub fn require_client() -> Result<UnifiClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNIFI_URL or UNIFI_API_KEY not configured".to_string(),
    })
}
