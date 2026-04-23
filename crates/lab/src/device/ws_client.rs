use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

use crate::device::queue::{DeviceOutboundQueue, QueuedEnvelope};
use crate::device::token;
use crate::dispatch::upstream::transport::websocket::{jitter_delay, reprobe_backoff};

const FLUSH_BATCH_SIZE: usize = 100;
const IDLE_FLUSH_INTERVAL: Duration = Duration::from_secs(10);
const STATUS_INTERVAL: Duration = Duration::from_secs(30);
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;
const MAX_FRAME_SIZE: usize = 128 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailnetIdentity {
    pub node_key: String,
    pub login_name: String,
    pub hostname: String,
}

impl TailnetIdentity {
    #[must_use]
    pub fn discover(hostname: &str) -> Self {
        Self {
            node_key: std::env::var("LAB_TAILNET_NODE_KEY").unwrap_or_else(|_| hostname.to_string()),
            login_name: std::env::var("LAB_TAILNET_LOGIN_NAME")
                .unwrap_or_else(|_| "unknown".to_string()),
            hostname: hostname.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WsClient {
    url: url::Url,
    device_id: String,
    token_path: PathBuf,
    connected: Arc<AtomicBool>,
}

impl WsClient {
    pub fn new(base_url: &str, device_id: impl Into<String>, token_path: impl AsRef<Path>) -> Result<Self> {
        let url = websocket_url_from_master_base(base_url)?;
        Ok(Self {
            url,
            device_id: device_id.into(),
            token_path: token_path.as_ref().to_path_buf(),
            connected: Arc::new(AtomicBool::new(false)),
        })
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub async fn run(self, queue: Arc<DeviceOutboundQueue>) {
        let mut attempt = 0_u32;
        loop {
            match self.connect_and_run_session(&queue).await {
                Ok(()) => {
                    attempt = 0;
                }
                Err(error) => {
                    self.connected.store(false, Ordering::Relaxed);
                    attempt = attempt.saturating_add(1);
                    let delay = jitter_delay(reprobe_backoff(attempt), stable_seed(&self.device_id, attempt));
                    tracing::warn!(
                        device_id = %self.device_id,
                        attempt,
                        backoff_ms = delay.as_millis(),
                        error = %error,
                        "ws.reconnect_attempt"
                    );
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }

    async fn connect_and_run_session(&self, queue: &DeviceOutboundQueue) -> Result<()> {
        let token = token::load_or_create(&self.token_path).await?;
        let tailnet_identity = TailnetIdentity::discover(&self.device_id);
        let request = self
            .url
            .to_string()
            .into_client_request()
            .map_err(|error| anyhow!("build websocket request: {error}"))?;
        let mut websocket_config = WebSocketConfig::default();
        websocket_config.max_message_size = Some(MAX_MESSAGE_SIZE);
        websocket_config.max_frame_size = Some(MAX_FRAME_SIZE);
        websocket_config.accept_unmasked_frames = false;
        let (mut socket, _) = connect_async_with_config(request, Some(websocket_config), false)
            .await
            .map_err(|error| anyhow!("connect websocket: {error}"))?;

        let initialize = build_initialize_request(&self.device_id, &token, &tailnet_identity);
        socket
            .send(Message::Text(serde_json::to_string(&initialize)?.into()))
            .await
            .context("send websocket initialize")?;
        let init_response = next_text_message(&mut socket).await?;
        validate_success_response(&init_response, 1)?;
        self.connected.store(true, Ordering::Relaxed);

        let mut next_id = 2_i64;
        let mut status_deadline = tokio::time::Instant::now() + STATUS_INTERVAL;

        loop {
            let ack_count = self.flush_queue_batch(&mut socket, queue, &mut next_id).await?;
            let now = tokio::time::Instant::now();
            if now >= status_deadline {
                self.send_status_update(&mut socket, next_id).await?;
                next_id += 1;
                status_deadline = now + STATUS_INTERVAL;
                continue;
            }
            if ack_count > 0 {
                continue;
            }
            tokio::time::sleep(IDLE_FLUSH_INTERVAL).await;
        }
    }

    #[cfg(test)]
    async fn flush_queue_once(&self, queue: &DeviceOutboundQueue) -> Result<()> {
        let token = token::load_or_create(&self.token_path).await?;
        let tailnet_identity = TailnetIdentity::discover(&self.device_id);
        let request = self
            .url
            .to_string()
            .into_client_request()
            .map_err(|error| anyhow!("build websocket request: {error}"))?;
        let mut websocket_config = WebSocketConfig::default();
        websocket_config.max_message_size = Some(MAX_MESSAGE_SIZE);
        websocket_config.max_frame_size = Some(MAX_FRAME_SIZE);
        websocket_config.accept_unmasked_frames = false;
        let (mut socket, _) = connect_async_with_config(request, Some(websocket_config), false)
            .await
            .map_err(|error| anyhow!("connect websocket: {error}"))?;
        let initialize = build_initialize_request(&self.device_id, &token, &tailnet_identity);
        socket
            .send(Message::Text(serde_json::to_string(&initialize)?.into()))
            .await
            .context("send websocket initialize")?;
        let init_response = next_text_message(&mut socket).await?;
        validate_success_response(&init_response, 1)?;
        let mut next_id = 2_i64;
        let _ = self.flush_queue_batch(&mut socket, queue, &mut next_id).await?;
        socket.close(None).await.context("close websocket")?;
        Ok(())
    }

    async fn flush_queue_batch(
        &self,
        socket: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        queue: &DeviceOutboundQueue,
        next_id: &mut i64,
    ) -> Result<usize> {
        let drained = queue.drain_batch(FLUSH_BATCH_SIZE).await?;
        let mut ack_count = 0usize;
        for envelope in drained {
            let result: Result<()> = async {
                let request = queue_envelope_to_request(&envelope, *next_id)?;
                socket
                    .send(Message::Text(serde_json::to_string(&request)?.into()))
                    .await
                    .context("send websocket queue request")?;
                let response = next_text_message(socket).await?;
                validate_success_response(&response, *next_id)
            }
            .await;
            if let Err(error) = result {
                queue.ack_drained(ack_count).await?;
                return Err(error);
            }
            *next_id += 1;
            ack_count += 1;
        }
        queue.ack_drained(ack_count).await?;
        Ok(ack_count)
    }

    async fn send_status_update(
        &self,
        socket: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        request_id: i64,
    ) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "fleet/status.push",
            "params": {
                "device_id": self.device_id,
                "connected": true,
                "cpu_percent": serde_json::Value::Null,
                "memory_used_bytes": serde_json::Value::Null,
                "storage_used_bytes": serde_json::Value::Null,
                "os": std::env::consts::OS,
                "ips": [],
            }
        });
        socket
            .send(Message::Text(serde_json::to_string(&request)?.into()))
            .await
            .context("send websocket status update")?;
        let response = next_text_message(socket).await?;
        validate_success_response(&response, request_id)?;
        Ok(())
    }
}

pub fn websocket_url_from_master_base(base_url: &str) -> Result<url::Url> {
    let mut url = url::Url::parse(base_url.trim()).context("parse master base url")?;
    let scheme = match url.scheme() {
        "http" => "ws",
        "https" => "wss",
        "ws" => "ws",
        "wss" => "wss",
        other => return Err(anyhow!("unsupported master base url scheme `{other}`")),
    };
    url.set_scheme(scheme).map_err(|_| anyhow!("set websocket scheme"))?;
    url.set_path("/v1/fleet/ws");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

pub fn build_initialize_request(
    device_id: &str,
    device_token: &str,
    tailnet_identity: &TailnetIdentity,
) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "lab-device",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "_meta": {
                "lab.device_id": device_id,
                "lab.device_token": device_token,
                "lab.tailnet_identity": tailnet_identity,
            }
        }
    })
}

pub fn queue_envelope_to_request(
    envelope: &QueuedEnvelope,
    id: i64,
) -> Result<serde_json::Value> {
    let method = match envelope.kind.as_str() {
        "syslog_batch" => "fleet/log.event",
        "status" => "fleet/status.push",
        "metadata" => "fleet/metadata.push",
        other => return Err(anyhow!("unsupported queued envelope kind `{other}`")),
    };
    Ok(serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": envelope.payload,
    }))
}

