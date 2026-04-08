//! Notification endpoints.
//!
//! Covers `/api/v3/notification` — Discord, Slack, email, webhook,
//! Pushover, etc. providers.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::notification::{Notification, NotificationId};

impl RadarrClient {
    /// List every configured notification provider.
    ///
    /// Maps to `GET /api/v3/notification`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn notification_list(&self) -> Result<Vec<Notification>, RadarrError> {
        self.http
            .get_json("/api/v3/notification")
            .await
            .map_err(RadarrError::from)
    }

    /// Fire a test notification.
    ///
    /// Maps to `POST /api/v3/notification/test`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn notification_test(&self, id: NotificationId) -> Result<(), RadarrError> {
        let body = serde_json::json!({ "id": id.0 });
        self.http
            .post_void("/api/v3/notification/test", &body)
            .await
            .map_err(RadarrError::from)
    }
}
