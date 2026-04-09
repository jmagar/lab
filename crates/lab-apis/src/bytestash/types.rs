//! Request models for documented ByteStash payloads.

use serde::{Deserialize, Serialize};

/// Username/password payload used by auth endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct AuthCredentials {
    /// Login or registration username.
    pub username: String,
    /// Login or registration password.
    pub password: String,
}

/// One file fragment inside a snippet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnippetFragment {
    /// File name shown in the UI.
    pub file_name: String,
    /// Snippet contents.
    pub code: String,
}

/// Create/update payload for a snippet.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct SnippetWriteRequest {
    /// Snippet title.
    pub title: String,
    /// Optional description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional language label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// File fragments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fragments: Vec<SnippetFragment>,
    /// Category names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
}

/// Create-share payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ShareCreateRequest {
    /// ID of the snippet to share.
    #[serde(rename = "snippetId")]
    pub snippet_id: String,
    /// Whether the share link requires auth to view.
    #[serde(rename = "requiresAuth", skip_serializing_if = "Option::is_none")]
    pub requires_auth: Option<bool>,
    /// Expiry in seconds from now (null = never).
    #[serde(rename = "expiresIn", skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
}
