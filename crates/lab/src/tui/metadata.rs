//! Collected `PluginMeta` references for every compiled-in service.
//! The TUI reads this list to render the plugin browser.

use lab_apis::core::ServiceClient;

use crate::tui::events::ServiceHealth;

/// One row in the plugin manager view.
#[derive(Debug, Clone)]
pub struct PluginRow {
    /// Service identifier.
    pub name: &'static str,
    /// Short description.
    pub description: &'static str,
    /// Category slug.
    pub category: &'static str,
}

/// Return every compiled-in plugin.
#[must_use]
pub fn all_plugins() -> Vec<PluginRow> {
    let mut rows = Vec::new();

    // extract is always-on (Bootstrap) — no feature flag needed
    {
        let meta = lab_apis::extract::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "radarr")]
    {
        let meta = lab_apis::radarr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "sonarr")]
    {
        let meta = lab_apis::sonarr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "prowlarr")]
    {
        let meta = lab_apis::prowlarr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "overseerr")]
    {
        let meta = lab_apis::overseerr::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "plex")]
    {
        let meta = lab_apis::plex::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "tautulli")]
    {
        let meta = lab_apis::tautulli::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "sabnzbd")]
    {
        let meta = lab_apis::sabnzbd::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "qbittorrent")]
    {
        let meta = lab_apis::qbittorrent::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "tailscale")]
    {
        let meta = lab_apis::tailscale::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "unraid")]
    {
        let meta = lab_apis::unraid::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "unifi")]
    {
        let meta = lab_apis::unifi::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "arcane")]
    {
        let meta = lab_apis::arcane::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "linkding")]
    {
        let meta = lab_apis::linkding::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "memos")]
    {
        let meta = lab_apis::memos::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "bytestash")]
    {
        let meta = lab_apis::bytestash::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "paperless")]
    {
        let meta = lab_apis::paperless::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "gotify")]
    {
        let meta = lab_apis::gotify::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "apprise")]
    {
        let meta = lab_apis::apprise::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "openai")]
    {
        let meta = lab_apis::openai::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "qdrant")]
    {
        let meta = lab_apis::qdrant::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    #[cfg(feature = "tei")]
    {
        let meta = lab_apis::tei::META;
        rows.push(PluginRow {
            name: meta.name,
            description: meta.description,
            category: meta.category.as_str(),
        });
    }

    rows
}

