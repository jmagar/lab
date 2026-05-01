//! NotebookLM client.
//!
//! This wraps Google NotebookLM's undocumented batchexecute RPC surface using
//! the request shapes documented by `teng-lin/notebooklm-py`.

pub mod client;
pub mod error;
pub mod types;

pub use client::NotebookLmClient;
pub use error::NotebookLmError;

use std::time::Instant;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_FIELD, TEXT_OPTIONAL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

pub const META: PluginMeta = PluginMeta {
    name: "notebooklm",
    display_name: "NotebookLM",
    description: "Google NotebookLM notebooks, sources, and research artifacts",
    category: Category::Ai,
    docs_url: "https://github.com/teng-lin/notebooklm-py",
    required_env: &[],
    optional_env: &[
        EnvVar {
            name: "NOTEBOOKLM_AUTH_JSON",
            description: "Inline Playwright storage_state.json containing Google session cookies",
            example: "{\"cookies\":[...]}",
            secret: true,
            ui: Some(&SECRET_FIELD),
        },
        EnvVar {
            name: "NOTEBOOKLM_STORAGE",
            description: "Path to a NotebookLM Playwright storage_state.json file",
            example: "~/.notebooklm/profiles/default/storage_state.json",
            secret: false,
            ui: Some(&TEXT_OPTIONAL_FIELD),
        },
        EnvVar {
            name: "NOTEBOOKLM_HOME",
            description: "NotebookLM config directory used when NOTEBOOKLM_STORAGE is not set",
            example: "~/.notebooklm",
            secret: false,
            ui: Some(&TEXT_OPTIONAL_FIELD),
        },
        EnvVar {
            name: "NOTEBOOKLM_PROFILE",
            description: "NotebookLM profile name under NOTEBOOKLM_HOME/profiles",
            example: "default",
            secret: false,
            ui: Some(&TEXT_OPTIONAL_FIELD),
        },
    ],
    default_port: None,
    supports_multi_instance: false,
};

impl ServiceClient for NotebookLmClient {
    fn name(&self) -> &'static str {
        "notebooklm"
    }

    fn service_type(&self) -> &'static str {
        "ai"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        let start = Instant::now();
        match self.list_notebooks().await {
            Ok(_) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: true,
                version: None,
                #[allow(clippy::cast_possible_truncation)]
                latency_ms: start.elapsed().as_millis() as u64,
                message: None,
            }),
            Err(NotebookLmError::Api(ApiError::Auth)) => Ok(ServiceStatus {
                reachable: true,
                auth_ok: false,
                version: None,
                #[allow(clippy::cast_possible_truncation)]
                latency_ms: start.elapsed().as_millis() as u64,
                message: Some("authentication failed".to_string()),
            }),
            Err(NotebookLmError::Api(ApiError::Network(e))) => Ok(ServiceStatus::unreachable(e)),
            Err(e) => Ok(ServiceStatus::degraded(e.to_string())),
        }
    }
}
