//! Collected service metadata for every compiled-in service.
//! The TUI reads from the shared tool registry to render the plugin browser,
//! avoiding duplication with `RegisteredService` in `mcp/registry.rs`.

use std::sync::OnceLock;

use lab_apis::core::ServiceClient;

use crate::registry::{RegisteredService, ToolRegistry, build_default_registry};
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

    // Macro for services with light probe methods returning `Result<T, E>`.
    macro_rules! spawn_health_expr {
        ($name:literal, $expr:expr) => {{
            let sem = sem.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire_owned().await.ok()?;
                let start = std::time::Instant::now();
                let result = $expr.await;
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

    macro_rules! spawn_health {
        ($name:literal, $client:expr) => {{ spawn_health_expr!($name, $client.health()) }};
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
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::unifi::client_from_env,
        ) {
            spawn_health_trait!("unifi", client);
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
        if let (Some(url), Some(key)) = (vars.get("OVERSEERR_URL"), vars.get("OVERSEERR_API_KEY")) {
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
                .map_or("https://api.tailscale.com/api/v2", String::as_str)
                .to_string();
            let tailnet = vars
                .get("TAILSCALE_TAILNET")
                .map_or("-", String::as_str)
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

    #[cfg(feature = "apprise")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::apprise::client_from_env,
        ) {
            spawn_health!("apprise", client);
        }
    }

    #[cfg(feature = "bytestash")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::bytestash::client_from_env,
        ) {
            spawn_health_trait!("bytestash", client);
        }
    }

    #[cfg(feature = "dozzle")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::dozzle::client_from_env,
        ) {
            spawn_health!("dozzle", client);
        }
    }

    #[cfg(feature = "adguard")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::adguard::client_from_env,
        ) {
            spawn_health!("adguard", client);
        }
    }

    #[cfg(feature = "glances")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::glances::client_from_env,
        ) {
            spawn_health!("glances", client);
        }
    }

    #[cfg(feature = "pihole")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::pihole::client_from_env,
        ) {
            spawn_health!("pihole", client);
        }
    }

    #[cfg(feature = "neo4j")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::neo4j::client_from_env,
        ) {
            spawn_health!("neo4j", client);
        }
    }

    #[cfg(feature = "uptime_kuma")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::uptime_kuma::client_from_env,
        ) {
            spawn_health!("uptime_kuma", client);
        }
    }

    #[cfg(feature = "immich")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::immich::client_from_env,
        ) {
            spawn_health!("immich", client);
        }
    }

    #[cfg(feature = "jellyfin")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::jellyfin::client_from_env,
        ) {
            spawn_health_trait!("jellyfin", client);
        }
    }

    #[cfg(feature = "navidrome")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::navidrome::client_from_env,
        ) {
            spawn_health!("navidrome", client);
        }
    }

    #[cfg(feature = "scrutiny")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::scrutiny::client_from_env,
        ) {
            spawn_health!("scrutiny", client);
        }
    }

    #[cfg(feature = "freshrss")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::freshrss::client_from_env,
        ) {
            spawn_health!("freshrss", client);
        }
    }

    #[cfg(feature = "loggifly")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::loggifly::client_from_env,
        ) {
            spawn_health!("loggifly", client);
        }
    }

    #[cfg(feature = "qdrant")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::qdrant::client_from_env,
        ) {
            spawn_health!("qdrant", client);
        }
    }

    #[cfg(feature = "tei")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::tei::client_from_env,
        ) {
            spawn_health!("tei", client);
        }
    }

    #[cfg(feature = "linkding")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::linkding::client_from_env,
        ) {
            spawn_health_expr!("linkding", client.probe());
        }
    }

    #[cfg(feature = "memos")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::memos::client_from_env,
        ) {
            spawn_health_expr!("memos", client.health());
        }
    }

    #[cfg(feature = "paperless")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::paperless::client_from_env,
        ) {
            spawn_health_expr!("paperless", client.probe());
        }
    }

    #[cfg(feature = "openai")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::openai::client_from_env,
        ) {
            spawn_health!("openai", client);
        }
    }

    #[cfg(feature = "openacp")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::openacp::client_from_env,
        ) {
            spawn_health_expr!("openacp", ServiceClient::health(&client));
        }
    }

    #[cfg(feature = "notebooklm")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::notebooklm::client_from_env,
        ) {
            spawn_health!("notebooklm", client);
        }
    }

    #[cfg(feature = "tautulli")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::tautulli::client_from_env,
        ) {
            spawn_health_expr!("tautulli", client.probe());
        }
    }

    #[cfg(feature = "arcane")]
    {
        if let Some(client) = crate::dispatch::helpers::with_env_override(
            vars.clone(),
            crate::dispatch::arcane::client_from_env,
        ) {
            spawn_health_expr!("arcane", client.health());
        }
    }

    #[cfg(feature = "qbittorrent")]
    {
        let qbittorrent_vars = vars.clone();
        let sem = sem.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire_owned().await.ok()?;
            let start = std::time::Instant::now();
            let result = async {
                let url = qbittorrent_vars
                    .get("QBITTORRENT_URL")
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
                    .ok_or_else(|| "QBITTORRENT_URL not configured".to_string())?;

                let sid = if let Some(sid) = qbittorrent_vars
                    .get("QBITTORRENT_SID")
                    .filter(|value| !value.trim().is_empty())
                    .cloned()
                {
                    sid
                } else {
                    let username = qbittorrent_vars
                        .get("QBITTORRENT_USERNAME")
                        .filter(|value| !value.trim().is_empty())
                        .cloned()
                        .ok_or_else(|| "QBITTORRENT_USERNAME not configured".to_string())?;
                    let password = qbittorrent_vars
                        .get("QBITTORRENT_PASSWORD")
                        .filter(|value| !value.trim().is_empty())
                        .cloned()
                        .ok_or_else(|| "QBITTORRENT_PASSWORD not configured".to_string())?;

                    let login_client = reqwest::Client::builder()
                        .connect_timeout(std::time::Duration::from_secs(5))
                        .timeout(std::time::Duration::from_secs(10))
                        .build()
                        .map_err(|e| e.to_string())?;
                    let login_url = format!("{}/api/v2/auth/login", url.trim_end_matches('/'));
                    let resp = login_client
                        .post(&login_url)
                        .form(&[("username", username), ("password", password)])
                        .send()
                        .await
                        .map_err(|e| e.to_string())?;
                    let set_cookie = resp
                        .headers()
                        .get("set-cookie")
                        .and_then(|value| value.to_str().ok())
                        .map(str::to_string);
                    let body = resp.text().await.map_err(|e| e.to_string())?;
                    if body.trim() != "Ok." {
                        return Err(
                            "qbittorrent login failed — check QBITTORRENT_USERNAME/PASSWORD"
                                .to_string(),
                        );
                    }
                    set_cookie
                        .as_deref()
                        .and_then(|value| {
                            value
                                .split(';')
                                .next()
                                .filter(|part| part.trim_start().starts_with("SID="))
                                .map(|part| part.trim().to_string())
                        })
                        .ok_or_else(|| "qbittorrent login did not return SID cookie".to_string())?
                };

                let client = lab_apis::qbittorrent::QbittorrentClient::new(&url, sid)
                    .map_err(|e| e.to_string())?;
                client.version().await.map_err(|e| e.to_string())
            }
            .await;
            let latency_ms = Some(u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX));
            let (reachable, auth_ok, message) = match result {
                Ok(_) => (true, true, None),
                Err(e) => {
                    let msg = e.to_string();
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
                service: "qbittorrent".to_owned(),
                reachable,
                auth_ok,
                latency_ms,
                message,
            })
        }));
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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn check_all_services_includes_qdrant_and_tei() {
        let qdrant = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/healthz"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&qdrant)
            .await;

        let tei = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&tei)
            .await;

        let temp_dir = TempDir::new().expect("temp dir");
        let env_path = temp_dir.path().join(".env");
        fs::write(
            &env_path,
            format!("QDRANT_URL={}\nTEI_URL={}\n", qdrant.uri(), tei.uri()),
        )
        .expect("write env");

        let results = super::check_all_services(&env_path).await;

        let qdrant_health = results
            .iter()
            .find(|health| health.service == "qdrant")
            .expect("qdrant health");
        assert!(qdrant_health.reachable);
        assert!(qdrant_health.auth_ok);

        let tei_health = results
            .iter()
            .find(|health| health.service == "tei")
            .expect("tei health");
        assert!(tei_health.reachable);
        assert!(tei_health.auth_ok);
    }
}
