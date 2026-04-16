use lab::device::checkin::{DeviceHello, DeviceStatus};
use lab::device::log_event::DeviceLogEvent;
use lab::device::runtime::DeviceRuntime;
use lab::device::store::DeviceFleetStore;

#[tokio::test]
async fn device_store_marks_hello_devices_connected_and_tracks_status() {
    let store = DeviceFleetStore::default();
    store
        .record_hello(DeviceHello {
            device_id: "tootie".into(),
            role: "master".into(),
            version: "1.0.0".into(),
        })
        .await;

    let snapshot = store.device("tootie").await.unwrap();
    assert!(snapshot.connected);

    store
        .record_status(DeviceStatus {
            device_id: "tootie".into(),
            connected: true,
            cpu_percent: Some(3.5),
            memory_used_bytes: Some(1024),
            storage_used_bytes: Some(2048),
            os: Some("linux".into()),
            ips: vec!["100.64.0.1".into()],
        })
        .await;

    let snapshot = store.device("tootie").await.unwrap();
    assert!(snapshot.connected);
    assert_eq!(snapshot.device_id, "tootie");
}

#[tokio::test]
async fn non_master_runtime_posts_hello_to_master() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/device/hello"))
        .respond_with(wiremock::ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let runtime = DeviceRuntime::non_master_for_test("dookie", server.uri()).unwrap();
    runtime.send_initial_hello().await.unwrap();
}

#[tokio::test]
async fn non_master_runtime_uploads_discovered_ai_cli_inventory() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join(".claude.json"),
        r#"{"mcpServers":{"lab":{"command":"lab","args":["serve"]}}}"#,
    )
    .unwrap();

    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/device/metadata"))
        .and(wiremock::matchers::body_string_contains(".claude.json"))
        .and(wiremock::matchers::body_string_contains("content_hash"))
        .respond_with(wiremock::ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let runtime =
        DeviceRuntime::non_master_for_test_with_home("dookie", server.uri(), temp.path()).unwrap();
    runtime.upload_initial_metadata().await.unwrap();
}

#[tokio::test]
async fn master_store_keeps_uploaded_logs_by_device() {
    let store = DeviceFleetStore::default();
    store
        .record_logs(
            "dookie",
            vec![DeviceLogEvent {
                device_id: "dookie".into(),
                source: "journald".into(),
                timestamp_unix_ms: 1,
                level: Some("info".into()),
                message: "hello".into(),
                fields: Default::default(),
            }],
        )
        .await;

    let snapshot = store.device("dookie").await.unwrap();
    assert_eq!(snapshot.logs.len(), 1);
}

#[tokio::test]
async fn flush_queue_acks_successes_before_returning_a_later_delivery_error() {
    let server = wiremock::MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/device/syslog/batch"))
        .respond_with(
            wiremock::ResponseTemplate::new(200).set_body_json(serde_json::json!({"ok": true})),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .and(wiremock::matchers::path("/v1/device/syslog/batch"))
        .respond_with(wiremock::ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let temp = tempfile::tempdir().unwrap();
    let runtime =
        DeviceRuntime::non_master_for_test_with_home("dookie", server.uri(), temp.path()).unwrap();
    runtime
        .queue_syslog_batch(vec![DeviceLogEvent {
            device_id: "dookie".into(),
            source: "journald".into(),
            timestamp_unix_ms: 1,
            level: Some("info".into()),
            message: "first".into(),
            fields: Default::default(),
        }])
        .await
        .unwrap();
    runtime
        .queue_syslog_batch(vec![DeviceLogEvent {
            device_id: "dookie".into(),
            source: "journald".into(),
            timestamp_unix_ms: 2,
            level: Some("warn".into()),
            message: "second".into(),
            fields: Default::default(),
        }])
        .await
        .unwrap();

    let error = runtime.flush_queue_once().await.unwrap_err();
    assert!(!error.to_string().is_empty());

    let queue = lab::device::queue::DeviceOutboundQueue::open(
        temp.path().join(".lab/device-runtime-queue.jsonl"),
    )
    .await
    .unwrap();
    let drained = queue.drain_batch(10).await.unwrap();
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].payload["events"][0]["message"], "second");
}

#[tokio::test]
async fn device_store_search_logs_applies_offset_limit_and_retention() {
    let store = DeviceFleetStore::default();
    for index in 0..10_100 {
        store
            .record_logs(
                "dookie",
                vec![DeviceLogEvent {
                    device_id: "dookie".into(),
                    source: "journald".into(),
                    timestamp_unix_ms: index,
                    level: Some("info".into()),
                    message: format!("hello-{index}"),
                    fields: Default::default(),
                }],
            )
            .await;
    }

    let retained = store.device("dookie").await.unwrap().logs;
    assert_eq!(retained.len(), 10_000);
    assert_eq!(retained.first().unwrap().message, "hello-100");

    let searched = store.search_logs_for_device("dookie", "hello", 5, 3).await;
    assert_eq!(searched.len(), 3);
    assert_eq!(searched.first().unwrap().message, "hello-105");
}
