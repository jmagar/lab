//! Movie types.
//!
//! Mirrors the `MovieResource` schema from the Radarr v3 OpenAPI spec.
//! Only the most commonly consumed fields are modeled here; the rest can
//! land as needed.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around an internal Radarr movie id.
///
/// Distinct from [`TmdbId`] and [`ImdbId`] so the type system prevents
/// accidental cross-wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MovieId(pub i64);

/// TMDB (The Movie Database) id — how Radarr uniquely identifies a movie
/// in its metadata provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TmdbId(pub i64);

/// IMDb id (e.g. "tt0133093").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImdbId(pub String);

/// A movie entry in the Radarr library.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Movie {
    pub id: MovieId,
    pub title: String,
    pub year: i32,
    pub tmdb_id: TmdbId,
    #[serde(default)]
    pub imdb_id: Option<ImdbId>,
    pub has_file: bool,
    pub monitored: bool,
    #[serde(default)]
    pub quality_profile_id: Option<i64>,
    #[serde(default)]
    pub root_folder_path: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub size_on_disk: i64,
}

/// Result from `/api/v3/movie/lookup`.
///
/// Not yet in the library — `id` will be absent or zero. Use the `tmdb_id`
/// to add.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MovieLookup {
    pub title: String,
    pub year: i32,
    pub tmdb_id: TmdbId,
    #[serde(default)]
    pub imdb_id: Option<ImdbId>,
    #[serde(default)]
    pub overview: Option<String>,
}
