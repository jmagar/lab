//! Shared catalog module — single source of truth for service + action
//! discovery, feeding three surfaces: the `lab.help` MCP meta-tool, the
//! `lab://catalog` MCP resource, and the `lab help` CLI subcommand.

use serde::{Deserialize, Serialize};

use crate::mcp::registry::ToolRegistry;

/// Top-level discovery document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    /// One entry per registered service.
    pub services: Vec<ServiceCatalog>,
}

/// Per-service slice of the discovery document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCatalog {
    /// Service identifier (matches the MCP tool name and CLI subcommand).
    pub name: String,
    /// Short human description from `PluginMeta::description`.
    pub description: String,
    /// Category slug (Media, Servarr, Notifications, etc.).
    pub category: String,
    /// List of actions exposed by the service.
    pub actions: Vec<ActionEntry>,
}

/// One action inside a service's catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    /// Dotted action name (e.g., `movie.search`).
    pub name: String,
    /// Short description.
    pub description: String,
    /// Whether the action mutates state and requires confirmation.
    pub destructive: bool,
}

/// Build a [`Catalog`] from the current tool registry.
#[must_use]
pub fn build_catalog(registry: &ToolRegistry) -> Catalog {
    let services = registry
        .services()
        .iter()
        .map(|svc| ServiceCatalog {
            name: svc.name.to_string(),
            description: svc.description.to_string(),
            category: svc.category.to_string(),
            actions: actions_for(svc.name),
        })
        .collect();

    Catalog { services }
}

/// Convert a service's `&[ActionSpec]` into `Vec<ActionEntry>` for the catalog.
fn convert_actions(specs: &[lab_apis::core::action::ActionSpec]) -> Vec<ActionEntry> {
    specs
        .iter()
        .map(|s| ActionEntry {
            name: s.name.into(),
            description: s.description.into(),
            destructive: s.destructive,
        })
        .collect()
}

fn actions_for(service: &str) -> Vec<ActionEntry> {
    match service {
        "extract" => convert_actions(crate::mcp::services::extract::ACTIONS),
        #[cfg(feature = "radarr")]
        "radarr" => convert_actions(crate::mcp::services::radarr::ACTIONS),
        #[cfg(feature = "sonarr")]
        "sonarr" => convert_actions(crate::mcp::services::sonarr::ACTIONS),
        #[cfg(feature = "prowlarr")]
        "prowlarr" => convert_actions(crate::mcp::services::prowlarr::ACTIONS),
        #[cfg(feature = "plex")]
        "plex" => convert_actions(crate::mcp::services::plex::ACTIONS),
        #[cfg(feature = "tautulli")]
        "tautulli" => convert_actions(crate::mcp::services::tautulli::ACTIONS),
        #[cfg(feature = "sabnzbd")]
        "sabnzbd" => convert_actions(crate::mcp::services::sabnzbd::ACTIONS),
        #[cfg(feature = "qbittorrent")]
        "qbittorrent" => convert_actions(crate::mcp::services::qbittorrent::ACTIONS),
        #[cfg(feature = "tailscale")]
        "tailscale" => convert_actions(crate::mcp::services::tailscale::ACTIONS),
        #[cfg(feature = "linkding")]
        "linkding" => convert_actions(crate::mcp::services::linkding::ACTIONS),
        #[cfg(feature = "memos")]
        "memos" => convert_actions(crate::mcp::services::memos::ACTIONS),
        #[cfg(feature = "bytestash")]
        "bytestash" => convert_actions(crate::services::bytestash::ACTIONS),
        #[cfg(feature = "paperless")]
        "paperless" => convert_actions(crate::mcp::services::paperless::ACTIONS),
        #[cfg(feature = "arcane")]
        "arcane" => convert_actions(crate::mcp::services::arcane::ACTIONS),
        #[cfg(feature = "unraid")]
        "unraid" => convert_actions(crate::mcp::services::unraid::ACTIONS),
        #[cfg(feature = "unifi")]
        "unifi" => convert_actions(crate::mcp::services::unifi::ACTIONS),
        #[cfg(feature = "overseerr")]
        "overseerr" => convert_actions(crate::mcp::services::overseerr::ACTIONS),
        #[cfg(feature = "gotify")]
        "gotify" => convert_actions(crate::mcp::services::gotify::ACTIONS),
        #[cfg(feature = "openai")]
        "openai" => convert_actions(crate::mcp::services::openai::ACTIONS),
        #[cfg(feature = "qdrant")]
        "qdrant" => convert_actions(crate::mcp::services::qdrant::ACTIONS),
        #[cfg(feature = "tei")]
        "tei" => convert_actions(crate::mcp::services::tei::ACTIONS),
        #[cfg(feature = "apprise")]
        "apprise" => convert_actions(crate::mcp::services::apprise::ACTIONS),
        _ => Vec::new(),
    }
}
