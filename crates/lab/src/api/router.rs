//! Top-level axum router — mounts `POST /v1/<service>` for every enabled service
//! and the MCP streamable HTTP transport at `/mcp`.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    Router,
    body::Body,
    extract::State,
    http::{HeaderName, HeaderValue, Request, StatusCode, header},
    middleware::Next,
    response::{Html, IntoResponse},
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

use lab_auth::error::AuthError as LabAuthError;

/// Constant-time byte comparison using `subtle::ConstantTimeEq` to prevent
/// timing-based token prefix leakage (lab-63jc).
fn tokens_equal(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).into()
}

fn parse_bearer_token(header_value: &str) -> Option<String> {
    let mut parts = header_value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    if parts.next().is_some() || !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    Some(token.to_string())
}

use super::{health, services, state::AppState};
use crate::dispatch::error::ToolError;

fn app_auth_state(state: &AppState) -> Result<lab_auth::state::AuthState, LabAuthError> {
    state
        .oauth_state
        .as_ref()
        .map(|state| (**state).clone())
        .ok_or_else(|| LabAuthError::Config("oauth auth state is not configured".to_string()))
}

async fn auth_authorization_server_metadata(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, LabAuthError> {
    Ok(lab_auth::metadata::authorization_server_metadata(State(app_auth_state(&state)?)).await)
}

async fn auth_protected_resource_metadata(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, LabAuthError> {
    Ok(lab_auth::metadata::protected_resource_metadata(State(app_auth_state(&state)?)).await)
}

async fn auth_jwks(State(state): State<AppState>) -> Result<impl IntoResponse, LabAuthError> {
    Ok(lab_auth::metadata::jwks(State(app_auth_state(&state)?)).await)
}

async fn auth_register(
    State(state): State<AppState>,
    body: axum::Json<lab_auth::types::ClientRegistrationRequest>,
) -> Result<impl IntoResponse, LabAuthError> {
    Ok(lab_auth::authorize::register_client(State(app_auth_state(&state)?), body).await?)
}

async fn auth_authorize(
    State(state): State<AppState>,
    query: axum::extract::Query<lab_auth::types::AuthorizeQuery>,
) -> Result<impl IntoResponse, LabAuthError> {
    Ok(lab_auth::authorize::authorize(State(app_auth_state(&state)?), query).await?)
}

async fn auth_callback(
    State(state): State<AppState>,
    query: axum::extract::Query<lab_auth::types::CallbackQuery>,
) -> Result<impl IntoResponse, LabAuthError> {
    Ok(lab_auth::authorize::callback(State(app_auth_state(&state)?), query).await?)
}

async fn auth_token(
    State(state): State<AppState>,
    form: axum::extract::Form<lab_auth::types::TokenRequest>,
) -> Result<impl IntoResponse, LabAuthError> {
    Ok(lab_auth::token::token(State(app_auth_state(&state)?), form).await?)
}

/// Build the `/v1` sub-router with all feature-gated service routes.
fn build_v1_router(state: &AppState) -> Router<AppState> {
    let openapi_spec: Arc<String> = super::openapi::build_openapi_spec(state.registry.services())
        .unwrap_or_else(|e| {
            tracing::error!(error = %e, "failed to serialize OpenAPI spec");
            Arc::new(String::from(r#"{"error":"spec generation failed"}"#))
        });
    let spec_for_route = openapi_spec;

    let mut v1 = Router::new()
        .route("/{service}/actions", get(service_actions))
        .nest("/gateway", services::gateway::routes(state.clone()))
        .route(
            "/openapi.json",
            get(move || {
                let spec = spec_for_route.clone();
                async move {
                    (
                        [
                            (header::CONTENT_TYPE, "application/json"),
                            (header::CACHE_CONTROL, "private, no-store"),
                        ],
                        (*spec).clone(),
                    )
                }
            }),
        )
        .route(
            "/docs",
            get(|| async { Html(include_str!("openapi_docs.html")) }),
        )
        .nest("/extract", services::extract::routes(state.clone()));

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

    v1
}

fn protect_with_auth(
    router: Router<AppState>,
    static_token: Option<Arc<str>>,
    auth_state_for_middleware: Option<Arc<lab_auth::state::AuthState>>,
    resource_url: Option<Arc<str>>,
) -> Router<AppState> {
    router.layer(axum::middleware::from_fn(move |mut request: Request<Body>, next: Next| {
        let static_token = static_token.clone();
        let auth_state = auth_state_for_middleware.clone();
        let resource_url = resource_url.clone();
        async move {
            fn auth_error_response(
                message: &str,
                resource_url: Option<&str>,
            ) -> axum::response::Response {
                let err = ToolError::Sdk {
                    sdk_kind: "auth_failed".into(),
                    message: message.into(),
                };
                let mut response = err.into_response();
                if let Some(url) = resource_url {
                    let www_auth = crate::api::oauth::www_authenticate_value(url);
                    if let Ok(value) = HeaderValue::from_str(&www_auth) {
                        response
                            .headers_mut()
                            .insert(header::WWW_AUTHENTICATE, value);
                    }
                }
                response
            }

            let auth_header = request
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_bearer_token);

            let Some(token) = auth_header else {
                return Ok(auth_error_response(
                    "missing bearer token",
                    resource_url.as_deref(),
                ));
            };

            if let Some(ref expected) = static_token
                && tokens_equal(&token, expected.as_ref())
            {
                request
                    .extensions_mut()
                    .insert(crate::api::oauth::AuthContext {
                        sub: "static-bearer".to_string(),
                        scopes: vec!["lab:read".to_string(), "lab:admin".to_string()],
                        issuer: "local".to_string(),
                    });
                return Ok::<_, std::convert::Infallible>(next.run(request).await);
            }

            if let Some(ref auth_state) = auth_state {
                let expected_aud = auth_state
                    .config
                    .public_url
                    .as_ref()
                    .map(|url| url.as_str().trim_end_matches('/').to_string());
                let Some(ref expected_aud) = expected_aud else {
                    return Ok(auth_error_response(
                        "server misconfigured: LAB_PUBLIC_URL required for JWT validation",
                        resource_url.as_deref(),
                    ));
                };
                match auth_state.signing_keys.validate_access_token(&token, expected_aud) {
                    Ok(claims) => {
                        if claims.iss != *expected_aud {
                            return Ok(auth_error_response(
                                "invalid bearer token",
                                resource_url.as_deref(),
                            ));
                        }

                        request.extensions_mut().insert(crate::api::oauth::AuthContext {
                            sub: claims.sub,
                            scopes: claims
                                .scope
                                .split_whitespace()
                                .filter(|scope| !scope.is_empty())
                                .map(ToOwned::to_owned)
                                .collect(),
                            issuer: claims.iss,
                        });
                        return Ok(next.run(request).await);
                    }
                    Err(error) => {
                        tracing::debug!(error = %error, "lab-auth JWT validation failed");
                    }
                }
            }

            Ok(auth_error_response(
                "invalid bearer token",
                resource_url.as_deref(),
            ))
        }
    }))
}

#[allow(clippy::too_many_lines)]
pub fn build_router(
    mut state: AppState,
    bearer_token: Option<String>,
    auth_state: Option<lab_auth::state::AuthState>,
    mcp_router: Option<Router<AppState>>,
    config_cors_origins: &[String],
) -> Router {
    if let Some(ref auth_state) = auth_state {
        state = state.with_oauth_state(auth_state.clone());
    }
    if bearer_token.is_none() && auth_state.is_none() {
        tracing::warn!(
            "HTTP API started without bearer token or OAuth auth state — all protected routes are unprotected"
        );
    }

    let v1 = build_v1_router(&state);

    let x_request_id = HeaderName::from_static("x-request-id");

    let mut protected = Router::new();

    // Apply layered auth to the protected sub-router (lab-y8aa).
    //
    // Auth priority: try static bearer first (cheapest), then OAuth JWT.
    // Either is accepted. If both are absent, no auth middleware is applied
    // (the safety gate in serve.rs prevents non-localhost bind without auth).
    //
    // `AuthContext` is injected as an Extension for downstream scope checks.
    // Static bearer tokens get implicit lab:admin scope (full access).
    let static_token = bearer_token.map(Arc::<str>::from);
    let auth_state = auth_state.map(Arc::new);
    let auth_state_for_middleware = auth_state.clone();
    let needs_auth = static_token.is_some() || auth_state.is_some();

    // Resolve resource_url once for WWW-Authenticate headers on 401.
    let resource_url: Option<Arc<str>> = auth_state
        .as_ref()
        .and_then(|state| state.config.public_url.as_ref().map(url::Url::as_str))
        .or_else(|| {
            state
                .auth_config
                .as_ref()
                .and_then(|cfg| cfg.public_url.as_ref().map(url::Url::as_str))
        })
        .map(Arc::from);

    let v1_router = if needs_auth && !state.web_ui_auth_disabled {
        protect_with_auth(
            Router::new().nest("/v1", v1),
            static_token.clone(),
            auth_state_for_middleware.clone(),
            resource_url.clone(),
        )
    } else {
        Router::new().nest("/v1", v1)
    };
    protected = protected.merge(v1_router);

    if let Some(mcp) = mcp_router {
        let mcp_router = if needs_auth {
            protect_with_auth(
                mcp,
                static_token.clone(),
                auth_state_for_middleware.clone(),
                resource_url.clone(),
            )
        } else {
            mcp
        };
        protected = protected.merge(mcp_router);
    }

    // Build the outer router: health probes + discovery (no auth) + protected routes (auth).
    // Layers apply bottom-up: last .layer() call = outermost middleware.
    // Desired execution order (outermost → innermost → handler):
    //   SetRequestId → TraceLayer → PropagateRequestId → Timeout → Compression → CORS → handler
    let mut router = Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .merge(protected);
    if let Some(auth_state) = auth_state.as_ref() {
        let _ = auth_state;
        router = router
            .route(
                "/.well-known/oauth-authorization-server",
                get(auth_authorization_server_metadata),
            )
            .route(
                "/.well-known/oauth-protected-resource",
                get(auth_protected_resource_metadata),
            )
            .route("/jwks", get(auth_jwks))
            .route("/register", axum::routing::post(auth_register))
            .route("/authorize", get(auth_authorize))
            .route("/auth/google/callback", get(auth_callback))
            .route("/token", axum::routing::post(auth_token));
    }

    if state.web_assets_dir.is_some() {
        router = router.fallback(crate::api::web::serve_web_request);
    }

    router
        .with_state(state)
        .layer(build_cors_layer(config_cors_origins))
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

#[allow(clippy::too_many_lines)]
#[allow(dead_code)]
pub fn build_router_with_bearer(
    state: AppState,
    bearer_token: Option<String>,
    mcp_router: Option<Router<AppState>>,
) -> Router {
    build_router(state, bearer_token, None, mcp_router, &[])
}

/// Build a `CorsLayer` that allows only explicit trusted origins.
///
/// Sources (env var overrides config.toml):
/// - `LAB_CORS_ORIGINS` env var (comma-separated `scheme://host[:port]`)
/// - `api.cors_origins` in config.toml (array of strings)
///
/// Always includes `http://localhost`, `http://127.0.0.1`, and `http://[::1]`
/// as safe loopback defaults.
fn build_cors_layer(config_origins: &[String]) -> CorsLayer {
    use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
    use axum::http::{HeaderValue, Method};

    // Env var overrides config.toml when present.
    let raw_origins: Vec<String> = match std::env::var("LAB_CORS_ORIGINS") {
        Ok(val) if !val.trim().is_empty() => val
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect(),
        _ => config_origins.to_vec(),
    };

    let env_origins: Vec<HeaderValue> = raw_origins
        .iter()
        .filter_map(|s| match s.parse::<HeaderValue>() {
            Ok(v) => Some(v),
            Err(e) => {
                tracing::warn!(
                    origin = s.as_str(),
                    error = %e,
                    "ignoring unparseable CORS origin"
                );
                None
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
    use std::fs;

    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;

    use super::*;

    #[tokio::test]
    async fn actions_known_service_returns_200() {
        let state = AppState::new();
        let app = build_router_with_bearer(state, None, None);
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
        let app = build_router_with_bearer(state, None, None);
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
        let app = build_router_with_bearer(state, Some("secret-token".into()), None);
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
        let app = build_router_with_bearer(state, Some("secret-token".into()), None);
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
    async fn auth_layer_accepts_case_insensitive_bearer_token() {
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()), None);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/extract/actions")
                    .header(header::AUTHORIZATION, "bearer   secret-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn web_ui_auth_bypass_opens_v1_only() {
        let state = AppState::new().with_web_ui_auth_disabled(true);
        let mcp_router: Router<AppState> =
            Router::new().route("/mcp", get(|| async { StatusCode::OK }));
        let app = build_router_with_bearer(state, Some("secret-token".into()), Some(mcp_router));

        let v1_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/extract/actions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(v1_response.status(), StatusCode::OK);

        let mcp_response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/mcp")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(mcp_response.status(), StatusCode::UNAUTHORIZED);
        let body = axum::body::to_bytes(mcp_response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["kind"], "auth_failed");
    }

    #[tokio::test]
    async fn health_endpoint_open_without_auth() {
        // /health must be reachable by monitoring probes without any token (lab-3qn.5).
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()), None);
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
        let app = build_router_with_bearer(state, Some("secret-token".into()), None);
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

    #[tokio::test]
    async fn openapi_json_requires_bearer_auth() {
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()), None);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/openapi.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn openapi_json_returns_spec_with_auth() {
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()), None);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/openapi.json")
                    .header(header::AUTHORIZATION, "Bearer secret-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let ct = response.headers().get(header::CONTENT_TYPE).unwrap();
        assert_eq!(ct, "application/json");
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let spec: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(spec["openapi"], "3.1.0");
        assert!(spec["info"]["title"].as_str().is_some());
        assert!(spec["paths"].as_object().is_some());
    }

    #[tokio::test]
    async fn docs_endpoint_returns_html_with_auth() {
        let state = AppState::new();
        let app = build_router_with_bearer(state, Some("secret-token".into()), None);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/docs")
                    .header(header::AUTHORIZATION, "Bearer secret-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("scalar"), "HTML should reference Scalar");
        assert!(
            html.contains("openapi.json"),
            "HTML should reference spec URL"
        );
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
        use crate::registry::ToolRegistry;

        // An empty registry = no services enabled at runtime.
        let registry = ToolRegistry::new();
        let state = AppState::from_registry(registry);
        let app = build_router_with_bearer(state, None, None);
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

    #[tokio::test]
    async fn bearer_mode_still_accepts_lab_mcp_http_token() {
        let state = AppState::new();
        let app = build_router(state, Some("secret-token".into()), None, None, &[]);
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
    async fn oauth_mode_accepts_lab_auth_jwt() {
        let state = AppState::new();
        let auth_state = test_lab_auth_state().await;
        let token = issue_test_lab_token(&auth_state);
        let app = build_router(state, None, Some(auth_state), None, &[]);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/v1/extract/actions")
                    .header(header::AUTHORIZATION, format!("Bearer {token}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn oauth_mode_missing_token_returns_www_authenticate_metadata_hint() {
        let state = AppState::new();
        let auth_state = test_lab_auth_state().await;
        let app = build_router(state, None, Some(auth_state), None, &[]);
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
        let header = response
            .headers()
            .get(header::WWW_AUTHENTICATE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(header.contains("resource_metadata="));
    }

    #[tokio::test]
    async fn serves_web_assets_for_browser_routes_when_configured() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "<html><body>Labby</body></html>").unwrap();

        let state = AppState::new().with_web_assets_dir(dir.path().to_path_buf());
        let app = build_router_with_bearer(state, None, None);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/gateways/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("Labby"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn rejects_symlinked_assets_outside_configured_web_root() {
        use std::os::unix::fs as unix_fs;

        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "<html><body>Labby</body></html>").unwrap();
        fs::write(outside.path().join("secret.txt"), "top-secret").unwrap();
        unix_fs::symlink(
            outside.path().join("secret.txt"),
            dir.path().join("secret.txt"),
        )
        .unwrap();

        let state = AppState::new().with_web_assets_dir(dir.path().to_path_buf());
        let app = build_router_with_bearer(state, None, None);
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/secret.txt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn v1_routes_still_win_over_web_asset_fallback() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "<html><body>Labby</body></html>").unwrap();

        let state = AppState::new().with_web_assets_dir(dir.path().to_path_buf());
        let app = build_router_with_bearer(state, None, None);
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
        let content_type = response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("");
        assert!(content_type.contains("application/json"));
    }

    async fn test_lab_auth_state() -> lab_auth::state::AuthState {
        let dir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let config = lab_auth::config::AuthConfig {
            mode: lab_auth::config::AuthMode::OAuth,
            public_url: Some(url::Url::parse("https://lab.example.com").unwrap()),
            sqlite_path: dir.path().join("auth.db"),
            key_path: dir.path().join("auth-jwt.pem"),
            bootstrap_secret: Some("bootstrap-secret".to_string()),
            google: lab_auth::config::GoogleConfig {
                client_id: "client-id".to_string(),
                client_secret: "client-secret".to_string(),
                callback_path: "/auth/google/callback".to_string(),
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            },
            ..lab_auth::config::AuthConfig::default()
        };
        lab_auth::state::AuthState::new(config).await.unwrap()
    }

    fn issue_test_lab_token(auth_state: &lab_auth::state::AuthState) -> String {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        auth_state
            .signing_keys
            .issue_access_token(lab_auth::jwt::AccessClaims {
                iss: "https://lab.example.com".to_string(),
                sub: "google-user".to_string(),
                aud: "https://lab.example.com".to_string(),
                exp: now + 3600,
                iat: now,
                jti: "test-jti".to_string(),
                scope: "lab".to_string(),
                azp: "client".to_string(),
            })
            .unwrap()
    }
}
