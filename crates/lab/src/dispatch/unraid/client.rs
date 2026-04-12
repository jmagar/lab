//! Client construction helpers for the `unraid` dispatch layer.

use lab_apis::core::Auth;
use lab_apis::unraid::UnraidClient;

use crate::config::scan_instances;
use crate::dispatch::error::ToolError;

/// Build an `UnraidClient` from the default-instance environment variables.
///
/// Reads `UNRAID_URL` and `UNRAID_API_KEY`. Returns `None` if either is
/// absent or empty. Called by `AppState::ServiceClients::from_env()` at
/// startup — must be pure (no side effects, no logging).
pub fn client_from_env() -> Option<UnraidClient> {
    let url = std::env::var("UNRAID_URL").ok().filter(|v| !v.is_empty())?;
    // Normalise: strip trailing /graphql if present so the URL stored in env
    // can be either the bare host (https://host:31337) or the full endpoint
    // (https://host:31337/graphql) — the client always appends /graphql itself.
    let url = url
        .trim_end_matches('/')
        .trim_end_matches("graphql")
        .trim_end_matches('/')
        .to_string();
    let key = std::env::var("UNRAID_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())?;
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

/// Build a `UnraidClient` for a named instance.
pub fn client_from_instance(label: &str) -> Result<UnraidClient, ToolError> {
    let label = label.trim();
    if label.is_empty() {
        return Err(ToolError::InvalidParam {
            message: "parameter `instance` must not be empty".to_string(),
            param: "instance".to_string(),
        });
    }
    let label = label.to_ascii_lowercase();
    let valid = {
        let mut labels: Vec<String> = scan_instances("UNRAID").keys().cloned().collect();
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
        ("UNRAID_URL".to_string(), "UNRAID_API_KEY".to_string())
    } else {
        let upper = label.to_ascii_uppercase();
        (
            format!("UNRAID_{upper}_URL"),
            format!("UNRAID_{upper}_API_KEY"),
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
    .map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("failed to initialize Unraid client: {e}"),
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
