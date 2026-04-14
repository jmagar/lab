use lab_apis::core::Auth;
use lab_apis::tailscale::TailscaleClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Resolve the tailnet value: env var overrides config.toml, default is `"-"`.
fn resolve_tailnet() -> String {
    env_non_empty("TAILSCALE_TAILNET")
        .or_else(|| {
            crate::config::load_toml(&crate::config::toml_candidates())
                .ok()
                .and_then(|cfg| cfg.services.tailscale.tailnet)
        })
        .unwrap_or_else(|| "-".to_string())
}

/// Build a `TailscaleClient` from environment variables + config.toml.
///
/// Returns `None` if `TAILSCALE_API_KEY` is absent or empty.
/// Called by `AppState` at startup — keep pure (no side effects, no logging).
pub fn client_from_env() -> Option<TailscaleClient> {
    let key = env_non_empty("TAILSCALE_API_KEY")?;
    let base_url = env_non_empty("TAILSCALE_BASE_URL")
        .unwrap_or_else(|| "https://api.tailscale.com/api/v2".to_string());
    let tailnet = resolve_tailnet();
    TailscaleClient::new(&base_url, Auth::Bearer { token: key }, tailnet).ok()
}

/// Return a client or a structured `internal_error`.
///
/// For MCP/CLI paths that do not have access to `AppState`.
pub fn require_client() -> Result<TailscaleClient, ToolError> {
    let key = env_non_empty("TAILSCALE_API_KEY").ok_or_else(not_configured_error)?;
    let base_url = env_non_empty("TAILSCALE_BASE_URL")
        .unwrap_or_else(|| "https://api.tailscale.com/api/v2".to_string());
    let tailnet = resolve_tailnet();
    TailscaleClient::new(&base_url, Auth::Bearer { token: key }, tailnet).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("Tailscale client init failed: {e}"),
    })
}

/// Structured error for callers holding a pre-built `Option<TailscaleClient>`.
/// The API handler calls this directly instead of re-reading env vars.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: "TAILSCALE_API_KEY not configured".to_string(),
    }
}
