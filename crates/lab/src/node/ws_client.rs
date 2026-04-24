use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

use crate::node::install::{
    AgentInstallParams, InstallComponentParams, InstallScope, handle_agent_install,
    handle_install_component,
};
use crate::node::queue::{NodeOutboundQueue, QueuedEnvelope};
use crate::node::token;
use crate::dispatch::upstream::transport::websocket::{jitter_delay, reprobe_backoff};

const FLUSH_BATCH_SIZE: usize = 100;
const IDLE_FLUSH_INTERVAL: Duration = Duration::from_secs(10);
const STATUS_INTERVAL: Duration = Duration::from_secs(30);
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;
const MAX_FRAME_SIZE: usize = 128 * 1024;

/// Size of the pending-response map and the progress forwarding channel.
const PENDING_CHANNEL_CAPACITY: usize = 64;

/// Maximum time to wait for a master response before giving up, removing the
/// pending entry, and surfacing an error. Prevents silent masters from
/// wedging the client and growing the pending map unbounded.
const REQUEST_RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);

/// Hard cap on pending in-flight request IDs. `send_and_await` rejects new
/// requests above this watermark so the HashMap cannot grow without bound.
const MAX_PENDING_INFLIGHT: usize = 256;

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
    node_id: String,
    token_path: PathBuf,
    connected: Arc<AtomicBool>,
}

impl WsClient {
    pub fn new(base_url: &str, node_id: impl Into<String>, token_path: impl AsRef<Path>) -> Result<Self> {
        let url = websocket_url_from_master_base(base_url)?;
        Ok(Self {
            url,
            node_id: node_id.into(),
            token_path: token_path.as_ref().to_path_buf(),
            connected: Arc::new(AtomicBool::new(false)),
        })
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }

