//! Integration tests for the upstream OAuth manager.
//!
//! These tests stand up a wiremock-backed OAuth authorization server and drive
//! `UpstreamOauthManager` against it. They exist to pin the wire-level behavior
//! of our outbound OAuth client — specifically the `resource` indicator on
//! authorize, issuer binding, S256 enforcement, and CIMD registration.

use base64::Engine;
use lab::config::{
    UpstreamConfig, UpstreamOauthConfig, UpstreamOauthMode, UpstreamOauthRegistration,
    canonicalize_upstream_url,
};
use lab::oauth::upstream::encryption::load_key;
use lab::oauth::upstream::manager::UpstreamOauthManager;
use lab::oauth::upstream::types::OauthError;
use lab_auth::sqlite::SqliteStore;
use serde_json::json;
use tempfile::TempDir;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------- harness ----------

struct Harness {
    mock: MockServer,
    _tmp: TempDir,
    sqlite: SqliteStore,
    key: lab::oauth::upstream::encryption::EncryptionKey,
}

impl Harness {
    async fn new() -> Self {
        let mock = MockServer::start().await;
        let tmp = TempDir::new().expect("tempdir");
        let sqlite = SqliteStore::open(tmp.path().join("auth.sqlite"))
            .await
            .expect("sqlite open");
        let key_b64 = base64::engine::general_purpose::STANDARD.encode([0u8; 32]);
        let key = load_key(&key_b64).expect("key");
        Self {
            mock,
            _tmp: tmp,
            sqlite,
            key,
        }
    }

    fn upstream_url(&self) -> String {
        self.mock.uri()
    }

    fn as_url(&self) -> String {
        self.mock.uri()
    }

    /// Mount an AS metadata document with the given fields. `issuer` and
    /// `code_challenge_methods_supported` are nullable so we can exercise
    /// degenerate configurations.
    async fn mount_metadata(
        &self,
        issuer: Option<&str>,
        methods: Option<&[&str]>,
        endpoint_override: Option<(&str, &str)>,
    ) {
        let (auth_ep, token_ep) = endpoint_override
            .map(|(a, t)| (a.to_string(), t.to_string()))
            .unwrap_or_else(|| {
                (
                    format!("{}/authorize", self.as_url()),
                    format!("{}/token", self.as_url()),
                )
            });
        let mut body = json!({
            "authorization_endpoint": auth_ep,
            "token_endpoint": token_ep,
        });
        if let Some(iss) = issuer {
            body["issuer"] = json!(iss);
        }
        if let Some(m) = methods {
            body["code_challenge_methods_supported"] = json!(m);
        }
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-authorization-server"))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&self.mock)
            .await;
    }

    /// Mock the resource-metadata probe to 404 so rmcp falls through to
    /// direct AS discovery at `.well-known/oauth-authorization-server`.
    async fn mount_no_resource_metadata(&self) {
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-protected-resource"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&self.mock)
            .await;
    }

    async fn mount_token_endpoint(&self) {
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "access-xyz",
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": "refresh-xyz",
                "scope": "read"
            })))
            .mount(&self.mock)
            .await;
    }

    fn upstream_cfg(&self, registration: UpstreamOauthRegistration) -> UpstreamConfig {
        UpstreamConfig {
            name: "test".into(),
            url: Some(self.upstream_url()),
            bearer_token_env: None,
            command: None,
            args: vec![],
            proxy_resources: false,
            expose_tools: None,
            oauth: Some(UpstreamOauthConfig {
                mode: UpstreamOauthMode::AuthorizationCodePkce,
                registration,
                scopes: Some(vec!["read".into()]),
            }),
        }
    }

    fn manager(&self, cfg: UpstreamConfig) -> UpstreamOauthManager {
        UpstreamOauthManager::new(
            self.sqlite.clone(),
            self.key.clone(),
            cfg,
            "https://lab.example/v1/gateway/oauth/callback".into(),
        )
    }
}

fn preregistered() -> UpstreamOauthRegistration {
    UpstreamOauthRegistration::Preregistered {
        client_id: "lab-client".into(),
        client_secret_env: None,
    }
}

// ---------- tests ----------

#[tokio::test]
async fn canonical_url_strips_default_port_and_lowercases_host() {
    assert_eq!(
        canonicalize_upstream_url("https://Example.COM:443/mcp").unwrap(),
        "https://example.com/mcp"
    );
    assert_eq!(
        canonicalize_upstream_url("http://Example.COM:80/mcp/").unwrap(),
        "http://example.com/mcp/"
    );
}

#[tokio::test]
async fn missing_code_challenge_methods_returns_unsupported() {
    let h = Harness::new().await;
    h.mount_no_resource_metadata().await;
    h.mount_metadata(Some(&h.as_url()), None, None).await;
    let m = h.manager(h.upstream_cfg(preregistered()));

    let err = m.begin_authorization("alice").await.unwrap_err();
    assert!(
        matches!(err, OauthError::UnsupportedMethod(_)),
        "expected UnsupportedMethod, got {err:?}"
    );
}

#[tokio::test]
async fn plain_pkce_only_returns_unsupported() {
    let h = Harness::new().await;
    h.mount_no_resource_metadata().await;
    h.mount_metadata(Some(&h.as_url()), Some(&["plain"]), None)
        .await;
    let m = h.manager(h.upstream_cfg(preregistered()));

    let err = m.begin_authorization("alice").await.unwrap_err();
    assert!(
        matches!(err, OauthError::UnsupportedMethod(_)),
        "expected UnsupportedMethod, got {err:?}"
    );
}

