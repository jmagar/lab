//! Shared application state for axum handlers.

use std::sync::Arc;

use crate::catalog::{Catalog, build_catalog};
use crate::mcp::registry::{ToolRegistry, build_default_registry};

/// Pre-built service clients, constructed once at startup from environment variables.
///
/// Each optional field is `None` when the required env vars are absent.
/// API handlers extract the pre-built client to avoid per-request env reads and
/// `reqwest::Client` (connection pool) construction.
///
/// # TODO(perf): sub-dispatcher threading
///
/// Radarr and `UniFi` use multi-level dispatch — their sub-dispatchers
/// (`movies`, `queue`, `devices`, etc.) each call `require_client()` independently.
/// Threading the pre-built client through those sub-dispatchers is a follow-on task.
/// `ByteStash` and `SABnzbd` are fully wired to use the client here.
#[derive(Clone, Default)]
pub struct ServiceClients {
    #[cfg(feature = "bytestash")]
    pub bytestash: Option<Arc<lab_apis::bytestash::ByteStashClient>>,
    #[cfg(feature = "radarr")]
    pub radarr: Option<Arc<lab_apis::radarr::RadarrClient>>,
    #[cfg(feature = "sabnzbd")]
    pub sabnzbd: Option<Arc<lab_apis::sabnzbd::SabnzbdClient>>,
    #[cfg(feature = "unifi")]
    pub unifi: Option<Arc<lab_apis::unifi::UnifiClient>>,
    #[cfg(feature = "unraid")]
    pub unraid: Option<Arc<lab_apis::unraid::UnraidClient>>,
    #[cfg(feature = "gotify")]
    pub gotify: Option<Arc<crate::dispatch::gotify::GotifyClients>>,
}

impl ServiceClients {
    /// Build all service clients from environment variables.
    ///
    /// Called once at startup. Returns `None` per field when env vars are missing.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            #[cfg(feature = "bytestash")]
            bytestash: crate::dispatch::bytestash::client_from_env().map(Arc::new),
            #[cfg(feature = "radarr")]
            radarr: crate::dispatch::radarr::client_from_env().map(Arc::new),
            #[cfg(feature = "sabnzbd")]
            sabnzbd: crate::dispatch::sabnzbd::client_from_env().map(Arc::new),
            #[cfg(feature = "unifi")]
            unifi: crate::dispatch::unifi::client_from_env().map(Arc::new),
            #[cfg(feature = "unraid")]
            unraid: crate::dispatch::unraid::client_from_env().map(Arc::new),
            #[cfg(feature = "gotify")]
            gotify: crate::dispatch::gotify::clients_from_env().map(Arc::new),
        }
    }
}

/// Application state passed to every axum handler via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    /// Pre-built service+action catalog for discovery endpoints.
    pub catalog: Arc<Catalog>,
    /// Tool registry with dispatch functions for each service.
    #[allow(dead_code)]
    pub registry: Arc<ToolRegistry>,
    /// Pre-built service clients for connection pool reuse.
    pub clients: Arc<ServiceClients>,
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
    #[must_use]
    pub fn from_registry(registry: ToolRegistry) -> Self {
        let catalog = Arc::new(build_catalog(&registry));
        let clients = Arc::new(ServiceClients::from_env());
        Self {
            catalog,
            registry: Arc::new(registry),
            clients,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
