//! Top-level axum router — mounts `POST /v1/<service>` for every enabled service.

use std::time::Duration;

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderName, Request, StatusCode, header},
    middleware::Next,
    response::Response,
    routing::get,
};
use subtle::ConstantTimeEq;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::Level;

/// Constant-time byte comparison using `subtle::ConstantTimeEq` to prevent
/// timing-based token prefix leakage (lab-63jc).
fn tokens_equal(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

use super::{health, services, state::AppState};
use crate::dispatch::error::ToolError;

pub fn build_router_with_bearer(state: AppState, bearer_token: Option<String>) -> Router {
    // lab-4v9: warn early so operators can see that auth is disabled.
    if bearer_token.is_none() {
        tracing::warn!("HTTP API started without bearer token — all /v1 routes are unprotected");
    }

    let mut v1 = Router::new()
        .route("/{service}/actions", get(service_actions))
        // always-on service
        .nest("/extract", services::extract::routes(state.clone()));

    // Feature-gated per-service route groups (lab-9ngs).
    //
    // Each invocation has two guards:
    //   1. Compile-time `#[cfg(feature)]` — the handler code must exist.
    //   2. Runtime registry check — only mount if the service was not filtered
    //      out by `--services` (or equivalent) at startup.
    //
    // Both guards are required: the feature flag ensures the handler module
    // compiles; the registry check ensures that `lab serve --services radarr`
    // cannot be bypassed by POSTing to `/v1/sonarr`.
    macro_rules! mount_if_enabled {
        ($v1:ident, $state:ident, $feat:literal, $name:literal, $mod:ident) => {
            #[cfg(feature = $feat)]
            if $state.registry.services().iter().any(|s| s.name == $name) {
                $v1 = $v1.nest(concat!("/", $name), services::$mod::routes($state.clone()));
            }
        };
    }

    mount_if_enabled!(v1, state, "radarr", "radarr", radarr);
    mount_if_enabled!(v1, state, "sonarr", "sonarr", sonarr);
    mount_if_enabled!(v1, state, "prowlarr", "prowlarr", prowlarr);
    mount_if_enabled!(v1, state, "plex", "plex", plex);
    mount_if_enabled!(v1, state, "tautulli", "tautulli", tautulli);
    mount_if_enabled!(v1, state, "sabnzbd", "sabnzbd", sabnzbd);
    mount_if_enabled!(v1, state, "qbittorrent", "qbittorrent", qbittorrent);
    mount_if_enabled!(v1, state, "tailscale", "tailscale", tailscale);
    mount_if_enabled!(v1, state, "linkding", "linkding", linkding);
    mount_if_enabled!(v1, state, "memos", "memos", memos);
    mount_if_enabled!(v1, state, "bytestash", "bytestash", bytestash);
    mount_if_enabled!(v1, state, "paperless", "paperless", paperless);
    mount_if_enabled!(v1, state, "arcane", "arcane", arcane);
    mount_if_enabled!(v1, state, "unraid", "unraid", unraid);
    mount_if_enabled!(v1, state, "unifi", "unifi", unifi);
    mount_if_enabled!(v1, state, "overseerr", "overseerr", overseerr);
    mount_if_enabled!(v1, state, "gotify", "gotify", gotify);
    mount_if_enabled!(v1, state, "openai", "openai", openai);
    mount_if_enabled!(v1, state, "qdrant", "qdrant", qdrant);
    mount_if_enabled!(v1, state, "tei", "tei", tei);
    mount_if_enabled!(v1, state, "apprise", "apprise", apprise);

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
        .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid));

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
        .filter_map(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                return None;
            }
            match trimmed.parse::<HeaderValue>() {
                Ok(v) => Some(v),
                Err(e) => {
                    tracing::warn!(
                        origin = trimmed,
                        error = %e,
                        "ignoring unparseable CORS origin from LAB_CORS_ORIGINS"
                    );
                    None
                }
            }
        })
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

pub async fn require_bearer_auth(
    State(expected_token): State<std::sync::Arc<str>>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, ToolError> {
    let provided_token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if provided_token == Some(expected_token.as_ref()) {
        Ok(next.run(request).await)
    } else {
        Err(ToolError::Sdk {
            sdk_kind: "auth_failed".into(),
            message: "missing or invalid bearer token".into(),
        })
    }
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

    /// When a service is absent from the runtime registry (e.g. filtered out by
    /// `--services`), its `/v1/<service>` routes must NOT be mounted — even if
    /// the feature flag for that service is compiled in.
    ///
    /// This test uses an empty registry to simulate `lab serve --services <other>`
    /// excluding `radarr`, then verifies that `POST /v1/radarr` returns 404 rather
    /// than reaching the handler.
    #[cfg(feature = "radarr")]
    #[tokio::test]
    async fn service_filtered_from_registry_has_no_http_route() {
        use crate::mcp::registry::ToolRegistry;

        // An empty registry = no services enabled at runtime.
        let registry = ToolRegistry::new();
        let state = AppState::from_registry(registry);
        let app = build_router_with_bearer(state, None);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/radarr")
                    .header(axum::http::header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"action":"help"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            StatusCode::NOT_FOUND,
            "radarr routes must not be mounted when radarr is absent from the runtime registry"
        );
    }
}
