//! Per-service HTTP route handlers.
//!
//! Versioned REST and action-dispatch route modules for the HTTP API.
//!
//! Most service modules expose `pub fn routes(state: AppState) -> Router` that
//! mounts a `POST /` action-dispatch handler matching the MCP `action + params`
//! shape. Modules may also expose versioned REST routers such as
//! `registry_v01`, which serves `/v0.1/servers/*`.

/// Shared dispatch wrapper: confirmation gate, timing, logging.
pub mod helpers;

pub mod acp;
pub mod doctor;
pub mod extract;
pub mod gateway;
pub mod logs;
pub mod marketplace;

#[cfg(feature = "radarr")]
pub mod radarr;

#[cfg(feature = "sonarr")]
pub mod sonarr;

#[cfg(feature = "prowlarr")]
pub mod prowlarr;

#[cfg(feature = "plex")]
pub mod plex;

#[cfg(feature = "tautulli")]
pub mod tautulli;

#[cfg(feature = "sabnzbd")]
pub mod sabnzbd;

#[cfg(feature = "qbittorrent")]
pub mod qbittorrent;

#[cfg(feature = "tailscale")]
pub mod tailscale;

#[cfg(feature = "linkding")]
pub mod linkding;

#[cfg(feature = "mcpregistry")]
pub mod mcpregistry;

#[cfg(feature = "mcpregistry")]
pub mod registry_v01;
#[cfg(feature = "memos")]
pub mod memos;

#[cfg(feature = "bytestash")]
pub mod bytestash;

#[cfg(feature = "paperless")]
pub mod paperless;

#[cfg(feature = "arcane")]
pub mod arcane;

#[cfg(feature = "unraid")]
pub mod unraid;

#[cfg(feature = "unifi")]
pub mod unifi;

#[cfg(feature = "overseerr")]
pub mod overseerr;

#[cfg(feature = "gotify")]
pub mod gotify;

#[cfg(feature = "openai")]
pub mod openai;

#[cfg(feature = "qdrant")]
pub mod qdrant;

#[cfg(feature = "tei")]
pub mod tei;

#[cfg(feature = "apprise")]
pub mod apprise;