    pub async fn run(self, queue: Arc<NodeOutboundQueue>) {
        let mut attempt = 0_u32;
        loop {
            match self.connect_and_run_session(&queue).await {
                Ok(()) => {
                    attempt = 0;
                }
                Err(error) => {
                    self.connected.store(false, Ordering::Relaxed);
                    attempt = attempt.saturating_add(1);
                    let delay = jitter_delay(reprobe_backoff(attempt), stable_seed(&self.node_id, attempt));
                    tracing::warn!(
                        node_id = %self.node_id,
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

    async fn connect_and_run_session(&self, queue: &NodeOutboundQueue) -> Result<()> {
        let token = token::load_or_create(&self.token_path).await?;
        let tailnet_identity = TailnetIdentity::discover(&self.node_id);
        let (socket, _) = self.open_websocket().await?;

        let initialize = build_initialize_request(&self.node_id, &token, &tailnet_identity);
        let (tx, rx) = mpsc::channel::<Message>(PENDING_CHANNEL_CAPACITY);
        tx.send(Message::Text(serde_json::to_string(&initialize)?.into()))
            .await
            .context("queue websocket initialize")?;

        // Pending response map: request id → oneshot sender.
        // The reader task resolves pending entries when it sees a JSON-RPC response.
        let pending: Arc<tokio::sync::Mutex<HashMap<i64, oneshot::Sender<String>>>> =
            Arc::new(tokio::sync::Mutex::new(HashMap::new()));

        // Split the raw socket into send + receive halves.
        let (sink, mut stream) = socket.split();

        // lab-kvhi.6 DECISION: spawn all three background tasks into a JoinSet so
        // that on session exit (success, error, or early-return via `?`) we can
        // call `abort_all()` and avoid leaking reader/writer/progress tasks
        // across reconnect attempts. A drop-guard ensures abort fires even when
        // the function returns via `?` from the main loop below.
        let mut session_tasks: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();

        // Writer task: drains `rx` → `sink`.
        session_tasks.spawn(async move {
            let mut sink = sink;
            let mut rx: mpsc::Receiver<Message> = rx;
            while let Some(msg) = rx.recv().await {
                if let Err(error) = sink.send(msg).await {
                    tracing::warn!(error = %error, "ws writer error");
                    break;
                }
            }
        });

        // Channel for progress notifications coming back from install handlers.
        // These are forwarded to `tx` as raw JSON text frames.
        let (progress_tx, mut progress_rx) = mpsc::channel::<String>(PENDING_CHANNEL_CAPACITY);

        // Reader + demux loop.
        let pending_clone = Arc::clone(&pending);
        let tx_clone = tx.clone();

        // Forward progress notifications to the write channel.
        {
            let tx_for_progress = tx.clone();
            session_tasks.spawn(async move {
                while let Some(notif) = progress_rx.recv().await {
                    tx_for_progress.send(Message::Text(notif.into())).await.ok();
                }
            });
        }

        // Await the initialize response via the main reader loop.
        // We use a oneshot to receive the init response while the reader loop is running.
        let (init_tx, init_rx) = oneshot::channel::<String>();
        let init_tx = Arc::new(tokio::sync::Mutex::new(Some(init_tx)));

        {
            let init_tx = Arc::clone(&init_tx);
            let pending = Arc::clone(&pending_clone);
            let tx = tx_clone.clone();
            let progress_tx = progress_tx.clone();
            let node_id = self.node_id.clone();
            session_tasks.spawn(async move {
                while let Some(message) = stream.next().await {
                    let text = match message {
                        Ok(Message::Text(t)) => t.to_string(),
                        Ok(Message::Ping(payload)) => {
                            if tx.send(Message::Pong(payload)).await.is_err() {
                                break;
                            }
                            continue;
                        }
                        Ok(Message::Pong(_) | Message::Frame(_)) => continue,
                        Ok(Message::Binary(_)) => {
                            tracing::warn!(node_id = %node_id, "ws binary frame ignored");
                            continue;
                        }
                        Ok(Message::Close(_)) | Err(_) => {
                            {
                                let mut guard = init_tx.lock().await;
                                guard.take();
                            }
                            let mut map = pending.lock().await;
                            map.clear();
                            break;
                        }
                    };

                    // Try to parse as JSON-RPC.
                    let parsed: Value = match serde_json::from_str(&text) {
                        Ok(v) => v,
                        Err(error) => {
                            tracing::warn!(node_id = %node_id, error = %error, "ws unparse-able frame");
                            continue;
                        }
                    };

                    // Dispatch: check whether the frame is a response (has `result`/`error`)
                    // or an inbound RPC request from master (has `method`).
                    if parsed.get("result").is_some() || parsed.get("error").is_some() {
                        if parsed.get("id").and_then(Value::as_i64) == Some(1) {
                            let mut guard = init_tx.lock().await;
                            if let Some(sender) = guard.take() {
                                drop(guard);
                                sender.send(text).ok();
                                continue;
                            }
                        }
                        // Response — resolve a pending oneshot.
                        if let Some(id) = parsed.get("id").and_then(Value::as_i64) {
                            let mut map = pending.lock().await;
                            if let Some(sender) = map.remove(&id) {
                                sender.send(text).ok();
                            }
                        }
                    } else if parsed.get("method").is_some() {
                        // Inbound RPC from master — dispatch it and send back the response.
                        let response = dispatch_inbound_rpc(parsed, &progress_tx).await;
                        let encoded = serde_json::to_string(&response)
                            .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"serialize error"}}"#.to_string());
                        if tx.send(Message::Text(encoded.into())).await.is_err() {
                            break;
                        }
                    }
                }
            });
        }

        // lab-kvhi.6 FACT: JoinSet::drop already aborts tasks, but we wrap the
        // remainder of the session in an inner async block so that `?` early
        // returns still fall through to the explicit `abort_all` + drain below.
        // This guarantees tasks are cancelled and the socket halves released
        // before `run()` proceeds to the next reconnect attempt.
        let session_result: Result<()> = async {
            let init_response =
                init_rx.await.context("initialize response channel closed")?;
            validate_success_response(&init_response, 1)?;
            self.connected.store(true, Ordering::Relaxed);

            let mut next_id = 2_i64;
            let mut status_deadline = tokio::time::Instant::now() + STATUS_INTERVAL;

            loop {
                let ack_count = self
                    .flush_queue_batch_async(queue, &tx, &pending_clone, &mut next_id)
                    .await?;
                let now = tokio::time::Instant::now();
                if now >= status_deadline {
                    self.send_status_update_async(&tx, &pending_clone, next_id).await?;
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
        .await;

        // Drop the outbound sender so the writer task exits, then abort the
        // reader/progress tasks and drain the set so the handles complete
        // before this function returns.
        drop(tx);
        drop(tx_clone);
        drop(progress_tx);
        session_tasks.abort_all();
        while session_tasks.join_next().await.is_some() {}

        session_result
    }

    /// Send a request over the channel and wait for the corresponding response
    /// via a oneshot in the pending map.
    ///
    /// Bounded by `REQUEST_RESPONSE_TIMEOUT` (removes the pending entry on
    /// timeout) and by `MAX_PENDING_INFLIGHT` (rejects new requests when the
    /// map is already full) so a silent master cannot wedge the client or
    /// leak pending senders.
    async fn send_and_await(
        tx: &mpsc::Sender<Message>,
        pending: &tokio::sync::Mutex<HashMap<i64, oneshot::Sender<String>>>,
        request: &Value,
        request_id: i64,
    ) -> Result<String> {
        let (resp_tx, resp_rx) = oneshot::channel::<String>();
        {
            let mut map = pending.lock().await;
            if map.len() >= MAX_PENDING_INFLIGHT {
                return Err(anyhow!(
                    "node ws_client pending map full ({} inflight); refusing request_id={}",
                    map.len(),
                    request_id
                ));
            }
            map.insert(request_id, resp_tx);
        }
        let send_result = tx
            .send(Message::Text(serde_json::to_string(request)?.into()))
            .await
            .context("send websocket request");
        if let Err(error) = send_result {
            pending.lock().await.remove(&request_id);
            return Err(error);
        }
        match tokio::time::timeout(REQUEST_RESPONSE_TIMEOUT, resp_rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                pending.lock().await.remove(&request_id);
                Err(anyhow!("response channel closed before reply"))
            }
            Err(_) => {
                pending.lock().await.remove(&request_id);
                Err(anyhow!(
                    "response timeout after {:?} for request_id={}",
                    REQUEST_RESPONSE_TIMEOUT,
                    request_id
                ))
            }
        }
    }

    async fn flush_queue_batch_async(
        &self,
        queue: &NodeOutboundQueue,
        tx: &mpsc::Sender<Message>,
        pending: &tokio::sync::Mutex<HashMap<i64, oneshot::Sender<String>>>,
        next_id: &mut i64,
    ) -> Result<usize> {
        let drained = queue.drain_batch(FLUSH_BATCH_SIZE).await?;
        let mut ack_count = 0usize;
        for envelope in drained {
            let result: Result<()> = async {
                let request = queue_envelope_to_request(&envelope, *next_id)?;
                let response = Self::send_and_await(tx, pending, &request, *next_id).await?;
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

    async fn send_status_update_async(
        &self,
        tx: &mpsc::Sender<Message>,
        pending: &tokio::sync::Mutex<HashMap<i64, oneshot::Sender<String>>>,
        request_id: i64,
    ) -> Result<()> {
        let request = json!({
            "jsonrpc": "2.0",
            "id": request_id,
            "method": "nodes/status.push",
            "params": {
                "node_id": self.node_id,
                "connected": true,
                "cpu_percent": Value::Null,
                "memory_used_bytes": Value::Null,
                "storage_used_bytes": Value::Null,
                "os": std::env::consts::OS,
                "ips": [],
            }
        });
        let response = Self::send_and_await(tx, pending, &request, request_id).await?;
        validate_success_response(&response, request_id)?;
        Ok(())
    }

    async fn open_websocket(
        &self,
    ) -> Result<(
        tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
        tokio_tungstenite::tungstenite::handshake::client::Response,
    )> {
        let request = self
            .url
            .to_string()
            .into_client_request()
            .map_err(|error| anyhow!("build websocket request: {error}"))?;
        let mut websocket_config = WebSocketConfig::default();
        websocket_config.max_message_size = Some(MAX_MESSAGE_SIZE);
        websocket_config.max_frame_size = Some(MAX_FRAME_SIZE);
        websocket_config.accept_unmasked_frames = false;
        connect_async_with_config(request, Some(websocket_config), false)
            .await
            .map_err(|error| anyhow!("connect websocket: {error}"))
    }

    #[cfg(test)]
    async fn flush_queue_once(&self, queue: &NodeOutboundQueue) -> Result<()> {
        let token = token::load_or_create(&self.token_path).await?;
        let tailnet_identity = TailnetIdentity::discover(&self.node_id);
        let (socket, _) = self.open_websocket().await?;

        let (tx, rx) = mpsc::channel::<Message>(PENDING_CHANNEL_CAPACITY);
        let pending: Arc<tokio::sync::Mutex<HashMap<i64, oneshot::Sender<String>>>> =
            Arc::new(tokio::sync::Mutex::new(HashMap::new()));

        let (sink, mut stream) = socket.split();

        let write_task = tokio::spawn(async move {
            let mut sink = sink;
            let mut rx: mpsc::Receiver<Message> = rx;
            while let Some(msg) = rx.recv().await {
                if sink.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Reader task: forward responses to pending map.
        let (init_tx, init_rx) = oneshot::channel::<String>();
        let init_tx = Arc::new(tokio::sync::Mutex::new(Some(init_tx)));
        let pending_clone = Arc::clone(&pending);
        let reader_task = tokio::spawn(async move {
            while let Some(message) = stream.next().await {
                let text = match message {
                    Ok(Message::Text(t)) => t.to_string(),
                    Ok(Message::Ping(payload)) => {
                        // No tx available here; just ignore pings in test helper.
                        drop(payload);
                        continue;
                    }
                    Ok(_) | Err(_) => break,
                };
                let mut guard = init_tx.lock().await;
                if let Some(sender) = guard.take() {
                    drop(guard);
                    sender.send(text).ok();
                    continue;
                }
                drop(guard);
                let parsed: Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                if let Some(id) = parsed.get("id").and_then(Value::as_i64) {
                    let mut map = pending_clone.lock().await;
                    if let Some(s) = map.remove(&id) {
                        s.send(text).ok();
                    }
                }
            }
        });

        // Send initialize.
        let initialize = build_initialize_request(&self.node_id, &token, &tailnet_identity);
        tx.send(Message::Text(serde_json::to_string(&initialize)?.into()))
            .await
            .context("send websocket initialize")?;

        let init_response = init_rx.await.context("init response channel closed")?;
        validate_success_response(&init_response, 1)?;

        let mut next_id = 2_i64;
        self.flush_queue_batch_async(queue, &tx, &pending, &mut next_id).await?;

        // Close the socket.
        tx.send(Message::Close(None)).await.ok();
        drop(tx);
        drop(write_task.await);
        reader_task.abort();
        Ok(())
    }
}

/// Dispatch an inbound RPC request from master to the appropriate handler.
///
/// Returns a JSON-RPC 2.0 response value (with `result` or `error`).
/// Progress notifications are sent via `progress_tx` as JSON-encoded strings.
async fn dispatch_inbound_rpc(frame: Value, progress_tx: &mpsc::Sender<String>) -> Value {
    let id = frame.get("id").cloned().unwrap_or(Value::Null);
    let method = match frame.get("method").and_then(Value::as_str) {
        Some(m) => m.to_string(),
        None => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32600, "message": "missing `method` field" }
            });
        }
    };
    let params = frame.get("params").cloned().unwrap_or(Value::Null);

    match method.as_str() {
        "marketplace.install_component" => {
            let install_params: InstallComponentParams =
                match serde_json::from_value(params.clone()) {
                    Ok(p) => p,
                    Err(error) => {
                        return json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32602,
                                "message": format!("invalid marketplace.install_component params: {error}")
                            }
                        });
                    }
                };

            // `component_files` is expected to be embedded in params as
            // `{ "files": [{ "path": "...", "content": "<base64 or utf8>" }] }`.
            // For this initial implementation we accept a `files` array where
            // each entry has `path` (string) and `content` (string, UTF-8).
            let raw_files = params
                .get("files")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let component_files: Vec<(String, Vec<u8>)> = raw_files
                .into_iter()
                .filter_map(|entry| {
                    let path = entry.get("path")?.as_str()?.to_string();
                    let content = entry.get("content")?.as_str()?.as_bytes().to_vec();
                    Some((path, content))
                })
                .collect();

            match handle_install_component(install_params, component_files, id.clone(), progress_tx)
                .await
            {
                Ok(result) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result,
                }),
                Err(error) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": format!("marketplace.install_component failed: {error}")
                    }
                }),
            }
        }

