//! Browser-facing routes for upstream MCP OAuth flows.
//!
//! Mounts at `/v1/upstream-oauth/:name/`:
//!
//! | Route | Purpose |
//! |-------|---------|
//! | `GET /v1/upstream-oauth/:name/start?subject=<sub>` | Begin authorization: redirect to AS |
//! | `GET /v1/upstream-oauth/:name/callback?code=…&state=…` | Complete: exchange code, show result |
//!
//! These routes are NOT protected by the bearer-token auth middleware — the
//! browser redirect from the AS cannot carry an `Authorization` header.  They
//! are still protected by the CSRF token embedded in the `state` parameter.
//!
//! ## Subject
//!
//! The `subject` is the lab user identity string.  In an OAuth-mode deployment it
//! should match the JWT `sub` claim of the authenticated session.  For the initial
//! implementation, callers pass it as a query parameter.  When session-based auth is
//! in place this will be replaced by session extraction.

use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::get,
};
use serde::Deserialize;
use tracing::{info, warn};

use crate::api::state::AppState;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/start", get(start))
        .route("/callback", get(callback))
}

#[derive(Deserialize)]
struct StartQuery {
    subject: String,
}

#[derive(Deserialize)]
struct CallbackQuery {
    code: String,
    state: String,
}

async fn start(
    State(state): State<AppState>,
    Path(upstream_name): Path<String>,
    Query(query): Query<StartQuery>,
) -> impl IntoResponse {
    let Some(managers) = state.upstream_oauth.as_ref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "upstream oauth is not configured",
        )
            .into_response();
    };

    let Some(manager) = managers.get(&upstream_name) else {
        return (
            StatusCode::NOT_FOUND,
            format!("upstream '{upstream_name}' not found or has no oauth config"),
        )
            .into_response();
    };

    match manager.begin_authorization(&query.subject).await {
        Ok(begin) => {
            info!(
                upstream = %upstream_name,
                subject = %query.subject,
                "upstream oauth: redirecting to AS"
            );
            Redirect::temporary(&begin.authorization_url).into_response()
        }
        Err(e) => {
            warn!(
                upstream = %upstream_name,
                subject = %query.subject,
                kind = e.kind(),
                error = %e,
                "upstream oauth: begin_authorization failed"
            );
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}

async fn callback(
    State(state): State<AppState>,
    Path(upstream_name): Path<String>,
    Query(query): Query<CallbackQuery>,
) -> impl IntoResponse {
    let Some(managers) = state.upstream_oauth.as_ref() else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "upstream oauth is not configured",
        )
            .into_response();
    };

    let Some(manager) = managers.get(&upstream_name) else {
        return (
            StatusCode::NOT_FOUND,
            format!("upstream '{upstream_name}' not found or has no oauth config"),
        )
            .into_response();
    };

    match manager
        .complete_authorization_callback_by_csrf(&query.code, &query.state)
        .await
    {
        Ok(()) => {
            info!(
                upstream = %upstream_name,
                "upstream oauth: authorization completed successfully"
            );
            Html(format!(
                "<html><body>\
                <h2>Authorization successful</h2>\
                <p>Upstream <strong>{upstream_name}</strong> has been authorized. \
                You may close this tab.</p>\
                </body></html>"
            ))
            .into_response()
        }
        Err(e) => {
            warn!(
                upstream = %upstream_name,
                kind = e.kind(),
                error = %e,
                "upstream oauth: callback failed"
            );
            (
                StatusCode::BAD_REQUEST,
                Html(format!(
                    "<html><body>\
                    <h2>Authorization failed</h2>\
                    <p>{e}</p>\
                    </body></html>"
                )),
            )
                .into_response()
        }
    }
}
