//! Shared application state for axum handlers.

use std::collections::HashSet;
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
    #[cfg(feature = "apprise")]
    pub apprise: Option<Arc<lab_apis::apprise::AppriseClient>>,
    #[cfg(feature = "bytestash")]
    pub bytestash: Option<Arc<lab_apis::bytestash::ByteStashClient>>,
    #[cfg(feature = "qdrant")]
    pub qdrant: Option<Arc<lab_apis::qdrant::QdrantClient>>,
    #[cfg(feature = "radarr")]
    pub radarr: Option<Arc<lab_apis::radarr::RadarrClient>>,
    #[cfg(feature = "sabnzbd")]
    pub sabnzbd: Option<Arc<lab_apis::sabnzbd::SabnzbdClient>>,
    #[cfg(feature = "tei")]
    pub tei: Option<Arc<lab_apis::tei::TeiClient>>,
    // Kept for future health-check and sub-dispatcher threading (see TODO above).
    #[cfg(feature = "unifi")]
    #[allow(dead_code)]
    pub unifi: Option<Arc<lab_apis::unifi::UnifiClient>>,
    #[cfg(feature = "unraid")]
    pub unraid: Option<Arc<lab_apis::unraid::UnraidClient>>,
    #[cfg(feature = "gotify")]
    pub gotify: Option<Arc<crate::dispatch::gotify::GotifyClients>>,
    #[cfg(feature = "linkding")]
    pub linkding: Option<Arc<lab_apis::linkding::LinkdingClient>>,
    #[cfg(feature = "paperless")]
    pub paperless: Option<Arc<lab_apis::paperless::PaperlessClient>>,
    #[cfg(feature = "prowlarr")]
    pub prowlarr: Option<Arc<lab_apis::prowlarr::ProwlarrClient>>,
}

impl ServiceClients {
    /// Build all service clients from environment variables.
    ///
    /// Called once at startup. Returns `None` per field when env vars are missing.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            #[cfg(feature = "apprise")]
            apprise: crate::dispatch::apprise::client_from_env().map(Arc::new),
            #[cfg(feature = "bytestash")]
            bytestash: crate::dispatch::bytestash::client_from_env().map(Arc::new),
            #[cfg(feature = "qdrant")]
            qdrant: crate::dispatch::qdrant::client_from_env().map(Arc::new),
            #[cfg(feature = "radarr")]
            radarr: crate::dispatch::radarr::client_from_env().map(Arc::new),
            #[cfg(feature = "sabnzbd")]
            sabnzbd: crate::dispatch::sabnzbd::client_from_env().map(Arc::new),
            #[cfg(feature = "tei")]
            tei: crate::dispatch::tei::client_from_env().map(Arc::new),
            #[cfg(feature = "unifi")]
            unifi: crate::dispatch::unifi::client_from_env().map(Arc::new),
            #[cfg(feature = "unraid")]
            unraid: crate::dispatch::unraid::client_from_env().map(Arc::new),
            #[cfg(feature = "gotify")]
            gotify: crate::dispatch::gotify::clients_from_env().map(Arc::new),
            #[cfg(feature = "linkding")]
            linkding: crate::dispatch::linkding::client_from_env().map(Arc::new),
            #[cfg(feature = "paperless")]
            paperless: crate::dispatch::paperless::client_from_env().map(Arc::new),
            #[cfg(feature = "prowlarr")]
            prowlarr: crate::dispatch::prowlarr::client_from_env().map(Arc::new),
        }
    }
}

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
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