        "agent.install" => {
            #[derive(serde::Deserialize)]
            struct AgentInstallEnvelope {
                #[serde(flatten)]
                params: AgentInstallParams,
                #[serde(default)]
                scope: Option<InstallScope>,
                project_path: Option<String>,
            }

            let envelope: AgentInstallEnvelope = match serde_json::from_value(params) {
                Ok(e) => e,
                Err(error) => {
                    return json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32602,
                            "message": format!("invalid agent.install params: {error}")
                        }
                    });
                }
            };

            let scope = envelope.scope.unwrap_or(InstallScope::Global);
            let project_path = envelope.project_path.as_deref();

            match handle_agent_install(envelope.params, scope, project_path, id.clone(), progress_tx)
                .await
            {
                Ok(result) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result,
                }),
                Err(error) => json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32000,
                        "message": format!("agent.install failed: {error}")
                    }
                }),
            }
        }

        other => {
            json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("unknown RPC method `{other}`"),
                    "data": { "kind": "unknown_action" }
                }
            })
        }
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
    url.set_path("/v1/nodes/ws");
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

pub fn build_initialize_request(
    node_id: &str,
    device_token: &str,
    tailnet_identity: &TailnetIdentity,
) -> Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "lab-node",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "_meta": {
                "lab.node_id": node_id,
                "lab.device_token": device_token,
                "lab.tailnet_identity": tailnet_identity,
            }
        }
    })
}

