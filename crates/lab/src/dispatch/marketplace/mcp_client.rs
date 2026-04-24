//! McpRegistry client construction helpers for `mcp.*` actions in marketplace dispatch.
//!
//! When `MCPREGISTRY_URL` is not set, `mcp.*` actions return a structured
//! `not_configured` error. Unlike the standalone mcpregistry dispatch which
//! falls back to the public default URL, the marketplace integration treats
//! absence of `MCPREGISTRY_URL` as "not configured" per the lab-zxx5.2 spec.

#[cfg(feature = "mcpregistry")]
use lab_apis::core::Auth;
#[cfg(feature = "mcpregistry")]
use lab_apis::mcpregistry::client::McpRegistryClient;

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

/// Return a McpRegistry client or a structured `not_configured` error.
///
/// Requires `MCPREGISTRY_URL` to be set; does NOT fall back to any default URL.
/// This intentionally differs from `mcpregistry::client::require_client()`.
#[cfg(feature = "mcpregistry")]
pub fn require_mcp_client() -> Result<McpRegistryClient, ToolError> {
    let url = env_non_empty("MCPREGISTRY_URL").ok_or_else(not_configured_error)?;
    McpRegistryClient::new(&url, Auth::None).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".to_string(),
        message: format!("McpRegistry client init failed: {e}"),
    })
}

/// Structured `not_configured` error for callers that hold a pre-built `Option<McpRegistryClient>`.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "not_configured".to_string(),
        message: "MCPREGISTRY_URL not set".to_string(),
    }
}

// Suppress unused warning when mcpregistry feature is off.
#[cfg(not(feature = "mcpregistry"))]
pub fn require_mcp_client() -> Result<(), ToolError> {
    Err(not_configured_error())
}

// TODO(lab-zxx5.3): Add ACP client stub/placeholder here once ACP registry
// client is available in lab-apis. The three-client architecture will be:
// 1. Marketplace filesystem client (always available, in client.rs)
// 2. McpRegistryClient (this file, requires MCPREGISTRY_URL)
// 3. AcpRegistryClient (placeholder — lab-zxx5.3)
