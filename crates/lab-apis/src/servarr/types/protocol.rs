//! Transfer protocol used by an indexer / download client / release.

use serde::{Deserialize, Serialize};

/// Which transfer protocol a release is distributed over.
///
/// Matches the `DownloadProtocol` enum in the Radarr / Sonarr / Prowlarr
/// OpenAPI specs. `Unknown` covers future values without breaking
/// deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadProtocol {
    /// Usenet (NZB) transfer.
    Usenet,
    /// BitTorrent transfer.
    Torrent,
    /// Unknown or future protocol value.
    Unknown,
}
