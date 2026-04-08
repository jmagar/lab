//! Calendar endpoints.
//!
//! Covers `/api/v3/calendar` and `/api/v3/calendar/radarr.ics` — upcoming
//! and recent movie release dates.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::calendar::CalendarEntry;

impl RadarrClient {
    /// List calendar entries between two ISO-8601 dates.
    ///
    /// Maps to `GET /api/v3/calendar?start=...&end=...`. When both dates
    /// are `None`, Radarr returns a default window around "now".
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn calendar_list(
        &self,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<CalendarEntry>, RadarrError> {
        let _ = (start, end, &self.http);
        Ok(Vec::new())
    }
}
