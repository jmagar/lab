use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use serde_json::Value;

use crate::api::services::helpers::handle_action;
use crate::api::{ActionRequest, state::AppState};
use crate::dispatch::dozzle::ACTIONS;
use crate::dispatch::error::ToolError;

/// Build the route group for the Dozzle service.
pub fn routes(_state: AppState) -> Router<AppState> {
    Router::new().route("/", post(handle))
}

async fn handle(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<ActionRequest>,
) -> Result<Json<Value>, ToolError> {
    let request_id = headers.get("x-request-id").and_then(|v| v.to_str().ok());
    let client = state.clients.dozzle.clone();
    handle_action(
        "dozzle",
        "api",
        request_id,
        req,
        ACTIONS,
        move |action, params| async move {
            let Some(client) = client.as_ref() else {
                return Err(crate::dispatch::dozzle::not_configured_error());
            };
            crate::dispatch::dozzle::dispatch_with_client(client, &action, params).await
        },
    )
    .await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::extract::State;
    use lab_apis::core::Auth;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::dispatch::clients::ServiceClients;

    fn state_with_client(client: lab_apis::dozzle::DozzleClient) -> AppState {
        let mut state = AppState::new();
        state.clients = Arc::new(ServiceClients {
            dozzle: Some(Arc::new(client)),
            ..ServiceClients::default()
        });
        state
    }

    #[tokio::test]
    async fn api_handler_returns_version_payload() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/version"))
            .respond_with(ResponseTemplate::new(200).set_body_string("<pre>v10.5.1</pre>"))
            .mount(&server)
            .await;
        let client = lab_apis::dozzle::DozzleClient::new(&server.uri(), Auth::None).unwrap();
        let req = ActionRequest {
            action: "server.version".to_string(),
            params: serde_json::json!({}),
        };

        let Json(value) = handle(
            State(state_with_client(client)),
            HeaderMap::new(),
            Json(req),
        )
        .await
        .unwrap();

        assert_eq!(value["version"], "v10.5.1");
    }

    #[tokio::test]
    async fn api_handler_rejects_unknown_action() {
        let state = AppState::new();
        let req = ActionRequest {
            action: "not.real".to_string(),
            params: serde_json::json!({}),
        };

        let err = handle(State(state), HeaderMap::new(), Json(req))
            .await
            .unwrap_err();

        assert_eq!(err.kind(), "unknown_action");
    }
}
