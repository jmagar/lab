use axum::{
    Json, Router,
    extract::{Extension, Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::api::state::AppState;
use crate::dispatch::error::ToolError;

pub fn gateway_routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/upstreams", get(upstreams))
        .route("/start", post(start))
        .route("/status", get(status))
        .route("/clear", post(clear))
}

pub fn browser_routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/auth/upstream/callback", get(callback))
        .route("/gateway/oauth/result", get(result_page))
}

#[derive(Debug, Deserialize)]
struct StartRequest {
    upstream: String,
}

#[derive(Debug, Deserialize)]
struct StatusQuery {
    upstream: String,
}

#[derive(Debug, Deserialize)]
struct ClearQuery {
    upstream: String,
    confirm: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
    upstream: String,
}

#[derive(Debug, Deserialize)]
struct ResultQuery {
    upstream: String,
    status: String,
}

#[derive(Debug, Serialize)]
struct StartResponse {
    authorization_url: String,
}

#[derive(Debug, Serialize)]
struct UpstreamEntry {
    name: String,
}

async fn upstreams(
    State(state): State<AppState>,
    Extension(_auth): Extension<crate::api::oauth::AuthContext>,
) -> Result<Json<Vec<UpstreamEntry>>, ToolError> {
    require_master(&state)?;
    let manager = state
        .gateway_manager
        .clone()
        .ok_or_else(|| ToolError::internal_message("gateway manager not wired"))?;
    let configs = manager.oauth_upstream_configs().await;
    Ok(Json(
        configs
            .into_iter()
            .map(|c| UpstreamEntry { name: c.name })
            .collect(),
    ))
}

fn require_master(state: &AppState) -> Result<(), ToolError> {
    if state.is_master() {
        Ok(())
    } else {
        Err(ToolError::Sdk {
            sdk_kind: "forbidden".to_string(),
            message: "upstream oauth routes are master-only".to_string(),
        })
    }
}

async fn callback_subject(
    state: &AppState,
    auth: Option<crate::api::oauth::AuthContext>,
    headers: &HeaderMap,
) -> Result<String, ToolError> {
    if let Some(auth) = auth {
        return Ok(auth.sub);
    }

    if let Some(auth_state) = state.oauth_state.as_ref()
        && let Some(session_id) = lab_auth::session::read_cookie(
            headers,
            lab_auth::session::BROWSER_SESSION_COOKIE_NAME,
        )
    {
        let session = auth_state
            .store
            .find_browser_session(&session_id)
            .await
            .map_err(|error| {
                ToolError::internal_message(format!("browser session lookup failed: {error}"))
            })?;
        if let Some(session) = session {
            return Ok(session.subject);
        }
    }

    Err(ToolError::Sdk {
        sdk_kind: "oauth_state_invalid".to_string(),
        message: "oauth_state_invalid: authenticated browser session required".to_string(),
    })
}

fn public_url(state: &AppState) -> Result<&url::Url, ToolError> {
    state
        .auth_config
        .as_ref()
        .and_then(|config| config.public_url.as_ref())
        .ok_or_else(|| ToolError::internal_message("LAB_PUBLIC_URL is required for upstream oauth"))
}

