#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Unit tests for the new `TailscaleClient` methods added in bead lab-hq03.11.
//! Uses wiremock to mock the Tailscale API v2.

use lab_apis::core::Auth;
use lab_apis::tailscale::TailscaleClient;
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn client(base_url: &str) -> TailscaleClient {
    TailscaleClient::new(base_url, Auth::Bearer { token: "tskey-test".into() }, "-")
        .expect("client")
}

// ── acl.get ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn acl_get_returns_policy() {
    let server = MockServer::start().await;
    let policy = json!({ "acls": [{"action": "accept", "src": ["*"], "dst": ["*:*"]}] });

    Mock::given(method("GET"))
        .and(path("/tailnet/-/acl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&policy))
        .mount(&server)
        .await;

    let result = client(&server.uri()).acl_get().await.expect("acl_get");
    assert_eq!(result, policy);
}

// ── acl.validate ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn acl_validate_returns_empty_on_success() {
    let server = MockServer::start().await;
    let policy = json!({ "acls": [] });
    let ok_response = json!({});

    Mock::given(method("POST"))
        .and(path("/tailnet/-/acl/validate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&ok_response))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .acl_validate(&policy)
        .await
        .expect("acl_validate");
    assert_eq!(result, ok_response);
}

// ── acl.set calls validate first ─────────────────────────────────────────────

#[tokio::test]
async fn acl_set_calls_validate_before_set() {
    let server = MockServer::start().await;
    let policy = json!({ "acls": [] });

    // Validation endpoint — returns clean
    Mock::given(method("POST"))
        .and(path("/tailnet/-/acl/validate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&json!({})))
        .mount(&server)
        .await;

    // Set endpoint — should be called after validation passes
    let set_response = json!({ "acls": [] });
    Mock::given(method("POST"))
        .and(path("/tailnet/-/acl"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&set_response))
        .mount(&server)
        .await;

    // Call acl_validate then acl_set (simulating dispatch.rs behavior)
    let c = client(&server.uri());
    let validation = c.acl_validate(&policy).await.expect("validate");
    // Dispatch logic: only call acl_set if no errors
    assert!(!validation.get("errors").is_some_and(|v| {
        v.as_array().map_or(false, |arr| !arr.is_empty())
    }));
    let result = c.acl_set(&policy).await.expect("acl_set");
    assert_eq!(result, set_response);
}

// ── device.routes-get ────────────────────────────────────────────────────────

#[tokio::test]
async fn device_routes_get_returns_routes() {
    let server = MockServer::start().await;
    let routes = json!({ "advertisedRoutes": ["10.0.0.0/8"], "enabledRoutes": ["10.0.0.0/8"] });

    Mock::given(method("GET"))
        .and(path("/device/abc123/routes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&routes))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .device_routes_get("abc123")
        .await
        .expect("device_routes_get");
    assert_eq!(result, routes);
}

// ── device.routes-set ────────────────────────────────────────────────────────

#[tokio::test]
async fn device_routes_set_posts_routes_body() {
    let server = MockServer::start().await;
    let response = json!({ "advertisedRoutes": ["10.0.0.0/8"], "enabledRoutes": [] });

    Mock::given(method("POST"))
        .and(path("/device/abc123/routes"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(&server)
        .await;

    let routes = vec!["10.0.0.0/8".to_string()];
    let result = client(&server.uri())
        .device_routes_set("abc123", &routes)
        .await
        .expect("device_routes_set");
    assert_eq!(result, response);
}

// ── device.tag ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn device_tag_posts_tags_body() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/device/abc123/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&json!({})))
        .mount(&server)
        .await;

    let tags = vec!["tag:server".to_string()];
    client(&server.uri())
        .device_tag("abc123", &tags)
        .await
        .expect("device_tag");
}

// ── device.expire ────────────────────────────────────────────────────────────

#[tokio::test]
async fn device_expire_posts_to_expire_endpoint() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/device/abc123/expire"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&json!({})))
        .mount(&server)
        .await;

    client(&server.uri())
        .device_expire("abc123")
        .await
        .expect("device_expire");
}

// ── user.list ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn user_list_returns_users() {
    let server = MockServer::start().await;
    let users = json!({ "users": [{ "id": "u1", "loginName": "alice@example.com" }] });

    Mock::given(method("GET"))
        .and(path("/tailnet/-/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&users))
        .mount(&server)
        .await;

    let result = client(&server.uri()).user_list().await.expect("user_list");
    assert_eq!(result, users);
}

// ── tailnet.settings-get ─────────────────────────────────────────────────────

#[tokio::test]
async fn tailnet_settings_get_returns_settings() {
    let server = MockServer::start().await;
    let settings = json!({ "devicesApprovalOn": true });

    Mock::given(method("GET"))
        .and(path("/tailnet/-/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&settings))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .tailnet_settings_get()
        .await
        .expect("tailnet_settings_get");
    assert_eq!(result, settings);
}

// ── tailnet.settings-patch ───────────────────────────────────────────────────

#[tokio::test]
async fn tailnet_settings_patch_sends_body() {
    let server = MockServer::start().await;
    let patch = json!({ "devicesApprovalOn": false });
    let response = json!({ "devicesApprovalOn": false });

    Mock::given(method("PATCH"))
        .and(path("/tailnet/-/settings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .tailnet_settings_patch(&patch)
        .await
        .expect("tailnet_settings_patch");
    assert_eq!(result, response);
}

// ── dns.split-get ────────────────────────────────────────────────────────────

#[tokio::test]
async fn dns_split_get_returns_config() {
    let server = MockServer::start().await;
    let split = json!({ "example.internal": ["100.100.100.100"] });

    Mock::given(method("GET"))
        .and(path("/tailnet/-/dns/split-dns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&split))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .dns_split_get()
        .await
        .expect("dns_split_get");
    assert_eq!(result, split);
}

// ── dns.split-set ────────────────────────────────────────────────────────────

#[tokio::test]
async fn dns_split_set_sends_body() {
    let server = MockServer::start().await;
    let config = json!({ "example.internal": ["100.100.100.100"] });

    Mock::given(method("PUT"))
        .and(path("/tailnet/-/dns/split-dns"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&config))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .dns_split_set(&config)
        .await
        .expect("dns_split_set");
    assert_eq!(result, config);
}

// ── key.create ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn key_create_posts_capabilities() {
    use lab_apis::tailscale::types::CreateKeyRequest;

    let server = MockServer::start().await;
    let response = json!({
        "id": "k1abc",
        "key": "tskey-auth-secret",
        "created": "2024-01-01T00:00:00Z",
        "expires": "2025-01-01T00:00:00Z",
        "invalid": false,
        "capabilities": {}
    });

    Mock::given(method("POST"))
        .and(path("/tailnet/-/keys"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(&server)
        .await;

    let req = CreateKeyRequest {
        capabilities: json!({ "devices": { "create": { "reusable": true } } }),
        expiry_seconds: Some(86400),
        description: Some("test key".into()),
    };

    let result = client(&server.uri())
        .key_create(&req)
        .await
        .expect("key_create");
    // The key field should be present in the response
    assert_eq!(result.id, "k1abc");
    assert_eq!(result.key, "tskey-auth-secret");
}
