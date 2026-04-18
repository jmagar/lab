use serde::Serialize;

use crate::dispatch::error::ToolError;
use crate::dispatch::gateway::manager::GatewayManager;
use crate::oauth::upstream::types::{BeginAuthorization, OauthError};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct UpstreamOauthStatusView {
    pub authenticated: bool,
    pub upstream: String,
    pub expires_within_5m: bool,
}

pub async fn begin_authorization(
    manager: &GatewayManager,
    upstream: &str,
    subject: &str,
) -> Result<BeginAuthorization, ToolError> {
    manager
        .begin_upstream_authorization(upstream, subject)
        .await
}

pub async fn complete_authorization_callback(
    manager: &GatewayManager,
    upstream: &str,
    subject: &str,
    code: &str,
    state: &str,
) -> Result<(), ToolError> {
    manager
        .complete_upstream_authorization_callback(upstream, subject, code, state)
        .await
}

pub async fn status(
    manager: &GatewayManager,
    upstream: &str,
    subject: &str,
) -> Result<UpstreamOauthStatusView, ToolError> {
    manager.upstream_oauth_status(upstream, subject).await
}

pub async fn clear(
    manager: &GatewayManager,
    upstream: &str,
    subject: &str,
) -> Result<(), ToolError> {
    manager.clear_upstream_credentials(upstream, subject).await
}

pub fn tool_error_from_oauth(error: OauthError) -> ToolError {
    ToolError::Sdk {
        sdk_kind: error.kind().to_string(),
        message: error.to_string(),
    }
}
