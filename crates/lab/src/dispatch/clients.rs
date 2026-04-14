use std::sync::Arc;

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
        }
    }
}
