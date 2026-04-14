#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::unraid::UnraidClient;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn graphql_response(data: serde_json::Value) -> serde_json::Value {
    serde_json::json!({ "data": data })
}

fn make_client(base_url: &str) -> UnraidClient {
    UnraidClient::new(
        base_url,
        Auth::ApiKey {
            header: "X-API-Key".into(),
            key: "test-key".into(),
        },
    )
    .expect("client construction should succeed")
}

// ---------------------------------------------------------------------------
// VM
// ---------------------------------------------------------------------------

#[tokio::test]
async fn vm_list_returns_vms() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "vms": [
            { "id": "vm1", "name": "ubuntu", "status": "running" },
            { "id": "vm2", "name": "windows", "status": "stopped" }
        ]
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let vms = client.vm_list().await.expect("vm_list");
    assert_eq!(vms.len(), 2);
    assert_eq!(vms[0].id, "vm1");
}

#[tokio::test]
async fn vm_start_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "vmAction": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.vm_start("vm1").await.expect("vm_start");
}

#[tokio::test]
async fn vm_stop_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "vmAction": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.vm_stop("vm1").await.expect("vm_stop");
}

#[tokio::test]
async fn vm_pause_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "vmAction": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.vm_pause("vm1").await.expect("vm_pause");
}

#[tokio::test]
async fn vm_resume_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "vmAction": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.vm_resume("vm1").await.expect("vm_resume");
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

#[tokio::test]
async fn notification_list_returns_notifications() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "notifications": [
            { "id": "n1", "title": "Array Started", "importance": "INFO" },
            { "id": "n2", "title": "Disk Warning", "importance": "WARNING" }
        ]
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let notifs = client.notification_list().await.expect("notification_list");
    assert_eq!(notifs.len(), 2);
    assert_eq!(notifs[0].id, "n1");
}

#[tokio::test]
async fn notification_create_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "createNotification": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .notification_create("Test title", Some("body text"), Some("INFO"), None)
        .await
        .expect("notification_create");
}

#[tokio::test]
async fn notification_archive_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "archiveNotification": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .notification_archive("n1")
        .await
        .expect("notification_archive");
}

// ---------------------------------------------------------------------------
// Parity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn parity_history_returns_entries() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "parityHistory": [
            { "date": "2024-01-01", "duration": 3600, "status": "ok", "errors": 0 }
        ]
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let history = client.parity_history().await.expect("parity_history");
    assert_eq!(history.len(), 1);
}

#[tokio::test]
async fn parity_check_start_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "parityCheck": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .parity_check_start(Some(true))
        .await
        .expect("parity_check_start");
}

#[tokio::test]
async fn parity_check_pause_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "parityCheck": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.parity_check_pause().await.expect("parity_check_pause");
}

#[tokio::test]
async fn parity_check_cancel_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "parityCheck": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .parity_check_cancel()
        .await
        .expect("parity_check_cancel");
}

// ---------------------------------------------------------------------------
// Share / Plugin / Network / UPS
// ---------------------------------------------------------------------------

#[tokio::test]
async fn share_list_returns_shares() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "shares": [
            { "id": "s1", "name": "media" },
            { "id": "s2", "name": "appdata" }
        ]
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let shares = client.share_list().await.expect("share_list");
    assert_eq!(shares.len(), 2);
}

#[tokio::test]
async fn plugin_list_returns_plugins() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "installedUnraidPlugins": [
            { "id": "p1", "name": "dynamix", "version": "1.0.0" }
        ]
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let plugins = client.plugin_list().await.expect("plugin_list");
    assert_eq!(plugins.len(), 1);
}

#[tokio::test]
async fn network_list_returns_value() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "network": { "id": "net1", "interfaces": [] }
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let network = client.network_list().await.expect("network_list");
    assert_eq!(network["id"], "net1");
}

#[tokio::test]
async fn ups_devices_returns_list() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "upsDevices": [
            { "id": "ups1", "name": "APC-750", "status": "ONLINE" }
        ]
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let devices = client.ups_devices().await.expect("ups_devices");
    assert_eq!(devices.len(), 1);
}

#[tokio::test]
async fn ups_config_returns_value() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "upsConfiguration": { "id": "cfg1", "upsName": "apc", "driver": "usbhid-ups" }
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let config = client.ups_config().await.expect("ups_config");
    assert_eq!(config["upsName"], "apc");
}

// ---------------------------------------------------------------------------
// Log file
// ---------------------------------------------------------------------------

#[tokio::test]
async fn log_read_returns_content() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "logFile": "line1\nline2\nline3"
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let content = client
        .log_read("/var/log/syslog", Some(50))
        .await
        .expect("log_read");
    assert!(content.contains("line1"));
}

#[tokio::test]
async fn log_read_null_returns_empty_string() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "logFile": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let content = client
        .log_read("/var/log/syslog", None)
        .await
        .expect("log_read null");
    assert!(content.is_empty());
}

// ---------------------------------------------------------------------------
// Flash
// ---------------------------------------------------------------------------

#[tokio::test]
async fn flash_status_returns_value() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({
        "flash": { "id": "flash1", "guid": "ABCD-1234", "state": "ok" }
    }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let status = client.flash_status().await.expect("flash_status");
    assert_eq!(status["guid"], "ABCD-1234");
}

#[tokio::test]
async fn flash_backup_sends_mutation() {
    let server = MockServer::start().await;
    let body = graphql_response(serde_json::json!({ "initiateFlashBackup": null }));

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.flash_backup().await.expect("flash_backup");
}
