//! Shared catalog module — single source of truth for service + action
//! discovery, feeding three surfaces: the `lab.help` MCP meta-tool, the
//! `lab://catalog` MCP resource, and the `lab help` CLI subcommand.

use serde::{Deserialize, Serialize};

use crate::registry::ToolRegistry;

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
    /// Implementation status: `"available"` or `"stub"`.
    ///
    /// Filter on `status == "available"` to find services that are callable.
    /// `"stub"` means compiled-in but not yet dispatching real actions.
    pub status: String,
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
            status: svc.status.to_string(),
            actions: svc
                .actions
                .iter()
                .map(|a| ActionEntry {
                    name: a.name.into(),
                    description: a.description.into(),
                    destructive: a.destructive,
                })
                .collect(),
        })
        .collect::<Vec<_>>();

    Catalog { services }
}
