//! `PluginMeta` — per-service compile-time constants.
//!
//! Each service module exposes `pub const META: PluginMeta` describing its
//! display name, category, docs URL, required/optional env vars, and default
//! port. Drives the TUI plugin manager, `lab install`, `lab doctor`, and the
//! `.mcp.json` patcher.

/// Per-service compile-time metadata.
#[derive(Debug, Clone, Copy)]
pub struct PluginMeta {
    /// Short module name, e.g. `"radarr"`. Matches feature flag and CLI subcommand.
    pub name: &'static str,
    /// Human-readable display name shown in TUI/help.
    pub display_name: &'static str,
    /// One-line description.
    pub description: &'static str,
    /// Logical grouping (used by the TUI plugin manager).
    pub category: Category,
    /// Upstream documentation URL.
    pub docs_url: &'static str,
    /// Env vars that must be set for this service to function.
    pub required_env: &'static [EnvVar],
    /// Env vars that the service understands but doesn't require.
    pub optional_env: &'static [EnvVar],
    /// Default upstream port if conventional, used by `lab doctor`.
    pub default_port: Option<u16>,
}

/// One declared environment variable for a plugin.
#[derive(Debug, Clone, Copy)]
pub struct EnvVar {
    /// Env var name, e.g. `"RADARR_API_KEY"`.
    pub name: &'static str,
    /// Description shown in `lab install` prompts and `lab doctor` output.
    pub description: &'static str,
    /// Example value — never a real credential.
    pub example: &'static str,
    /// True if this is sensitive: TUI masks input, doctor never echoes it.
    pub secret: bool,
}

/// Logical category used by the TUI plugin manager and `lab help`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Media servers (Plex, Tautulli).
    Media,
    /// Servarr stack (Radarr, Sonarr).
    Servarr,
    /// Indexer managers (Prowlarr).
    Indexer,
    /// Download clients (`SABnzbd`, `qBittorrent`).
    Download,
    /// Note-taking and bookmarks (`Memos`, `Linkding`, `ByteStash`).
    Notes,
    /// Document management (Paperless).
    Documents,
    /// Network and infrastructure (Tailscale, `UniFi`, Unraid, Arcane).
    Network,
    /// Push-notification dispatchers (Gotify, Apprise).
    Notifications,
    /// AI / inference / vector search (`OpenAI`, Qdrant, TEI).
    Ai,
    /// Bootstrap utilities (extract, init, doctor).
    Bootstrap,
    /// Marketplace and registry services (Marketplace, MCP Registry).
    Marketplace,
}

impl Category {
    /// Return a lowercase static string label for this category.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Media => "media",
            Self::Servarr => "servarr",
            Self::Indexer => "indexer",
            Self::Download => "download",
            Self::Notes => "notes",
            Self::Documents => "documents",
            Self::Network => "network",
            Self::Notifications => "notifications",
            Self::Ai => "ai",
            Self::Bootstrap => "bootstrap",
            Self::Marketplace => "marketplace",
        }
    }
}
