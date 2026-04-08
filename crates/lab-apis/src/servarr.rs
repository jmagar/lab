//! Shared primitives for the *arr family (Radarr, Sonarr, Prowlarr, …).
//!
//! Types here have identical shape across every *arr service. Service-specific
//! types (e.g. history records that reference `MovieId` or `SeriesId`) stay
//! in the per-service module.
//!
//! This module is types-only — no HTTP client, no endpoints, no `PluginMeta`.
//! Each *arr service re-exports these shapes from its own `types/*.rs` files
//! so downstream callers can keep writing `radarr::types::Quality` without
//! caring where the definition lives. Errors are the shared
//! [`crate::core::ApiError`] — servarr does not define its own error type.

pub mod types;
