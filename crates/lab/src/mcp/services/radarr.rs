//! MCP dispatch for the Radarr tool.

use anyhow::Result;
use serde_json::{Value, json};

use lab_apis::core::Auth;
use lab_apis::radarr::RadarrClient;

/// Build a Radarr client from the default-instance env vars.
#[must_use]
pub fn client_from_env() -> Option<RadarrClient> {
    let url = std::env::var("RADARR_URL").ok()?;
    let key = std::env::var("RADARR_API_KEY").ok()?;
    Some(RadarrClient::new(
        &url,
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key,
        },
    ))
}

/// Dispatch one MCP call against the Radarr tool.
///
/// # Errors
pub async fn dispatch(action: &str, _params: Value) -> Result<Value> {
    match action {
        "help" => Ok(json!({
            "service": "radarr",
            "actions": [
                { "name": "system.status", "description": "Return Radarr system status", "destructive": false },
                { "name": "help", "description": "Show this catalog", "destructive": false },
            ]
        })),
        "system.status" => {
            let client = client_from_env()
                .ok_or_else(|| anyhow::anyhow!("missing RADARR_URL or RADARR_API_KEY"))?;
            let status = client.system_status().await?;
            Ok(serde_json::to_value(status)?)
        }
        unknown => {
            anyhow::bail!("unknown action `radarr.{unknown}` — call `radarr.help` for the catalog")
        }
    }
}
