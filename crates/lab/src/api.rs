//! HTTP API surface for `lab`.
//!
//! Thin axum layer over `lab-apis` clients. Mirrors the MCP dispatch shape:
//! one route group per service, action + params dispatch, structured JSON
//! error envelopes with stable `kind` tags.
//!
//! The API is an optional surface — the binary can run as CLI-only, MCP-only,
//! HTTP-only, or any combination. Routes are feature-gated per service.

/// Shared application state (service clients, config).
pub mod state;

/// `ApiError` → HTTP response mapping with structured envelopes.
pub mod error;

/// Router builder — composes all feature-gated route groups.
pub mod router;

/// `GET /health` and `GET /ready` liveness/readiness probes.
pub mod health;

#[allow(unused_imports)]
pub use error::{ApiError, ApiResult};
#[allow(unused_imports)]
pub use router::build_router;
#[allow(unused_imports)]
pub use state::AppState;
