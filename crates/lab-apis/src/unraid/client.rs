//! `UnraidClient` — async methods for the Unraid GraphQL API.
//!
//! All operations POST to `{base_url}/graphql` using the shared
//! `HttpClient::post_graphql` helper.
//!
//! # Rate limits
//! Unraid enforces approximately 100 requests per 10 seconds. This client
//! performs no in-process rate limiting — callers must respect this bound.

use serde_json::json;

use crate::core::{Auth, HttpClient};

use super::{
    UnraidError,
    types::{
        ArrayData, ArrayStatus, DiskInfo, DisksData, DockerContainer, DockerData, DockerStartData,
        DockerStopData, InfoData, MetricsData, OnlineData, SystemInfo, SystemMetrics,
    },
};

// ---------------------------------------------------------------------------
// GraphQL query / mutation strings
// ---------------------------------------------------------------------------

const QUERY_ONLINE: &str = "query { online }";

const QUERY_INFO: &str = r#"
query {
  info {
    id
    os { id hostname platform distro release kernel arch uptime }
    cpu { id brand manufacturer cores threads speed }
    system { id manufacturer model serial }
    versions { id core { unraid api kernel } }
  }
}
"#;

const QUERY_METRICS: &str = r#"
query {
  metrics {
    id
    cpu { id percentTotal }
    memory { id total used free percentTotal }
  }
}
"#;

const QUERY_ARRAY: &str = r#"
query {
  array {
    id
    state
    disks  { id name device size status temp type }
    parities { id name device size status temp type }
    caches { id name device size status temp type }
  }
}
"#;

const QUERY_DOCKER: &str = r#"
query {
  docker {
    id
    containers {
      id
      names
      image
      imageId
      state
      status
      autoStart
    }
  }
}
"#;

const QUERY_DISKS: &str = r#"
query {
  disks {
    id
    name
    device
    vendor
    size
    type
    smartStatus
    temperature
    serialNum
  }
}
"#;

const MUTATION_DOCKER_START: &str = r#"
mutation DockerStart($id: PrefixedID!) {
  docker {
    start(id: $id) {
      id names image imageId state status autoStart
    }
  }
}
"#;

const MUTATION_DOCKER_STOP: &str = r#"
mutation DockerStop($id: PrefixedID!) {
  docker {
    stop(id: $id) {
      id names image imageId state status autoStart
    }
  }
}
"#;

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Client for the Unraid GraphQL API.
pub struct UnraidClient {
    pub(crate) http: HttpClient,
}

impl UnraidClient {
    /// Construct a new client.
    ///
    /// Uses `Auth::ApiKey { header: "X-API-Key", key }` as required by the
    /// Unraid Connect API.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] if TLS initialisation fails.
    pub fn new(base_url: &str, auth: Auth) -> Result<Self, UnraidError> {
        Ok(Self {
            http: HttpClient::new(base_url, auth)?,
        })
    }

    /// Build a client from the default-instance environment variables.
    ///
    /// Reads `UNRAID_URL` and `UNRAID_API_KEY`. Returns `None` if either is
    /// absent or empty.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("UNRAID_URL").ok().filter(|v| !v.is_empty())?;
        let key = std::env::var("UNRAID_API_KEY")
            .ok()
            .filter(|v| !v.is_empty())?;
        Self::new(
            &url,
            Auth::ApiKey {
                header: "X-API-Key".into(),
                key,
            },
        )
        .ok()
    }

    // -----------------------------------------------------------------------
    // System queries
    // -----------------------------------------------------------------------

    /// Return `true` if the Unraid server considers itself online.
    ///
    /// Intended as a lightweight health probe.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn system_online(&self) -> Result<bool, UnraidError> {
        let data: OnlineData = self.http.post_graphql("/graphql", QUERY_ONLINE, None).await?;
        Ok(data.online)
    }

    /// Return detailed system information.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn system_info(&self) -> Result<SystemInfo, UnraidError> {
        let data: InfoData = self.http.post_graphql("/graphql", QUERY_INFO, None).await?;
        Ok(data.info)
    }

    /// Return current CPU and memory utilisation metrics.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn system_metrics(&self) -> Result<SystemMetrics, UnraidError> {
        let data: MetricsData = self
            .http
            .post_graphql("/graphql", QUERY_METRICS, None)
            .await?;
        Ok(data.metrics)
    }

    /// Return current array state and disk list.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn system_array(&self) -> Result<ArrayStatus, UnraidError> {
        let data: ArrayData = self.http.post_graphql("/graphql", QUERY_ARRAY, None).await?;
        Ok(data.array)
    }

    // -----------------------------------------------------------------------
    // Docker queries
    // -----------------------------------------------------------------------

    /// List all Docker containers.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn docker_list(&self) -> Result<Vec<DockerContainer>, UnraidError> {
        let data: DockerData = self
            .http
            .post_graphql("/graphql", QUERY_DOCKER, None)
            .await?;
        Ok(data.docker.containers)
    }

    /// Start a Docker container by its prefixed ID.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn docker_start(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id });
        let _data: DockerStartData = self
            .http
            .post_graphql("/graphql", MUTATION_DOCKER_START, Some(&vars))
            .await?;
        Ok(())
    }

    /// Stop a Docker container by its prefixed ID.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn docker_stop(&self, id: &str) -> Result<(), UnraidError> {
        let vars = json!({ "id": id });
        let _data: DockerStopData = self
            .http
            .post_graphql("/graphql", MUTATION_DOCKER_STOP, Some(&vars))
            .await?;
        Ok(())
    }

    /// Restart a Docker container by stopping then starting it.
    ///
    /// The Unraid GraphQL API does not expose a native `restart` mutation; this
    /// method calls `stop` followed by `start`.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn docker_restart(&self, id: &str) -> Result<(), UnraidError> {
        self.docker_stop(id).await?;
        self.docker_start(id).await?;
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Disk queries
    // -----------------------------------------------------------------------

    /// Return the list of physical disks attached to the server.
    ///
    /// # Errors
    /// Returns [`UnraidError::Http`] on transport or GraphQL error.
    pub async fn disk_list(&self) -> Result<Vec<DiskInfo>, UnraidError> {
        let data: DisksData = self
            .http
            .post_graphql("/graphql", QUERY_DISKS, None)
            .await?;
        Ok(data.disks)
    }
}
