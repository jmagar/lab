//! Runtime tool registry. Services register themselves here during
//! startup; the MCP server walks the registry to expose tools and the
//! catalog module walks it to produce discovery docs.

use lab_apis::core::action::ActionSpec;

/// Metadata the registry keeps about each registered service.
#[derive(Debug, Clone)]
pub struct RegisteredService {
    /// Service / tool name.
    pub name: &'static str,
    /// Short description from `PluginMeta::description`.
    pub description: &'static str,
    /// Category slug.
    pub category: &'static str,
    /// Actions exposed by this service.
    pub actions: &'static [ActionSpec],
}

/// Collection of registered services, built at startup.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    services: Vec<RegisteredService>,
}

impl ToolRegistry {
    /// Create an empty registry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    /// Register a service. Duplicates are ignored (first registration wins).
    pub fn register(&mut self, service: RegisteredService) {
        if !self.services.iter().any(|s| s.name == service.name) {
            self.services.push(service);
        }
    }

    /// Borrow the current service list.
    #[must_use]
    pub fn services(&self) -> &[RegisteredService] {
        &self.services
    }
}

/// Build a registry with every feature-enabled service registered.
///
/// This is the single place feature flags gate MCP tool availability.
/// Service entries are added in alphabetical order as services come
/// online.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn build_default_registry() -> ToolRegistry {
    let mut reg = ToolRegistry::new();

    // extract is always-on (no feature flag).
    {
        let meta = lab_apis::extract::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::extract::ACTIONS,
        });
    }

    #[cfg(feature = "radarr")]
    {
        let meta = lab_apis::radarr::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::radarr::ACTIONS,
        });
    }

    #[cfg(feature = "sonarr")]
    {
        let meta = lab_apis::sonarr::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::sonarr::ACTIONS,
        });
    }

    #[cfg(feature = "prowlarr")]
    {
        let meta = lab_apis::prowlarr::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::prowlarr::ACTIONS,
        });
    }

    #[cfg(feature = "plex")]
    {
        let meta = lab_apis::plex::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::plex::ACTIONS,
        });
    }

    #[cfg(feature = "tautulli")]
    {
        let meta = lab_apis::tautulli::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::tautulli::ACTIONS,
        });
    }

    #[cfg(feature = "sabnzbd")]
    {
        let meta = lab_apis::sabnzbd::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::sabnzbd::ACTIONS,
        });
    }

    #[cfg(feature = "qbittorrent")]
    {
        let meta = lab_apis::qbittorrent::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::qbittorrent::ACTIONS,
        });
    }

    #[cfg(feature = "tailscale")]
    {
        let meta = lab_apis::tailscale::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::tailscale::ACTIONS,
        });
    }

    #[cfg(feature = "linkding")]
    {
        let meta = lab_apis::linkding::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::linkding::ACTIONS,
        });
    }

    #[cfg(feature = "memos")]
    {
        let meta = lab_apis::memos::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::memos::ACTIONS,
        });
    }

    #[cfg(feature = "bytestash")]
    {
        let meta = lab_apis::bytestash::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::services::bytestash::ACTIONS,
        });
    }

    #[cfg(feature = "paperless")]
    {
        let meta = lab_apis::paperless::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::paperless::ACTIONS,
        });
    }

    #[cfg(feature = "arcane")]
    {
        let meta = lab_apis::arcane::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::arcane::ACTIONS,
        });
    }

    #[cfg(feature = "unraid")]
    {
        let meta = lab_apis::unraid::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::unraid::ACTIONS,
        });
    }

    #[cfg(feature = "unifi")]
    {
        let meta = lab_apis::unifi::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::unifi::ACTIONS,
        });
    }

    #[cfg(feature = "overseerr")]
    {
        let meta = lab_apis::overseerr::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::overseerr::ACTIONS,
        });
    }

    #[cfg(feature = "gotify")]
    {
        let meta = lab_apis::gotify::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::gotify::ACTIONS,
        });
    }

    #[cfg(feature = "openai")]
    {
        let meta = lab_apis::openai::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::openai::ACTIONS,
        });
    }

    #[cfg(feature = "qdrant")]
    {
        let meta = lab_apis::qdrant::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::qdrant::ACTIONS,
        });
    }

    #[cfg(feature = "tei")]
    {
        let meta = lab_apis::tei::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::tei::ACTIONS,
        });
    }

    #[cfg(feature = "apprise")]
    {
        let meta = lab_apis::apprise::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::apprise::ACTIONS,
        });
    }

    reg
}

