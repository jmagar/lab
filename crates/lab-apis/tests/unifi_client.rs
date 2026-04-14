#![cfg(feature = "unifi")]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::unifi::UnifiClient;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn make_client(base_url: &str) -> UnifiClient {
    let auth = Auth::ApiKey {
        header: "X-API-KEY".into(),
        key: "test-api-key".into(),
    };
    UnifiClient::new(base_url, auth).expect("client construction")
}

const API_PREFIX: &str = "/proxy/network/integration/v1";

// ── client.history ────────────────────────────────────────────────────────────

#[tokio::test]
async fn client_history_returns_value() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"data": [{"ts": 1234567890, "event": "connected"}]});
    Mock::given(method("GET"))
        .and(path(format!(
            "{API_PREFIX}/sites/site-abc/clients/aa:bb:cc:dd:ee:ff/history"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .get_value("/sites/site-abc/clients/aa:bb:cc:dd:ee:ff/history")
        .await
        .expect("client_history");
    assert!(result["data"].is_array());
}

// ── client.block ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn client_block_posts_to_block_endpoint() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"status": "blocked"});
    Mock::given(method("POST"))
        .and(path(format!(
            "{API_PREFIX}/sites/site-abc/clients/aa:bb:cc:dd:ee:ff/block"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .post_value(
            "/sites/site-abc/clients/aa:bb:cc:dd:ee:ff/block",
            &serde_json::json!({}),
        )
        .await
        .expect("client_block");
    assert_eq!(result["status"], "blocked");
}

// ── client.unblock ────────────────────────────────────────────────────────────

#[tokio::test]
async fn client_unblock_posts_to_unblock_endpoint() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"status": "unblocked"});
    Mock::given(method("POST"))
        .and(path(format!(
            "{API_PREFIX}/sites/site-abc/clients/aa:bb:cc:dd:ee:ff/unblock"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .post_value(
            "/sites/site-abc/clients/aa:bb:cc:dd:ee:ff/unblock",
            &serde_json::json!({}),
        )
        .await
        .expect("client_unblock");
    assert_eq!(result["status"], "unblocked");
}

// ── device.update ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn device_update_puts_to_device_endpoint() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": "dev-001", "name": "My AP"});
    Mock::given(method("PUT"))
        .and(path(format!("{API_PREFIX}/sites/site-abc/devices/dev-001")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let update = serde_json::json!({"name": "My AP"});
    let result = client
        .put_value("/sites/site-abc/devices/dev-001", &update)
        .await
        .expect("device_update");
    assert_eq!(result["name"], "My AP");
}

// ── wifi.update ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn wifi_update_puts_to_wifi_endpoint() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": "wifi-001", "ssid": "HomeNet"});
    Mock::given(method("PUT"))
        .and(path(format!("{API_PREFIX}/sites/site-abc/wifi/wifi-001")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let update = serde_json::json!({"ssid": "HomeNet"});
    let result = client
        .put_value("/sites/site-abc/wifi/wifi-001", &update)
        .await
        .expect("wifi_update");
    assert_eq!(result["ssid"], "HomeNet");
}

// ── port-profile.list ─────────────────────────────────────────────────────────

#[tokio::test]
async fn port_profile_list_returns_page() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"data": [{"id": "pp-001", "name": "Default"}]});
    Mock::given(method("GET"))
        .and(path(format!(
            "{API_PREFIX}/sites/site-abc/switching/port-profiles"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .get_value("/sites/site-abc/switching/port-profiles")
        .await
        .expect("port_profile_list");
    assert!(result["data"].is_array());
}

// ── port-profile.create ───────────────────────────────────────────────────────

#[tokio::test]
async fn port_profile_create_posts_and_returns_profile() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": "pp-new", "name": "My Profile"});
    Mock::given(method("POST"))
        .and(path(format!(
            "{API_PREFIX}/sites/site-abc/switching/port-profiles"
        )))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let new_profile = serde_json::json!({"name": "My Profile"});
    let result = client
        .post_value("/sites/site-abc/switching/port-profiles", &new_profile)
        .await
        .expect("port_profile_create");
    assert_eq!(result["name"], "My Profile");
}

// ── port-profile.update ───────────────────────────────────────────────────────

#[tokio::test]
async fn port_profile_update_puts_to_profile_endpoint() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": "pp-001", "name": "Updated Profile"});
    Mock::given(method("PUT"))
        .and(path(format!(
            "{API_PREFIX}/sites/site-abc/switching/port-profiles/pp-001"
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let update = serde_json::json!({"name": "Updated Profile"});
    let result = client
        .put_value("/sites/site-abc/switching/port-profiles/pp-001", &update)
        .await
        .expect("port_profile_update");
    assert_eq!(result["name"], "Updated Profile");
}

// ── wan.get ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn wan_get_returns_wan_object() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": "wan-001", "type": "Internet"});
    Mock::given(method("GET"))
        .and(path(format!("{API_PREFIX}/sites/site-abc/wans/wan-001")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .get_value("/sites/site-abc/wans/wan-001")
        .await
        .expect("wan_get");
    assert_eq!(result["type"], "Internet");
}
