//! Language types.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around a language id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LanguageId(pub i32);

/// A language known to an *arr service (for audio / subtitle tracking).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Language {
    pub id: LanguageId,
    pub name: String,
    #[serde(default)]
    pub name_lower: Option<String>,
}
