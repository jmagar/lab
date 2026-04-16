use std::net::SocketAddr;
use std::time::Duration;

use axum::{Json, extract::State};
use serde::Deserialize;

use crate::api::{ToolError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct StartOauthRelayRequest {
    pub bind_addr: SocketAddr,
    pub target_url: String,
    #[serde(default)]
    pub default_port: Option<u16>,
    #[serde(default)]
    pub request_timeout_ms: Option<u64>,
}

#[derive(Debug, serde::Serialize)]
pub struct StartOauthRelayResponse {
    pub ok: bool,
    pub bind_addr: SocketAddr,
}

fn validate_bind_addr(bind_addr: SocketAddr) -> Result<(), ToolError> {
    if !bind_addr.ip().is_loopback() {
        return Err(ToolError::InvalidParam {
            message: "bind_addr must be a loopback address".to_string(),
            param: "bind_addr".to_string(),
        });
    }

    Ok(())
}

pub async fn handle_start(
    State(_state): State<AppState>,
    Json(payload): Json<StartOauthRelayRequest>,
) -> Result<Json<StartOauthRelayResponse>, ToolError> {
    validate_bind_addr(payload.bind_addr)?;
    let resolved_target =
        crate::oauth::target::resolve_explicit_target(&payload.target_url, payload.default_port)
            .map_err(|error| ToolError::InvalidParam {
                message: error.to_string(),
                param: "target_url".to_string(),
            })?;
    let timeout = Duration::from_millis(payload.request_timeout_ms.unwrap_or(30_000));

    let bound_addr =
        crate::device::oauth::start_local_oauth_relay(payload.bind_addr, resolved_target, timeout)
            .await
            .map_err(|error| ToolError::Sdk {
                sdk_kind: "internal_error".to_string(),
                message: error.to_string(),
            })?;

    Ok(Json(StartOauthRelayResponse {
        ok: true,
        bind_addr: bound_addr,
    }))
}
