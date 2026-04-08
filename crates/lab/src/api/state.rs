//! Shared application state for axum handlers.

use std::sync::Arc;

use crate::catalog::{Catalog, build_catalog};
use crate::mcp::registry::build_default_registry;

/// Application state passed to every axum handler via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// Pre-built service+action catalog for discovery endpoints.
    pub catalog: Arc<Catalog>,
}

impl AppState {
    /// Build state from the default (all enabled features) registry.
    #[must_use]
    pub fn new() -> Self {
        let registry = build_default_registry();
        Self {
            catalog: Arc::new(build_catalog(&registry)),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
