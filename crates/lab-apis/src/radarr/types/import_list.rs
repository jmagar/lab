//! Import list types.

use serde::{Deserialize, Serialize};

use super::movie::TmdbId;

/// Newtype wrapper around an import-list id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImportListId(pub i64);

/// Newtype wrapper around an import-list-exclusion id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ImportListExclusionId(pub i64);

/// A configured import list (TMDB collection, Trakt user list, `IMDb` list,
/// Letterboxd list, …).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportList {
    pub id: ImportListId,
    pub name: String,
    pub implementation: String,
    pub enabled: bool,
    pub enable_auto: bool,
    pub monitor: String,
    pub quality_profile_id: i64,
    pub root_folder_path: String,
    #[serde(default)]
    pub tags: Vec<i64>,
    #[serde(default)]
    pub fields: serde_json::Value,
}

/// A single import-list exclusion. Exclusions are keyed on TMDB id so
/// the same movie never gets auto-added again regardless of which list
/// surfaced it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportListExclusion {
    pub id: ImportListExclusionId,
    pub tmdb_id: TmdbId,
    pub movie_title: String,
    pub movie_year: i32,
}
