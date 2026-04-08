//! Notification provider types.

use serde::{Deserialize, Serialize};

use super::tag::TagId;

/// Newtype wrapper around a notification id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NotificationId(pub i64);

/// A configured notification provider.
///
/// Mirrors `NotificationResource` from the Radarr v3 / Sonarr v3 `OpenAPI`
/// specs. Service-specific event flags (radarr's `on_movie_delete`, sonarr's
/// `on_series_delete`, etc.) are intentionally not modeled here — they round-
/// trip through `fields` as raw JSON. If a caller needs strict typing for
/// those flags, it should define a per-service extension struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct Notification {
    /// Notification id.
    pub id: NotificationId,
    /// Display name.
    pub name: String,
    /// Provider implementation (e.g. `"Discord"`, `"Telegram"`).
    pub implementation: String,
    /// Fire on release grab.
    #[serde(default)]
    pub on_grab: bool,
    /// Fire on successful download.
    #[serde(default)]
    pub on_download: bool,
    /// Fire on an upgrade import.
    #[serde(default)]
    pub on_upgrade: bool,
    /// Fire on media rename.
    #[serde(default)]
    pub on_rename: bool,
    /// Fire on a new health-check finding.
    #[serde(default)]
    pub on_health_issue: bool,
    /// Tag ids scoping which resources trigger this provider.
    #[serde(default)]
    pub tags: Vec<TagId>,
    /// Provider-specific configuration bag — shape varies per implementation.
    #[serde(default)]
    pub fields: serde_json::Value,
}
