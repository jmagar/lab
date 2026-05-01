//! Uptime Kuma client.

use std::sync::Arc;
use std::time::Duration;

use rust_socketio::Payload;
use rust_socketio::asynchronous::{Client, ClientBuilder};
use serde_json::{Value, json};
use tokio::sync::{Mutex, oneshot};
use tokio::time::timeout;

use crate::core::error::ApiError;
use crate::core::{Auth, HttpClient};

use super::error::UptimeKumaError;
use super::types::UptimeKumaResponse;

const ACK_TIMEOUT: Duration = Duration::from_secs(15);

/// Client for an Uptime Kuma instance.
#[derive(Clone)]
pub struct UptimeKumaClient {
    http: HttpClient,
    username: Option<String>,
    password: Option<String>,
    socket: Arc<Mutex<Option<Client>>>,
    jwt: Arc<Mutex<Option<String>>>,
}

impl UptimeKumaClient {
    /// Build a client against `base_url`.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, UptimeKumaError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
            username: None,
            password: None,
            socket: Arc::new(Mutex::new(None)),
            jwt: Arc::new(Mutex::new(None)),
        })
    }

    /// Build a client with web-account credentials for the Socket.IO API.
    pub fn with_login(
        base_url: &str,
        username: String,
        password: String,
    ) -> Result<Self, UptimeKumaError> {
        if username.trim().is_empty() || password.is_empty() {
            return Err(ApiError::Internal(
                "UPTIME_KUMA_USERNAME and UPTIME_KUMA_PASSWORD must not be empty".into(),
            )
            .into());
        }
        Ok(Self {
            http: HttpClient::new(base_url, Auth::None)?,
            username: Some(username),
            password: Some(password),
            socket: Arc::new(Mutex::new(None)),
            jwt: Arc::new(Mutex::new(None)),
        })
    }

    /// Probe the web UI root. Uptime Kuma does not expose a stable REST health endpoint.
    pub async fn health(&self) -> Result<(), UptimeKumaError> {
        self.http.get_text("/").await?;
        Ok(())
    }

    /// Return the supported integration contract.
    pub fn contract_status(&self) -> UptimeKumaResponse {
        UptimeKumaResponse {
            value: json!({
                "service": "uptime-kuma",
                "transport": "socket.io",
                "base_url": self.http.base_url(),
                "credentials_configured": self.username.is_some() && self.password.is_some(),
                "live_socket_reads": self.username.is_some() && self.password.is_some(),
                "read_actions": [
                    "monitor.list",
                    "monitor.get",
                    "heartbeat.list",
                    "status.summary",
                    "notification.list"
                ],
                "write_actions": [],
                "note": "Live reads use Uptime Kuma's internal Socket.IO API. Mutating monitor and notification actions are intentionally deferred."
            }),
        }
    }

    pub async fn monitor_list(&self) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("getMonitorList", Value::Null).await
    }

    pub async fn monitor_get(&self, id: i64) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("getMonitor", json!(id)).await
    }

    pub async fn heartbeat_list(
        &self,
        monitor_id: Option<i64>,
        hours: u32,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        if !(1..=168).contains(&hours) {
            return Err(UptimeKumaError::InvalidParam(
                "hours must be between 1 and 168".into(),
            ));
        }
        let payload = match monitor_id {
            Some(id) => json!({ "monitorID": id, "hours": hours }),
            None => json!({ "hours": hours }),
        };
        self.emit_value("getMonitorBeats", payload).await
    }

    pub async fn status_summary(&self) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("getStats", Value::Null).await
    }

    pub async fn notification_list(&self) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("getNotificationList", Value::Null).await
    }

    async fn emit_value(
        &self,
        event: &'static str,
        payload: Value,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        let socket = self.ensure_socket().await?;
        let ack = emit_ack(socket, event, payload).await?;
        Ok(UptimeKumaResponse {
            value: redact_sensitive(ack),
        })
    }

    async fn ensure_socket(&self) -> Result<Client, UptimeKumaError> {
        let mut guard = self.socket.lock().await;
        if let Some(socket) = guard.as_ref() {
            return Ok(socket.clone());
        }
        let socket = ClientBuilder::new(self.http.base_url())
            .reconnect(true)
            .reconnect_on_disconnect(true)
            .reconnect_delay(1000, 30_000)
            .connect()
            .await
            .map_err(|e| UptimeKumaError::Socket(e.to_string()))?;
        self.login(&socket).await?;
        *guard = Some(socket.clone());
        Ok(socket)
    }

    async fn login(&self, socket: &Client) -> Result<(), UptimeKumaError> {
        if let Some(token) = self.jwt.lock().await.clone() {
            if self.login_by_token(socket, &token).await.is_ok() {
                return Ok(());
            }
            *self.jwt.lock().await = None;
        }

        let (Some(username), Some(password)) = (&self.username, &self.password) else {
            return Err(UptimeKumaError::Auth(
                "UPTIME_KUMA_USERNAME and UPTIME_KUMA_PASSWORD are required for live Socket.IO reads"
                    .into(),
            ));
        };
        let value = emit_ack(
            socket.clone(),
            "login",
            json!({ "username": username, "password": password }),
        )
        .await?;
        validate_login_ack(&value)?;
        if let Some(token) = value.get("token").and_then(Value::as_str) {
            *self.jwt.lock().await = Some(token.to_string());
        }
        Ok(())
    }

    async fn login_by_token(&self, socket: &Client, token: &str) -> Result<(), UptimeKumaError> {
        let value = emit_ack(socket.clone(), "loginByToken", json!(token)).await?;
        validate_login_ack(&value)
    }
}

