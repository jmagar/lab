//! Shared application state for axum handlers.
//!
//! Holds pre-constructed `lab-apis` clients so handlers don't rebuild
//! HTTP clients per request. Cloned per handler invocation — all fields
//! should be cheap to clone (typically `Arc`-wrapped internally).

use std::sync::Arc;

#[cfg(feature = "radarr")]
use lab_apis::radarr::RadarrClient;

/// Application state passed to every axum handler via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    #[cfg(feature = "radarr")]
    radarr: Option<RadarrClient>,
}

impl AppState {
    /// Construct a new `AppState` by reading env vars for each enabled service.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                #[cfg(feature = "radarr")]
                radarr: crate::mcp::services::radarr::client_from_env(),
            }),
        }
    }

    /// Borrow the optional Radarr client.
    #[cfg(feature = "radarr")]
    #[must_use]
    pub fn radarr(&self) -> Option<&RadarrClient> {
        self.inner.radarr.as_ref()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
