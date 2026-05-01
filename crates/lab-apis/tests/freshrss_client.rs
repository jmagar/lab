use lab_apis::freshrss::FreshrssClient;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> FreshrssClient {
    FreshrssClient::with_credentials(&server.uri(), "alice".into(), "secret".into()).unwrap()
}

async fn mock_login(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/accounts/ClientLogin"))
        .and(body_string_contains("Email=alice"))
        .and(body_string_contains("Passwd=secret"))
        .respond_with(ResponseTemplate::new(200).set_body_string("SID=x\nAuth=token123\n"))
        .mount(server)
        .await;
}

#[tokio::test]
async fn subscriptions_login_then_fetches_reader_endpoint() {
    let server = MockServer::start().await;
    mock_login(&server).await;
    Mock::given(method("GET"))
        .and(path("/reader/api/0/subscription/list"))
        .and(header("authorization", "Token token123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "subscriptions": [{"id": "feed/1", "title": "Lab"}]
        })))
        .mount(&server)
        .await;

    let response = client(&server).subscriptions().await.unwrap();

    assert_eq!(response.value["subscriptions"][0]["title"], "Lab");
}

#[tokio::test]
async fn stream_items_clamps_item_count() {
    let server = MockServer::start().await;
    let err = client(&server).stream_items(101, None).await.unwrap_err();

    assert!(err.to_string().contains("n must be between 1 and 100"));
}

#[tokio::test]
async fn stream_items_sends_count_and_continuation() {
    let server = MockServer::start().await;
    mock_login(&server).await;
    Mock::given(method("GET"))
        .and(path("/reader/api/0/stream/contents/reading-list"))
        .and(query_param("n", "2"))
        .and(query_param("c", "next"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "items": [{"id": "item/1"}]
        })))
        .mount(&server)
        .await;

    let response = client(&server).stream_items(2, Some("next")).await.unwrap();

    assert_eq!(response.value["items"][0]["id"], "item/1");
}
