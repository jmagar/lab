//! Command endpoints.
//!
//! Covers `/api/v3/command` — the async job queue Radarr uses for anything
//! that takes time (refresh metadata, rescan disk, search, RSS sync, etc.).
//! Every POST here returns a `Command` object whose `id` can be polled.

use super::RadarrClient;
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
        let _ = movie_id;
        Err(RadarrError::Api(crate::core::error::ApiError::Internal(
            "command_refresh_movie not yet implemented".into(),
        )))
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
        let _ = movie_ids;
        Err(RadarrError::Api(crate::core::error::ApiError::Internal(
            "command_movies_search not yet implemented".into(),
        )))
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
        let _ = id;
        Err(RadarrError::NotFound { kind: "command", id: id.0 })
    }
}
