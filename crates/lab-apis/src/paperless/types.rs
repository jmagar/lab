//! Paperless-ngx request/response types.
//!
//! TODO: add document, correspondent, tag, and document type types from the Paperless-ngx API spec.

use serde::{Deserialize, Serialize};

/// A Paperless-ngx document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: u64,
    pub title: String,
    pub content: Option<String>,
    pub created: String,
    pub modified: String,
    pub added: String,
    pub correspondent: Option<u64>,
    #[serde(default)]
    pub tags: Vec<u64>,
}
