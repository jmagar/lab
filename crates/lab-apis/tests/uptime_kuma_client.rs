use lab_apis::uptime_kuma::{UptimeKumaClient, UptimeKumaError};

#[test]
fn login_client_requires_non_empty_credentials() {
    let result = UptimeKumaClient::with_login("http://localhost:3001", String::new(), "pw".into());
    assert!(result.is_err(), "invalid credentials should fail");
    let err = match result {
        Err(err) => err,
        Ok(_) => return,
    };

    assert!(matches!(err, UptimeKumaError::Api(_)));
}

#[test]
fn contract_reports_live_reads_when_credentials_configured() {
    let client = UptimeKumaClient::with_login(
        "http://localhost:3001",
        "admin".into(),
        "secret-password".into(),
    )
    .unwrap();

    let contract = client.contract_status();

    assert_eq!(contract.value["transport"], "socket.io");
    assert_eq!(contract.value["credentials_configured"], true);
    assert_eq!(contract.value["live_socket_reads"], true);
}
