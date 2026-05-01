//! Neo4j Bolt service module.

pub mod client;
pub mod error;
pub mod types;

pub use client::{Neo4jClient, Neo4jConfig, SanitizedUri};
pub use error::Neo4jError;

use crate::core::error::ApiError;
use crate::core::plugin::{Category, EnvVar, PluginMeta};
use crate::core::plugin_ui::{SECRET_OPTIONAL_FIELD, TEXT_OPTIONAL_FIELD, URL_FIELD};
use crate::core::status::ServiceStatus;
use crate::core::traits::ServiceClient;

/// Compile-time metadata for the Neo4j service.
pub const META: PluginMeta = PluginMeta {
    name: "neo4j",
    display_name: "Neo4j",
    description: "Neo4j graph database over the Bolt protocol",
    category: Category::Ai,
    docs_url: "https://neo4j.com/docs/",
    required_env: &[
        EnvVar {
            name: "NEO4J_URL",
            description: "Bolt URL for Neo4j",
            example: "neo4j://localhost:7687",
            secret: false,
            ui: Some(&URL_FIELD),
        },
        EnvVar {
            name: "NEO4J_USER",
            description: "Neo4j username",
            example: "neo4j",
            secret: false,
            ui: Some(&TEXT_OPTIONAL_FIELD),
        },
        EnvVar {
            name: "NEO4J_PASSWORD",
            description: "Neo4j password",
            example: "change-me",
            secret: true,
            ui: Some(&SECRET_OPTIONAL_FIELD),
        },
    ],
    optional_env: &[
        EnvVar {
            name: "NEO4J_DB",
            description: "Default database name",
            example: "neo4j",
            secret: false,
            ui: Some(&TEXT_OPTIONAL_FIELD),
        },
        EnvVar {
            name: "NEO4J_POOL_SIZE",
            description: "Maximum Bolt connection pool size",
            example: "16",
            secret: false,
            ui: Some(&TEXT_OPTIONAL_FIELD),
        },
        EnvVar {
            name: "NEO4J_CA_CERT_PATH",
            description: "Optional CA certificate path for self-signed TLS",
            example: "/etc/ssl/neo4j-ca.pem",
            secret: false,
            ui: Some(&TEXT_OPTIONAL_FIELD),
        },
    ],
    default_port: Some(7687),
    supports_multi_instance: false,
};

impl ServiceClient for Neo4jClient {
    fn name(&self) -> &'static str {
        "neo4j"
    }

    fn service_type(&self) -> &'static str {
        "database"
    }

    async fn health(&self) -> Result<ServiceStatus, ApiError> {
        self.server_status()
            .await
            .map_err(|e| ApiError::Network(e.to_string()))?;
        Ok(ServiceStatus {
            reachable: true,
            auth_ok: true,
            version: None,
            latency_ms: 0,
            message: None,
        })
    }
}