const fn category_slug(cat: lab_apis::core::Category) -> &'static str {
    use lab_apis::core::Category;
    match cat {
        Category::Media => "media",
        Category::Servarr => "servarr",
        Category::Indexer => "indexer",
        Category::Download => "download",
        Category::Notes => "notes",
        Category::Documents => "documents",
        Category::Network => "network",
        Category::Notifications => "notifications",
        Category::Ai => "ai",
        Category::Bootstrap => "bootstrap",
    }
}

#[cfg(test)]
mod tests {
    use super::build_default_registry;

    #[test]
    fn extract_is_always_registered() {
        let reg = build_default_registry();
        assert!(
            reg.services().iter().any(|s| s.name == "extract"),
            "extract must be in the default registry"
        );
    }

    #[test]
    fn all_features_registers_all_services() {
        let reg = build_default_registry();
        let names: Vec<&str> = reg.services().iter().map(|s| s.name).collect();
        // extract is always-on (no feature flag)
        assert!(names.contains(&"extract"), "extract missing");
        // feature-gated services — present only when the flag is enabled
        #[cfg(feature = "radarr")]
        assert!(names.contains(&"radarr"), "radarr missing");
        #[cfg(feature = "sonarr")]
        assert!(names.contains(&"sonarr"), "sonarr missing");
        #[cfg(feature = "prowlarr")]
        assert!(names.contains(&"prowlarr"), "prowlarr missing");
        #[cfg(feature = "plex")]
        assert!(names.contains(&"plex"), "plex missing");
        #[cfg(feature = "tautulli")]
        assert!(names.contains(&"tautulli"), "tautulli missing");
        #[cfg(feature = "sabnzbd")]
        assert!(names.contains(&"sabnzbd"), "sabnzbd missing");
        #[cfg(feature = "qbittorrent")]
        assert!(names.contains(&"qbittorrent"), "qbittorrent missing");
        #[cfg(feature = "tailscale")]
        assert!(names.contains(&"tailscale"), "tailscale missing");
        #[cfg(feature = "linkding")]
        assert!(names.contains(&"linkding"), "linkding missing");
        #[cfg(feature = "memos")]
        assert!(names.contains(&"memos"), "memos missing");
        #[cfg(feature = "bytestash")]
        assert!(names.contains(&"bytestash"), "bytestash missing");
        #[cfg(feature = "paperless")]
        assert!(names.contains(&"paperless"), "paperless missing");
        #[cfg(feature = "arcane")]
        assert!(names.contains(&"arcane"), "arcane missing");
        #[cfg(feature = "unraid")]
        assert!(names.contains(&"unraid"), "unraid missing");
        #[cfg(feature = "unifi")]
        assert!(names.contains(&"unifi"), "unifi missing");
        #[cfg(feature = "overseerr")]
        assert!(names.contains(&"overseerr"), "overseerr missing");
        #[cfg(feature = "gotify")]
        assert!(names.contains(&"gotify"), "gotify missing");
        #[cfg(feature = "openai")]
        assert!(names.contains(&"openai"), "openai missing");
        #[cfg(feature = "qdrant")]
        assert!(names.contains(&"qdrant"), "qdrant missing");
        #[cfg(feature = "tei")]
        assert!(names.contains(&"tei"), "tei missing");
        #[cfg(feature = "apprise")]
        assert!(names.contains(&"apprise"), "apprise missing");
    }
}
