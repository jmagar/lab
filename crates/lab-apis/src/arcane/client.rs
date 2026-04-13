//! `ArcaneClient` — Docker management methods.
//!
//! Arcane is a multi-environment Docker management UI. Environments represent
//! Docker hosts. Containers are scoped per environment.

use crate::core::{Auth, HttpClient};

use super::error::ArcaneError;
use super::types::{
    ApiResponse, Container, ContainerActionResult, Environment, HealthResponse,
    PaginatedResponse,
};

/// Client for an Arcane Docker management UI instance.
pub struct ArcaneClient {
    http: HttpClient,
}

impl ArcaneClient {
    /// Build a client against `base_url` with the given auth.
    ///
    /// # Errors
    /// Returns [`ArcaneError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, ArcaneError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    // ── Health ────────────────────────────────────────────────────────────────

    /// Check Arcane API health (`GET /health`).
    ///
    /// Does not require auth per the Arcane API spec.
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn health(&self) -> Result<HealthResponse, ArcaneError> {
        let resp: HealthResponse = self.http.get_json("/health").await?;
        Ok(resp)
    }

    // ── Environments ──────────────────────────────────────────────────────────

    /// List all registered Docker environments (`GET /environments`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn environments_list(&self) -> Result<Vec<Environment>, ArcaneError> {
        let resp: PaginatedResponse<Environment> = self.http.get_json("/environments").await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Get a single environment by ID (`GET /environments/{id}`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP or decode failure.
    pub async fn environment_get(&self, id: &str) -> Result<Environment, ArcaneError> {
        let resp: ApiResponse<Environment> =
            self.http.get_json(&format!("/environments/{id}")).await?;
        Ok(resp.data)
    }

    // ── Containers ────────────────────────────────────────────────────────────

    /// List containers for an environment (`GET /environments/{id}/containers`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn containers_list(&self, env_id: &str) -> Result<Vec<Container>, ArcaneError> {
        let resp: PaginatedResponse<Container> = self
            .http
            .get_json(&format!("/environments/{env_id}/containers"))
            .await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Get a single container (`GET /environments/{id}/containers/{containerId}`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn container_get(
        &self,
        env_id: &str,
        container_id: &str,
    ) -> Result<Container, ArcaneError> {
        let resp: ApiResponse<Container> = self
            .http
            .get_json(&format!(
                "/environments/{env_id}/containers/{container_id}"
            ))
            .await?;
        Ok(resp.data)
    }

    /// Start a container (`POST /environments/{id}/containers/{containerId}/start`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn container_start(
        &self,
        env_id: &str,
        container_id: &str,
    ) -> Result<ContainerActionResult, ArcaneError> {
        let resp: ApiResponse<ContainerActionResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/containers/{container_id}/start"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }

    /// Stop a container (`POST /environments/{id}/containers/{containerId}/stop`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn container_stop(
        &self,
        env_id: &str,
        container_id: &str,
    ) -> Result<ContainerActionResult, ArcaneError> {
        let resp: ApiResponse<ContainerActionResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/containers/{container_id}/stop"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }

    /// Restart a container (`POST /environments/{id}/containers/{containerId}/restart`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn container_restart(
        &self,
        env_id: &str,
        container_id: &str,
    ) -> Result<ContainerActionResult, ArcaneError> {
        let resp: ApiResponse<ContainerActionResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/containers/{container_id}/restart"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }

    /// Redeploy a container (`POST /environments/{id}/containers/{containerId}/redeploy`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn container_redeploy(
        &self,
        env_id: &str,
        container_id: &str,
    ) -> Result<ContainerActionResult, ArcaneError> {
        let resp: ApiResponse<ContainerActionResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/containers/{container_id}/redeploy"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }
}
