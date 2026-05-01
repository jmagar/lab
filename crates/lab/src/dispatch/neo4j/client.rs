use std::path::PathBuf;

use lab_apis::neo4j::{Neo4jClient, Neo4jConfig};

use crate::dispatch::error::ToolError;
use crate::dispatch::helpers::env_non_empty;

const DEFAULT_POOL_SIZE: usize = 16;

/// Build a Neo4j client from env.
pub fn client_from_env() -> Option<Neo4jClient> {
    let url = env_non_empty("NEO4J_URL")?;
    let user = env_non_empty("NEO4J_USER")?;
    let password = env_non_empty("NEO4J_PASSWORD")?;
    let pool_size = env_non_empty("NEO4J_POOL_SIZE")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(DEFAULT_POOL_SIZE);
    let config = Neo4jConfig {
        url,
        user,
        password,
        database: env_non_empty("NEO4J_DB"),
        pool_size,
        ca_cert_path: env_non_empty("NEO4J_CA_CERT_PATH").map(PathBuf::from),
    };
    Neo4jClient::new(config).ok()
}

/// Return a client or a structured error distinguishing missing config from init failure.
pub fn require_client() -> Result<Neo4jClient, ToolError> {
    let url = env_non_empty("NEO4J_URL").ok_or_else(not_configured_error)?;
    let user = env_non_empty("NEO4J_USER").ok_or_else(not_configured_error)?;
    let password = env_non_empty("NEO4J_PASSWORD").ok_or_else(not_configured_error)?;
    let pool_size = env_non_empty("NEO4J_POOL_SIZE")
        .map(|value| {
            value.parse::<usize>().map_err(|_| ToolError::InvalidParam {
                message: "NEO4J_POOL_SIZE must be a positive integer".into(),
                param: "NEO4J_POOL_SIZE".into(),
            })
        })
        .transpose()?
        .unwrap_or(DEFAULT_POOL_SIZE);
    let config = Neo4jConfig {
        url,
        user,
        password,
        database: env_non_empty("NEO4J_DB"),
        pool_size,
        ca_cert_path: env_non_empty("NEO4J_CA_CERT_PATH").map(PathBuf::from),
    };
    Neo4jClient::new(config).map_err(ToolError::from)
}

/// Structured error returned when required env vars are absent.
pub fn not_configured_error() -> ToolError {
    ToolError::Sdk {
        sdk_kind: "internal_error".into(),
        message: "NEO4J_URL, NEO4J_USER, and NEO4J_PASSWORD are required".into(),
    }
}
