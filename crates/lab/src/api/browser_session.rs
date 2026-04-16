use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::api::state::AppState;

use lab_auth::session::{BROWSER_CSRF_HEADER_NAME, BROWSER_SESSION_COOKIE_NAME};

fn oauth_state(state: &AppState) -> Option<lab_auth::state::AuthState> {
    state.oauth_state.as_ref().map(|state| (**state).clone())
}

fn no_store_json(body: serde_json::Value) -> Response {
    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "private, no-store")],
        axum::Json(body),
    )
        .into_response()
}

fn unauthenticated_session_response() -> Response {
    no_store_json(serde_json::json!({ "authenticated": false }))
}

fn session_cookie(headers: &HeaderMap) -> Option<String> {
    lab_auth::session::read_cookie(headers, BROWSER_SESSION_COOKIE_NAME)
}

async fn load_browser_session(
    auth_state: &lab_auth::state::AuthState,
    headers: &HeaderMap,
) -> Result<Option<lab_auth::types::BrowserSessionRow>, lab_auth::error::AuthError> {
    let has_cookie_header = headers.contains_key(header::COOKIE);
    let browser_session_cookie = session_cookie(headers);
    let has_browser_session_cookie = browser_session_cookie.is_some();
    tracing::info!(
        has_cookie_header,
        has_browser_session_cookie,
        "auth session request received"
    );

    let Some(session_id) = browser_session_cookie else {
        return Ok(None);
    };

    match auth_state.store.find_browser_session(&session_id).await {
        Ok(session) => {
            tracing::info!(
                has_cookie_header,
                has_browser_session_cookie,
                session_found = session.is_some(),
                "auth session lookup completed"
            );
            Ok(session)
        }
        Err(error) => {
            tracing::warn!(
                error = %error,
                has_cookie_header,
                has_browser_session_cookie,
                "auth session lookup failed"
            );
            Err(error)
        }
    }
}

fn invalid_csrf_response() -> Response {
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        axum::Json(serde_json::json!({
            "kind": "validation_failed",
            "message": "missing or invalid csrf token",
        })),
    )
        .into_response()
}

pub async fn auth_session(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(auth_state) = oauth_state(&state) else {
        return unauthenticated_session_response();
    };

    let body = match load_browser_session(&auth_state, &headers).await {
        Ok(Some(session)) => serde_json::json!({
            "authenticated": true,
            "user": {
                "sub": session.subject,
                "email": session.email,
            },
            "expires_at": session.expires_at,
            "csrf_token": session.csrf_token,
        }),
        Ok(None) => return unauthenticated_session_response(),
        Err(error) => {
            tracing::error!(error = %error, "failed to load browser session for auth session");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    no_store_json(body)
}

pub async fn auth_logout(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(auth_state) = oauth_state(&state) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let mut response = StatusCode::NO_CONTENT.into_response();
    if let Some(session_id) = session_cookie(&headers) {
        let csrf = headers
            .get(BROWSER_CSRF_HEADER_NAME)
            .and_then(|value| value.to_str().ok());
        match auth_state.store.find_browser_session(&session_id).await {
            Ok(Some(session)) => {
                if csrf != Some(session.csrf_token.as_str()) {
                    tracing::warn!(
                        has_csrf_header = csrf.is_some(),
                        "auth logout rejected: missing or invalid csrf token"
                    );
                    return invalid_csrf_response();
                }
                if let Err(error) = auth_state.store.revoke_browser_session(&session_id).await {
                    tracing::error!(error = %error, "failed to revoke browser session");
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            }
            Ok(None) => {}
            Err(error) => {
                tracing::error!(error = %error, "failed to load browser session for logout");
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    lab_auth::session::append_set_cookie(
        &mut response,
        &lab_auth::session::clear_browser_session_cookie(&auth_state),
    );
    response
}
