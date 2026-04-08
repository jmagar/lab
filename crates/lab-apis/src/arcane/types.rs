//! Arcane request/response types.
//!
//! TODO: add container, image, volume, and network types from the Arcane API spec.

use serde::{Deserialize, Serialize};

/// A Docker container as reported by Arcane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
}
