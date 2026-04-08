//! Shared application state for axum handlers.
//!
//! Intentionally minimal — service clients are constructed per-request
//! inside each handler's MCP dispatch call via `client_from_env()`.

/// Application state passed to every axum handler via `State<AppState>`.
#[derive(Clone, Default)]
pub struct AppState;

impl AppState {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
