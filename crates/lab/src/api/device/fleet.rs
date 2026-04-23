use std::time::Instant;

use axum::{
    Json,
    extract::{
        Path, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;

use crate::api::{ToolError, state::AppState};
use crate::config::DeviceRole;
use crate::device::enrollment::store::{
    EnrollmentAttempt, EnrollmentDecision, EnrollmentStore, TailnetIdentity,
};
use crate::device::checkin::{DeviceHello, DeviceMetadataUpload, DeviceStatus};
use crate::device::log_event::DeviceLogEvent;

pub async fn list_devices(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ToolError> {
    let store = require_master_store(&state)?;
    let devices = store.list_devices().await;
    Ok(Json(serde_json::Value::Array(
        devices
            .into_iter()
            .map(|snapshot| {
                json!({
                    "device_id": snapshot.device_id,
                    "connected": snapshot.connected,
                    "role": snapshot.role,
                    "log_count": snapshot.logs.len(),
                    "discovered_config_count": snapshot
                        .metadata
                        .as_ref()
                        .map(|metadata| metadata.discovered_configs.len())
                        .unwrap_or(0),
                })
            })
            .collect(),
    )))
}

pub async fn get_device(
    State(state): State<AppState>,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, ToolError> {
    let store = require_master_store(&state)?;
    let device_id = super::normalize_device_id_value(&device_id, "device_id")?;
    let snapshot = store
        .device(&device_id)
        .await
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: format!("unknown device `{device_id}`"),
        })?;
    Ok(Json(json!({
        "device_id": snapshot.device_id,
        "connected": snapshot.connected,
        "role": snapshot.role,
        "status": snapshot.status,
        "metadata": snapshot.metadata,
        "log_count": snapshot.logs.len(),
    })))
}

pub(crate) fn require_master_store(
    state: &AppState,
) -> Result<std::sync::Arc<crate::device::store::DeviceFleetStore>, ToolError> {
    if matches!(state.device_role, Some(DeviceRole::NonMaster)) {
        return Err(ToolError::Sdk {
            sdk_kind: "not_found".to_string(),
            message: "device fleet queries are only available on the master".to_string(),
        });
    }
    state
        .device_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("device store is not configured"))
}

pub(crate) fn require_enrollment_store(
    state: &AppState,
) -> Result<std::sync::Arc<EnrollmentStore>, ToolError> {
    state
        .enrollment_store
        .clone()
        .ok_or_else(|| ToolError::internal_message("enrollment store is not configured"))
}

pub async fn websocket_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<Response, ToolError> {
    let store = require_master_store(&state)?;
    let enrollment_store = require_enrollment_store(&state)?;
    Ok(ws.max_message_size(10 * 1024 * 1024).on_upgrade(move |socket| async move {
        if let Err(error) = handle_websocket(socket, store, enrollment_store).await {
            tracing::warn!(error = %error, "fleet websocket session failed");
        }
    }))
}

async fn handle_websocket(
    mut socket: WebSocket,
    store: std::sync::Arc<crate::device::store::DeviceFleetStore>,
    enrollment_store: std::sync::Arc<EnrollmentStore>,
) -> Result<(), anyhow::Error> {
    let mut session_device_id: Option<String> = None;

    while let Some(message) = socket.next().await {
        match message? {
            Message::Text(text) => {
                let request: RpcRequest =
                    serde_json::from_str(&text).map_err(|error| anyhow::anyhow!(error))?;
                let response =
                    handle_rpc_request(request, &store, &enrollment_store, &mut session_device_id)
                        .await;
                socket
                    .send(Message::Text(response.to_string().into()))
                    .await?;
            }
            Message::Ping(payload) => {
                socket.send(Message::Pong(payload)).await?;
            }
            Message::Pong(_) => {}
            Message::Binary(_) => {
                socket
                    .send(Message::Text(
                        error_response(None, -32600, "binary websocket frames are not supported")
                            .to_string()
                            .into(),
                    ))
                    .await?;
            }
            Message::Close(_) => break,
        }
    }

    if let Some(device_id) = session_device_id {
        store.set_connected(&device_id, false).await;
    }

    Ok(())
}

async fn handle_rpc_request(
    request: RpcRequest,
    store: &crate::device::store::DeviceFleetStore,
    enrollment_store: &EnrollmentStore,
    session_device_id: &mut Option<String>,
) -> serde_json::Value {
    match request.method.as_str() {
        "initialize" => {
            let params: InitializeParams = match serde_json::from_value(
                request.params.unwrap_or(serde_json::Value::Null),
            ) {
                Ok(params) => params,
                Err(error) => {
                    return error_response(
                        request.id,
                        -32602,
                        format!("invalid initialize params: {error}"),
                    );
                }
            };

            match handle_initialize(store, enrollment_store, &params).await {
                Ok(initialized) => {
                    *session_device_id = Some(initialized.device_id.clone());
                    success_response(
                        request.id,
                        json!({
                            "protocolVersion": params.protocol_version,
                            "serverInfo": {
                                "name": "lab-fleet",
                                "version": env!("CARGO_PKG_VERSION"),
                            },
                            "_meta": {
                                "lab.device_id": initialized.device_id,
                            }
                        }),
                    )
                }
                Err(error) => tool_error_response(request.id, &error),
            }
        }
        "fleet/status.push" => match require_initialized_device_id(session_device_id)
            .and_then(|device_id| {
                let params = request.params.unwrap_or(serde_json::Value::Null);
                parse_status_params(params, &device_id)
            }) {
            Ok(status) => {
                store.record_status(status).await;
                success_response(request.id, json!({}))
            }
            Err(error) => tool_error_response(request.id, &error),
        },
        "fleet/metadata.push" => match require_initialized_device_id(session_device_id)
            .and_then(|device_id| {
                let params = request.params.unwrap_or(serde_json::Value::Null);
                parse_metadata_params(params, &device_id)
            }) {
            Ok(metadata) => {
                store.record_metadata(metadata).await;
                success_response(request.id, json!({}))
            }
            Err(error) => tool_error_response(request.id, &error),
        },
        "fleet/log.event" => {
            let start = Instant::now();
            match require_initialized_device_id(session_device_id).and_then(|device_id| {
                let params = request.params.unwrap_or(serde_json::Value::Null);
                parse_log_events(params, &device_id).map(|events| (device_id, events))
            }) {
                Ok((device_id, events)) => {
                    let event_count = events.len();
                    store.record_logs(&device_id, events).await;
                    tracing::info!(
                        surface = "api",
                        service = "fleet",
                        action = "ws.log.event",
                        device_id = %device_id,
                        event_count,
                        elapsed_ms = start.elapsed().as_millis(),
                        "fleet websocket log batch recorded"
                    );
                    success_response(request.id, json!({}))
                }
                Err(error) => tool_error_response(request.id, &error),
            }
        }
        other => error_response(
            request.id,
            -32601,
            format!("unsupported fleet websocket method `{other}`"),
        ),
    }
}

async fn handle_initialize(
    store: &crate::device::store::DeviceFleetStore,
    enrollment_store: &EnrollmentStore,
    params: &InitializeParams,
) -> Result<InitializedDevice, ToolError> {
    let meta = params.meta.as_ref().ok_or_else(|| ToolError::InvalidParam {
        message: "initialize params must include `_meta`".to_string(),
        param: "_meta".to_string(),
    })?;
    let device_id = super::normalize_device_id_value(&meta.device_id, "_meta.lab.device_id")?;
    if meta.device_token.trim().is_empty() {
        return Err(ToolError::InvalidParam {
            message: "initialize `_meta.lab.device_token` must not be empty".to_string(),
            param: "_meta.lab.device_token".to_string(),
        });
    }
    let tailnet_identity = meta
        .tailnet_identity
        .clone()
        .ok_or_else(|| ToolError::InvalidParam {
            message: "initialize `_meta.lab.tailnet_identity` must be present".to_string(),
            param: "_meta.lab.tailnet_identity".to_string(),
        })?;

    match enrollment_store
        .validate(&device_id, &meta.device_token)
        .await
        .map_err(|error| ToolError::internal_message(format!("validate enrollment: {error}")))?
    {
        EnrollmentDecision::Approved(_) => {}
        EnrollmentDecision::PendingRequired => {
            enrollment_store
                .record_pending(EnrollmentAttempt {
                    device_id: device_id.clone(),
                    token: meta.device_token.clone(),
                    tailnet_identity,
                    client_version: params.client_info.version.clone(),
                    metadata: None,
                })
                .await
                .map_err(|error| ToolError::internal_message(format!("record pending enrollment: {error}")))?;
            return Err(ToolError::Sdk {
                sdk_kind: "enrollment_required".to_string(),
                message: format!("device `{device_id}` is pending enrollment approval"),
            });
        }
        EnrollmentDecision::Denied(_) => {
            return Err(ToolError::Sdk {
                sdk_kind: "access_denied".to_string(),
                message: format!("device `{device_id}` has been denied enrollment"),
            });
        }
        EnrollmentDecision::TokenMismatch(_) => {
            return Err(ToolError::Sdk {
                sdk_kind: "auth_failed".to_string(),
                message: format!("device `{device_id}` presented an unexpected token"),
            });
        }
    }

    store
        .record_hello(DeviceHello {
            device_id: device_id.clone(),
            role: "device".to_string(),
            version: params.client_info.version.clone(),
        })
        .await;
    store.set_connected(&device_id, true).await;
    Ok(InitializedDevice { device_id })
}

fn parse_status_params(
    params: serde_json::Value,
    session_device_id: &str,
) -> Result<DeviceStatus, ToolError> {
    let mut status: DeviceStatus =
        serde_json::from_value(params).map_err(|error| ToolError::InvalidParam {
            message: format!("invalid fleet/status.push params: {error}"),
            param: "params".to_string(),
        })?;
    status.device_id = super::normalize_device_id_value(&status.device_id, "params.device_id")?;
    if status.device_id != session_device_id {
        return Err(ToolError::InvalidParam {
            message: format!(
                "status device_id `{}` does not match initialized device `{session_device_id}`",
                status.device_id
            ),
            param: "params.device_id".to_string(),
        });
    }
    Ok(status)
}

fn parse_log_events(
    params: serde_json::Value,
    session_device_id: &str,
) -> Result<Vec<DeviceLogEvent>, ToolError> {
    let payload: FleetLogEventParams =
        serde_json::from_value(params).map_err(|error| ToolError::InvalidParam {
            message: format!("invalid fleet/log.event params: {error}"),
            param: "params".to_string(),
        })?;
    let device_id = super::normalize_device_id_value(&payload.device_id, "params.device_id")?;
    if device_id != session_device_id {
        return Err(ToolError::InvalidParam {
            message: format!(
                "log batch device_id `{device_id}` does not match initialized device `{session_device_id}`"
            ),
            param: "params.device_id".to_string(),
        });
    }

    let mut events = payload.events;
    for (index, event) in events.iter_mut().enumerate() {
        event.device_id =
            super::normalize_device_id_value(&event.device_id, &format!("events[{index}].device_id"))?;
        if event.device_id != device_id {
            return Err(ToolError::InvalidParam {
                message: format!(
                    "events[{index}].device_id must match batch device_id `{device_id}`"
                ),
                param: format!("events[{index}].device_id"),
            });
        }
    }
    Ok(events)
}

fn parse_metadata_params(
    params: serde_json::Value,
    session_device_id: &str,
) -> Result<DeviceMetadataUpload, ToolError> {
    let mut metadata: DeviceMetadataUpload =
        serde_json::from_value(params).map_err(|error| ToolError::InvalidParam {
            message: format!("invalid fleet/metadata.push params: {error}"),
            param: "params".to_string(),
        })?;
    metadata.device_id =
        super::normalize_device_id_value(&metadata.device_id, "params.device_id")?;
    if metadata.device_id != session_device_id {
        return Err(ToolError::InvalidParam {
            message: format!(
                "metadata device_id `{}` does not match initialized device `{session_device_id}`",
                metadata.device_id
            ),
            param: "params.device_id".to_string(),
        });
    }
    Ok(metadata)
}

fn require_initialized_device_id(session_device_id: &Option<String>) -> Result<String, ToolError> {
    session_device_id
        .clone()
        .ok_or_else(|| ToolError::Sdk {
            sdk_kind: "auth_failed".to_string(),
            message: "websocket session must send initialize before fleet methods".to_string(),
    })
}

fn tool_error_response(id: Option<serde_json::Value>, error: &ToolError) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": tool_error_code(error),
            "message": error.to_string(),
            "data": error,
        }
    })
}

