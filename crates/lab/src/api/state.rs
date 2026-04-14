//! Shared application state for axum handlers.

use std::collections::HashSet;
use std::sync::Arc;

use crate::catalog::{Catalog, build_catalog};
use crate::dispatch::clients::ServiceClients;
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
    /// Optional JWKS manager for OAuth JWT validation.
    ///
    /// `None` when `LAB_OAUTH_ISSUER` is not configured — only static bearer
    /// auth is available. When `Some`, the auth middleware tries JWT validation
    /// after a static bearer mismatch.
    pub jwks: Option<Arc<crate::api::oauth::JwksManager>>,
    /// Resolved OAuth configuration, if present.
    ///
    /// Stored in `AppState` so that handlers (e.g. protected resource metadata,
    /// WWW-Authenticate headers) can read from resolved config rather than
    /// re-reading env vars at request time.
    pub oauth_config: Option<Arc<crate::config::OAuthConfig>>,
    /// Optional upstream MCP server pool for gateway proxy dispatch.
    ///
    /// `None` when no `[[upstream]]` entries are configured in `config.toml`.
    pub upstream_pool: Option<Arc<crate::dispatch::upstream::pool::UpstreamPool>>,
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
            jwks: None,
            oauth_config: None,
            upstream_pool: None,
        }
    }

    /// Attach a pre-built `JwksManager` for OAuth JWT validation.
    #[must_use]
    pub fn with_jwks(mut self, jwks: Arc<crate::api::oauth::JwksManager>) -> Self {
        self.jwks = Some(jwks);
        self
    }

    /// Attach the resolved OAuth configuration.
    #[must_use]
    pub fn with_oauth_config(mut self, config: crate::config::OAuthConfig) -> Self {
        self.oauth_config = Some(Arc::new(config));
        self
    }

    /// Attach an upstream MCP server pool for gateway proxy dispatch.
    #[must_use]
    #[allow(dead_code)] // Will be called when `lab serve` wires [[upstream]] config.
    pub fn with_upstream_pool(
        mut self,
        pool: Arc<crate::dispatch::upstream::pool::UpstreamPool>,
    ) -> Self {
        self.upstream_pool = Some(pool);
        self
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
