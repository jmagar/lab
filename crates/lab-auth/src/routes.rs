use axum::Router;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::{self, Next};
use axum::response::Response;
use axum::routing::{get, post};
use std::time::Instant;

use crate::authorize::{authorize, browser_login, callback, register_client};
use crate::error::AuthErrorKind;
use crate::metadata::{authorization_server_metadata, jwks, protected_resource_metadata};
use crate::state::AuthState;
use crate::token::token;

pub fn router(state: AuthState) -> Router {
    Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(authorization_server_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(protected_resource_metadata),
        )
        .route("/jwks", get(jwks))
        .route("/register", post(register_client))
        .route("/authorize", get(authorize))
        .route("/auth/login", get(browser_login))
        .route("/auth/google/callback", get(callback))
        .route("/token", post(token))
        .with_state(state)
        .layer(middleware::from_fn(auth_dispatch_observability))
}

async fn auth_dispatch_observability(request: Request, next: Next) -> Response {
    let action = auth_dispatch_action(request.uri().path());
    let request_id = request_id(request.headers()).map(ToOwned::to_owned);
    let start = Instant::now();
    let response = next.run(request).await;
    let elapsed_ms = start.elapsed().as_millis();
    let status = response.status();
    let kind = response
        .extensions()
        .get::<AuthErrorKind>()
        .map(|kind| kind.0)
        .or_else(|| status_error_kind(status));

    if status.is_server_error() || status.is_client_error() {
        tracing::warn!(
            surface = "api",
            service = "auth",
            action,
            request_id = request_id.as_deref(),
            elapsed_ms,
            kind,
            status = status.as_u16(),
            "dispatch.error"
        );
    } else {
        tracing::info!(
            surface = "api",
            service = "auth",
            action,
            request_id = request_id.as_deref(),
            elapsed_ms,
            status = status.as_u16(),
            "dispatch.finish"
        );
    }

    response
}

fn request_id(headers: &HeaderMap) -> Option<&str> {
    headers.get("x-request-id").and_then(|value| value.to_str().ok())
}

fn status_error_kind(status: StatusCode) -> Option<&'static str> {
    if status.is_client_error() {
        Some("request_failed")
    } else if status.is_server_error() {
        Some("internal_error")
    } else {
        None
    }
}

fn auth_dispatch_action(path: &str) -> &'static str {
    match path {
        "/.well-known/oauth-authorization-server" => "oauth.metadata.authorization_server",
        "/.well-known/oauth-protected-resource" => "oauth.metadata.protected_resource",
        "/jwks" => "oauth.jwks",
        "/register" => "oauth.register",
        "/authorize" => "oauth.authorize",
        "/auth/login" => "oauth.browser_login",
        "/auth/google/callback" => "oauth.callback",
        "/token" => "oauth.token",
        _ => "oauth.unknown",
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request as HttpRequest, StatusCode};
    use tower::util::ServiceExt;
    use tracing_subscriber::layer::SubscriberExt;

    use super::*;
    use crate::authorize::tests::test_auth_state;

    #[test]
    fn auth_dispatch_action_names_are_stable() {
        assert_eq!(
            auth_dispatch_action("/.well-known/oauth-authorization-server"),
            "oauth.metadata.authorization_server"
        );
        assert_eq!(auth_dispatch_action("/register"), "oauth.register");
        assert_eq!(auth_dispatch_action("/authorize"), "oauth.authorize");
        assert_eq!(auth_dispatch_action("/token"), "oauth.token");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn auth_dispatch_logs_request_id_action_elapsed_and_failure_kind() {
        let _tracing_lock = crate::test_support::TRACING_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let buf = crate::test_support::SharedBuf::default();
        let subscriber = tracing_subscriber::registry()
            .with(tracing_subscriber::EnvFilter::new("lab_auth=info"))
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(buf.clone())
                    .with_ansi(false)
                    .without_time(),
            );
        let _ = tracing::subscriber::set_global_default(subscriber);

        let app = router(test_auth_state().await);
        let response = app
            .oneshot(
                HttpRequest::builder()
                    .method("POST")
                    .uri("/register")
                    .header("content-type", "application/json")
                    .header("x-request-id", "req-auth-1")
                    .body(Body::from(r#"{"redirect_uris":[]}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

        let logs = crate::test_support::captured_logs(&buf);
        for expected in [
            "\"surface\":\"api\"",
            "\"service\":\"auth\"",
            "\"action\":\"oauth.register\"",
            "\"request_id\":\"req-auth-1\"",
            "\"kind\":\"validation_failed\"",
            "\"status\":422",
            "\"dispatch.error\"",
        ] {
            assert!(
                logs.contains(expected),
                "missing auth dispatch log field `{expected}` in:\n{logs}"
            );
        }
        assert!(
            logs.contains("\"elapsed_ms\":"),
            "missing elapsed_ms in:\n{logs}"
        );
    }
}
