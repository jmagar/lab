//! Calendar entry types.

use serde::{Deserialize, Serialize};

use super::movie::{Movie, MovieId};

/// A single calendar entry. Radarr reuses the full `Movie` shape here —
/// the calendar response is just a filtered movie list windowed by
/// release date.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalendarEntry {
    pub id: MovieId,
    pub title: String,
    #[serde(default)]
    pub in_cinemas: Option<String>,
    #[serde(default)]
    pub physical_release: Option<String>,
    #[serde(default)]
    pub digital_release: Option<String>,
    #[serde(flatten)]
    pub movie: Option<Box<Movie>>,
}
