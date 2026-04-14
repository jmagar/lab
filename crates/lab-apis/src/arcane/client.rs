//! `ArcaneClient` — Docker management methods.
//!
//! Arcane is a multi-environment Docker management UI. Environments represent
//! Docker hosts. Containers are scoped per environment.

use crate::core::{Auth, HttpClient};

use super::error::ArcaneError;
use super::types::{
    ApiResponse, Container, ContainerActionResult, Environment, HealthResponse, Image,
    ImagePruneResult, ImagePullResult, ImageUpdateSummary, PaginatedResponse, Project,
    ProjectActionResult, PruneResult, Volume,
};

/// Client for an Arcane Docker management UI instance.
pub struct ArcaneClient {
    http: HttpClient,
}

impl ArcaneClient {
    fn normalize_base_url(base_url: &str) -> String {
        let trimmed = base_url.trim_end_matches('/');
        if trimmed.ends_with("/api") {
            trimmed.to_string()
        } else {
            format!("{trimmed}/api")
        }
    }

    /// Build a client against `base_url` with the given auth.
    ///
    /// # Errors
    /// Returns [`ArcaneError::Api`] if the TLS backend fails to initialise.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, ArcaneError> {
        Ok(Self {
            http: HttpClient::new(Self::normalize_base_url(base_url), auth)?,
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
            .get_json(&format!("/environments/{env_id}/containers/{container_id}"))
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

    // ── Projects ──────────────────────────────────────────────────────────────

    /// List all projects for an environment (`GET /environments/{envId}/projects`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn projects_list(&self, env_id: &str) -> Result<Vec<Project>, ArcaneError> {
        let resp: PaginatedResponse<Project> = self
            .http
            .get_json(&format!("/environments/{env_id}/projects"))
            .await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Create a project in an environment (`POST /environments/{envId}/projects`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn project_create(
        &self,
        env_id: &str,
        body: &serde_json::Value,
    ) -> Result<Project, ArcaneError> {
        let resp: ApiResponse<Project> = self
            .http
            .post_json(&format!("/environments/{env_id}/projects"), body)
            .await?;
        Ok(resp.data)
    }

    /// Bring a project up (`POST /environments/{envId}/projects/{projId}/up`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn project_up(
        &self,
        env_id: &str,
        project_id: &str,
    ) -> Result<ProjectActionResult, ArcaneError> {
        let resp: ApiResponse<ProjectActionResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/projects/{project_id}/up"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }

    /// Bring a project down (`POST /environments/{envId}/projects/{projId}/down`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn project_down(
        &self,
        env_id: &str,
        project_id: &str,
    ) -> Result<ProjectActionResult, ArcaneError> {
        let resp: ApiResponse<ProjectActionResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/projects/{project_id}/down"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }

    /// Redeploy a project (`POST /environments/{envId}/projects/{projId}/redeploy`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn project_redeploy(
        &self,
        env_id: &str,
        project_id: &str,
    ) -> Result<ProjectActionResult, ArcaneError> {
        let resp: ApiResponse<ProjectActionResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/projects/{project_id}/redeploy"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }

    // ── Volumes ───────────────────────────────────────────────────────────────

    /// List volumes for an environment (`GET /environments/{envId}/volumes`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn volumes_list(&self, env_id: &str) -> Result<Vec<Volume>, ArcaneError> {
        let resp: PaginatedResponse<Volume> = self
            .http
            .get_json(&format!("/environments/{env_id}/volumes"))
            .await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Delete a volume by name (`DELETE /environments/{envId}/volumes/{name}`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn volume_delete(&self, env_id: &str, name: &str) -> Result<(), ArcaneError> {
        self.http
            .delete(&format!("/environments/{env_id}/volumes/{name}"))
            .await?;
        Ok(())
    }

    /// Prune unused volumes (`POST /environments/{envId}/volumes/prune`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn volumes_prune(&self, env_id: &str) -> Result<PruneResult, ArcaneError> {
        let resp: ApiResponse<PruneResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/volumes/prune"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }

    // ── Images ────────────────────────────────────────────────────────────────

    /// List images for an environment (`GET /environments/{envId}/images`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn images_list(&self, env_id: &str) -> Result<Vec<Image>, ArcaneError> {
        let resp: PaginatedResponse<Image> = self
            .http
            .get_json(&format!("/environments/{env_id}/images"))
            .await?;
        Ok(resp.data.unwrap_or_default())
    }

    /// Pull an image (`POST /environments/{envId}/images/pull`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn image_pull(
        &self,
        env_id: &str,
        image: &str,
    ) -> Result<ImagePullResult, ArcaneError> {
        let body = serde_json::json!({ "image": image });
        let resp: ApiResponse<ImagePullResult> = self
            .http
            .post_json(&format!("/environments/{env_id}/images/pull"), &body)
            .await?;
        Ok(resp.data)
    }

    /// Prune unused images (`POST /environments/{envId}/images/prune`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn images_prune(&self, env_id: &str) -> Result<ImagePruneResult, ArcaneError> {
        let resp: ApiResponse<ImagePruneResult> = self
            .http
            .post_json(
                &format!("/environments/{env_id}/images/prune"),
                &serde_json::Value::Null,
            )
            .await?;
        Ok(resp.data)
    }

    /// Get image update summary (`GET /environments/{envId}/image-updates/summary`).
    ///
    /// # Errors
    /// Returns `ArcaneError::Api` on HTTP failure.
    pub async fn image_update_summary(
        &self,
        env_id: &str,
    ) -> Result<ImageUpdateSummary, ArcaneError> {
        let resp: ApiResponse<ImageUpdateSummary> = self
            .http
            .get_json(&format!("/environments/{env_id}/image-updates/summary"))
            .await?;
        Ok(resp.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client(base_url: &str) -> ArcaneClient {
        ArcaneClient::new(base_url, Auth::None).expect("client init")
    }

    #[tokio::test]
    async fn health_uses_api_prefix() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/health"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "UP"
            })))
            .mount(&server)
            .await;
        let c = client(&server.uri());
        let result = c.health().await.expect("should succeed");
        assert_eq!(result.status, "UP");
    }

    #[tokio::test]
    async fn projects_list_returns_projects() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/environments/env1/projects"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "id": "proj1", "name": "myproject" }]
            })))
            .mount(&server)
            .await;
        let c = client(&server.uri());
        let result = c.projects_list("env1").await.expect("should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "proj1");
    }

    #[tokio::test]
    async fn volumes_list_returns_volumes() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/environments/env1/volumes"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "name": "myvol" }]
            })))
            .mount(&server)
            .await;
        let c = client(&server.uri());
        let result = c.volumes_list("env1").await.expect("should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "myvol");
    }

    #[tokio::test]
    async fn images_list_returns_images() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/environments/env1/images"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": [{ "id": "sha256:abc" }]
            })))
            .mount(&server)
            .await;
        let c = client(&server.uri());
        let result = c.images_list("env1").await.expect("should succeed");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "sha256:abc");
    }

    #[tokio::test]
    async fn image_update_summary_returns_summary() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/environments/env1/image-updates/summary"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "data": { "updatesAvailable": 3 }
            })))
            .mount(&server)
            .await;
        let c = client(&server.uri());
        let result = c
            .image_update_summary("env1")
            .await
            .expect("should succeed");
        assert_eq!(result.updates_available, Some(3));
    }

    #[tokio::test]
    async fn volume_delete_ok() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/environments/env1/volumes/myvol"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&server)
            .await;
        let c = client(&server.uri());
        c.volume_delete("env1", "myvol")
            .await
            .expect("should succeed");
    }
}