async fn emit_ack(
    socket: Client,
    event: &'static str,
    payload: Value,
) -> Result<Value, UptimeKumaError> {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let tx_cb = tx.clone();
    socket
        .emit_with_ack(event, payload, ACK_TIMEOUT, move |payload, _| {
            let tx_cb = tx_cb.clone();
            Box::pin(async move {
                if let Some(sender) = tx_cb.lock().await.take() {
                    let _sent = sender.send(payload_to_value(payload));
                }
            })
        })
        .await
        .map_err(|e| UptimeKumaError::Socket(e.to_string()))?;

    timeout(ACK_TIMEOUT, rx)
        .await
        .map_err(|_| UptimeKumaError::Timeout(ACK_TIMEOUT.as_secs()))?
        .map_err(|_| UptimeKumaError::Socket("ack callback dropped".into()))
}

fn payload_to_value(payload: Payload) -> Value {
    match payload {
        Payload::Text(mut values) => {
            if values.len() == 1 {
                values.remove(0)
            } else {
                Value::Array(values)
            }
        }
        Payload::Binary(bytes) => Value::Array(bytes.iter().map(|byte| json!(byte)).collect()),
        #[allow(deprecated)]
        Payload::String(value) => serde_json::from_str(&value).unwrap_or(Value::String(value)),
    }
}

fn validate_login_ack(value: &Value) -> Result<(), UptimeKumaError> {
    let ok = value.get("ok").and_then(Value::as_bool).unwrap_or(false);
    if ok {
        Ok(())
    } else {
        Err(UptimeKumaError::Auth(
            value
                .get("msg")
                .and_then(Value::as_str)
                .unwrap_or("login rejected")
                .to_string(),
        ))
    }
}

fn redact_sensitive(mut value: Value) -> Value {
    fn strip(value: &mut Value) {
        match value {
            Value::Object(map) => {
                for key in map.keys().cloned().collect::<Vec<_>>() {
                    let lower = key.to_ascii_lowercase();
                    if lower.contains("password")
                        || lower.contains("token")
                        || lower.contains("secret")
                        || lower.contains("auth")
                        || lower.contains("webhook")
                    {
                        map.insert(key, Value::String("[redacted]".into()));
                    }
                }
                for value in map.values_mut() {
                    strip(value);
                }
            }
            Value::Array(values) => {
                for value in values {
                    strip(value);
                }
            }
            _ => {}
        }
    }
    strip(&mut value);
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_ack_rejects_failed_login_message() {
        let err = validate_login_ack(&json!({"ok": false, "msg": "bad login"})).unwrap_err();
        assert!(matches!(err, UptimeKumaError::Auth(_)));
    }

    #[test]
    fn redacts_sensitive_response_fields() {
        let value = redact_sensitive(json!({
            "name": "monitor",
            "headers": { "Authorization": "Bearer abc" },
            "notification": { "webhookURL": "https://example.test/hook" },
            "password": "pw"
        }));
        assert_eq!(value["password"], "[redacted]");
        assert_eq!(value["headers"]["Authorization"], "[redacted]");
        assert_eq!(value["notification"]["webhookURL"], "[redacted]");
        assert_eq!(value["name"], "monitor");
    }
}