fn tool_error_code(error: &ToolError) -> i64 {
    match error.kind() {
        "invalid_param" | "missing_param" | "validation_failed" => -32602,
        "auth_failed" | "access_denied" | "enrollment_required" => -32001,
        _ => -32000,
    }
}

fn success_response(id: Option<serde_json::Value>, result: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result,
    })
}

fn error_response(id: Option<serde_json::Value>, code: i64, message: impl Into<String>) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into(),
        }
    })
}

#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InitializeParams {
    protocol_version: String,
    client_info: ClientInfo,
    #[serde(rename = "_meta")]
    meta: Option<InitializeMeta>,
}

#[derive(Debug, Deserialize)]
struct ClientInfo {
    version: String,
}

#[derive(Debug, Deserialize)]
struct InitializeMeta {
    #[serde(rename = "lab.device_id")]
    device_id: String,
    #[serde(rename = "lab.device_token")]
    device_token: String,
    #[serde(rename = "lab.tailnet_identity")]
    tailnet_identity: Option<TailnetIdentity>,
}

#[derive(Debug, Deserialize)]
struct FleetLogEventParams {
    device_id: String,
    events: Vec<DeviceLogEvent>,
}

struct InitializedDevice {
    device_id: String,
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{Router, routing::get};
    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::{connect_async, tungstenite::Message};

