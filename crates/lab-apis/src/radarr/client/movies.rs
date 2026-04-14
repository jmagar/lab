//! Movie endpoints.
//!
//! Covers `/api/v3/movie`, `/api/v3/movie/lookup`, and `/api/v3/moviefile`.
//! All methods are stubs pending implementation from the OpenAPI spec.

use super::RadarrClient;
use crate::core::ApiError;
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
        self.http
            .get_json("/api/v3/movie")
            .await
            .map_err(RadarrError::from)
    }

    /// Fetch a single movie by its internal Radarr id.
    ///
    /// Maps to `GET /api/v3/movie/{id}`.
    ///
    /// # Errors
    /// Returns `RadarrError::NotFound` if the id does not exist,
    /// `RadarrError::Api` on any other HTTP failure.
    pub async fn movie_get(&self, id: MovieId) -> Result<Movie, RadarrError> {
        self.http
            .get_json(&format!("/api/v3/movie/{}", id.0))
            .await
            .map_err(|e| match e {
                ApiError::NotFound => RadarrError::NotFound {
                    kind: "movie",
                    id: id.0,
                },
                other => RadarrError::Api(other),
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
        let q = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("term", term)
            .finish();
        self.http
            .get_json(&format!("/api/v3/movie/lookup?{q}"))
            .await
            .map_err(RadarrError::from)
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
        self.http
            .post_json("/api/v3/movie", movie)
            .await
            .map_err(RadarrError::from)
    }

    /// Update an existing movie resource.
    ///
    /// Maps to `PUT /api/v3/movie/{id}`. The `body` should be the full movie
    /// resource JSON (typically fetched via [`Self::movie_get`], modified, then
    /// sent back). Returns the updated movie.
    ///
    /// # Errors
    /// Returns `RadarrError::NotFound` if the id does not exist,
    /// `RadarrError::Api` on any other HTTP failure.
    pub async fn movie_edit(
        &self,
        id: MovieId,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value, RadarrError> {
        self.http
            .put_json(&format!("/api/v3/movie/{}", id.0), body)
            .await
            .map_err(|e| match e {
                ApiError::NotFound => RadarrError::NotFound {
                    kind: "movie",
                    id: id.0,
                },
                other => RadarrError::Api(other),
            })
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
        self.http
            .delete(&format!(
                "/api/v3/movie/{}?deleteFiles={delete_files}",
                id.0
            ))
            .await
            .map_err(|e| match e {
                ApiError::NotFound => RadarrError::NotFound {
                    kind: "movie",
                    id: id.0,
                },
                other => RadarrError::Api(other),
            })
    }
}