fn append_public_path(base: &url::Url, path: &str) -> Result<url::Url, ToolError> {
    let mut url = base.clone();
    let base_path = url.path().trim_end_matches('/');
    let path = path.trim_start_matches('/');
    let next = if base_path.is_empty() {
        format!("/{path}")
    } else {
        format!("{base_path}/{path}")
    };
    url.set_path(&next);
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

async fn start(
    State(state): State<AppState>,
    Extension(auth): Extension<crate::api::oauth::AuthContext>,
    Json(body): Json<StartRequest>,
) -> Result<Json<StartResponse>, ToolError> {
    let started = std::time::Instant::now();
    require_master(&state)?;
    let manager = state
        .gateway_manager
        .clone()
        .ok_or_else(|| ToolError::internal_message("gateway manager not wired"))?;
    let begin =
        crate::dispatch::gateway::oauth::begin_authorization(&manager, &body.upstream, &auth.sub)
            .await
            .inspect_err(|error| {
                warn!(
                    surface = "api",
                    service = "upstream_oauth",
                    action = "start",
                    elapsed_ms = started.elapsed().as_millis(),
                    kind = error.kind(),
                    "upstream oauth start failed"
                );
            })?;
    info!(
        surface = "api",
        service = "upstream_oauth",
        action = "start",
        elapsed_ms = started.elapsed().as_millis(),
        upstream = %body.upstream,
        "upstream oauth authorization started"
    );
    Ok(Json(StartResponse {
        authorization_url: begin.authorization_url,
    }))
}

async fn status(
    State(state): State<AppState>,
    Query(query): Query<StatusQuery>,
    Extension(auth): Extension<crate::api::oauth::AuthContext>,
) -> Result<Json<crate::dispatch::gateway::oauth::UpstreamOauthStatusView>, ToolError> {
    let started = std::time::Instant::now();
    require_master(&state)?;
    let manager = state
        .gateway_manager
        .clone()
        .ok_or_else(|| ToolError::internal_message("gateway manager not wired"))?;
    let status = crate::dispatch::gateway::oauth::status(&manager, &query.upstream, &auth.sub)
        .await
        .inspect_err(|error| {
            warn!(
                surface = "api",
                service = "upstream_oauth",
                action = "status",
                elapsed_ms = started.elapsed().as_millis(),
                kind = error.kind(),
                "upstream oauth status failed"
            );
        })?;
    info!(
        surface = "api",
        service = "upstream_oauth",
        action = "status",
        elapsed_ms = started.elapsed().as_millis(),
        upstream = %query.upstream,
        "upstream oauth status retrieved"
    );
    Ok(Json(status))
}

async fn clear(
    State(state): State<AppState>,
    Query(query): Query<ClearQuery>,
    Extension(auth): Extension<crate::api::oauth::AuthContext>,
) -> impl IntoResponse {
    let started = std::time::Instant::now();
    if let Err(error) = require_master(&state) {
        return error.into_response();
    }
    if query.confirm != Some(true) {
        return ToolError::Sdk {
            sdk_kind: "confirmation_required".to_string(),
            message: "set ?confirm=true to clear upstream oauth credentials".to_string(),
        }
        .into_response();
    }
    let manager = match state.gateway_manager.clone() {
        Some(manager) => manager,
        None => return ToolError::internal_message("gateway manager not wired").into_response(),
    };
    if let Err(error) =
        crate::dispatch::gateway::oauth::clear(&manager, &query.upstream, &auth.sub).await
    {
        warn!(
            surface = "api",
            service = "upstream_oauth",
            action = "clear",
            elapsed_ms = started.elapsed().as_millis(),
            kind = error.kind(),
            "upstream oauth clear failed"
        );
        return error.into_response();
    }
    info!(
        surface = "api",
        service = "upstream_oauth",
        action = "clear",
        elapsed_ms = started.elapsed().as_millis(),
        upstream = %query.upstream,
        "upstream oauth credentials cleared"
    );
    (StatusCode::OK, Json(serde_json::json!({"ok": true}))).into_response()
}

async fn callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
    auth: Option<Extension<crate::api::oauth::AuthContext>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let started = std::time::Instant::now();
    if let Err(error) = require_master(&state) {
        return error.into_response();
    }

    let manager = match state.gateway_manager.clone() {
        Some(manager) => manager,
        None => return ToolError::internal_message("gateway manager not wired").into_response(),
    };
    let subject = match callback_subject(&state, auth.map(|e| e.0), &headers).await {
        Ok(subject) => subject,
        Err(error) => return error.into_response(),
    };
    let base = match public_url(&state) {
        Ok(url) => url.clone(),
        Err(error) => return error.into_response(),
    };
    let mut redirect_url = match append_public_path(&base, "/gateway/oauth/result") {
        Ok(url) => url,
        Err(error) => return error.into_response(),
    };

    let result = crate::dispatch::gateway::oauth::complete_authorization_callback(
        &manager,
        &query.upstream,
        &subject,
        &query.code,
        &query.state,
    )
    .await;

    let status = if let Err(error) = &result {
        warn!(
            surface = "api",
            service = "upstream_oauth",
            action = "callback",
            elapsed_ms = started.elapsed().as_millis(),
            upstream = %query.upstream,
            kind = error.kind(),
            "upstream oauth callback failed"
        );
        "fail"
    } else {
        info!(
            surface = "api",
            service = "upstream_oauth",
            action = "callback",
            elapsed_ms = started.elapsed().as_millis(),
            upstream = %query.upstream,
            "upstream oauth callback completed"
        );
        "ok"
    };
    redirect_url
        .query_pairs_mut()
        .append_pair("upstream", &query.upstream)
        .append_pair("status", status);

    if let Err(error) = result {
        redirect_url
            .query_pairs_mut()
            .append_pair("error_kind", error.kind());
        return Redirect::to(redirect_url.as_str()).into_response();
    }

    Redirect::to(redirect_url.as_str()).into_response()
}

