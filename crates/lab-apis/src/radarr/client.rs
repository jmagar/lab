//! `RadarrClient` — struct definition, constructor, and per-resource `impl`
//! block modules.
//!
//! The client itself is intentionally tiny: it owns an `HttpClient` and a
//! `health()` method. **All endpoint methods live in the resource-scoped
//! sub-modules** (`movies`, `queue`, `indexers`, `quality`, `commands`),
//! each of which adds its own `impl RadarrClient { ... }` block. Rust
//! allows unlimited `impl` blocks across files for the same type, so this
//! gives us one cohesive client surface without a 5000-line `client.rs`.
//!
//! Adding a new resource is purely additive:
//!
//! 1. Create `radarr/client/foo.rs` with `impl RadarrClient { ... }`
//! 2. Create `radarr/types/foo.rs` with the request/response types
//! 3. Add `pub mod foo;` to this file and to `radarr/types.rs`
//!
//! No existing code changes — the new methods just appear on `RadarrClient`.

/// Movie CRUD, lookup, file management.
pub mod movies;

/// Download queue inspection and manipulation.
pub mod queue;

/// Indexer configuration (read-only wrappers around `/indexer`).
pub mod indexers;

/// Quality profiles and quality definitions.
pub mod quality;

/// System commands (refresh, search, rename, rescan).
pub mod commands;

/// History records and blocklist.
pub mod history;

/// Calendar / upcoming releases.
pub mod calendar;

/// Download clients (SAB, qBit, …) and remote-path mappings.
pub mod download_clients;

/// Import lists (TMDB/Trakt/Letterboxd/IMDb) and exclusions.
pub mod import_lists;

/// Notification providers (Discord, Slack, email, webhook, …).
pub mod notifications;

/// Metadata writers (Kodi/Plex/Emby) and media cover art.
pub mod metadata;

/// Host / UI / naming / media-management config endpoints.
pub mod config;

/// System status, health, logs, updates, disk space.
pub mod system;

/// Tag CRUD and usage detail.
pub mod tags;

/// Root folders.
pub mod root_folders;

/// Filesystem browsing and manual import.
pub mod filesystem;

/// Release search results.
pub mod releases;

/// Session login/logout (rarely used — most deploys use API-key auth).
pub mod auth;

/// Languages and localization.
pub mod language;

use crate::core::{Auth, HttpClient};

use super::error::RadarrError;

/// Client for a Radarr v3 instance.
///
/// Construct once and share across tasks — `HttpClient` is internally
/// `Clone`-able and cheap to reuse.
pub struct RadarrClient {
    pub(crate) http: HttpClient,
}

impl RadarrClient {
    /// Build a client against `base_url` with a Radarr API key.
    ///
    /// `base_url` should be the root of the Radarr instance (e.g.
    /// `http://radarr.lan:7878`) — the `/api/v3` prefix is added
    /// automatically by each endpoint method.
    ///
    /// Radarr uses header-token auth:
    /// `Auth::ApiKey { header: "X-Api-Key".into(), key: <api_key> }`.
    #[must_use]
    pub fn new(base_url: &str, auth: Auth) -> Self {
        Self {
            http: HttpClient::new(base_url, auth),
        }
    }

    /// Probe `GET /api/v3/system/status` as a liveness check.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` if the request fails or the server
    /// returns a non-2xx status.
    pub async fn health(&self) -> Result<(), RadarrError> {
        // TODO: GET /api/v3/system/status
        Ok(())
    }
}
