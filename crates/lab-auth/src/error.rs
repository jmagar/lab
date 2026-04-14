use std::path::PathBuf;

use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("{0}")]
    Config(String),

    #[error("{0}")]
    Storage(String),

    #[error("{0}")]
    InvalidGrant(String),

    #[error("{0}")]
    AuthFailed(String),

    #[error("{0}")]
    Validation(String),

    #[error("{message}")]
    RateLimited { message: String, retry_after_ms: u64 },

    #[error("invalid access token")]
    InvalidAccessToken,

    #[error("access token verifier is not configured")]
    UnconfiguredVerifier,

    #[error("path `{path}` has insecure permissions")]
    InsecurePermissions { path: PathBuf },
}

impl AuthError {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Config(_) | Self::Storage(_) | Self::UnconfiguredVerifier | Self::InsecurePermissions { .. } => "internal_error",
            Self::InvalidGrant(_) => "invalid_grant",
            Self::AuthFailed(_) | Self::InvalidAccessToken => "auth_failed",
            Self::Validation(_) => "validation_failed",
            Self::RateLimited { .. } => "rate_limited",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidGrant(_) => StatusCode::BAD_REQUEST,
            Self::AuthFailed(_) | Self::InvalidAccessToken => StatusCode::UNAUTHORIZED,
            Self::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Config(_)
            | Self::Storage(_)
            | Self::UnconfiguredVerifier
            | Self::InsecurePermissions { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = axum::Json(serde_json::json!({
            "kind": self.kind(),
            "message": self.to_string(),
        }));
        let mut response = (status, body).into_response();
        if let Self::RateLimited { retry_after_ms, .. } = self {
            if let Ok(value) = HeaderValue::from_str(&(retry_after_ms / 1_000).max(1).to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }
}