    use super::*;

    #[tokio::test]
    async fn websocket_initialize_metadata_status_and_logs_round_trip_into_store() {
        let store = Arc::new(crate::device::store::DeviceFleetStore::default());
        let enrollment_store = Arc::new(
            EnrollmentStore::open(test_enrollment_store_path("fleet-happy"))
                .await
                .expect("open enrollment store"),
        );
        enrollment_store
            .record_pending(EnrollmentAttempt {
                device_id: "device-1".to_string(),
                token: "token-1".to_string(),
                tailnet_identity: TailnetIdentity {
                    node_key: "node-key".to_string(),
                    login_name: "user@example.com".to_string(),
                    hostname: "device-1".to_string(),
                },
                client_version: "0.7.3".to_string(),
                metadata: None,
            })
            .await
            .expect("record pending");
        enrollment_store
            .approve("device-1", None)
            .await
            .expect("approve");
        let state = AppState::new()
            .with_device_store(store.clone())
            .with_enrollment_store(enrollment_store);
        let app = Router::new()
            .route("/v1/fleet/ws", get(websocket_upgrade))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        let (mut socket, _) = connect_async(format!("ws://{addr}/v1/fleet/ws"))
            .await
            .expect("connect");

        socket
            .send(Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {
                            "name": "lab-device",
                            "version": "0.7.3",
                        },
                        "_meta": {
                            "lab.device_id": "device-1",
                            "lab.device_token": "token-1",
                            "lab.tailnet_identity": {
                                "node_key": "node-key",
                                "login_name": "user@example.com",
                                "hostname": "device-1",
                            }
                        }
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .expect("send initialize");
        let init_response = next_text(&mut socket).await;
        assert_eq!(init_response["result"]["_meta"]["lab.device_id"], "device-1");

        socket
            .send(Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": "fleet/metadata.push",
                    "params": {
                        "device_id": "device-1",
                        "discovered_configs": []
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .expect("send metadata");
        let metadata_response = next_text(&mut socket).await;
        assert!(metadata_response.get("error").is_none());

        socket
            .send(Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": "fleet/status.push",
                    "params": {
                        "device_id": "device-1",
                        "connected": true,
                        "cpu_percent": 12.5,
                        "memory_used_bytes": 1024,
                        "storage_used_bytes": 2048,
                        "os": "linux",
                        "ips": ["100.64.0.1"]
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .expect("send status");
        let status_response = next_text(&mut socket).await;
        assert!(status_response.get("error").is_none());

        socket
            .send(Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "id": 4,
                    "method": "fleet/log.event",
                    "params": {
                        "device_id": "device-1",
                        "events": [{
                            "device_id": "device-1",
                            "source": "syslog",
                            "timestamp_unix_ms": 1234,
                            "level": "info",
                            "message": "hello from ws",
                            "fields": {}
                        }]
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .expect("send logs");
        let log_response = next_text(&mut socket).await;
        assert!(log_response.get("error").is_none());

        socket.close(None).await.expect("close");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let snapshot = store.device("device-1").await.expect("snapshot");
        assert!(!snapshot.connected);
        assert_eq!(snapshot.role.as_deref(), Some("device"));
        assert_eq!(
            snapshot
                .metadata
                .as_ref()
                .map(|metadata| metadata.discovered_configs.len()),
            Some(0)
        );
        assert_eq!(snapshot.status.as_ref().and_then(|s| s.os.as_deref()), Some("linux"));
        assert_eq!(snapshot.logs.len(), 1);
        assert_eq!(snapshot.logs[0].message, "hello from ws");

        server.abort();
    }

    #[tokio::test]
    async fn initialize_unknown_device_creates_pending_and_rejects() {
        let store = Arc::new(crate::device::store::DeviceFleetStore::default());
        let enrollment_store =
            Arc::new(EnrollmentStore::open(test_enrollment_store_path("fleet-unknown")).await.expect("open"));
        let state = AppState::new()
            .with_device_store(store)
            .with_enrollment_store(enrollment_store.clone());
        let app = Router::new()
            .route("/v1/fleet/ws", get(websocket_upgrade))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });
        let (mut socket, _) = connect_async(format!("ws://{addr}/v1/fleet/ws")).await.expect("connect");

        send_initialize(&mut socket, "device-unknown", "token-unknown").await;
        let response = next_text(&mut socket).await;
        assert_eq!(response["error"]["data"]["kind"], "enrollment_required");

        let snapshot = enrollment_store.list().await.expect("list");
        assert!(snapshot.pending.contains_key("device-unknown"));
        assert!(snapshot.approved.is_empty());

        server.abort();
    }

    #[tokio::test]
    async fn initialize_approved_device_is_admitted() {
        let state = approved_ws_state("device-1", "token-1").await;
        let app = Router::new()
            .route("/v1/fleet/ws", get(websocket_upgrade))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });
        let (mut socket, _) = connect_async(format!("ws://{addr}/v1/fleet/ws")).await.expect("connect");

        send_initialize(&mut socket, "device-1", "token-1").await;
        let response = next_text(&mut socket).await;
        assert!(response.get("error").is_none());
        assert_eq!(response["result"]["_meta"]["lab.device_id"], "device-1");

        server.abort();
    }

    #[tokio::test]
    async fn initialize_denied_device_is_rejected() {
        let store = Arc::new(crate::device::store::DeviceFleetStore::default());
        let enrollment_store =
            Arc::new(EnrollmentStore::open(test_enrollment_store_path("fleet-denied")).await.expect("open"));
        enrollment_store
            .record_pending(EnrollmentAttempt {
                device_id: "device-1".to_string(),
                token: "token-1".to_string(),
                tailnet_identity: TailnetIdentity {
                    node_key: "node-key".to_string(),
                    login_name: "user@example.com".to_string(),
                    hostname: "device-1".to_string(),
                },
                client_version: "0.7.3".to_string(),
                metadata: None,
            })
            .await
            .expect("record pending");
        enrollment_store
            .deny("device-1", Some("no".to_string()))
            .await
            .expect("deny");
        let state = AppState::new()
            .with_device_store(store)
            .with_enrollment_store(enrollment_store);
        let app = Router::new()
            .route("/v1/fleet/ws", get(websocket_upgrade))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });
        let (mut socket, _) = connect_async(format!("ws://{addr}/v1/fleet/ws")).await.expect("connect");

        send_initialize(&mut socket, "device-1", "token-1").await;
        let response = next_text(&mut socket).await;
        assert_eq!(response["error"]["data"]["kind"], "access_denied");

        server.abort();
    }

    #[tokio::test]
    async fn initialize_wrong_token_for_approved_device_is_rejected() {
        let state = approved_ws_state("device-1", "token-1").await;
        let app = Router::new()
            .route("/v1/fleet/ws", get(websocket_upgrade))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });
        let (mut socket, _) = connect_async(format!("ws://{addr}/v1/fleet/ws")).await.expect("connect");