pub fn queue_envelope_to_request(
    envelope: &QueuedEnvelope,
    id: i64,
) -> Result<Value> {
    let method = match envelope.kind.as_str() {
        "syslog_batch" => "nodes/log.event",
        "status" => "nodes/status.push",
        "metadata" => "nodes/metadata.push",
        other => return Err(anyhow!("unsupported queued envelope kind `{other}`")),
    };
    Ok(serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": envelope.payload,
    }))
}

fn validate_success_response(payload: &str, expected_id: i64) -> Result<()> {
    let value: Value = serde_json::from_str(payload).context("decode websocket response")?;
    let response_id = value
        .get("id")
        .and_then(Value::as_i64)
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
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        return Err(anyhow!("websocket request failed ({kind}): {error}"));
    }
    Ok(())
}

fn stable_seed(node_id: &str, attempt: u32) -> u64 {
    let mut hash = 1_469_598_103_934_665_603_u64;
    for byte in node_id.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(1_099_511_628_211);
    }
    hash ^ u64::from(attempt)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::api::{nodes::fleet, state::AppState};
    use crate::node::queue::NodeOutboundQueue;
    use axum::{Router, routing::get};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::Message;

    #[test]
    fn websocket_url_rewrites_http_base() {
        let url = websocket_url_from_master_base("http://master:8765").expect("url");
        assert_eq!(url.as_str(), "ws://master:8765/v1/nodes/ws");
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
        assert_eq!(request["params"]["_meta"]["lab.node_id"], "device-1");
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
        assert_eq!(syslog["method"], "nodes/log.event");

        let status = queue_envelope_to_request(
            &QueuedEnvelope::status(serde_json::json!({"connected": true})),
            3,
        )
        .expect("status");
        assert_eq!(status["method"], "nodes/status.push");

        let metadata = queue_envelope_to_request(
            &QueuedEnvelope::metadata(serde_json::json!({"node_id": "device-1"})),
            4,
        )
        .expect("metadata");
        assert_eq!(metadata["method"], "nodes/metadata.push");
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
                let payload: Value =
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
        let queue = NodeOutboundQueue::open(tempdir.path().join("node-runtime-queue.jsonl"))
            .await
            .expect("open queue");
        queue
            .push(QueuedEnvelope::metadata(serde_json::json!({
                "node_id": "device-1",
                "discovered_configs": []
            })))
            .await
            .expect("push metadata");
        queue
            .push(QueuedEnvelope::syslog_batch(serde_json::json!({
                "node_id": "device-1",
                "events": [{"message": "first"}]
            })))
            .await
            .expect("push syslog");
        queue
            .push(QueuedEnvelope::status(serde_json::json!({
                "node_id": "device-1",
                "connected": true
            })))
            .await
            .expect("push status");

        let client = WsClient::new(
            &format!("http://{addr}"),
            "device-1",
            tempdir.path().join("node-token"),
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
                "nodes/metadata.push".to_string(),
                "nodes/log.event".to_string(),
                "nodes/status.push".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn flush_queue_once_drains_into_real_fleet_websocket_handler() {
        let store = Arc::new(crate::node::store::NodeStore::default());
        let enrollment_store = Arc::new(
            crate::node::enrollment::store::EnrollmentStore::open(
                std::env::temp_dir().join(format!("lab-ws-client-{}.json", uuid::Uuid::new_v4())),
            )
            .await
            .expect("open enrollment store"),
        );
        enrollment_store
            .record_pending(crate::node::enrollment::store::EnrollmentAttempt {
                node_id: "device-1".to_string(),
                token: "token".to_string(),
                tailnet_identity: crate::node::enrollment::store::TailnetIdentity {
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
            .with_node_store(store.clone())
            .with_enrollment_store(enrollment_store);
        let app = Router::new()
            .route("/v1/nodes/ws", get(fleet::websocket_upgrade))
            .with_state(state);

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind listener");
        let addr = listener.local_addr().expect("listener addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        let tempdir = tempfile::tempdir().expect("tempdir");
        let queue = NodeOutboundQueue::open(tempdir.path().join("node-runtime-queue.jsonl"))
            .await
            .expect("open queue");
        queue
            .push(QueuedEnvelope::metadata(serde_json::json!({
                "node_id": "device-1",
                "discovered_configs": []
            })))
            .await
            .expect("push metadata");
        queue
            .push(QueuedEnvelope::syslog_batch(serde_json::json!({
                "node_id": "device-1",
                "events": [{
                    "node_id": "device-1",
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
                "node_id": "device-1",
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
            tempdir.path().join("node-token"),
        )
        .expect("client");
        tokio::fs::write(tempdir.path().join("node-token"), "token")
            .await
            .expect("write token");
        client.flush_queue_once(&queue).await.expect("flush");

        let remaining = queue.drain_batch(10).await.expect("remaining");
        assert!(remaining.is_empty(), "queue should be acked");

        let snapshot = store.node("device-1").await.expect("snapshot");
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
                let payload: Value = serde_json::from_str(&text).expect("json");
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
        let queue = NodeOutboundQueue::open(tempdir.path().join("node-runtime-queue.jsonl"))
            .await
            .expect("open queue");
        queue
            .push(QueuedEnvelope::metadata(json!({
                "node_id": "device-1",
                "discovered_configs": []
            })))
            .await
            .expect("metadata");
        queue
            .push(QueuedEnvelope::syslog_batch(json!({
                "node_id": "device-1",
                "events": [{"message": "first"}]
            })))
            .await
            .expect("log");
        tokio::fs::write(tempdir.path().join("node-token"), "token")
            .await
            .expect("write token");

        let client = WsClient::new(
            &format!("http://{addr}"),
            "device-1",
            tempdir.path().join("node-token"),
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
                "nodes/metadata.push".to_string(),
                "nodes/log.event".to_string()
            ]
        );

        run.abort();
    }

    #[tokio::test]
    async fn dispatch_inbound_rpc_unknown_method_returns_error() {
        let (progress_tx, _rx) = mpsc::channel(8);
        let frame = json!({
            "jsonrpc": "2.0",
            "id": 42,
            "method": "unknown.method",
            "params": {}
        });
        let response = dispatch_inbound_rpc(frame, &progress_tx).await;
        assert_eq!(response["id"], 42);
        assert!(response.get("error").is_some());
        assert_eq!(response["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn dispatch_inbound_rpc_marketplace_install_component_path_traversal() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let (progress_tx, _rx) = mpsc::channel(8);
        let frame = json!({
            "jsonrpc": "2.0",
            "id": 10,
            "method": "marketplace.install_component",
            "params": {
                "plugin_id": "evil@marketplace",
                "components": ["../etc/passwd"],
                "scope": "project",
                "project_path": tempdir.path().to_str().unwrap(),
                "files": [
                    { "path": "../etc/passwd", "content": "evil content" }
                ]
            }
        });
        let response = dispatch_inbound_rpc(frame, &progress_tx).await;
        // Should succeed at the RPC level but report errors in the result.
        assert_eq!(response["id"], 10);
        // Either we get an error at RPC level or the result has errors field.
        let has_error = response.get("error").is_some()
            || response["result"]["errors"]
                .as_array()
                .map(|e| !e.is_empty())
                .unwrap_or(false);
        assert!(has_error, "expected path traversal to be rejected: {response}");
    }

    #[tokio::test]
    async fn dispatch_inbound_rpc_agent_install_unknown_method_variation() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let (progress_tx, _rx) = mpsc::channel(8);
        let frame = json!({
            "jsonrpc": "2.0",
            "id": 20,
            "method": "agent.install",
            "params": {
                "agent_id": "test-agent",
                "distribution": {
                    "type": "npx",
                    "package": "@anthropic/test-agent",
                    "version": "1.0.0"
                },
                "scope": "project",
                "project_path": tempdir.path().to_str().unwrap()
            }
        });
        let response = dispatch_inbound_rpc(frame, &progress_tx).await;
        assert_eq!(response["id"], 20);
        assert!(response.get("result").is_some(), "expected success: {response}");
        assert_eq!(response["result"]["written"].as_array().map(|a| a.len()), Some(1));
    }
}
