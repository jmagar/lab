//! Shared application state for axum handlers.
//!
//! Holds pre-constructed `lab-apis` clients so handlers don't rebuild
//! HTTP clients per request. Cloned per handler invocation — all fields
//! should be cheap to clone (typically `Arc`-wrapped internally).

use std::sync::Arc;

/// Application state passed to every axum handler via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    inner: Arc<AppStateInner>,
}

#[allow(dead_code)]
struct AppStateInner {
    // Service clients live here, e.g.:
    // #[cfg(feature = "radarr")]
    // radarr: lab_apis::radarr::RadarrClient,
}

impl AppState {
    /// Construct a new `AppState` from loaded config.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {}),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