#[tokio::test]
async fn authorize_url_carries_canonical_resource_indicator() {
    let h = Harness::new().await;
    h.mount_no_resource_metadata().await;
    h.mount_metadata(Some(&h.as_url()), Some(&["S256"]), None)
        .await;
    let expected_resource = canonicalize_upstream_url(&h.upstream_url()).unwrap();
    let m = h.manager(h.upstream_cfg(preregistered()));

    let begin = m.begin_authorization("alice").await.expect("begin");
    let u = Url::parse(&begin.authorization_url).expect("authorize url parses");
    let resource = u
        .query_pairs()
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v.into_owned());
    assert_eq!(
        resource.as_deref(),
        Some(expected_resource.as_str()),
        "authorize url missing canonical resource indicator"
    );
    let method = u
        .query_pairs()
        .find(|(k, _)| k == "code_challenge_method")
        .map(|(_, v)| v.into_owned());
    assert_eq!(method.as_deref(), Some("S256"));
}

#[tokio::test]
async fn token_exchange_carries_canonical_resource_indicator() {
    let h = Harness::new().await;
    h.mount_no_resource_metadata().await;
    h.mount_metadata(Some(&h.as_url()), Some(&["S256"]), None)
        .await;
    h.mount_token_endpoint().await;
    let expected_resource = canonicalize_upstream_url(&h.upstream_url()).unwrap();
    let m = h.manager(h.upstream_cfg(preregistered()));

    let begin = m.begin_authorization("alice").await.expect("begin");
    let authorize_url = Url::parse(&begin.authorization_url).unwrap();
    let state = authorize_url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.into_owned())
        .expect("state");
    m.complete_authorization_callback("alice", "fake-code", &state)
        .await
        .expect("exchange");

    let recorded = h.mock.received_requests().await.expect("record enabled");
    let token_req = recorded
        .iter()
        .find(|r| r.method.as_str() == "POST" && r.url.path() == "/token")
        .expect("token request recorded");
    let body = std::str::from_utf8(&token_req.body).expect("utf8 body");
    let resource_value = url::form_urlencoded::parse(body.as_bytes())
        .find(|(k, _)| k == "resource")
        .map(|(_, v)| v.into_owned());
    assert_eq!(
        resource_value.as_deref(),
        Some(expected_resource.as_str()),
        "token exchange body missing canonical resource indicator; body was: {body}"
    );
}

#[tokio::test]
async fn issuer_missing_returns_issuer_mismatch() {
    let h = Harness::new().await;
    h.mount_no_resource_metadata().await;
    h.mount_metadata(None, Some(&["S256"]), None).await;
    let m = h.manager(h.upstream_cfg(preregistered()));

    let err = m.begin_authorization("alice").await.unwrap_err();
    assert!(
        matches!(err, OauthError::IssuerMismatch(_)),
        "expected IssuerMismatch for missing issuer, got {err:?}"
    );
}

#[tokio::test]
async fn issuer_endpoint_host_mismatch_returns_issuer_mismatch() {
    let h = Harness::new().await;
    h.mount_no_resource_metadata().await;
    h.mount_metadata(
        Some("https://good.example"),
        Some(&["S256"]),
        Some((
            "https://evil.example/authorize",
            "https://evil.example/token",
        )),
    )
    .await;
    let m = h.manager(h.upstream_cfg(preregistered()));

    let err = m.begin_authorization("alice").await.unwrap_err();
    assert!(
        matches!(err, OauthError::IssuerMismatch(_)),
        "expected IssuerMismatch for endpoint host drift, got {err:?}"
    );
}

#[tokio::test]
async fn cimd_registration_uses_metadata_url_as_client_id() {
    let h = Harness::new().await;
    h.mount_no_resource_metadata().await;
    h.mount_metadata(Some(&h.as_url()), Some(&["S256"]), None)
        .await;
    let cimd_url = "https://lab.example/.well-known/oauth-client-metadata";
    let m = h.manager(
        h.upstream_cfg(UpstreamOauthRegistration::ClientMetadataDocument {
            url: cimd_url.into(),
        }),
    );

    let begin = m.begin_authorization("alice").await.expect("begin");
    let u = Url::parse(&begin.authorization_url).unwrap();
    let client_id = u
        .query_pairs()
        .find(|(k, _)| k == "client_id")
        .map(|(_, v)| v.into_owned());
    assert_eq!(client_id.as_deref(), Some(cimd_url));
}

#[tokio::test]
async fn subject_lookup_survives_restart_for_saved_state() {
    let h = Harness::new().await;
    h.mount_no_resource_metadata().await;
    h.mount_metadata(Some(&h.as_url()), Some(&["S256"]), None)
        .await;
    let manager = h.manager(h.upstream_cfg(preregistered()));

    let begin = manager.begin_authorization("alice").await.expect("begin");
    let authorize_url = Url::parse(&begin.authorization_url).unwrap();
    let state = authorize_url
        .query_pairs()
        .find(|(k, _)| k == "state")
        .map(|(_, v)| v.into_owned())
        .expect("state");

    let restarted_manager = h.manager(h.upstream_cfg(preregistered()));
    let subject = restarted_manager
        .subject_for_state(&state)
        .await
        .expect("lookup");
    assert_eq!(subject.as_deref(), Some("alice"));
}
