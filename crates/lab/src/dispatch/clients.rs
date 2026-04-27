use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use tokio::sync::RwLock;

/// Pre-built service clients, constructed once at startup from environment variables.
///
/// Each optional field is `None` when the required env vars are absent.
/// Surfaces extract the pre-built client to avoid per-request env reads and
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
    #[cfg(feature = "plex")]
    pub plex: Option<Arc<lab_apis::plex::PlexClient>>,
    #[cfg(feature = "sonarr")]
    pub sonarr: Option<Arc<lab_apis::sonarr::SonarrClient>>,
    #[cfg(feature = "overseerr")]
    pub overseerr: Option<Arc<lab_apis::overseerr::OverseerrClient>>,
    #[cfg(feature = "openai")]
    pub openai: Option<Arc<lab_apis::openai::OpenAiClient>>,
    #[cfg(feature = "memos")]
    pub memos: Option<Arc<lab_apis::memos::MemosClient>>,
    #[cfg(feature = "tailscale")]
    pub tailscale: Option<Arc<lab_apis::tailscale::TailscaleClient>>,
    #[cfg(feature = "tautulli")]
    pub tautulli: Option<Arc<lab_apis::tautulli::TautulliClient>>,
    #[cfg(feature = "arcane")]
    pub arcane: Option<Arc<lab_apis::arcane::ArcaneClient>>,
    #[cfg(feature = "qbittorrent")]
    pub qbittorrent: Option<Arc<lab_apis::qbittorrent::QbittorrentClient>>,
    #[cfg(feature = "beads")]
    pub beads: Option<Arc<lab_apis::beads::BeadsClient>>,
    // [lab-scaffold: state-fields]
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
            #[cfg(feature = "plex")]
            plex: crate::dispatch::plex::client_from_env().map(Arc::new),
            #[cfg(feature = "sonarr")]
            sonarr: crate::dispatch::sonarr::client_from_env().map(Arc::new),
            #[cfg(feature = "overseerr")]
            overseerr: crate::dispatch::overseerr::client_from_env().map(Arc::new),
            #[cfg(feature = "openai")]
            openai: crate::dispatch::openai::client_from_env().map(Arc::new),
            #[cfg(feature = "memos")]
            memos: crate::dispatch::memos::client_from_env().map(Arc::new),
            #[cfg(feature = "tailscale")]
            tailscale: crate::dispatch::tailscale::client_from_env().map(Arc::new),
            #[cfg(feature = "tautulli")]
            tautulli: crate::dispatch::tautulli::client_from_env().map(Arc::new),
            #[cfg(feature = "arcane")]
            arcane: crate::dispatch::arcane::client_from_env().map(Arc::new),
            #[cfg(feature = "qbittorrent")]
            qbittorrent: crate::dispatch::qbittorrent::client_from_env().map(Arc::new),
            #[cfg(feature = "beads")]
            beads: crate::dispatch::beads::client_from_env().map(Arc::new),
            // [lab-scaffold: state-from-env]
        }
    }

    #[must_use]
    pub fn from_env_map(values: HashMap<String, String>) -> Self {
        crate::dispatch::helpers::with_env_override(values, Self::from_env)
    }
}

#[derive(Clone, Default)]
pub struct SharedServiceClients {
    inner: Arc<RwLock<ServiceClients>>,
    #[cfg(test)]
    refresh_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl SharedServiceClients {
    #[must_use]
    pub fn from_clients(clients: ServiceClients) -> Self {
        Self {
            inner: Arc::new(RwLock::new(clients)),
            #[cfg(test)]
            refresh_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn from_env() -> Self {
        Self::from_clients(ServiceClients::from_env())
    }

    #[allow(dead_code)]
    pub async fn snapshot(&self) -> ServiceClients {
        self.inner.read().await.clone()
    }

    pub async fn refresh_from_env_path(&self, path: &Path) -> anyhow::Result<()> {
        let values = dotenvy::from_path_iter(path)
            .ok()
            .map(|iter| iter.filter_map(Result::ok).collect())
            .unwrap_or_default();
        *self.inner.write().await = ServiceClients::from_env_map(values);
        #[cfg(test)]
        self.refresh_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }

    #[cfg(test)]
    pub fn refresh_count(&self) -> usize {
        self.refresh_count.load(std::sync::atomic::Ordering::SeqCst)
    }
}
