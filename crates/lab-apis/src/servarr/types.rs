//! Shared *arr types, one sub-module per resource.
//!
//! Each sub-module mirrors the matching `*Resource` shape from the Radarr /
//! Sonarr / Prowlarr `OpenAPI` specs. Only the fields that are identical across
//! services live here; service-specific fields stay in the per-service
//! `types/<name>.rs` file alongside a `pub use` of these shapes.

pub mod auth;
pub mod command;
pub mod download_client;
pub mod filesystem;
pub mod indexer;
pub mod language;
pub mod metadata;
pub mod notification;
pub mod protocol;
pub mod quality;
pub mod release;
pub mod root_folder;
pub mod system;
pub mod tag;
