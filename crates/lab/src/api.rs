//! API surface for `lab`.
//!
//! Thin axum layer over `lab-apis` clients. Mirrors the MCP dispatch shape:
//! one route group per service, action + params dispatch, structured JSON
//! error envelopes with stable `kind` tags.
//!
//! The API is an optional surface — the binary can run as CLI-only, MCP-only,
//! HTTP-only, or any combination. Routes are feature-gated per service.

/// Shared application state (service clients, config).
pub mod state;

/// HTTP error re-exports — canonical type is `ToolError` from `dispatch::error`.
pub mod error;

/// Router builder — composes all feature-gated route groups.
pub mod router;

/// `GET /health` and `GET /ready` liveness/readiness probes.
pub mod health;

/// HTTP auth helpers for bearer-or-OAuth mode (metadata, WWW-Authenticate).
pub mod oauth;

/// Per-service HTTP route handlers (one module per service).
pub mod services;

/// Shared request type for all service action dispatchers.
///
/// Every service handler deserializes `POST /v1/<service>` bodies into this
/// struct and forwards `action` + `params` to the corresponding MCP dispatch
/// function, keeping the HTTP and MCP surfaces byte-compatible.
#[derive(Debug, serde::Deserialize)]
pub struct ActionRequest {
    pub action: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[allow(unused_imports)]
pub use error::ToolError;
#[allow(unused_imports)]
pub use state::AppState;
