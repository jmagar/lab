//! Top-level axum router builder.
//!
//! Composes feature-gated service routers under `/v1/<service>` and mounts
//! cross-cutting middleware (tracing, CORS, compression, timeout).

use std::time::Duration;

use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

use super::{error::ApiResult, health, state::AppState};

/// Build the full `lab` HTTP router with all enabled service route groups
/// and the standard middleware stack applied.
pub fn build_router(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready));

    #[cfg(feature = "radarr")]
    {
        router = router.route("/v1/radarr/system/status", get(radarr_system_status));
    }

    router
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
}

#[cfg(feature = "radarr")]
async fn radarr_system_status(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    let Some(client) = state.radarr() else {
        return Err(super::error::ApiError::UnknownInstance("radarr".into()));
    };
    let status = client.system_status().await.map_err(|e| match e {
        lab_apis::radarr::RadarrError::Api(sdk_err) => super::error::ApiError::Sdk(sdk_err),
        lab_apis::radarr::RadarrError::NotFound { kind, id } => {
            super::error::ApiError::UnknownAction(format!("{kind} id {id} not found"))
        }
    })?;
    Ok(Json(serde_json::to_value(status).unwrap_or_default()))
}
