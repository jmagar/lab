use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SurfaceStateView {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub connected: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SurfaceStatesView {
    #[serde(default)]
    pub cli: SurfaceStateView,
    #[serde(default)]
    pub api: SurfaceStateView,
    #[serde(default)]
    pub mcp: SurfaceStateView,
    #[serde(default)]
    pub webui: SurfaceStateView,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerWarningView {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerConfigSummaryView {
    #[serde(default)]
    pub transport: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerView {
    pub id: String,
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub configured: bool,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub connected: bool,
    #[serde(default)]
    pub surfaces: SurfaceStatesView,
    #[serde(default)]
    pub warnings: Vec<ServerWarningView>,
    #[serde(default)]
    pub config_summary: ServerConfigSummaryView,
}
