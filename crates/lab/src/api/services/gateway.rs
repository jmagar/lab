use std::sync::Arc;

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::error::ToolError;

pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let manager = state
        .gateway_manager
        .clone()
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "internal_error".to_string(),
            message: "gateway manager not wired".to_string(),
        })?;

    handle_action(
        "gateway",
        "api",
        request_id,
        req,
        crate::dispatch::gateway::ACTIONS,
        move |action, params| {
            let manager = Arc::clone(&manager);
            async move {
                crate::dispatch::gateway::dispatch_with_manager(&manager, &action, params).await
            }
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
    };
    use serde_json::json;
    use tower::ServiceExt;

    use crate::api::{router::build_router_with_bearer, state::AppState};
    use crate::dispatch::gateway::manager::{GatewayManager, GatewayRuntimeHandle};
    use crate::mcp::registry::build_default_registry;

    fn test_app() -> Router {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(1);
        let path = std::env::temp_dir().join(format!(
            "lab-gateway-api-test-{}-{}.toml",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        let state = AppState::from_registry(build_default_registry()).with_gateway_manager(
            Arc::new(GatewayManager::new(path, GatewayRuntimeHandle::default())),
        );
        build_router_with_bearer(state, None, None)
    }

    async fn post_gateway(app: Router, body: serde_json::Value) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/gateway")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("request"),
        )
        .await
        .expect("response")
    }

    async fn post_gateway_fresh(body: serde_json::Value) -> axum::response::Response {
        post_gateway(test_app(), body).await
    }

    async fn get_gateway_actions(app: Router) -> axum::response::Response {
        app.oneshot(
            Request::builder()
                .method("GET")
                .uri("/v1/gateway/actions")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response")
    }

    #[tokio::test]
    async fn gateway_list_route_exists() {
        let response = post_gateway_fresh(json!({"action":"gateway.list","params":{}})).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn gateway_get_returns_not_found_for_missing_gateway() {
        let response =
            post_gateway_fresh(json!({"action":"gateway.get","params":{"name":"fixture-http"}}))
                .await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn gateway_test_accepts_proposed_spec() {
        let response = post_gateway_fresh(json!({
            "action":"gateway.test",
            "params":{"spec":{"name":"fixture-stdio","command":"echo","args":["hello"]}}
        }))
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn gateway_add_update_remove_reload_routes_exist() {
        let app = test_app();

        let added = post_gateway(app.clone(), json!({
            "action":"gateway.add",
            "params":{"spec":{"name":"fixture-http","url":"http://127.0.0.1:9001","bearer_token_env":"FIXTURE_HTTP_TOKEN"}}
        }))
        .await;
        assert_eq!(added.status(), StatusCode::OK);

        let updated = post_gateway(
            app.clone(),
            json!({
                "action":"gateway.update",
                "params":{"name":"fixture-http","patch":{"proxy_resources":true}}
            }),
        )
        .await;
        assert_eq!(updated.status(), StatusCode::OK);

        let removed = post_gateway(
            app.clone(),
            json!({"action":"gateway.remove","params":{"name":"fixture-http"}}),
        )
        .await;
        assert_eq!(removed.status(), StatusCode::OK);

        let reloaded = post_gateway(app, json!({"action":"gateway.reload","params":{}})).await;
        assert_eq!(reloaded.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn gateway_actions_endpoint_is_registered() {
        let response = get_gateway_actions(test_app()).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}
