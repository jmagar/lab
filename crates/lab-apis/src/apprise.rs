//! Apprise notification dispatcher client.
//!
//! Apprise is a universal notification library that speaks to 100+ upstream
//! services (Slack, Discord, Telegram, Gotify, Pushover, email, webhooks,
//! etc.) behind a single unified URL scheme (e.g. `slack://`, `tgram://`).
//!
//! This module targets `apprise-api` — the Flask HTTP wrapper maintained at
//! `github.com/caronc/apprise-api`. It exposes a small REST surface:
//!
//! - `POST /notify` — stateless: caller supplies URLs + payload in one call
//! - `POST /notify/{key}` — stateful: send via URLs persisted under `key`
//! - `POST /add/{key}` — persist a config (URL list or YAML/TEXT blob)
//! - `GET  /get/{key}` — retrieve the config for a key
//! - `POST /del/{key}` — delete a key
//!
//! Spec: `docs/api-specs/apprise.md` (mirrored from the upstream README —
//! apprise-api does not publish an OpenAPI document).
//!
//! Auth is optional: apprise-api can run unauthenticated, or behind a reverse
//! proxy that injects basic-auth / bearer headers.

/// Public request/response types (serde).
pub mod types;

/// `AppriseError` (thiserror).
pub mod error;

/// `AppriseClient` — notify, persist configs, dispatch.
pub mod client;

pub use client::AppriseClient;
pub use error::AppriseError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};

/// Compile-time metadata for the apprise module.
pub const META: PluginMeta = PluginMeta {
    name: "apprise",
    display_name: "Apprise",
    description: "Universal notification dispatcher (100+ services behind one URL scheme)",
    category: Category::Notifications,
    docs_url: "https://github.com/caronc/apprise-api",
    required_env: &[EnvVar {
        name: "APPRISE_URL",
        description: "Base URL of the apprise-api server",
        example: "http://localhost:8000",
        secret: false,
    }],
    optional_env: &[EnvVar {
        name: "APPRISE_TOKEN",
        description: "Bearer token if behind auth proxy",
        example: "abc123...",
        secret: true,
    }],
    default_port: Some(8000),
};
