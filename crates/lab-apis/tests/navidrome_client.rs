use lab_apis::navidrome::NavidromeClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> NavidromeClient {
    NavidromeClient::with_subsonic_auth(
        &server.uri(),
        "alice".into(),
        "token123".into(),
        "salt456".into(),
    )
    .unwrap()
}

#[tokio::test]
async fn ping_sends_subsonic_auth_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/ping.view"))
        .and(query_param("u", "alice"))
        .and(query_param("t", "token123"))
        .and(query_param("s", "salt456"))
        .and(query_param("f", "json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "subsonic-response": {"status": "ok", "version": "1.16.1"}
        })))
        .mount(&server)
        .await;

    let response = client(&server).ping().await.unwrap();

    assert_eq!(response.value["status"], "ok");
}

#[tokio::test]
async fn album_list_clamps_size() {
    let server = MockServer::start().await;
    let err = client(&server)
        .album_list("newest", 101, 0)
        .await
        .unwrap_err();

    assert!(err.to_string().contains("size must be between 1 and 100"));
}

#[tokio::test]
async fn search_sends_counts_and_offsets() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/rest/search3.view"))
        .and(query_param("query", "matrix"))
        .and(query_param("artistCount", "5"))
        .and(query_param("songOffset", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "subsonic-response": {"status": "ok", "searchResult3": {}}
        })))
        .mount(&server)
        .await;

    let response = client(&server).search("matrix", 5, 10).await.unwrap();

    assert_eq!(response.value["status"], "ok");
}