async fn result_page(Query(query): Query<ResultQuery>) -> Html<String> {
    let upstream = html_escape(&query.upstream);
    let status = if query.status == "ok" {
        "successful"
    } else {
        "failed"
    };
    Html(format!(
        "<html><body><h2>Authorization {status}</h2><p>Upstream <strong>{upstream}</strong> has been processed. You may close this tab.</p></body></html>"
    ))
}

#[cfg(test)]
mod tests {
    use axum::{
        Extension,
        body::{self, Body},
        http::{Request, StatusCode, header},
    };
    use lab_auth::{
        config::{AuthConfig, AuthMode, GoogleConfig},
        state::AuthState,
    };
    use tower::ServiceExt;

    use super::{browser_routes, callback_subject, gateway_routes};
    use crate::{api::oauth::AuthContext, api::state::AppState, config::DeviceRole};

    #[tokio::test]
    async fn callback_rejects_non_master_requests() {
        let state = AppState::new().with_device_role(DeviceRole::NonMaster);
        let app = browser_routes(state.clone()).with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/auth/upstream/callback?upstream=test&code=code&state=csrf")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn callback_requires_authenticated_browser_session() {
        let state = AppState::new();
        let parts = Request::new(()).into_parts().0;
        let error = callback_subject(&state, &parts).await.unwrap_err();
        assert_eq!(error.kind(), "oauth_state_invalid");
    }

    #[tokio::test]
    async fn result_page_escapes_upstream_name() {
        let state = AppState::new();
        let app = browser_routes(state.clone()).with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/gateway/oauth/result?upstream=%3Cscript%3Ealert(1)%3C%2Fscript%3E&status=ok")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8(body.to_vec()).unwrap();
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>alert(1)</script>"));
    }

    #[tokio::test]
    async fn clear_requires_explicit_confirmation() {
        let state = AppState::new();
        let app = gateway_routes(state.clone())
            .layer(Extension(test_auth_context()))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/clear?upstream=test")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        let body = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["kind"], "confirmation_required");
    }

    fn test_auth_context() -> AuthContext {
        AuthContext {
            sub: "browser-user".to_string(),
            scopes: vec!["lab:admin".to_string()],
            issuer: "https://issuer.example".to_string(),
            via_session: true,
            csrf_token: Some("csrf-123".to_string()),
            email: Some("browser@example.com".to_string()),
        }
    }

    #[allow(dead_code)]
    async fn test_auth_state() -> AuthState {
        let dir = tempfile::tempdir().unwrap();
        let config = AuthConfig {
            mode: AuthMode::OAuth,
            public_url: Some(url::Url::parse("https://lab.example.com").unwrap()),
            sqlite_path: dir.path().join("auth.db"),
            key_path: dir.path().join("auth-jwt.pem"),
            bootstrap_secret: Some("bootstrap-secret".to_string()),
            google: GoogleConfig {
                client_id: "client-id".to_string(),
                client_secret: "client-secret".to_string(),
                callback_path: "/auth/google/callback".to_string(),
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            },
            ..AuthConfig::default()
        };
        AuthState::new(config).await.unwrap()
    }
}
