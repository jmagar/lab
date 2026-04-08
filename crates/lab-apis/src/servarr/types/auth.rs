//! Session auth types.

use serde::Serialize;

/// Body for `POST /login` when an *arr service is configured with forms auth.
#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remember_me: Option<bool>,
}
