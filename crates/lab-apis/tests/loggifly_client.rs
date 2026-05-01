use lab_apis::core::Auth;
use lab_apis::loggifly::LoggiflyClient;

#[test]
fn contract_status_exposes_local_safe_actions() {
    let client = LoggiflyClient::new("http://localhost", Auth::None).unwrap();
    let status = client.contract_status();

    assert_eq!(status.status, "local_contract_implemented");
    assert_eq!(
        status.safe_v1_actions,
        &["contract.status", "health.status", "config.summary"]
    );
    assert!(status.deferred.contains(&"docker.logs"));
}

#[tokio::test]
async fn health_status_reads_heartbeat_file() {
    let dir = tempfile::tempdir().unwrap();
    let heartbeat = dir.path().join("heartbeat");
    tokio::fs::write(&heartbeat, b"ok").await.unwrap();
    let client = LoggiflyClient::with_local_paths(None, Some(heartbeat), Some(60));

    let status = client.health_status().await.unwrap();

    assert_eq!(status.status, "healthy");
}

#[tokio::test]
async fn config_summary_omits_raw_config() {
    let dir = tempfile::tempdir().unwrap();
    tokio::fs::write(
        dir.path().join("config.yaml"),
        "containers:\nnotifications:\n  ntfy:\n    token: secret\n",
    )
    .await
    .unwrap();
    let client = LoggiflyClient::with_local_paths(Some(dir.path().to_path_buf()), None, None);

    let summary = client.config_summary().await.unwrap();

    assert!(summary.exists);
    assert!(summary.containers_section);
    assert!(summary.notifications_section);
    assert!(!summary.raw_config_returned);
}
