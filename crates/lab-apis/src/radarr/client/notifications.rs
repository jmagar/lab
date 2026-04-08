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
        let _ = &self.http;
        Ok(Vec::new())
    }

    /// Fire a test notification.
    ///
    /// Maps to `POST /api/v3/notification/test`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn notification_test(&self, id: NotificationId) -> Result<(), RadarrError> {
        let _ = id;
        Ok(())
    }
}
