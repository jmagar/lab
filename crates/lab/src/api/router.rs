//! Top-level axum router builder.
//!
//! Composes feature-gated service routers under `/v1/<service>` and mounts
//! cross-cutting middleware (tracing, CORS, compression, timeout).

use std::time::Duration;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, timeout::TimeoutLayer,
    trace::TraceLayer,
};

use super::{health, state::AppState};

/// Build the full `lab` HTTP router with all enabled service route groups
/// and the standard middleware stack applied.
#[must_use]
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::GATEWAY_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(CompressionLayer::new())
        .layer(CorsLayer::permissive())
}