        send_initialize(&mut socket, "device-1", "wrong-token").await;
        let response = next_text(&mut socket).await;
        assert_eq!(response["error"]["data"]["kind"], "auth_failed");

        server.abort();
    }

    #[tokio::test]
    async fn fleet_methods_before_initialize_return_request_error_without_closing_socket() {
        let state = approved_ws_state("device-1", "token-1").await;
        let app = Router::new()
            .route("/v1/fleet/ws", get(websocket_upgrade))
            .with_state(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });
        let (mut socket, _) = connect_async(format!("ws://{addr}/v1/fleet/ws")).await.expect("connect");

        socket
            .send(Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "id": 99,
                    "method": "fleet/status.push",
                    "params": {
                        "device_id": "device-1",
                        "connected": true,
                        "ips": [],
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .expect("send pre-init status");
        let pre_init_response = next_text(&mut socket).await;
        assert_eq!(pre_init_response["error"]["data"]["kind"], "auth_failed");

        send_initialize(&mut socket, "device-1", "token-1").await;
        let init_response = next_text(&mut socket).await;
        assert!(init_response.get("error").is_none());

        server.abort();
    }

    async fn approved_ws_state(device_id: &str, token: &str) -> AppState {
        let store = Arc::new(crate::device::store::DeviceFleetStore::default());
        let enrollment_store =
            Arc::new(EnrollmentStore::open(test_enrollment_store_path("fleet-approved")).await.expect("open"));
        enrollment_store
            .record_pending(EnrollmentAttempt {
                device_id: device_id.to_string(),
                token: token.to_string(),
                tailnet_identity: TailnetIdentity {
                    node_key: "node-key".to_string(),
                    login_name: "user@example.com".to_string(),
                    hostname: device_id.to_string(),
                },
                client_version: "0.7.3".to_string(),
                metadata: None,
            })
            .await
            .expect("record pending");
        enrollment_store.approve(device_id, None).await.expect("approve");
        AppState::new()
            .with_device_store(store)
            .with_enrollment_store(enrollment_store)
    }

    async fn send_initialize(
        socket: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        device_id: &str,
        token: &str,
    ) {
        socket
            .send(Message::Text(
                json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "initialize",
                    "params": {
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "clientInfo": {
                            "name": "lab-device",
                            "version": "0.7.3",
                        },
                        "_meta": {
                            "lab.device_id": device_id,
                            "lab.device_token": token,
                            "lab.tailnet_identity": {
                                "node_key": "node-key",
                                "login_name": "user@example.com",
                                "hostname": device_id,
                            }
                        }
                    }
                })
                .to_string()
                .into(),
            ))
            .await
            .expect("send initialize");
    }

    fn test_enrollment_store_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("lab-{name}-{}.json", uuid::Uuid::new_v4()))
    }

    async fn next_text(
        socket: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    ) -> serde_json::Value {
        match socket.next().await.expect("message").expect("ok") {
            Message::Text(text) => serde_json::from_str(&text).expect("json"),
            other => panic!("unexpected websocket message: {other:?}"),
        }
    }
}
