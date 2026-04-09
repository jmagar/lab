//! Runtime tool registry. Services register themselves here during
//! startup; the MCP server walks the registry to expose tools and the
//! catalog module walks it to produce discovery docs.

use std::future::Future;
use std::pin::Pin;

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::services::error::ToolError;

/// A dispatch function pointer: takes an owned action name and params,
/// returns a boxed future resolving to `Result<Value, ToolError>`.
pub type DispatchFn =
    fn(String, Value) -> Pin<Box<dyn Future<Output = Result<Value, ToolError>> + Send>>;

/// Wrap an `async fn(&str, Value) -> Result<Value, ToolError>` into a [`DispatchFn`].
///
/// Bridges the `&str`-taking dispatch signatures into the owned-`String`
/// function pointer stored in the registry.
macro_rules! dispatch_fn {
    ($f:path) => {
        |action: String, params: serde_json::Value| -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<serde_json::Value, $crate::services::error::ToolError>,
                    > + Send,
            >,
        > { Box::pin(async move { $f(&action, params).await }) }
    };
}

/// Metadata the registry keeps about each registered service.
#[derive(Clone)]
pub struct RegisteredService {
    /// Service / tool name.
    pub name: &'static str,
    /// Short description from `PluginMeta::description`.
    pub description: &'static str,
    /// Category slug.
    pub category: &'static str,
    /// Actions exposed by this service.
    pub actions: &'static [ActionSpec],
    /// Dispatch function for routing action calls.
    pub dispatch: DispatchFn,
}

impl std::fmt::Debug for RegisteredService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RegisteredService")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("category", &self.category)
            .field("actions", &self.actions)
            .finish_non_exhaustive()
    }
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
            dispatch: dispatch_fn!(crate::mcp::services::extract::dispatch),
        });
    }

    #[cfg(feature = "radarr")]
    {
        let meta = lab_apis::radarr::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::radarr::actions(),
            dispatch: dispatch_fn!(crate::mcp::services::radarr::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::sonarr::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::prowlarr::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::plex::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::tautulli::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::sabnzbd::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::qbittorrent::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::tailscale::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::linkding::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::memos::dispatch),
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
            dispatch: dispatch_fn!(crate::services::bytestash::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::paperless::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::arcane::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::unraid::dispatch),
        });
    }

    #[cfg(feature = "unifi")]
    {
        let meta = lab_apis::unifi::META;
        reg.register(RegisteredService {
            name: meta.name,
            description: meta.description,
            category: category_slug(meta.category),
            actions: crate::mcp::services::unifi::actions(),
            dispatch: dispatch_fn!(crate::mcp::services::unifi::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::overseerr::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::gotify::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::openai::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::qdrant::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::tei::dispatch),
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
            dispatch: dispatch_fn!(crate::mcp::services::apprise::dispatch),
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

    #[tokio::test]
    async fn dispatch_fn_round_trips() {
        let reg = build_default_registry();
        let extract = reg
            .services()
            .iter()
            .find(|s| s.name == "extract")
            .expect("extract must be registered");
        let result = (extract.dispatch)("help".to_string(), serde_json::json!({})).await;
        assert!(result.is_ok(), "extract help dispatch should succeed");
    }
}
