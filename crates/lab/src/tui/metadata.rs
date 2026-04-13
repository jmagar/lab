//! Collected service metadata for every compiled-in service.
//! The TUI reads from the shared tool registry to render the plugin browser,
//! avoiding duplication with `RegisteredService` in `mcp/registry.rs`.

use std::sync::OnceLock;

use lab_apis::core::ServiceClient;

use crate::mcp::registry::{RegisteredService, ToolRegistry, build_default_registry};
use crate::tui::events::ServiceHealth;

/// Lazily-initialized default registry shared by the TUI.
static DEFAULT_REGISTRY: OnceLock<ToolRegistry> = OnceLock::new();

fn registry() -> &'static ToolRegistry {
    DEFAULT_REGISTRY.get_or_init(build_default_registry)
}

/// Return every compiled-in service from the shared registry.
#[must_use]
pub fn all_services() -> &'static [RegisteredService] {
    registry().services()
}

/// Run health checks for all enabled services concurrently (max 5 in parallel).
///
/// Called from a background tokio task — never from the render thread.
/// Services without a usable client (stub-only) are silently skipped.
#[allow(clippy::too_many_lines)]
pub async fn check_all_services(env: &std::path::Path) -> Vec<ServiceHealth> {
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    // Load env vars.
    let vars: HashMap<String, String> = dotenvy::from_path_iter(env)
        .ok()
        .map(|it| it.filter_map(Result::ok).collect())
        .unwrap_or_default();

    let sem = Arc::new(Semaphore::new(5));
    let mut handles: Vec<tokio::task::JoinHandle<Option<ServiceHealth>>> = Vec::new();

    // Macro for services with `health() -> Result<(), ServiceError>` (most services).
    macro_rules! spawn_health {
        ($name:literal, $client:expr) => {{
            let sem = sem.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire_owned().await.ok()?;
                let start = std::time::Instant::now();
                let result = $client.health().await;
                let latency_ms =
                    Some(u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX));
                let (reachable, auth_ok, message) = match result {
                    Ok(_) => (true, true, None),
                    Err(e) => {
                        let msg = e.to_string();
                        // Classify error by string heuristics since error types differ.
                        let auth_fail = msg.contains("401")
                            || msg.contains("403")
                            || msg.contains("auth")
                            || msg.contains("Auth")
                            || msg.contains("Unauthorized")
                            || msg.contains("Forbidden");
                        (false, !auth_fail, Some(msg))
                    }
                };
                if reachable && auth_ok {
                    tracing::info!(
                        surface = "tui",
                        service = $name,
                        operation = "health",
                        "health ok"
                    );
                } else {
                    tracing::warn!(
                        surface = "tui",
                        service = $name,
                        operation = "health",
                        "health issue"
                    );
                }
                Some(ServiceHealth {
                    service: $name.to_owned(),
                    reachable,
                    auth_ok,
                    latency_ms,
                    message,
                })
            }));
        }};
    }

    // Macro for services that implement `ServiceClient` trait health() -> Result<ServiceStatus, ApiError>.
    macro_rules! spawn_health_trait {
        ($name:literal, $client:expr) => {{
            let sem = sem.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire_owned().await.ok()?;
                let start = std::time::Instant::now();
                let result = $client.health().await;
                let latency_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
                let (reachable, auth_ok, message, latency) = match result {
                    Ok(status) => (
                        status.reachable,
                        status.auth_ok,
                        status.message,
                        Some(status.latency_ms),
                    ),
                    Err(e) => (false, false, Some(e.to_string()), Some(latency_ms)),
                };
                if reachable && auth_ok {
                    tracing::info!(
                        surface = "tui",
                        service = $name,
                        operation = "health",
                        elapsed_ms = latency_ms,
                        "health ok"
                    );
                } else {
                    tracing::warn!(
                        surface = "tui",
                        service = $name,
                        operation = "health",
                        elapsed_ms = latency_ms,
                        "health issue"
                    );
                }
                Some(ServiceHealth {
                    service: $name.to_owned(),
                    reachable,
                    auth_ok,
                    latency_ms: latency,
                    message,
                })
            }));
        }};
    }

    #[cfg(feature = "radarr")]
    {
        if let (Some(url), Some(key)) = (vars.get("RADARR_URL"), vars.get("RADARR_API_KEY")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::radarr::RadarrClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Api-Key".to_owned(),
                    key: key.clone(),
                },
            ) {
                spawn_health!("radarr", client);
            }
        }
    }

    #[cfg(feature = "sonarr")]
    {
        if let (Some(url), Some(key)) = (vars.get("SONARR_URL"), vars.get("SONARR_API_KEY")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::sonarr::SonarrClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Api-Key".to_owned(),
                    key: key.clone(),
                },
            ) {
                spawn_health!("sonarr", client);
            }
        }
    }

    #[cfg(feature = "prowlarr")]
    {
        if let (Some(url), Some(key)) = (vars.get("PROWLARR_URL"), vars.get("PROWLARR_API_KEY")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::prowlarr::ProwlarrClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Api-Key".to_owned(),
                    key: key.clone(),
                },
            ) {
                spawn_health!("prowlarr", client);
            }
        }
    }

    #[cfg(feature = "sabnzbd")]
    {
        if let (Some(url), Some(key)) = (vars.get("SABNZBD_URL"), vars.get("SABNZBD_API_KEY"))
            && let Ok(client) = lab_apis::sabnzbd::SabnzbdClient::new(url, key.clone())
        {
            spawn_health_trait!("sabnzbd", client);
        }
    }

    #[cfg(feature = "unifi")]
    {
        if let (Some(url), Some(key)) = (vars.get("UNIFI_URL"), vars.get("UNIFI_API_KEY")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::unifi::UnifiClient::new(
                url,
                Auth::ApiKey {
                    header: "X-API-KEY".to_owned(),
                    key: key.clone(),
                },
            ) {
                spawn_health_trait!("unifi", client);
            }
        }
    }

    #[cfg(feature = "unraid")]
    {
        if let (Some(url), Some(key)) = (vars.get("UNRAID_URL"), vars.get("UNRAID_API_KEY")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::unraid::UnraidClient::new(
                url,
                Auth::ApiKey {
                    header: "X-API-Key".into(),
                    key: key.clone(),
                },
            ) {
                spawn_health_trait!("unraid", client);
            }
        }
    }

    #[cfg(feature = "gotify")]
    {
        let token = vars
            .get("GOTIFY_CLIENT_TOKEN")
            .or_else(|| vars.get("GOTIFY_TOKEN"));
        if let (Some(url), Some(token)) = (vars.get("GOTIFY_URL"), token) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::gotify::GotifyClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Gotify-Key".into(),
                    key: token.clone(),
                },
            ) {
                spawn_health_trait!("gotify", client);
            }
        }
    }

    #[cfg(feature = "plex")]
    {
        if let (Some(url), Some(token)) = (vars.get("PLEX_URL"), vars.get("PLEX_TOKEN")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::plex::PlexClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Plex-Token".to_owned(),
                    key: token.clone(),
                },
            ) {
                spawn_health_trait!("plex", client);
            }
        }
    }

    #[cfg(feature = "overseerr")]
    {
        if let (Some(url), Some(key)) =
            (vars.get("OVERSEERR_URL"), vars.get("OVERSEERR_API_KEY"))
        {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::overseerr::OverseerrClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Api-Key".to_owned(),
                    key: key.clone(),
                },
            ) {
                spawn_health_trait!("overseerr", client);
            }
        }
    }

    #[cfg(feature = "tailscale")]
    {
        if let Some(key) = vars.get("TAILSCALE_API_KEY") {
            use lab_apis::core::Auth;
            let base_url = vars
                .get("TAILSCALE_BASE_URL")
                .map(String::as_str)
                .unwrap_or("https://api.tailscale.com/api/v2")
                .to_string();
            let tailnet = vars
                .get("TAILSCALE_TAILNET")
                .map(String::as_str)
                .unwrap_or("-")
                .to_string();
            if let Ok(client) = lab_apis::tailscale::TailscaleClient::new(
                &base_url,
                Auth::Bearer { token: key.clone() },
                tailnet,
            ) {
                spawn_health_trait!("tailscale", client);
            }
        }
    }

    // Collect results.
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(Some(h)) = handle.await {
            results.push(h);
        }
    }
    results
}
