use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use lab_apis::core::Auth;
use lab_apis::openacp::OpenAcpClient;

use crate::config::scan_instances;
use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

struct OpenAcpInstancePool {
    clients: HashMap<String, Arc<OpenAcpClient>>,
    all_labels: Vec<String>,
}

static POOL: OnceLock<OpenAcpInstancePool> = OnceLock::new();

fn build_client(url: &str, token: &str) -> Result<OpenAcpClient, lab_apis::openacp::OpenAcpError> {
    let auth = Auth::Bearer {
        token: token.to_string(),
    };
    OpenAcpClient::new(url, auth)
}

fn pool() -> &'static OpenAcpInstancePool {
    POOL.get_or_init(|| {
        let mut clients = HashMap::new();
        let mut all_labels = Vec::new();
        for (label, vars) in scan_instances("OPENACP") {
            all_labels.push(label.clone());
            let Some(url) = vars
                .get("URL")
                .map(|v| v.expose().to_string())
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            let Some(token) = vars
                .get("TOKEN")
                .map(|v| v.expose().to_string())
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            if let Ok(client) = build_client(&url, &token) {
                clients.insert(label, Arc::new(client));
            }
        }
        OpenAcpInstancePool {
            clients,
            all_labels,
        }
    })
}

/// Build a default OpenACP client from env vars.
pub fn client_from_env() -> Option<OpenAcpClient> {
    let url = env_non_empty("OPENACP_URL")?;
    let token = env_non_empty("OPENACP_TOKEN")?;
    build_client(&url, &token)
        .map_err(|e| tracing::warn!(error = %e, "openacp client construction failed"))
        .ok()
}

/// Resolve a named OpenACP instance from `{OPENACP}_{LABEL}_URL` / `TOKEN`.
pub fn client_from_instance(label: Option<&str>) -> Result<Arc<OpenAcpClient>, ToolError> {
    let label = label.unwrap_or("default").trim();
    if label.is_empty() {
        return Err(ToolError::InvalidParam {
            message: "parameter `instance` must not be empty".to_string(),
            param: "instance".to_string(),
        });
    }
    let label = label.to_ascii_lowercase();
    if let Some(client) = pool().clients.get(&label) {
        return Ok(client.clone());
    }
    if pool().all_labels.iter().any(|known| known == &label) {
        return Err(ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: format!(
                "instance `{label}` is configured but OPENACP_URL or OPENACP_TOKEN is missing or invalid"
            ),
        });
    }
    let mut valid: Vec<String> = pool().clients.keys().cloned().collect();
    valid.sort();
    Err(ToolError::UnknownInstance {
        message: format!("unknown instance `{label}`"),
        valid,
    })
}

/// Return a default client or a structured not-configured error.
pub fn require_client() -> Result<OpenAcpClient, ToolError> {
    client_from_env().ok_or_else(not_configured_error)
}

/// Structured error returned when OpenACP env vars are absent.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "OPENACP_URL and OPENACP_TOKEN not configured".to_string(),
    }
}
