use lab::device::checkin::{DeviceHello, DeviceStatus};
use lab::device::runtime::DeviceRuntime;
use lab::device::store::DeviceFleetStore;

#[tokio::test]
async fn device_store_tracks_last_seen_status_and_metadata() {
    let store = DeviceFleetStore::default();
    store
        .record_hello(DeviceHello {
            device_id: "tootie".into(),
            role: "master".into(),
            version: "1.0.0".into(),
        })
        .await;

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

    let runtime = DeviceRuntime::non_master_for_test("dookie", server.uri());
    runtime.send_initial_hello().await.unwrap();
}
