//! Public marketplace types. Serde shapes match the gateway-admin TS types.

use serde::{Deserialize, Serialize};

/// Marketplace source kind. Matches `MarketplaceSource` on the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginSource {
    Github,
    Git,
    Local,
}

/// A configured marketplace (local JSON file or remote repo).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marketplace {
    pub id: String,
    pub name: String,
    pub owner: String,
    #[serde(rename = "ghUser")]
    pub gh_user: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    pub source: PluginSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub desc: String,
    #[serde(rename = "autoUpdate")]
    pub auto_update: bool,
    #[serde(rename = "totalPlugins")]
    pub total_plugins: u32,
    #[serde(rename = "lastUpdated")]
    pub last_updated: String,
}

/// A plugin entry within a marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    /// Marketplace id this plugin belongs to.
    pub mkt: String,
    pub ver: String,
    pub desc: String,
    pub tags: Vec<String>,
    pub installed: bool,
    #[serde(rename = "hasUpdate", skip_serializing_if = "Option::is_none")]
    pub has_update: Option<bool>,
    #[serde(rename = "installedAt", skip_serializing_if = "Option::is_none")]
    pub installed_at: Option<String>,
    #[serde(rename = "updatedAt", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Syntax-highlight hint for plugin artifact files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArtifactLang {
    Json,
    Yaml,
    Markdown,
    Bash,
    Toml,
    Text,
}

/// A single file shipped with an installed plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub path: String,
    pub lang: ArtifactLang,
    pub content: String,
}
