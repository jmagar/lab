//! Quality profile and definition types.

use serde::{Deserialize, Serialize};

/// Newtype wrapper around a quality-profile id.
///
/// Distinct from indexer / movie / command ids so the type system rejects
/// cross-wiring.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct QualityProfileId(pub i64);

/// A quality profile — the set of qualities an *arr service is willing to
/// accept, plus the upgrade rules between them.
///
/// Mirrors `QualityProfileResource` from the Radarr v3 / Sonarr v3 `OpenAPI` specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityProfile {
    pub id: QualityProfileId,
    pub name: String,
    pub upgrade_allowed: bool,
    pub cutoff: i32,
    #[serde(default)]
    pub items: serde_json::Value,
}

/// A quality definition — size/megabit rules per quality level.
///
/// Mirrors `QualityDefinitionResource` from the Radarr v3 / Sonarr v3 `OpenAPI` specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityDefinition {
    pub id: i64,
    pub title: String,
    pub weight: i32,
    pub min_size: Option<f64>,
    pub max_size: Option<f64>,
    pub preferred_size: Option<f64>,
}
