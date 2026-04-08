//! Gotify push-notification server client.
//!
//! Gotify exposes a small REST API (Swagger 2.0) for sending messages,
//! managing applications, and managing clients. Auth is a header token —
//! `X-Gotify-Key` — scoped per-app (send) or per-client (read).
//!
//! Spec: `docs/api-specs/gotify.openapi.json` (mirrored from
//! `github.com/gotify/server/blob/master/docs/spec.json`).

/// Public request/response types (serde).
pub mod types;

/// `GotifyError` (thiserror).
pub mod error;

/// `GotifyClient` — send messages, manage apps/clients/messages.
pub mod client;

pub use client::GotifyClient;
pub use error::GotifyError;

use crate::core::plugin::{Category, EnvVar, PluginMeta};

/// Compile-time metadata for the gotify module.
pub const META: PluginMeta = PluginMeta {
    name: "gotify",
    display_name: "Gotify",
    description: "Self-hosted push notification server",
    category: Category::Notifications,
    docs_url: "https://gotify.net/api-docs",
    required_env: &[
        EnvVar {
            name: "GOTIFY_URL",
            description: "Base URL of the Gotify server",
            example: "http://localhost:8080",
            secret: false,
        },
        EnvVar {
            name: "GOTIFY_TOKEN",
            description: "App token from Gotify (X-Gotify-Key)",
            example: "A1b2C3d4E5...",
            secret: true,
        },
    ],
    optional_env: &[],
    default_port: Some(80),
};
