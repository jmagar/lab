//! Shared application state for axum handlers.

use std::collections::HashSet;
use std::sync::Arc;

use crate::catalog::{Catalog, build_catalog};
use crate::dispatch::clients::ServiceClients;
// NOTE: The API surface imports ToolRegistry for service metadata (names,
// descriptions, categories) used in route mounting and OpenAPI generation.
// It does NOT use the dispatch function pointers. `build_default_registry` is
// called in `AppState::new()` to construct the registry. A future refactor
// could extract ServiceMetadata into dispatch/ to eliminate this cross-surface
// import.
use crate::mcp::registry::{ToolRegistry, build_default_registry};

/// Application state passed to every axum handler via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// Pre-built service+action catalog for discovery endpoints.
    pub catalog: Arc<Catalog>,
    /// Tool registry with dispatch functions for each service.
    ///
    /// Used by `build_router_with_bearer` to enforce runtime service filtering:
    /// only services present in the registry get their HTTP routes mounted,
    /// even when their compile-time feature flag is enabled.
    pub registry: Arc<ToolRegistry>,
    /// Pre-built service clients for connection pool reuse.
    pub clients: Arc<ServiceClients>,
    /// Runtime-enabled service names derived from the registry.
    ///
    /// The HTTP router checks this set to decide which per-service route groups
    /// to mount.  When `--services` filtering is applied, only the listed names
    /// appear here, so filtered-out services have no reachable POST endpoint.
    #[allow(dead_code)]
    pub enabled_services: Arc<HashSet<String>>,
    /// Resolved auth configuration, if present.
    ///
    /// Stored in `AppState` so that handlers (e.g. protected resource metadata,
    /// WWW-Authenticate headers) can read from resolved config rather than
    /// re-reading env vars at request time.
    pub auth_config: Option<Arc<lab_auth::config::AuthConfig>>,
    /// OAuth-mode auth server state, mounted only when LAB_AUTH_MODE=oauth.
    pub oauth_state: Option<Arc<lab_auth::state::AuthState>>,
    /// Shared gateway manager for runtime upstream pool access and config mutation.
    ///
    /// `None` when gateway management is not wired for this process.
    pub gateway_manager: Option<Arc<crate::dispatch::gateway::manager::GatewayManager>>,
}

impl AppState {
    /// Build state from the default (all enabled features) registry.
    #[must_use]
    pub fn new() -> Self {
        let registry = build_default_registry();
        Self::from_registry(registry)
    }

    /// Build state from a pre-filtered or pre-built registry.
    ///
    /// Use this when the caller has already applied service filtering (e.g.
    /// `--services` on `lab serve --transport http`) so that the HTTP surface
    /// respects the same service set as the stdio surface.
    ///
    /// `enabled_services` is derived from the registry entries so the router
    /// can skip mounting handlers for services that were filtered out.
    #[must_use]
    pub fn from_registry(registry: ToolRegistry) -> Self {
        let enabled_services: HashSet<String> = registry
            .services()
            .iter()
            .map(|e| e.name.to_string())
            .collect();
        let catalog = Arc::new(build_catalog(&registry));
        let clients = Arc::new(ServiceClients::from_env());
        Self {
            catalog,
            registry: Arc::new(registry),
            clients,
            enabled_services: Arc::new(enabled_services),
            auth_config: None,
            oauth_state: None,
            gateway_manager: None,
        }
    }

    /// Attach the resolved auth configuration.
    #[must_use]
    pub fn with_auth_config(mut self, config: lab_auth::config::AuthConfig) -> Self {
        self.auth_config = Some(Arc::new(config));
        self
    }

    #[must_use]
    pub fn with_oauth_state(mut self, auth_state: lab_auth::state::AuthState) -> Self {
        self.oauth_state = Some(Arc::new(auth_state));
        self
    }

    /// Attach the shared gateway manager.
    #[must_use]
    #[allow(dead_code)] // Called by `lab serve` when gateway runtime is wired.
    pub fn with_gateway_manager(
        mut self,
        manager: Arc<crate::dispatch::gateway::manager::GatewayManager>,
    ) -> Self {
        self.gateway_manager = Some(manager);
        self
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
