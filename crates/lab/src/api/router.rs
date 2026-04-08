//! Top-level axum router — mounts `POST /v1/<service>` for every enabled service.

use std::time::Duration;

use axum::{
    Router,
    http::{HeaderName, StatusCode},
    routing::get,
};
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::Level;

use super::{health, services, state::AppState};

#[allow(clippy::too_many_lines)]
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
        router = router.nest(
            "/v1/qbittorrent",
            services::qbittorrent::routes(state.clone()),
        );
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

    let x_request_id = HeaderName::from_static("x-request-id");

    router
        .with_state(state)
        // TraceLayer reads x-request-id set by SetRequestId below.
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &axum::http::Request<_>| {
                let request_id = req
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("-");
                tracing::span!(
                    Level::INFO,
                    "request",
                    method = %req.method(),
                    path = %req.uri().path(),
                    request_id,
                    status = tracing::field::Empty,
                )
            }),
        )
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
        // PropagateRequestId echoes the id back in the response header.
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        // SetRequestId generates a UUID for every request that lacks one.
        .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
}
