//! Error type for Neo4j.

use crate::core::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum Neo4jError {
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error("invalid parameter: {0}")]
    InvalidParam(String),
    #[error("insecure or unsupported Neo4j URI scheme: {0}")]
    InsecureScheme(String),
    #[error("Neo4j query timed out after {0}s")]
    Timeout(u64),
}
