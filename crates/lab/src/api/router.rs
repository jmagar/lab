//! Top-level axum router — mounts `POST /v1/<service>` for every enabled service.

use std::time::Duration;

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderName, Request, StatusCode, header},
    middleware::Next,
    routing::get,
};
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::Level;

/// Constant-time byte comparison to prevent timing-based token prefix leakage.
///
/// Length mismatch still reveals length information, but that is unavoidable
/// without padding and is acceptable for a pre-shared homelab token.
fn tokens_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.as_bytes()
        .iter()
        .zip(b.as_bytes().iter())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

use super::{health, services, state::AppState};
use crate::dispatch::error::ToolError;

#[allow(clippy::too_many_lines)]
pub fn build_router_with_bearer(state: AppState, bearer_token: Option<String>) -> Router {
    let mut v1 = Router::new()
        .route("/{service}/actions", get(service_actions))
        // always-on service
        .nest("/extract", services::extract::routes(state.clone()));

    // Feature-gated per-service route groups.
    #[cfg(feature = "radarr")]
    {
        v1 = v1.nest("/radarr", services::radarr::routes(state.clone()));
    }
    #[cfg(feature = "sonarr")]
    {
        v1 = v1.nest("/sonarr", services::sonarr::routes(state.clone()));
    }
    #[cfg(feature = "prowlarr")]
    {
        v1 = v1.nest("/prowlarr", services::prowlarr::routes(state.clone()));
    }
    #[cfg(feature = "plex")]
    {
        v1 = v1.nest("/plex", services::plex::routes(state.clone()));
    }
    #[cfg(feature = "tautulli")]
    {
        v1 = v1.nest("/tautulli", services::tautulli::routes(state.clone()));
    }
    #[cfg(feature = "sabnzbd")]
    {
        v1 = v1.nest("/sabnzbd", services::sabnzbd::routes(state.clone()));
    }
    #[cfg(feature = "qbittorrent")]
    {
        v1 = v1.nest("/qbittorrent", services::qbittorrent::routes(state.clone()));
    }
    #[cfg(feature = "tailscale")]
    {
        v1 = v1.nest("/tailscale", services::tailscale::routes(state.clone()));
    }
    #[cfg(feature = "linkding")]
    {
        v1 = v1.nest("/linkding", services::linkding::routes(state.clone()));
    }
    #[cfg(feature = "memos")]
    {
        v1 = v1.nest("/memos", services::memos::routes(state.clone()));
    }
    #[cfg(feature = "bytestash")]
    {
        v1 = v1.nest("/bytestash", services::bytestash::routes(state.clone()));
    }
    #[cfg(feature = "paperless")]
    {
        v1 = v1.nest("/paperless", services::paperless::routes(state.clone()));
    }
    #[cfg(feature = "arcane")]
    {
        v1 = v1.nest("/arcane", services::arcane::routes(state.clone()));
    }
    #[cfg(feature = "unraid")]
    {
        v1 = v1.nest("/unraid", services::unraid::routes(state.clone()));
    }
    #[cfg(feature = "unifi")]
    {
        v1 = v1.nest("/unifi", services::unifi::routes(state.clone()));
    }
    #[cfg(feature = "overseerr")]
    {
        v1 = v1.nest("/overseerr", services::overseerr::routes(state.clone()));
    }
    #[cfg(feature = "gotify")]
    {
        v1 = v1.nest("/gotify", services::gotify::routes(state.clone()));
    }
    #[cfg(feature = "openai")]
    {
        v1 = v1.nest("/openai", services::openai::routes(state.clone()));
    }
    #[cfg(feature = "qdrant")]
    {
        v1 = v1.nest("/qdrant", services::qdrant::routes(state.clone()));
    }
    #[cfg(feature = "tei")]
    {
        v1 = v1.nest("/tei", services::tei::routes(state.clone()));
    }
    #[cfg(feature = "apprise")]
    {
        v1 = v1.nest("/apprise", services::apprise::routes(state.clone()));
    }

    let x_request_id = HeaderName::from_static("x-request-id");

    // Apply bearer auth as a layer on just the /v1 sub-router while it is
    // still `Router<AppState>`. This scopes auth to /v1 only — health probes
    // added to the outer router below are never wrapped by it (lab-3qn.5).
    //
    // Use `from_fn` (captures token by closure) rather than `from_fn_with_state`
    // so no new state type is introduced that would prevent `with_state(AppState)`
    // from resolving the whole-router state in one shot.
    let v1 = if let Some(token) = bearer_token {
        let token = std::sync::Arc::<str>::from(token);
        v1.layer(axum::middleware::from_fn(
            move |request: Request<Body>, next: Next| {
                let token = token.clone();
                async move {
                    let provided = request
                        .headers()
                        .get(header::AUTHORIZATION)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|v| v.strip_prefix("Bearer "));
                    if provided.is_some_and(|t| tokens_equal(t, token.as_ref())) {
                        Ok::<_, ToolError>(next.run(request).await)
                    } else {
                        Err(ToolError::Sdk {
                            sdk_kind: "auth_failed".into(),
                            message: "missing or invalid bearer token".into(),
                        })
                    }
                }
            },
        ))
    } else {
        v1
    };

    // Build the outer router: health probes are outside bearer auth, /v1 is inside.
    // Layers apply bottom-up: last .layer() call = outermost middleware.
    // Desired execution order (outermost → innermost → handler):
    //   SetRequestId → TraceLayer → PropagateRequestId → Timeout → Compression → CORS → handler
    Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .nest("/v1", v1)
        .with_state(state)
        .layer(build_cors_layer())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(30),
        ))
        // PropagateRequestId echoes the id back in the response header.
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        // TraceLayer reads x-request-id set by SetRequestId (outermost).
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &Request<_>| {
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
        // SetRequestId generates a UUID for every request that lacks one.
        .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
}

