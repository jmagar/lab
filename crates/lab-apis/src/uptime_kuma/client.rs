//! Uptime Kuma client.

use std::sync::Arc;
use std::time::Duration;

use rust_socketio::Payload;
use rust_socketio::asynchronous::{Client, ClientBuilder};
use serde_json::{Value, json};
use tokio::sync::{Mutex, oneshot};
use tokio::time::timeout;
use url::Url;

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
                "write_actions": [
                    "monitor.create",
                    "monitor.update",
                    "monitor.delete",
                    "monitor.pause",
                    "monitor.resume",
                    "notification.test",
                    "notification.add"
                ],
                "note": "Live actions use Uptime Kuma's internal Socket.IO API. Mutating monitor and notification actions are exposed as destructive actions by the Lab surfaces."
            }),
        }
    }

    pub async fn monitor_list(&self) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("getMonitorList", Value::Null).await
    }

    pub async fn monitor_get(&self, id: i64) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("getMonitor", json!(id)).await
    }

    pub async fn monitor_create(
        &self,
        config: Value,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        validate_object_config(&config, "config")?;
        self.emit_value("add", config).await
    }

    pub async fn monitor_update(
        &self,
        id: i64,
        config: Value,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        validate_object_config(&config, "config")?;
        self.emit_values("editMonitor", vec![json!(id), config])
            .await
    }

    pub async fn monitor_delete(&self, id: i64) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("deleteMonitor", json!(id)).await
    }

    pub async fn monitor_pause(&self, id: i64) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("pauseMonitor", json!(id)).await
    }

    pub async fn monitor_resume(&self, id: i64) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("resumeMonitor", json!(id)).await
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

    pub async fn notification_test(
        &self,
        notification_id: i64,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_value("testNotification", json!(notification_id))
            .await
    }

    pub async fn notification_add(
        &self,
        config: Value,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        validate_notification_config(&config)?;
        self.emit_value("addNotification", config).await
    }

    async fn emit_value(
        &self,
        event: &'static str,
        payload: Value,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_payload(event, payload).await
    }

    async fn emit_values(
        &self,
        event: &'static str,
        payload: Vec<Value>,
    ) -> Result<UptimeKumaResponse, UptimeKumaError> {
        self.emit_payload(event, payload).await
    }

    async fn emit_payload(
        &self,
        event: &'static str,
        payload: impl Into<Payload>,
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
    payload: impl Into<Payload>,
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

fn validate_object_config(value: &Value, param: &str) -> Result<(), UptimeKumaError> {
    if value.as_object().is_none() {
        return Err(UptimeKumaError::InvalidParam(format!(
            "{param} must be a JSON object"
        )));
    }
    Ok(())
}

fn validate_notification_config(value: &Value) -> Result<(), UptimeKumaError> {
    validate_object_config(value, "config")?;
    reject_template_markers(value)?;
    reject_unsafe_notification_urls(value)?;
    Ok(())
}

fn reject_template_markers(value: &Value) -> Result<(), UptimeKumaError> {
    match value {
        Value::String(raw) => {
            if raw.contains("{{") || raw.contains("${") || raw.contains("<%") || raw.contains("{%")
            {
                return Err(UptimeKumaError::InvalidParam(
                    "notification config must not contain template markers".into(),
                ));
            }
        }
        Value::Array(values) => {
            for value in values {
                reject_template_markers(value)?;
            }
        }
        Value::Object(map) => {
            for value in map.values() {
                reject_template_markers(value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn reject_unsafe_notification_urls(value: &Value) -> Result<(), UptimeKumaError> {
    fn visit(key: Option<&str>, value: &Value) -> Result<(), UptimeKumaError> {
        match value {
            Value::Object(map) => {
                for (key, value) in map {
                    visit(Some(key), value)?;
                }
            }
            Value::Array(values) => {
                for value in values {
                    visit(key, value)?;
                }
            }
            Value::String(raw) => {
                let lower_key = key.unwrap_or_default().to_ascii_lowercase();
                let looks_like_target = lower_key.contains("webhook")
                    || lower_key.contains("url")
                    || lower_key.contains("endpoint");
                if looks_like_target && raw.starts_with("http") {
                    validate_external_url(raw)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    visit(None, value)
}

fn validate_external_url(raw: &str) -> Result<(), UptimeKumaError> {
    let parsed = Url::parse(raw).map_err(|_| {
        UptimeKumaError::InvalidParam("notification URL fields must be valid URLs".into())
    })?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(UptimeKumaError::InvalidParam(
            "notification URL fields must use http or https".into(),
        ));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(UptimeKumaError::InvalidParam(
            "notification URL fields must not include userinfo".into(),
        ));
    }
    let Some(host) = parsed.host_str() else {
        return Err(UptimeKumaError::InvalidParam(
            "notification URL fields must include a host".into(),
        ));
    };
    let host = host.trim_matches(['[', ']']).to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") {
        return Err(UptimeKumaError::InvalidParam(
            "notification URL fields must not target localhost".into(),
        ));
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        if ip.is_loopback()
            || ip.is_unspecified()
            || ip.is_multicast()
            || match ip {
                std::net::IpAddr::V4(ip) => {
                    ip.is_private()
                        || ip.is_link_local()
                        || ip.octets()[0] == 169 && ip.octets()[1] == 254
                }
                std::net::IpAddr::V6(ip) => {
                    ip.is_loopback() || ip.is_unspecified() || ip.is_unique_local()
                }
            }
        {
            return Err(UptimeKumaError::InvalidParam(
                "notification URL fields must not target local, private, or metadata networks"
                    .into(),
            ));
        }
    }
    Ok(())
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

    #[test]
    fn notification_config_rejects_template_markers() {
        let err = validate_notification_config(&json!({
            "type": "discord",
            "name": "Discord",
            "message": "{{ env.PASSWORD }}"
        }))
        .unwrap_err();

        assert!(matches!(err, UptimeKumaError::InvalidParam(_)));
    }

    #[test]
    fn notification_config_rejects_private_webhook_url() {
        let err = validate_notification_config(&json!({
            "type": "webhook",
            "name": "Webhook",
            "webhookURL": "http://192.168.1.10/hook"
        }))
        .unwrap_err();

        assert!(matches!(err, UptimeKumaError::InvalidParam(_)));
    }

    #[test]
    fn notification_config_allows_public_webhook_url() {
        validate_notification_config(&json!({
            "type": "webhook",
            "name": "Webhook",
            "webhookURL": "https://hooks.example.com/service"
        }))
        .unwrap();
    }
}
