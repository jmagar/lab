//! Config types.
//!
//! Each `*Config` type here maps to one endpoint under `/api/v3/config/*`.
//! When this file grows, nest into `types/config/{host,naming,ui,…}.rs`
//! the same way the client side splits.

use serde::{Deserialize, Serialize};

/// Host config — bind address, URL base, auth mode, SSL.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostConfig {
    pub id: i64,
    pub bind_address: String,
    pub port: i32,
    #[serde(default)]
    pub url_base: Option<String>,
    pub authentication_method: String,
    #[serde(default)]
    pub authentication_required: Option<String>,
    pub analytics_enabled: bool,
    #[serde(default)]
    pub instance_name: Option<String>,
    pub log_level: String,
    pub launch_browser: bool,
}

/// Naming config — movie filename and folder templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NamingConfig {
    pub id: i64,
    pub rename_movies: bool,
    pub replace_illegal_characters: bool,
    pub colon_replacement_format: Option<String>,
    #[serde(default)]
    pub standard_movie_format: Option<String>,
    #[serde(default)]
    pub movie_folder_format: Option<String>,
}

/// UI config — theme, date/time format, first day of week.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiConfig {
    pub id: i64,
    #[serde(default)]
    pub first_day_of_week: i32,
    #[serde(default)]
    pub calendar_week_column_header: Option<String>,
    #[serde(default)]
    pub movie_runtime_format: Option<String>,
    #[serde(default)]
    pub short_date_format: Option<String>,
    #[serde(default)]
    pub long_date_format: Option<String>,
    #[serde(default)]
    pub time_format: Option<String>,
    #[serde(default)]
    pub show_relative_dates: bool,
    #[serde(default)]
    pub enable_color_impaired_mode: bool,
    #[serde(default)]
    pub theme: Option<String>,
}