/// Build a `CorsLayer` that allows only explicit trusted origins.
///
/// Reads `LAB_CORS_ORIGINS` (comma-separated list of `scheme://host[:port]`
/// values) from the environment; always includes `http://localhost`,
/// `http://127.0.0.1`, and `http://[::1]` as safe loopback defaults.
///
/// This replaces `CorsLayer::permissive()` which would allow any browser page
/// on the local network to issue cross-origin requests to destructive endpoints.
fn build_cors_layer() -> CorsLayer {
    use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
    use axum::http::{HeaderValue, Method};

    let env_origins: Vec<HeaderValue> = std::env::var("LAB_CORS_ORIGINS")
        .unwrap_or_default()
        .split(',')
        .filter_map(|s| s.trim().parse::<HeaderValue>().ok())
        .collect();

    // Include common dev ports so browser clients on localhost:PORT work
    // without explicit LAB_CORS_ORIGINS configuration (lab-3qn.19).
    let mut origins: Vec<HeaderValue> = vec![
        HeaderValue::from_static("http://localhost"),
        HeaderValue::from_static("http://localhost:3000"),
        HeaderValue::from_static("http://localhost:5173"),
        HeaderValue::from_static("http://localhost:8080"),
        HeaderValue::from_static("http://127.0.0.1"),
        HeaderValue::from_static("http://127.0.0.1:3000"),
        HeaderValue::from_static("http://127.0.0.1:5173"),
        HeaderValue::from_static("http://127.0.0.1:8080"),
        HeaderValue::from_static("http://[::1]"),
    ];
    origins.extend(env_origins);

    // Explicit allowlist instead of Any — prevents arbitrary headers from
    // allowed origins reaching destructive endpoints (lab-3qn.7).
    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            AUTHORIZATION,
            CONTENT_TYPE,
            HeaderName::from_static("x-request-id"),
        ])
}

async fn service_actions(
    State(state): State<AppState>,
    axum::extract::Path(service): axum::extract::Path<String>,
) -> Result<axum::Json<serde_json::Value>, ToolError> {
    let entry = state
        .catalog
        .services
        .iter()
        .find(|s| s.name == service)
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("unknown service `{service}`"),
        })?;
    let actions = serde_json::to_value(&entry.actions).map_err(|e| ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: format!("serialize actions: {e}"),
    })?;
    Ok(axum::Json(actions))
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn actions_known_service_returns_200() {
        let state = AppState::new();
        let app = build_router_with_bearer(state, None);
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
        let app = build_router_with_bearer(state, None);
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

    #[tokio::test]
    async fn auth_layer_rejects_missing_bearer_token() {
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()));
        // /v1/extract/actions is behind bearer auth; /health is NOT (lab-3qn.5).
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
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["kind"], "auth_failed");
    }

    #[tokio::test]
    async fn auth_layer_accepts_valid_bearer_token() {
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()));
        // Confirm that a valid token reaches the protected /v1 route.
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/extract/actions")
                    .header(header::AUTHORIZATION, "Bearer secret-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn health_endpoint_open_without_auth() {
        // /health must be reachable by monitoring probes without any token (lab-3qn.5).
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ready_endpoint_open_without_auth() {
        // /ready must be reachable by monitoring probes without any token (lab-3qn.5).
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()));
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