async fn next_text_message(
    socket: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
) -> Result<String> {
    while let Some(message) = socket.next().await {
        match message.context("read websocket message")? {
            Message::Text(text) => return Ok(text.to_string()),
            Message::Ping(payload) => {
                socket.send(Message::Pong(payload)).await.context("send websocket pong")?;
            }
            Message::Pong(_) | Message::Frame(_) => {}
            Message::Binary(_) => return Err(anyhow!("binary websocket frames are not supported")),
            Message::Close(_) => return Err(anyhow!("websocket closed before response")),
        }
    }
    Err(anyhow!("websocket closed before response"))
}

fn validate_success_response(payload: &str, expected_id: i64) -> Result<()> {
    let value: serde_json::Value = serde_json::from_str(payload).context("decode websocket response")?;
    let response_id = value
        .get("id")
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| anyhow!("websocket response missing numeric id"))?;
    if response_id != expected_id {
        return Err(anyhow!(
            "websocket response id mismatch: expected {expected_id}, got {response_id}"
        ));
    }
    if let Some(error) = value.get("error") {
        let kind = error
            .get("data")
            .and_then(|data| data.get("kind"))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        return Err(anyhow!("websocket request failed ({kind}): {error}"));
    }
    Ok(())
}

fn stable_seed(device_id: &str, attempt: u32) -> u64 {
    let mut hash = 1_469_598_103_934_665_603_u64;
    for byte in device_id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash ^ u64::from(attempt)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::api::{device::fleet, state::AppState};
    use crate::device::queue::DeviceOutboundQueue;
    use axum::{Router, routing::get};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::Message;

    #[test]
    fn websocket_url_rewrites_http_base() {
        let url = websocket_url_from_master_base("http://master:8765").expect("url");
        assert_eq!(url.as_str(), "ws://master:8765/v1/fleet/ws");
    }

    #[test]
    fn initialize_request_includes_required_meta_fields() {
        let identity = TailnetIdentity {
            node_key: "node-key".to_string(),
            login_name: "user@example.com".to_string(),
            hostname: "host".to_string(),
        };
        let request = build_initialize_request("device-1", "token-1", &identity);
        assert_eq!(request["method"], "initialize");
        assert_eq!(request["params"]["_meta"]["lab.device_id"], "device-1");
        assert_eq!(request["params"]["_meta"]["lab.device_token"], "token-1");
        assert_eq!(
            request["params"]["_meta"]["lab.tailnet_identity"]["node_key"],
            "node-key"
        );
        assert_eq!(
            request["params"]["_meta"]["lab.tailnet_identity"]["login_name"],
            "user@example.com"
        );
    }

    #[test]
    fn queue_envelope_maps_to_fleet_methods() {
        let syslog = queue_envelope_to_request(
            &QueuedEnvelope::syslog_batch(serde_json::json!({"events": []})),
            2,
        )
        .expect("syslog");
        assert_eq!(syslog["method"], "fleet/log.event");

        let status = queue_envelope_to_request(
            &QueuedEnvelope::status(serde_json::json!({"connected": true})),
            3,
        )
        .expect("status");
        assert_eq!(status["method"], "fleet/status.push");

        let metadata = queue_envelope_to_request(
            &QueuedEnvelope::metadata(serde_json::json!({"device_id": "device-1"})),
            4,
        )
        .expect("metadata");
        assert_eq!(metadata["method"], "fleet/metadata.push");
    }

    #[tokio::test]
    async fn flush_queue_once_drains_and_acks_entries_over_websocket() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut socket = accept_async(stream).await.expect("accept websocket");
            let mut received_methods = Vec::new();

            while let Some(message) = socket.next().await {
                let text = match message.expect("message") {
                    Message::Text(text) => text.to_string(),
                    Message::Close(_) => break,
                    other => panic!("unexpected websocket message: {other:?}"),
                };
                let payload: serde_json::Value =
                    serde_json::from_str(&text).expect("parse request");
                received_methods.push(
                    payload["method"]
                        .as_str()
                        .expect("method")
                        .to_string(),
                );
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": payload["id"],
                    "result": {},
                });
                socket
                    .send(Message::Text(response.to_string().into()))
                    .await
                    .expect("send response");
            }

            received_methods
        });

        let tempdir = tempfile::tempdir().expect("tempdir");
        let queue = DeviceOutboundQueue::open(tempdir.path().join("device-runtime-queue.jsonl"))
            .await
            .expect("open queue");
        queue
            .push(QueuedEnvelope::metadata(serde_json::json!({
                "device_id": "device-1",
                "discovered_configs": []
            })))
            .await
            .expect("push metadata");
        queue
            .push(QueuedEnvelope::syslog_batch(serde_json::json!({
                "device_id": "device-1",
                "events": [{"message": "first"}]
            })))
            .await
            .expect("push syslog");
        queue
            .push(QueuedEnvelope::status(serde_json::json!({
                "device_id": "device-1",
                "connected": true
            })))
            .await
            .expect("push status");

        let client = WsClient::new(
            &format!("http://{addr}"),
            "device-1",
            tempdir.path().join("device-token"),
        )
        .expect("client");
        client.flush_queue_once(&queue).await.expect("flush");

        let remaining = queue.drain_batch(10).await.expect("remaining");
        assert!(remaining.is_empty(), "queue should be acked");

        let methods = server.await.expect("server task");
        assert_eq!(
            methods,
            vec![
                "initialize".to_string(),
                "fleet/metadata.push".to_string(),
                "fleet/log.event".to_string(),
                "fleet/status.push".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn flush_queue_once_drains_into_real_fleet_websocket_handler() {
        let store = Arc::new(crate::device::store::DeviceFleetStore::default());
        let enrollment_store = Arc::new(
            crate::device::enrollment::store::EnrollmentStore::open(
                std::env::temp_dir().join(format!("lab-ws-client-{}.json", uuid::Uuid::new_v4())),
            )
            .await
            .expect("open enrollment store"),
        );
        enrollment_store
            .record_pending(crate::device::enrollment::store::EnrollmentAttempt {
                device_id: "device-1".to_string(),
                token: "token".to_string(),
                tailnet_identity: crate::device::enrollment::store::TailnetIdentity {
                    node_key: "node-key".to_string(),
                    login_name: "user@example.com".to_string(),
                    hostname: "device-1".to_string(),
                },
                client_version: "0.7.3".to_string(),
                metadata: None,
            })
            .await
            .expect("record pending");
        enrollment_store.approve("device-1", None).await.expect("approve");
        let state = AppState::new()
            .with_device_store(store.clone())
            .with_enrollment_store(enrollment_store);
        let app = Router::new()
            .route("/v1/fleet/ws", get(fleet::websocket_upgrade))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        let tempdir = tempfile::tempdir().expect("tempdir");
        let queue = DeviceOutboundQueue::open(tempdir.path().join("device-runtime-queue.jsonl"))
            .await
            .expect("open queue");
        queue
            .push(QueuedEnvelope::metadata(serde_json::json!({
                "device_id": "device-1",
                "discovered_configs": []
            })))
            .await
            .expect("push metadata");
        queue
            .push(QueuedEnvelope::syslog_batch(serde_json::json!({
                "device_id": "device-1",
                "events": [{
                    "device_id": "device-1",
                    "source": "syslog",
                    "timestamp_unix_ms": 1234,
                    "level": "info",
                    "message": "first",
                    "fields": {}
                }]
            })))
            .await
            .expect("push syslog");
        queue
            .push(QueuedEnvelope::status(serde_json::json!({
                "device_id": "device-1",
                "connected": true,
                "cpu_percent": 12.5,
                "memory_used_bytes": 1024,
                "storage_used_bytes": 2048,
                "os": "linux",
                "ips": ["100.64.0.1"]
            })))
            .await
            .expect("push status");

        let client = WsClient::new(
            &format!("http://{addr}"),
            "device-1",
            tempdir.path().join("device-token"),
        )
        .expect("client");
        tokio::fs::write(tempdir.path().join("device-token"), "token")
            .await
            .expect("write token");
        client.flush_queue_once(&queue).await.expect("flush");

        let remaining = queue.drain_batch(10).await.expect("remaining");
        assert!(remaining.is_empty(), "queue should be acked");

        let snapshot = store.device("device-1").await.expect("snapshot");
        assert!(!snapshot.connected);
        assert_eq!(
            snapshot
                .metadata
                .as_ref()
                .map(|metadata| metadata.discovered_configs.len()),
            Some(0)
        );
        assert_eq!(snapshot.logs.len(), 1);
        assert_eq!(snapshot.logs[0].message, "first");
        assert_eq!(
            snapshot.status.as_ref().map(|status| status.connected),
            Some(false)
        );
        assert_eq!(
            snapshot.status.as_ref().and_then(|status| status.os.as_deref()),
            Some("linux")
        );

        server.abort();
    }

    #[tokio::test]
    async fn approved_device_keeps_socket_open_for_multiple_messages() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut socket = accept_async(stream).await.expect("accept websocket");
            let mut methods = Vec::new();

            while let Some(message) = socket.next().await {
                let text = match message.expect("message") {
                    Message::Text(text) => text.to_string(),
                    Message::Close(_) => break,
                    other => panic!("unexpected websocket message: {other:?}"),
                };
                let payload: serde_json::Value = serde_json::from_str(&text).expect("json");
                methods.push(
                    payload["method"]
                        .as_str()
                        .expect("method")
                        .to_string(),
                );
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": payload["id"],
                    "result": {},
                });
                socket
                    .send(Message::Text(response.to_string().into()))
                    .await
                    .expect("send");
                if methods.len() >= 3 {
                    break;
                }
            }
            methods
        });

        let tempdir = tempfile::tempdir().expect("tempdir");
        let queue = DeviceOutboundQueue::open(tempdir.path().join("device-runtime-queue.jsonl"))
            .await
            .expect("open queue");
        queue
            .push(QueuedEnvelope::metadata(json!({
                "device_id": "device-1",
                "discovered_configs": []
            })))
            .await
            .expect("metadata");
        queue
            .push(QueuedEnvelope::syslog_batch(json!({
                "device_id": "device-1",
                "events": [{"message": "first"}]
            })))
            .await
            .expect("log");
        tokio::fs::write(tempdir.path().join("device-token"), "token")
            .await
            .expect("write token");

        let client = WsClient::new(
            &format!("http://{addr}"),
            "device-1",
            tempdir.path().join("device-token"),
        )
        .expect("client");
        let queue = Arc::new(queue);
        let run = tokio::spawn({
            let queue = queue.clone();
            let client = client.clone();
            async move {
                tokio::time::timeout(Duration::from_secs(2), client.connect_and_run_session(&queue))
                    .await
                    .ok();
            }
        });

        let methods = server.await.expect("server");
        assert_eq!(
            methods,
            vec![
                "initialize".to_string(),
                "fleet/metadata.push".to_string(),
                "fleet/log.event".to_string()
            ]
        );

        run.abort();
    }
}
