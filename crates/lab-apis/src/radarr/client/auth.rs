//! Session auth endpoints.
//!
//! Covers `/login` and `/logout`. Most Radarr deployments use static
//! API-key auth (via the `X-Api-Key` header), which this client already
//! does through `HttpClient` — these endpoints are only used when Radarr
//! is configured with forms/basic auth and the caller needs to mint a
//! session cookie.

use super::RadarrClient;
use crate::radarr::error::RadarrError;
use crate::radarr::types::auth::LoginRequest;

impl RadarrClient {
    /// Exchange a username/password for a session cookie.
    ///
    /// Maps to `POST /login`. Rarely needed — prefer API-key auth.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn login(&self, req: &LoginRequest) -> Result<(), RadarrError> {
        self.http
            .post_void("/login", req)
            .await
            .map_err(RadarrError::from)
    }

    /// Invalidate the current session cookie.
    ///
    /// Maps to `GET /logout`.
    ///
    /// # Errors
    /// Returns `RadarrError::Api` on HTTP failure.
    pub async fn logout(&self) -> Result<(), RadarrError> {
        // Radarr invalidates the session via GET /logout; we discard the redirect body.
        self.http
            .get_void("/logout")
            .await
            .map_err(RadarrError::from)
    }
}
