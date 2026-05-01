use lab_apis::core::Auth;
use lab_apis::dozzle::DozzleClient;
use lab_apis::dozzle::types::{LogFetchRequest, ReadLimits};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> DozzleClient {
    DozzleClient::new(&server.uri(), Auth::None).unwrap()
}

fn log_request() -> LogFetchRequest {
    LogFetchRequest {
        host: "local".to_string(),
        container_id: "abc123".to_string(),
        stdout: true,
        stderr: true,
        limits: ReadLimits::default(),
    }
}

#[tokio::test]
async fn health_uses_healthcheck() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/healthcheck"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let response = client(&server).health().await.unwrap();

    assert!(response.reachable);
}

#[tokio::test]
async fn version_strips_pre_wrapper() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/version"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<pre>v10.5.1</pre>"))
        .mount(&server)
        .await;

    let response = client(&server).version().await.unwrap();

    assert_eq!(response.version, "v10.5.1");
}

#[tokio::test]
async fn containers_list_reads_first_containers_changed_event() {
    let server = MockServer::start().await;
    let body = "event: containers-changed\n\
data: [{\"id\":\"abc123\",\"name\":\"dozzle\"}]\n\n";
    Mock::given(method("GET"))
        .and(path("/api/events/stream"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let response = client(&server)
        .containers_list(ReadLimits::default())
        .await
        .unwrap();

    assert_eq!(response.containers[0]["id"], "abc123");
    assert_eq!(response.meta.events_read, Some(1));
}

#[tokio::test]
async fn logs_fetch_parses_jsonl_and_sends_query_flags() {
    let server = MockServer::start().await;
    let body = "{\"id\":\"1\",\"m\":\"hello\",\"stream\":\"stdout\"}\n\
{\"id\":\"2\",\"m\":\"world\",\"stream\":\"stderr\"}\n";
    Mock::given(method("GET"))
        .and(path("/api/hosts/local/containers/abc123/logs"))
        .and(query_param("stdout", "1"))
        .and(query_param("stderr", "1"))
        .and(query_param("everything", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/x-jsonl")
                .set_body_string(body),
        )
        .mount(&server)
        .await;

    let response = client(&server).logs_fetch(&log_request()).await.unwrap();

    assert_eq!(response.events.len(), 2);
    assert_eq!(response.events[0]["m"], "hello");
    assert!(!response.meta.truncated);
}

#[tokio::test]
async fn logs_fetch_plain_requests_text_plain() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/hosts/local/containers/abc123/logs"))
        .and(header("accept", "text/plain"))
        .and(query_param("everything", "1"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/plain")
                .set_body_string("hello\n"),
        )
        .mount(&server)
        .await;

    let response = client(&server)
        .logs_fetch_plain(&log_request())
        .await
        .unwrap();

    assert_eq!(response.text, "hello\n");
}

#[tokio::test]
async fn session_auth_sends_cookie_without_special_casing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/version"))
        .and(header("cookie", "jwt=secret"))
        .respond_with(ResponseTemplate::new(200).set_body_string("<pre>v10.5.1</pre>"))
        .mount(&server)
        .await;

    let client = DozzleClient::new(
        &server.uri(),
        Auth::Session {
            cookie: "jwt=secret".to_string(),
        },
    )
    .unwrap();

    let response = client.version().await.unwrap();

    assert_eq!(response.version, "v10.5.1");
}

#[tokio::test]
async fn hostile_path_segments_are_encoded() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/api/hosts/local%2Fwith%3Fslash/containers/abc%2F123/logs",
        ))
        .and(query_param("stdout", "1"))
        .and(query_param("everything", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"m\":\"ok\"}\n"))
        .mount(&server)
        .await;

    let mut request = log_request();
    request.host = "local/with?slash".to_string();
    request.container_id = "abc/123".to_string();
    request.stderr = false;

    let response = client(&server).logs_fetch(&request).await.unwrap();

    assert_eq!(response.events[0]["m"], "ok");
}

#[tokio::test]
async fn max_lines_truncates_jsonl_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/hosts/local/containers/abc123/logs"))
        .and(query_param("stdout", "1"))
        .and(query_param("stderr", "1"))
        .and(query_param("everything", "1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string("{\"m\":\"one\"}\n{\"m\":\"two\"}\n"),
        )
        .mount(&server)
        .await;

    let mut request = log_request();
    request.limits.max_lines = 1;

    let response = client(&server).logs_fetch(&request).await.unwrap();

    assert_eq!(response.events.len(), 1);
    assert!(response.meta.truncated);
    assert_eq!(response.meta.limit_kind.as_deref(), Some("max_lines"));
}
