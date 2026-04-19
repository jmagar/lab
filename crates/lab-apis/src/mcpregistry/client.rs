//! `McpRegistryClient` — read-only MCP Registry API client.

use std::time::Duration;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::redirect;

use crate::core::{ApiError, Auth, HttpClient};

use super::error::RegistryError;
use super::types::{
    HealthBody, ListServersParams, ServerJSON, ServerListResponse, ServerResponse, ValidationResult,
};

/// Default registry base URL; overridden by `MCPREGISTRY_URL` env var.
pub const REGISTRY_DEFAULT_URL: &str = "https://registry.modelcontextprotocol.io";

/// Path-segment encoding set: encodes everything except `A-Za-z0-9`, `-`, `.`, `_`, `~`.
///
/// This preserves RFC 3986 unreserved characters (including hyphens and dots that
/// appear in reverse-DNS server names) while correctly percent-encoding `/` so that
/// names like `io.github.user/my-server` survive path-segment round-trips.
const PATH_SEGMENT: AsciiSet = NON_ALPHANUMERIC.remove(b'-').remove(b'.').remove(b'_').remove(b'~');

/// Client for the official MCP Registry at <https://registry.modelcontextprotocol.io>.
///
/// All endpoints are unauthenticated read-only (except POST /validate which is
/// also unauthenticated). Uses a custom `reqwest::Client` with:
/// - 20 s request timeout (fires before the axum 30 s `TimeoutLayer`)
/// - 5 s connect timeout
/// - No redirect following (prevents SSRF via registry-hosted redirect chains)
pub struct McpRegistryClient {
    http: HttpClient,
}

