//! Top-level axum router — mounts `POST /v1/<service>` for every enabled service.

use std::time::Duration;

use axum::{
    Router,
    extract::State,
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
        .route("/ready", get(health::ready))
        .route("/v1/{service}/actions", get(service_actions));

    // Mount each feature-gated service. Adding a new service: add one line here.
    macro_rules! mount {
        ($feat:literal => $module:ident) => {
            #[cfg(feature = $feat)]
            {
                router = router.nest(
                    concat!("/v1/", $feat),
                    services::$module::routes(state.clone()),
                );
            }
        };
    }

    router = router.nest("/v1/extract", services::extract::routes(state.clone()));
    mount!("radarr"      => radarr);
    mount!("sonarr"      => sonarr);
    mount!("prowlarr"    => prowlarr);
    mount!("plex"        => plex);
    mount!("tautulli"    => tautulli);
    mount!("sabnzbd"     => sabnzbd);
    mount!("qbittorrent" => qbittorrent);
    mount!("tailscale"   => tailscale);
    mount!("linkding"    => linkding);
    mount!("memos"       => memos);
    mount!("bytestash"   => bytestash);
    mount!("paperless"   => paperless);
    mount!("arcane"      => arcane);
    mount!("unraid"      => unraid);
    mount!("unifi"       => unifi);
    mount!("overseerr"   => overseerr);
    mount!("gotify"      => gotify);
    mount!("openai"      => openai);
    mount!("qdrant"      => qdrant);
    mount!("tei"         => tei);
    mount!("apprise"     => apprise);

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

async fn service_actions(
    State(state): State<AppState>,
    axum::extract::Path(service): axum::extract::Path<String>,
) -> Result<axum::Json<serde_json::Value>, crate::mcp::envelope::ToolError> {
    let entry = state
        .catalog
        .services
        .iter()
        .find(|s| s.name == service)
        .ok_or_else(|| crate::mcp::envelope::ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("unknown service `{service}`"),
        })?;
    let actions =
        serde_json::to_value(&entry.actions).map_err(|e| crate::mcp::envelope::ToolError::Sdk {
            sdk_kind: "internal_error".into(),
            message: format!("serialize actions: {e}"),
        })?;
    Ok(axum::Json(actions))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn actions_known_service_returns_200() {
        let state = AppState::new();
        let app = build_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/extract/actions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.is_array(), "body should be a JSON array of actions");
    }

    #[tokio::test]
    async fn actions_unknown_service_returns_404() {
        let state = AppState::new();
        let app = build_router(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/doesnotexist/actions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["kind"], "not_found");
    }
}
