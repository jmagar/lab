use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use lab_apis::core::Auth;
use lab_apis::unifi::UnifiClient;

use crate::config::scan_instances;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Module-level cache of named `UniFi` clients, built once on first access.
static NAMED_CLIENTS: OnceLock<HashMap<String, Arc<UnifiClient>>> = OnceLock::new();

/// Return (or lazily build) the map of named `UniFi` clients.
///
/// The map is keyed by label (e.g. `"default"`, `"home"`) and built once
/// by scanning env vars at first call. Subsequent calls are lock-free reads.
fn named_clients() -> &'static HashMap<String, Arc<UnifiClient>> {
    NAMED_CLIENTS.get_or_init(|| {
        let mut map = HashMap::new();
        for (label, _vars) in scan_instances("UNIFI") {
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
        map
    })
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

/// Build a `UniFi` client for a named instance.
///
/// The label must exist in the scanned env set, e.g. `default` or `node2`.
/// Returns an `Arc`-wrapped client from the module-level cache — no new
/// `reqwest::Client` is created after the first call.
pub fn client_from_instance(label: &str) -> Result<Arc<UnifiClient>, ToolError> {
    let label = label.trim();
    if label.is_empty() {
        return Err(ToolError::InvalidParam {
            message: "parameter `instance` must not be empty".to_string(),
            param: "instance".to_string(),
        });
    }
    let label = label.to_ascii_lowercase();
    let clients = named_clients();
    clients.get(&label).cloned().ok_or_else(|| {
        let mut valid: Vec<String> = clients.keys().cloned().collect();
        valid.sort();
        ToolError::UnknownInstance {
            message: format!("unknown instance `{label}`"),
            valid,
        }
    })
}

pub fn require_client() -> Result<UnifiClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNIFI_URL or UNIFI_API_KEY not configured".to_string(),
    })
}
