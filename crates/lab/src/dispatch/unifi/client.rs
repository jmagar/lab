use lab_apis::core::Auth;
use lab_apis::unifi::UnifiClient;

use crate::config::scan_instances;
use crate::dispatch::error::ToolError;

/// Build a `UniFi` client from the default-instance env vars.
pub fn client_from_env() -> Option<UnifiClient> {
    let url = std::env::var("UNIFI_URL").ok().filter(|v| !v.is_empty())?;
    let key = std::env::var("UNIFI_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())?;
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
pub fn client_from_instance(label: &str) -> Result<UnifiClient, ToolError> {
    let label = label.trim();
    if label.is_empty() {
        return Err(ToolError::InvalidParam {
            message: "parameter `instance` must not be empty".to_string(),
            param: "instance".to_string(),
        });
    }
    let label = label.to_ascii_lowercase();
    let valid = {
        let mut labels: Vec<String> = scan_instances("UNIFI").keys().cloned().collect();
        labels.sort();
        labels
    };
    if !valid.iter().any(|valid_label| valid_label == &label) {
        return Err(ToolError::UnknownInstance {
            message: format!("unknown instance `{label}`"),
            valid,
        });
    }

    let (url_key, key_key) = if label == "default" {
        ("UNIFI_URL".to_string(), "UNIFI_API_KEY".to_string())
    } else {
        let upper = label.to_ascii_uppercase();
        (
            format!("UNIFI_{upper}_URL"),
            format!("UNIFI_{upper}_API_KEY"),
        )
    };

    let url = std::env::var(&url_key)
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: format!("{url_key} not configured"),
        })?;
    let key = std::env::var(&key_key)
        .ok()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: format!("{key_key} not configured"),
        })?;

    UnifiClient::new(
        &url,
        Auth::ApiKey {
            header: "X-API-KEY".into(),
            key,
        },
    )
    .map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("failed to initialize UniFi client: {e}"),
    })
}

pub fn require_client() -> Result<UnifiClient, ToolError> {
    client_from_env().ok_or_else(|| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "UNIFI_URL or UNIFI_API_KEY not configured".to_string(),
    })
}
