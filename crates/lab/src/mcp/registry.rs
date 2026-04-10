//! Runtime tool registry. Services register themselves here during
//! startup; the MCP server walks the registry to expose tools and the
//! catalog module walks it to produce discovery docs.

use std::future::Future;
use std::pin::Pin;

use lab_apis::core::action::ActionSpec;
use serde_json::Value;

use crate::dispatch::error::ToolError;

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
        |action: String,
         params: serde_json::Value|
         -> std::pin::Pin<
            Box<
                dyn std::future::Future<
                        Output = Result<serde_json::Value, $crate::dispatch::error::ToolError>,
                    > + Send,
            >,
        > { Box::pin(async move { $f(&action, params).await }) }
    };
}

/// Register a standard service (feature name == module name, uses `mcp::services::$svc`).
///
/// Expands to the `#[cfg(feature)] { reg.register(RegisteredService { ... }) }` block,
/// eliminating the 7-line boilerplate that would otherwise be repeated per service.
///
/// Two forms:
/// - Default: `register_service!(reg, "foo", foo)` — uses `mcp::services::foo::ACTIONS` and
///   `mcp::services::foo::dispatch`.
/// - Override: `register_service!(reg, "foo", foo, actions = $expr, dispatch = $expr)` —
///   for migrated services whose catalog and/or dispatch live outside `mcp::services`.
macro_rules! register_service {
    // Full override: custom actions expr and dispatch expr (for migrated services).
    ($reg:expr, $feature:literal, $svc:ident, actions = $actions:expr, dispatch = $dispatch:expr) => {
        #[cfg(feature = $feature)]
        {
            let meta = lab_apis::$svc::META;
            $reg.register(RegisteredService {
                name: meta.name,
                description: meta.description,
                category: category_slug(meta.category),
                actions: $actions,
                dispatch: $dispatch,
            });
        }
    };
    // Default: use mcp::services::$svc ACTIONS const and dispatch fn.
    ($reg:expr, $feature:literal, $svc:ident) => {
        #[cfg(feature = $feature)]
        {
            let meta = lab_apis::$svc::META;
            $reg.register(RegisteredService {
                name: meta.name,
                description: meta.description,
                category: category_slug(meta.category),
                actions: crate::mcp::services::$svc::ACTIONS,
                dispatch: dispatch_fn!(crate::mcp::services::$svc::dispatch),
            });
        }
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

    register_service!(reg, "radarr", radarr,
        actions = crate::mcp::services::radarr::actions(),
        dispatch = dispatch_fn!(crate::mcp::services::radarr::dispatch));

    register_service!(reg, "sonarr", sonarr);
    register_service!(reg, "prowlarr", prowlarr);
    register_service!(reg, "plex", plex);
    register_service!(reg, "tautulli", tautulli);

    register_service!(reg, "sabnzbd", sabnzbd,
        actions = crate::dispatch::sabnzbd::ACTIONS,
        dispatch = dispatch_fn!(crate::mcp::services::sabnzbd::dispatch));

    register_service!(reg, "qbittorrent", qbittorrent);
    register_service!(reg, "tailscale", tailscale);
    register_service!(reg, "linkding", linkding);
    register_service!(reg, "memos", memos);

    register_service!(reg, "bytestash", bytestash,
        actions = crate::dispatch::bytestash::ACTIONS,
        dispatch = dispatch_fn!(crate::dispatch::bytestash::dispatch));

    register_service!(reg, "paperless", paperless);
    register_service!(reg, "arcane", arcane);
    register_service!(reg, "unraid", unraid);

    register_service!(reg, "unifi", unifi,
        actions = crate::dispatch::unifi::actions(),
        dispatch = dispatch_fn!(crate::dispatch::unifi::dispatch));

    register_service!(reg, "overseerr", overseerr);
    register_service!(reg, "gotify", gotify);
    register_service!(reg, "openai", openai);
    register_service!(reg, "qdrant", qdrant);
    register_service!(reg, "tei", tei);
    register_service!(reg, "apprise", apprise);

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
