//! Movie endpoints.
//!
//! Covers `/api/v3/movie`, `/api/v3/movie/lookup`, and `/api/v3/moviefile`.
//! All methods are stubs pending implementation from the OpenAPI spec.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::movie::{Movie, MovieId, MovieLookup};

impl RadarrClient {
    /// List every movie in the library.
    ///
    /// Maps to `GET /api/v3/movie`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn movie_list(&self) -> Result<Vec<Movie>, RadarrError> {
        // TODO: GET /api/v3/movie
        let _ = &self.http;
        Ok(Vec::new())
    }

    /// Fetch a single movie by its internal Radarr id.
    ///
    /// Maps to `GET /api/v3/movie/{id}`.
    ///
    /// # Errors
    /// Returns `RadarrError::NotFound` if the id does not exist,
    /// `RadarrError::Api` on any other HTTP failure.
    pub async fn movie_get(&self, id: MovieId) -> Result<Movie, RadarrError> {
        let _ = id;
        Err(RadarrError::NotFound {
            kind: "movie",
            id: id.0,
        })
    }

    /// Search the Radarr metadata provider (TMDB-backed) for a title.
    ///
    /// Maps to `GET /api/v3/movie/lookup?term=...`. Does **not** add the
    /// movie — use [`Self::movie_add`] for that.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn movie_lookup(&self, term: &str) -> Result<Vec<MovieLookup>, RadarrError> {
        let _ = term;
        Ok(Vec::new())
    }

    /// Add a previously-looked-up movie to the library.
    ///
    /// Maps to `POST /api/v3/movie`. The caller is responsible for picking
    /// a quality profile and root folder — typically by first calling
    /// [`Self::quality_profile_list`] and [`Self::movie_lookup`].
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn movie_add(&self, movie: &Movie) -> Result<Movie, RadarrError> {
        let _ = movie;
        Err(RadarrError::Api(crate::core::error::ApiError::Internal(
            "movie_add not yet implemented".into(),
        )))
    }

    /// Delete a movie by id.
    ///
    /// Maps to `DELETE /api/v3/movie/{id}`. When `delete_files` is true,
    /// Radarr also removes the on-disk files — destructive, callers should
    /// confirm at the CLI/MCP boundary.
    ///
    /// # Errors
    /// Returns `RadarrError::NotFound` if the id does not exist,
    /// `RadarrError::Api` on any other HTTP failure.
    pub async fn movie_delete(&self, id: MovieId, delete_files: bool) -> Result<(), RadarrError> {
        let _ = (id, delete_files);
        Err(RadarrError::NotFound {
            kind: "movie",
            id: id.0,
        })
    }
}
