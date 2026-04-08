//! Apprise request/response types.
//!
//! Modeled on the apprise-api Flask surface. Apprise does not publish an
//! OpenAPI spec; these types are distilled from the project README.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around an apprise-api config key (the path segment in
/// `/notify/{key}`, `/add/{key}`, etc.). Keys are arbitrary strings chosen
/// by the caller — typically one per logical recipient group.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConfigKey(pub String);

/// Severity/type of a notification. Apprise uses this to color messages and
/// optionally filter delivery on a per-URL basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotifyType {
    Info,
    Success,
    Warning,
    Failure,
}

/// Body format of the notification `body` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BodyFormat {
    Text,
    Html,
    Markdown,
}

/// Request body for `POST /notify` (stateless) and `POST /notify/{key}`
/// (stateful — `urls` is then ignored).
#[derive(Debug, Clone, Serialize)]
pub struct NotifyRequest {
    /// The message body.
    pub body: String,

    /// Optional title. Apprise falls back to a sensible default per-service
    /// if omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Stateless mode only: one or more Apprise URLs (e.g. `slack://...`,
    /// `tgram://...`, comma-separated in a single string, or — when using
    /// the JSON content type — sent as a list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<Vec<String>>,

    /// Notification type (default: `info`).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub notify_type: Option<NotifyType>,

    /// Body format (default: `text`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<BodyFormat>,

    /// Optional tag filter — when a stored config contains tags, only URLs
    /// matching the specified tags are notified. Accepts `"all"` as a
    /// wildcard.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
}

/// Request body for `POST /add/{key}` — persist a config under `key`.
///
/// Exactly one of `urls` or `config` should be set:
///
/// - `urls`: a newline- or comma-separated list of Apprise URLs.
/// - `config`: a YAML or TEXT config blob (more expressive — supports tags).
#[derive(Debug, Clone, Serialize)]
pub struct AddRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<String>,

    /// `"yaml"` or `"text"` — required when `config` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}
