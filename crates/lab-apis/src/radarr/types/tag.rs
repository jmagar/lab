//! Radarr tag types — the core `Tag` / `TagId` / `TagDetail` shapes live in
//! [`crate::servarr::types::tag`]; only the radarr-specific `movie_ids`
//! association is added here.

use serde::{Deserialize, Serialize};

pub use crate::servarr::types::tag::{Tag, TagDetail, TagId};

use super::movie::MovieId;

/// Movies carrying a given tag. Radarr returns this as part of its
/// `TagDetailsResource`; the shared servarr `TagDetail` covers the
/// cross-service id arrays (indexers, download clients, notifications).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RadarrTagMovies {
    /// Tag id.
    pub id: TagId,
    /// Movies currently tagged.
    pub movie_ids: Vec<MovieId>,
}