/// Run health checks for all enabled services concurrently (max 5 in parallel).
///
/// Called from a background tokio task — never from the render thread.
/// Services without a usable client (stub-only) are silently skipped.
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
                let result: Result<(), _> = $client.health().await;
                let latency_ms = Some(start.elapsed().as_millis() as u64);
                let (reachable, auth_ok, message) = match result {
                    Ok(()) => (true, true, None),
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
                let status: lab_apis::core::ServiceStatus = $client.health().await.ok()?;
                Some(ServiceHealth {
                    service: $name.to_owned(),
                    reachable: status.reachable,
                    auth_ok: status.auth_ok,
                    latency_ms: Some(status.latency_ms),
                    message: status.message,
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

    #[cfg(feature = "plex")]
    {
        if let (Some(url), Some(token)) = (vars.get("PLEX_URL"), vars.get("PLEX_TOKEN")) {
            use lab_apis::core::Auth;
            if let Ok(client) =
                lab_apis::plex::PlexClient::new(url, Auth::Token { token: token.clone() })
            {
                spawn_health!("plex", client);
            }
        }
    }

    #[cfg(feature = "sabnzbd")]
    {
        if let (Some(url), Some(key)) = (vars.get("SABNZBD_URL"), vars.get("SABNZBD_API_KEY")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::sabnzbd::SabnzbdClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Api-Key".to_owned(),
                    key: key.clone(),
                },
            ) {
                spawn_health!("sabnzbd", client);
            }
        }
    }

    #[cfg(feature = "linkding")]
    {
        if let (Some(url), Some(token)) = (vars.get("LINKDING_URL"), vars.get("LINKDING_TOKEN")) {
            use lab_apis::core::Auth;
            if let Ok(client) =
                lab_apis::linkding::LinkdingClient::new(url, Auth::Token { token: token.clone() })
            {
                spawn_health!("linkding", client);
            }
        }
    }

    #[cfg(feature = "memos")]
    {
        if let (Some(url), Some(token)) = (vars.get("MEMOS_URL"), vars.get("MEMOS_TOKEN")) {
            use lab_apis::core::Auth;
            if let Ok(client) =
                lab_apis::memos::MemosClient::new(url, Auth::Token { token: token.clone() })
            {
                spawn_health!("memos", client);
            }
        }
    }

    #[cfg(feature = "bytestash")]
    {
        if let (Some(url), Some(token)) = (vars.get("BYTESTASH_URL"), vars.get("BYTESTASH_TOKEN")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::bytestash::ByteStashClient::new(
                url,
                Auth::Bearer { token: token.clone() },
            ) {
                spawn_health_trait!("bytestash", client);
            }
        }
    }

    #[cfg(feature = "paperless")]
    {
        if let (Some(url), Some(token)) =
            (vars.get("PAPERLESS_URL"), vars.get("PAPERLESS_TOKEN"))
        {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::paperless::PaperlessClient::new(
                url,
                Auth::Token { token: token.clone() },
            ) {
                spawn_health!("paperless", client);
            }
        }
    }

    #[cfg(feature = "gotify")]
    {
        if let (Some(url), Some(token)) = (vars.get("GOTIFY_URL"), vars.get("GOTIFY_TOKEN")) {
            use lab_apis::core::Auth;
            if let Ok(client) = lab_apis::gotify::GotifyClient::new(
                url,
                Auth::ApiKey {
                    header: "X-Gotify-Key".to_owned(),
                    key: token.clone(),
                },
            ) {
                spawn_health!("gotify", client);
            }
        }
    }

    #[cfg(feature = "apprise")]
    {
        if let Some(url) = vars.get("APPRISE_URL") {
            use lab_apis::core::Auth;
            let auth = vars
                .get("APPRISE_TOKEN")
                .map(|t| Auth::Bearer { token: t.clone() })
                .unwrap_or(Auth::None);
            if let Ok(client) = lab_apis::apprise::AppriseClient::new(url, auth) {
                spawn_health!("apprise", client);
            }
        }
    }

    #[cfg(feature = "openai")]
    {
        if let Some(key) = vars.get("OPENAI_API_KEY") {
            use lab_apis::core::Auth;
            let base = vars
                .get("OPENAI_URL")
                .map(String::as_str)
                .unwrap_or("https://api.openai.com");
            if let Ok(client) = lab_apis::openai::OpenAiClient::new(
                base,
                Auth::Bearer { token: key.clone() },
            ) {
                spawn_health!("openai", client);
            }
        }
    }

    #[cfg(feature = "qdrant")]
    {
        if let Some(url) = vars.get("QDRANT_URL") {
            use lab_apis::core::Auth;
            let auth = vars
                .get("QDRANT_API_KEY")
                .map(|k| Auth::ApiKey {
                    header: "api-key".to_owned(),
                    key: k.clone(),
                })
                .unwrap_or(Auth::None);
            if let Ok(client) = lab_apis::qdrant::QdrantClient::new(url, auth) {
                spawn_health!("qdrant", client);
            }
        }
    }

    #[cfg(feature = "tei")]
    {
        if let Some(url) = vars.get("TEI_URL") {
            use lab_apis::core::Auth;
            let auth = vars
                .get("TEI_API_KEY")
                .map(|k| Auth::ApiKey {
                    header: "api-key".to_owned(),
                    key: k.clone(),
                })
                .unwrap_or(Auth::None);
            if let Ok(client) = lab_apis::tei::TeiClient::new(url, auth) {
                spawn_health!("tei", client);
            }
        }
    }

    // unifi: UnifiClient does not implement ServiceClient health() — skip health probing.
    #[cfg(feature = "unifi")]
    let _ = vars.get("UNIFI_URL"); // suppress unused-vars lint

    #[cfg(feature = "arcane")]
    {
        if let Some(url) = vars.get("ARCANE_URL") {
            use lab_apis::core::Auth;
            let auth = vars
                .get("ARCANE_TOKEN")
                .map(|t| Auth::Bearer { token: t.clone() })
                .unwrap_or(Auth::None);
            if let Ok(client) = lab_apis::arcane::ArcaneClient::new(url, auth) {
                spawn_health!("arcane", client);
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
