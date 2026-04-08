//! Top-level axum router — mounts `POST /v1/<service>` for every enabled service.

use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

use super::{health, services, state::AppState};

#[must_use]
pub fn build_router(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready));

    router = router.nest("/v1/extract", services::extract::routes(state.clone()));

    #[cfg(feature = "radarr")]
    {
        router = router.nest("/v1/radarr", services::radarr::routes(state.clone()));
    }
    #[cfg(feature = "sonarr")]
    {
        router = router.nest("/v1/sonarr", services::sonarr::routes(state.clone()));
    }
    #[cfg(feature = "prowlarr")]
    {
        router = router.nest("/v1/prowlarr", services::prowlarr::routes(state.clone()));
    }
    #[cfg(feature = "plex")]
    {
        router = router.nest("/v1/plex", services::plex::routes(state.clone()));
    }
    #[cfg(feature = "tautulli")]
    {
        router = router.nest("/v1/tautulli", services::tautulli::routes(state.clone()));
    }
    #[cfg(feature = "sabnzbd")]
    {
        router = router.nest("/v1/sabnzbd", services::sabnzbd::routes(state.clone()));
    }
    #[cfg(feature = "qbittorrent")]
    {
        router = router.nest("/v1/qbittorrent", services::qbittorrent::routes(state.clone()));
    }
    #[cfg(feature = "tailscale")]
    {
        router = router.nest("/v1/tailscale", services::tailscale::routes(state.clone()));
    }
    #[cfg(feature = "linkding")]
    {
        router = router.nest("/v1/linkding", services::linkding::routes(state.clone()));
    }
    #[cfg(feature = "memos")]
    {
        router = router.nest("/v1/memos", services::memos::routes(state.clone()));
    }
    #[cfg(feature = "bytestash")]
    {
        router = router.nest("/v1/bytestash", services::bytestash::routes(state.clone()));
    }
    #[cfg(feature = "paperless")]
    {
        router = router.nest("/v1/paperless", services::paperless::routes(state.clone()));
    }
    #[cfg(feature = "arcane")]
    {
        router = router.nest("/v1/arcane", services::arcane::routes(state.clone()));
    }
    #[cfg(feature = "unraid")]
    {
        router = router.nest("/v1/unraid", services::unraid::routes(state.clone()));
    }
    #[cfg(feature = "unifi")]
    {
        router = router.nest("/v1/unifi", services::unifi::routes(state.clone()));
    }
    #[cfg(feature = "overseerr")]
    {
        router = router.nest("/v1/overseerr", services::overseerr::routes(state.clone()));
    }
    #[cfg(feature = "gotify")]
    {
        router = router.nest("/v1/gotify", services::gotify::routes(state.clone()));
    }
    #[cfg(feature = "openai")]
    {
        router = router.nest("/v1/openai", services::openai::routes(state.clone()));
    }
    #[cfg(feature = "qdrant")]
    {
        router = router.nest("/v1/qdrant", services::qdrant::routes(state.clone()));
    }
    #[cfg(feature = "tei")]
    {
        router = router.nest("/v1/tei", services::tei::routes(state.clone()));
    }
    #[cfg(feature = "apprise")]
    {
        router = router.nest("/v1/apprise", services::apprise::routes(state.clone()));
    }

    router
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
}
