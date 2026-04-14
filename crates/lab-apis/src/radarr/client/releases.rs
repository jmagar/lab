//! Release endpoints.
//!
//! Covers `/api/v3/release` (indexer search results — what Radarr found
//! for a movie) and `/api/v3/release/push` (manual release grabs).

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::movie::MovieId;
use crate::radarr::types::release::Release;

impl RadarrClient {
    /// Search every indexer for releases matching a given movie.
    ///
    /// Maps to `GET /api/v3/release?movieId=...`. Note this is distinct
    /// from [`Self::command_movies_search`] — that queues an async grab,
    /// this one returns the raw results synchronously.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn release_search(&self, movie_id: MovieId) -> Result<Vec<Release>, RadarrError> {
        self.http
            .get_json(&format!("/api/v3/release?movieId={}", movie_id.0))
            .await
            .map_err(RadarrError::from)
    }

    /// Grab (download) a specific release.
    ///
    /// Maps to `POST /api/v3/release`. The `release` body should be the full
    /// release object returned from [`Self::release_search`] (Radarr echoes
    /// key fields back). Returns the grabbed release resource as a JSON value.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn release_grab(
        &self,
        release: &serde_json::Value,
    ) -> Result<serde_json::Value, RadarrError> {
        self.http
            .post_json("/api/v3/release", release)
            .await
            .map_err(RadarrError::from)
    }
}
