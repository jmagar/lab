//! Top-level axum router — mounts `POST /v1/<service>` for every enabled service.

use std::time::Duration;

use axum::{
    Router,
    extract::State,
    http::HeaderMap,
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

use super::{ActionRequest, health, state::AppState};
use crate::dispatch::context::DispatchContext;
use crate::dispatch::error::ToolError;

#[allow(clippy::too_many_lines)]
pub fn build_router(state: AppState) -> Router {
    let v1 = Router::new()
        .route("/{service}/actions", get(service_actions))
        .route("/{service}", axum::routing::post(dispatch_service));

    let router = Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .nest("/v1", v1);

    let x_request_id = HeaderName::from_static("x-request-id");

    // Layers apply bottom-up: last .layer() call = outermost middleware.
    // Desired execution order (outermost → innermost → handler):
    //   SetRequestId → TraceLayer → PropagateRequestId → Timeout → Compression → CORS → handler
    router
        .with_state(state)
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(30),
        ))
        // PropagateRequestId echoes the id back in the response header.
        .layer(PropagateRequestIdLayer::new(x_request_id.clone()))
        // TraceLayer reads x-request-id set by SetRequestId (outermost).
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
        // SetRequestId generates a UUID for every request that lacks one.
        .layer(SetRequestIdLayer::new(x_request_id, MakeRequestUuid))
}

/// Generic dispatch handler: looks up the service in the registry and
/// calls its `DispatchFn` through the shared `handle_action` wrapper.
async fn dispatch_service(
    State(state): State<AppState>,
    axum::extract::Path(service): axum::extract::Path<String>,
    headers: HeaderMap,
    axum::Json(req): axum::Json<ActionRequest>,
) -> Result<axum::Json<serde_json::Value>, ToolError> {
    let svc = state
        .registry
        .services()
        .iter()
        .find(|s| s.name == service)
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".into(),
            message: format!("unknown service `{service}`"),
        })?;

    let dispatch = svc.dispatch;
    let actions = svc.actions;
    let ctx = DispatchContext {
        surface: "api",
        instance: None,
    };
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());

    super::services::helpers::handle_action(
        svc.name,
        ctx,
        request_id,
        req,
        actions,
        move |action, params| dispatch(action, params),
        Some(&headers),
    )
    .await
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
