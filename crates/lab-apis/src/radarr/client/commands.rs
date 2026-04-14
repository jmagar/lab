//! Command endpoints.
//!
//! Covers `/api/v3/command` — the async job queue Radarr uses for anything
//! that takes time (refresh metadata, rescan disk, search, RSS sync, etc.).
//! Every POST here returns a `Command` object whose `id` can be polled.

use super::RadarrClient;
use crate::core::ApiError;
use crate::radarr::error::RadarrError;
use crate::radarr::types::command::{Command, CommandId};
use crate::radarr::types::movie::MovieId;

impl RadarrClient {
    /// Queue a `RefreshMovie` command.
    ///
    /// When `movie_id` is `None`, Radarr refreshes every movie in the
    /// library; otherwise only the specified movie.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn command_refresh_movie(
        &self,
        movie_id: Option<MovieId>,
    ) -> Result<Command, RadarrError> {
        let body = movie_id.map_or_else(
            || serde_json::json!({ "name": "RefreshMovie" }),
            |id| serde_json::json!({ "name": "RefreshMovie", "movieIds": [id.0] }),
        );
        self.http
            .post_json("/api/v3/command", &body)
            .await
            .map_err(RadarrError::from)
    }

    /// Queue a `MoviesSearch` command — tells Radarr to search the
    /// configured indexers for the given movie ids immediately.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn command_movies_search(
        &self,
        movie_ids: &[MovieId],
    ) -> Result<Command, RadarrError> {
        let ids: Vec<i64> = movie_ids.iter().map(|m| m.0).collect();
        let body = serde_json::json!({ "name": "MoviesSearch", "movieIds": ids });
        self.http
            .post_json("/api/v3/command", &body)
            .await
            .map_err(RadarrError::from)
    }

    /// Poll a queued command by id.
    ///
    /// Maps to `GET /api/v3/command/{id}`. The returned `Command.status`
    /// cycles through `queued` → `started` → `completed` / `failed`.
    ///
    /// # Errors
    /// Returns `RadarrError::NotFound` if the id does not exist,
    /// `RadarrError::Api` on any other HTTP failure.
    pub async fn command_get(&self, id: CommandId) -> Result<Command, RadarrError> {
        self.http
            .get_json(&format!("/api/v3/command/{}", id.0))
            .await
            .map_err(|e| match e {
                ApiError::NotFound => RadarrError::NotFound {
                    kind: "command",
                    id: id.0,
                },
                other => RadarrError::Api(other),
            })
    }

    /// Execute a named scheduled task immediately.
    ///
    /// Maps to `POST /api/v3/command` with body `{"name": "<TaskName>"}`.
    /// `task_name` must match one of Radarr's internal task names (e.g.
    /// `"RssSync"`, `"RefreshMonitoredDownloads"`, `"ApplicationCheckUpdate"`).
    /// Returns the queued `Command` object whose `id` can be polled.
    ///
    /// Destructive in the sense that it triggers background processing that
    /// may alter library state.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn system_task_execute(&self, task_name: &str) -> Result<Command, RadarrError> {
        let body = serde_json::json!({ "name": task_name });
        self.http
            .post_json("/api/v3/command", &body)
            .await
            .map_err(RadarrError::from)
    }
}
