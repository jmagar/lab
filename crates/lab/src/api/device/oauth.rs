use std::net::SocketAddr;
use std::time::Duration;

use axum::{Json, extract::State};
use serde::Deserialize;

use crate::api::{ToolError, device::DeviceAck, state::AppState};

#[derive(Debug, Deserialize)]
pub struct StartOauthRelayRequest {
    pub bind_addr: SocketAddr,
    pub target_url: String,
    #[serde(default)]
    pub default_port: Option<u16>,
    #[serde(default)]
    pub request_timeout_ms: Option<u64>,
}

pub async fn handle_start(
    State(_state): State<AppState>,
    Json(payload): Json<StartOauthRelayRequest>,
) -> Result<Json<DeviceAck>, ToolError> {
    let resolved_target =
        crate::oauth::target::resolve_explicit_target(&payload.target_url, payload.default_port)
            .map_err(|error| ToolError::InvalidParam {
                message: error.to_string(),
                param: "target_url".to_string(),
            })?;
    let timeout = Duration::from_millis(payload.request_timeout_ms.unwrap_or(30_000));

    tokio::spawn(async move {
        if let Err(error) = crate::device::oauth::start_local_oauth_relay(
            payload.bind_addr,
            resolved_target,
            timeout,
        )
        .await
        {
            tracing::warn!(error = %error, "device oauth relay exited");
        }
    });

    Ok(super::ok())
}
