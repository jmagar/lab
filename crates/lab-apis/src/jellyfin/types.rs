//! Jellyfin request and response types.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// System information returned by `GET /System/Info`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SystemInfo {
    pub id: Option<String>,
    pub server_name: Option<String>,
    pub version: Option<String>,
    pub product_name: Option<String>,
    pub operating_system: Option<String>,
    pub operating_system_display_name: Option<String>,
    pub startup_wizard_completed: Option<bool>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Public system information returned by `GET /System/Info/Public`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PublicSystemInfo {
    pub id: Option<String>,
    pub server_name: Option<String>,
    pub version: Option<String>,
    pub product_name: Option<String>,
    pub operating_system: Option<String>,
    pub local_address: Option<String>,
    pub startup_wizard_completed: Option<bool>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Jellyfin user summary returned by user read endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct User {
    pub id: Option<String>,
    pub name: Option<String>,
    pub server_id: Option<String>,
    pub server_name: Option<String>,
    pub last_login_date: Option<String>,
    pub last_activity_date: Option<String>,
    pub has_password: Option<bool>,
    pub has_configured_password: Option<bool>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Virtual folder/library summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct VirtualFolder {
    pub name: Option<String>,
    pub collection_type: Option<String>,
    pub item_id: Option<String>,
    pub locations: Option<Vec<String>>,
    pub refresh_status: Option<String>,
    pub refresh_progress: Option<f64>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Jellyfin item summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Item {
    pub id: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "Type")]
    pub item_type: Option<String>,
    pub server_id: Option<String>,
    pub parent_id: Option<String>,
    pub collection_type: Option<String>,
    pub is_folder: Option<bool>,
    pub run_time_ticks: Option<i64>,
    pub production_year: Option<i32>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Result wrapper returned by `GET /Items`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemsResult {
    #[serde(default)]
    pub items: Vec<Item>,
    pub total_record_count: Option<u32>,
    pub start_index: Option<u32>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Counts returned by `GET /Items/Counts`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ItemCounts {
    pub movie_count: Option<u32>,
    pub series_count: Option<u32>,
    pub episode_count: Option<u32>,
    pub song_count: Option<u32>,
    pub album_count: Option<u32>,
    pub artist_count: Option<u32>,
    pub item_count: Option<u32>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Active session summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionInfo {
    pub id: Option<String>,
    pub user_id: Option<String>,
    pub user_name: Option<String>,
    pub client: Option<String>,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub application_version: Option<String>,
    pub is_active: Option<bool>,
    pub last_activity_date: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Installed plugin summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PluginInfo {
    pub id: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub has_image: Option<bool>,
    pub can_uninstall: Option<bool>,
    #[serde(flatten)]
    pub extra: Value,
}

/// Supported bounded `GET /Items` query parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemsQuery {
    pub user_id: Option<String>,
    pub search_term: Option<String>,
    pub parent_id: Option<String>,
    pub include_item_types: Option<Vec<String>>,
    pub recursive: Option<bool>,
    pub start_index: Option<u32>,
    pub limit: Option<u32>,
}

impl ItemsQuery {
    pub(crate) fn to_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        if let Some(value) = &self.user_id {
            pairs.push(("userId".to_string(), value.clone()));
        }
        if let Some(value) = &self.search_term {
            pairs.push(("searchTerm".to_string(), value.clone()));
        }
        if let Some(value) = &self.parent_id {
            pairs.push(("parentId".to_string(), value.clone()));
        }
        if let Some(values) = &self.include_item_types
            && !values.is_empty()
        {
            pairs.push(("includeItemTypes".to_string(), values.join(",")));
        }
        if let Some(value) = self.recursive {
            pairs.push(("recursive".to_string(), value.to_string()));
        }
        if let Some(value) = self.start_index {
            pairs.push(("startIndex".to_string(), value.to_string()));
        }
        if let Some(value) = self.limit {
            pairs.push(("limit".to_string(), value.to_string()));
        }
        pairs
    }
}
