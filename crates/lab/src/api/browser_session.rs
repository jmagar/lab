use axum::extract::State;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;

use crate::api::state::AppState;

pub async fn auth_session(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(auth_state) = state.oauth_state.as_ref().map(|state| (**state).clone()) else {
        return (
            StatusCode::OK,
            [(header::CACHE_CONTROL, "private, no-store")],
            axum::Json(serde_json::json!({ "authenticated": false })),
        )
            .into_response();
    };

    let cookie_name = lab_auth::session::BROWSER_SESSION_COOKIE_NAME;
    let has_cookie_header = headers.contains_key(header::COOKIE);
    let browser_session_cookie = lab_auth::session::read_cookie(&headers, cookie_name);
    let has_browser_session_cookie = browser_session_cookie.is_some();
    tracing::info!(
        has_cookie_header,
        has_browser_session_cookie,
        "auth session request received"
    );
    let maybe_session = browser_session_cookie.map(|session_id| async move {
        match auth_state.store.find_browser_session(&session_id).await {
            Ok(session) => {
                tracing::info!(
                    has_cookie_header,
                    has_browser_session_cookie,
                    session_found = session.is_some(),
                    "auth session lookup completed"
                );
                session
            }
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    has_cookie_header,
                    has_browser_session_cookie,
                    "auth session lookup failed"
                );
                None
            }
        }
    });
    let session = match maybe_session {
        Some(fut) => fut.await,
        None => None,
    };

    let body = match session {
        Some(session) => serde_json::json!({
            "authenticated": true,
            "user": {
                "sub": session.subject,
                "email": session.email,
            },
            "expires_at": session.expires_at,
            "csrf_token": session.csrf_token,
        }),
        None => serde_json::json!({ "authenticated": false }),
    };

    (
        StatusCode::OK,
        [(header::CACHE_CONTROL, "private, no-store")],
        axum::Json(body),
    )
        .into_response()
}

pub async fn auth_logout(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    let Some(auth_state) = state.oauth_state.as_ref().map(|state| (**state).clone()) else {
        return StatusCode::NO_CONTENT.into_response();
    };

    let mut response = StatusCode::NO_CONTENT.into_response();
    if let Some(session_id) =
        lab_auth::session::read_cookie(&headers, lab_auth::session::BROWSER_SESSION_COOKIE_NAME)
    {
        let csrf = headers
            .get(lab_auth::session::BROWSER_CSRF_HEADER_NAME)
            .and_then(|value| value.to_str().ok());
        match auth_state.store.find_browser_session(&session_id).await {
            Ok(Some(session)) => {
                if csrf != Some(session.csrf_token.as_str()) {
                    return (
                        StatusCode::UNPROCESSABLE_ENTITY,
                        axum::Json(serde_json::json!({
                            "kind": "validation_failed",
                            "message": "missing or invalid csrf token",
                        })),
                    )
                        .into_response();
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
