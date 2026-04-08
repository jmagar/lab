//! Radarr request/response types, split by resource.
//!
//! Each sibling module under `types/` owns the types for one API resource
//! group. Downstream code can either reach in directly
//! (`radarr::types::movie::Movie`) or use the flat re-exports below for
//! commonly-used types.

/// Movies and movie-file types.
pub mod movie;

/// Download-queue types.
pub mod queue;

/// Indexer configuration types.
pub mod indexer;

/// Quality profile and definition types.
pub mod quality;

/// Command (async job) types.
pub mod command;

/// History and blocklist types.
pub mod history;

/// Calendar entry types.
pub mod calendar;

/// Download client and remote-path-mapping types.
pub mod download_client;

/// Import list types.
pub mod import_list;

/// Notification provider types.
pub mod notification;

/// Metadata writer types.
pub mod metadata;

/// Host / UI / naming config types.
pub mod config;

/// System status, health, log, update, disk-space types.
pub mod system;

/// Tag types.
pub mod tag;

/// Root folder types.
pub mod root_folder;

/// Filesystem browsing and manual-import types.
pub mod filesystem;

/// Release search types.
pub mod release;

/// Session auth types.
pub mod auth;

/// Language types.
pub mod language;

// Flat re-exports for the most-used types. Keep this list short — callers
// wanting deeper types should reach into the per-resource module.
pub use command::{Command, CommandId, CommandStatus};
pub use indexer::{Indexer, IndexerId};
pub use movie::{Movie, MovieId, MovieLookup};
pub use quality::{QualityDefinition, QualityProfile, QualityProfileId};
pub use queue::{QueueItem, QueueItemId};
