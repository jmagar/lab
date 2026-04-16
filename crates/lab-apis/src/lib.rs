//! Pure Rust SDK for homelab service APIs.
//!
//! `lab-apis` wraps every service that `lab` (the binary) talks to. It is
//! intentionally free of `clap`, `rmcp`, `ratatui`, and `anyhow` so that any
//! Rust program can pull it in as a standalone SDK.
//!
//! Each service lives behind a feature flag so consumers compile only what
//! they need:
//!
//! ```toml
//! lab-apis = { version = "0.1", default-features = false, features = ["radarr", "plex"] }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

/// Cross-cutting primitives: HTTP client, auth, errors, status, action specs.
pub mod core;

/// Shared `*arr` primitives used by Radarr/Sonarr/Prowlarr.
#[cfg(feature = "servarr")]
pub mod servarr;

/// Bootstrap utility: extract API keys from existing service config files.
pub mod extract;

/// Device-runtime control-plane client shared by CLI and runtime code.
pub mod device_runtime;

/// Radarr movie management client.
#[cfg(feature = "radarr")]
pub mod radarr;

/// Sonarr TV management client.
#[cfg(feature = "sonarr")]
pub mod sonarr;

/// Prowlarr indexer management client.
#[cfg(feature = "prowlarr")]
pub mod prowlarr;

/// Overseerr media request management client.
#[cfg(feature = "overseerr")]
pub mod overseerr;

/// Plex media server client.
#[cfg(feature = "plex")]
pub mod plex;

/// Tautulli Plex analytics client.
#[cfg(feature = "tautulli")]
pub mod tautulli;

/// SABnzbd usenet downloader client.
#[cfg(feature = "sabnzbd")]
pub mod sabnzbd;

/// qBittorrent torrent client.
#[cfg(feature = "qbittorrent")]
pub mod qbittorrent;

/// Tailscale VPN client.
#[cfg(feature = "tailscale")]
pub mod tailscale;

/// Linkding bookmark manager client.
#[cfg(feature = "linkding")]
pub mod linkding;

/// Memos note-taking client.
#[cfg(feature = "memos")]
pub mod memos;

/// ByteStash code snippet client.
#[cfg(feature = "bytestash")]
pub mod bytestash;

/// Paperless-ngx document management client.
#[cfg(feature = "paperless")]
pub mod paperless;

/// Arcane Docker management UI client.
#[cfg(feature = "arcane")]
pub mod arcane;

/// Unraid GraphQL API client.
#[cfg(feature = "unraid")]
pub mod unraid;

/// UniFi Network Application local API client.
#[cfg(feature = "unifi")]
pub mod unifi;

/// Gotify push-notification server client.
#[cfg(feature = "gotify")]
pub mod gotify;

/// OpenAI API client (chat, embeddings, models, images, audio).
#[cfg(feature = "openai")]
pub mod openai;

/// Qdrant vector database client.
#[cfg(feature = "qdrant")]
pub mod qdrant;

/// Hugging Face Text Embeddings Inference (TEI) client.
#[cfg(feature = "tei")]
pub mod tei;

/// Apprise universal notification dispatcher client.
#[cfg(feature = "apprise")]
pub mod apprise;