impl McpRegistryClient {
    /// Construct a new client targeting `base_url`.
    ///
    /// # Errors
    /// Returns [`RegistryError::Api`] wrapping [`ApiError::Internal`] if the TLS
    /// backend fails to initialise.
    pub fn new(base_url: &str, _auth: Auth) -> Result<Self, RegistryError> {
        let inner = reqwest::Client::builder()
            .user_agent(concat!("lab-apis/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .redirect(redirect::Policy::none())
            .build()
            .map_err(|e| {
                RegistryError::Api(ApiError::Internal(format!("reqwest::Client::build: {e}")))
            })?;
        Ok(Self {
            http: HttpClient::from_parts(base_url, Auth::None, inner),
        })
    }

    fn encode_name(name: &str) -> String {
        utf8_percent_encode(name, &PATH_SEGMENT).to_string()
    }

    fn validate_name(name: &str) -> Result<(), RegistryError> {
        if name.trim().is_empty() {
            return Err(RegistryError::InvalidInput {
                message: "server name must not be empty".to_string(),
            });
        }
        Ok(())
    }

    /// List MCP servers from the registry with optional filtering and pagination.
    ///
    /// Maps to `GET /v0.1/servers`.
    ///
    /// # Errors
    /// Returns [`RegistryError`] on HTTP or decode failure.
    pub async fn list_servers(
        &self,
        params: ListServersParams,
    ) -> Result<ServerListResponse, RegistryError> {
        let query = params.to_query_pairs();
        Ok(self.http.get_json_query("/v0.1/servers", &query).await?)
    }

    /// Get a specific MCP server by name and version.
    ///
    /// Maps to `GET /v0.1/servers/{name}/versions/{version}`.
    /// Pass `version = "latest"` to fetch the most recent version.
    ///
    /// # Errors
    /// Returns [`RegistryError::InvalidInput`] if `name` is empty.
    /// Returns [`RegistryError::Api`] on HTTP or decode failure.
    pub async fn get_server(
        &self,
        name: &str,
        version: &str,
    ) -> Result<ServerResponse, RegistryError> {
        Self::validate_name(name)?;
        let encoded = Self::encode_name(name);
        let path = format!("/v0.1/servers/{encoded}/versions/{version}");
        Ok(self.http.get_json(&path).await?)
    }

    /// List all versions of a specific MCP server.
    ///
    /// Maps to `GET /v0.1/servers/{name}/versions`.
    ///
    /// # Errors
    /// Returns [`RegistryError::InvalidInput`] if `name` is empty.
    /// Returns [`RegistryError::Api`] on HTTP or decode failure.
    pub async fn list_versions(&self, name: &str) -> Result<ServerListResponse, RegistryError> {
        Self::validate_name(name)?;
        let encoded = Self::encode_name(name);
        let path = format!("/v0.1/servers/{encoded}/versions");
        Ok(self.http.get_json(&path).await?)
    }

    /// Validate a server JSON without publishing it to the registry.
    ///
    /// Maps to `POST /v0.1/validate`.
    ///
    /// # Errors
    /// Returns [`RegistryError::Api`] on HTTP or decode failure.
    pub async fn validate(&self, server_json: &ServerJSON) -> Result<ValidationResult, RegistryError> {
        Ok(self.http.post_json("/v0.1/validate", server_json).await?)
    }

    /// Raw health probe — returns the health body for the registry.
    pub(super) async fn health_probe(&self) -> Result<HealthBody, RegistryError> {
        Ok(self.http.get_json("/v0.1/health").await?)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path, path_regex, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn fixture_server_list() -> serde_json::Value {
        serde_json::json!({
            "servers": [
                {
                    "server": {
                        "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
                        "name": "io.github.user/weather",
                        "description": "Weather MCP server",
                        "version": "1.0.0",
                        "packages": [],
                        "remotes": [{ "type": "streamable-http", "url": "https://api.example.com/mcp" }]
                    },
                    "_meta": {
                        "io.modelcontextprotocol.registry/official": {
                            "isLatest": true,
                            "publishedAt": "2025-01-01T00:00:00Z",
                            "status": "active",
                            "statusChangedAt": "2025-01-01T00:00:00Z"
                        }
                    }
                }
            ],
            "metadata": { "count": 1 }
        })
    }

    fn make_client(base_url: &str) -> McpRegistryClient {
        McpRegistryClient::new(base_url, Auth::None).expect("client construction should succeed")
    }

    #[tokio::test]
    async fn test_list_servers_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v0.1/servers"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture_server_list()))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client.list_servers(ListServersParams::default()).await.unwrap();
        assert_eq!(result.servers.len(), 1);
        assert_eq!(result.metadata.count, 1);
        assert_eq!(result.servers[0].server.name, "io.github.user/weather");
    }

    #[tokio::test]
    async fn test_get_server_encodes_name_in_path() {
        let server = MockServer::start().await;
        // Name "io.github.user/weather" must have the "/" encoded as "%2F"
        Mock::given(method("GET"))
            .and(path("/v0.1/servers/io.github.user%2Fweather/versions/latest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "server": {
                    "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
                    "name": "io.github.user/weather",
                    "description": "Weather MCP server",
                    "version": "1.0.0"
                },
                "_meta": {}
            })))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client.get_server("io.github.user/weather", "latest").await;
        assert!(result.is_ok(), "expected Ok, got: {result:?}");
    }

    #[tokio::test]
    async fn test_get_server_latest() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/v0\.1/servers/.+/versions/latest$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "server": {
                    "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
                    "name": "io.github.user/weather",
                    "description": "Weather MCP server",
                    "version": "1.0.0"
                },
                "_meta": {}
            })))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client.get_server("io.github.user/weather", "latest").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().server.version, "1.0.0");
    }

    #[tokio::test]
    async fn test_get_server_empty_name_returns_error() {
        let server = MockServer::start().await;
        // No mock mounted — any HTTP call would fail the test via wiremock's strict mode

        let client = make_client(&server.uri());
        let result = client.get_server("", "latest").await;
        assert!(
            matches!(result, Err(RegistryError::InvalidInput { .. })),
            "expected InvalidInput error, got: {result:?}"
        );
        // Wiremock will assert zero requests were received when server drops
    }

    #[tokio::test]
    async fn test_get_server_whitespace_only_name_returns_error() {
        let server = MockServer::start().await;
        let client = make_client(&server.uri());
        let result = client.get_server("   ", "latest").await;
        assert!(
            matches!(result, Err(RegistryError::InvalidInput { .. })),
            "expected InvalidInput error, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_get_server_no_redirects_followed() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/v0\.1/servers/.+/versions/latest$"))
            .respond_with(ResponseTemplate::new(301).insert_header("Location", "http://evil.internal/"))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client.get_server("io.github.user/weather", "latest").await;
        // Should return an error (3xx treated as non-success), not follow the redirect
        assert!(result.is_err(), "expected error on redirect, got Ok");
    }

    #[tokio::test]
    async fn test_get_server_404_maps_to_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/v0\.1/servers/.+/versions/latest$"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client.get_server("io.github.user/weather", "latest").await;
        assert!(
            matches!(result, Err(RegistryError::Api(ApiError::NotFound))),
            "expected NotFound error, got: {result:?}"
        );
    }

    #[tokio::test]
    async fn test_list_versions_round_trips() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v0.1/servers/io.github.user%2Fweather/versions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture_server_list()))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client.list_versions("io.github.user/weather").await.unwrap();
        assert_eq!(result.servers.len(), 1);
    }

    #[tokio::test]
    async fn test_validate_posts_server_json() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v0.1/validate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "valid": true,
                "issues": []
            })))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let server_json = ServerJSON {
            schema: Some("https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json".into()),
            name: "io.github.user/weather".into(),
            description: "Weather MCP server".into(),
            version: "1.0.0".into(),
            title: None,
            packages: vec![],
            remotes: vec![],
            repository: None,
            icons: vec![],
            website_url: None,
        };
        let result = client.validate(&server_json).await.unwrap();
        assert!(result.valid);
        assert!(result.issues.is_empty());
    }

    #[tokio::test]
    async fn test_response_meta_dotted_key_deserializes() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/v0\.1/servers/.+/versions/latest$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "server": {
                    "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
                    "name": "io.github.user/weather",
                    "description": "Weather MCP server",
                    "version": "1.0.0"
                },
                "_meta": {
                    "io.modelcontextprotocol.registry/official": {
                        "isLatest": true,
                        "publishedAt": "2025-01-01T00:00:00Z",
                        "status": "active",
                        "statusChangedAt": "2025-01-01T00:00:00Z"
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client.get_server("io.github.user/weather", "latest").await.unwrap();
        let official = result
            .meta
            .as_ref()
            .and_then(|m| m.official.as_ref())
            .expect("official extension should be present");
        assert_eq!(official.status, "active");
        assert!(official.is_latest);
    }

    #[tokio::test]
    async fn test_unknown_fields_ignored() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r"^/v0\.1/servers/.+/versions/latest$"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "server": {
                    "$schema": "https://static.modelcontextprotocol.io/schemas/2025-12-11/server.schema.json",
                    "name": "io.github.user/weather",
                    "description": "Weather MCP server",
                    "version": "1.0.0",
                    "future_field_not_in_spec": "should be ignored",
                    "another_unknown": 42
                },
                "_meta": {},
                "extra_top_level": "ignored"
            })))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client.get_server("io.github.user/weather", "latest").await;
        assert!(result.is_ok(), "unknown fields should be silently ignored: {result:?}");
    }

    #[tokio::test]
    async fn test_list_servers_with_search_query_param() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v0.1/servers"))
            .and(query_param("search", "postgres"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "servers": [],
                "metadata": { "count": 0 }
            })))
            .mount(&server)
            .await;

        let client = make_client(&server.uri());
        let result = client
            .list_servers(ListServersParams {
                search: Some("postgres".into()),
                ..Default::default()
            })
            .await
            .unwrap();
        assert_eq!(result.metadata.count, 0);
    }
}
