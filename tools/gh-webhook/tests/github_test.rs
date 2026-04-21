use gh_webhook::github::GithubClient;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path, query_param},
};

#[tokio::test]
async fn list_pr_comments_dedups_across_endpoints() {
    let server = MockServer::start().await;
    // pulls endpoint returns comments with ids 1, 2
    Mock::given(method("GET"))
        .and(path("/repos/o/r/pulls/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id":1,"user":{"login":"a"},"body":"hi","created_at":"2026-04-20T00:00:00Z","updated_at":"2026-04-20T00:00:00Z","html_url":"x"},
            {"id":2,"user":{"login":"b"},"body":"yo","created_at":"2026-04-20T00:00:01Z","updated_at":"2026-04-20T00:00:01Z","html_url":"y"}
        ])))
        .mount(&server)
        .await;
    // issues endpoint returns overlapping id 2 plus new id 3
    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {"id":2,"user":{"login":"b"},"body":"yo","created_at":"2026-04-20T00:00:01Z","updated_at":"2026-04-20T00:00:01Z","html_url":"y"},
            {"id":3,"user":{"login":"c"},"body":"ok","created_at":"2026-04-20T00:00:02Z","updated_at":"2026-04-20T00:00:02Z","html_url":"z"}
        ])))
        .mount(&server)
        .await;

    let c = GithubClient::new(server.uri(), "tok".into()).unwrap();
    let out = c.list_pr_comments("o", "r", 1, None).await.unwrap();
    assert_eq!(out.len(), 3);
    let ids: Vec<u64> = out.iter().map(|c| c.id).collect();
    assert_eq!(ids, vec![1, 2, 3]);
}

#[tokio::test]
async fn list_pr_comments_follows_pagination() {
    let server = MockServer::start().await;
    let next = format!(
        "<{}/repos/o/r/pulls/1/comments?page=2>; rel=\"next\"",
        server.uri()
    );
    Mock::given(method("GET"))
        .and(path("/repos/o/r/pulls/1/comments"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("Link", next.as_str())
            .set_body_json(serde_json::json!([{"id":1,"user":{"login":"a"},"body":"hi","created_at":"2026-04-20T00:00:00Z","updated_at":"2026-04-20T00:00:00Z","html_url":"x"}])))
        .up_to_n_times(1)
        .mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/repos/o/r/pulls/1/comments"))
        .and(query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200)
            .set_body_json(serde_json::json!([{"id":2,"user":{"login":"b"},"body":"ok","created_at":"2026-04-20T00:00:01Z","updated_at":"2026-04-20T00:00:01Z","html_url":"y"}])))
        .mount(&server).await;
    // issues endpoint returns empty
    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let c = GithubClient::new(server.uri(), "tok".into()).unwrap();
    let out = c.list_pr_comments("o", "r", 1, None).await.unwrap();
    assert_eq!(out.len(), 2);
}

#[tokio::test]
async fn pagination_ssrf_guard_stops_on_cross_host() {
    let server = MockServer::start().await;
    // evil next link pointing at a different host
    let evil = "<http://evil.example.com/repos/o/r/pulls/1/comments?page=2>; rel=\"next\"";
    Mock::given(method("GET"))
        .and(path("/repos/o/r/pulls/1/comments"))
        .respond_with(ResponseTemplate::new(200)
            .insert_header("Link", evil)
            .set_body_json(serde_json::json!([{"id":1,"user":{"login":"a"},"body":"hi","created_at":"2026-04-20T00:00:00Z","updated_at":"2026-04-20T00:00:00Z","html_url":"x"}])))
        .mount(&server).await;
    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;

    let c = GithubClient::new(server.uri(), "tok".into()).unwrap();
    let out = c.list_pr_comments("o", "r", 1, None).await.unwrap();
    // We should get the 1 comment from the first page and stop — not attempt evil.example.com.
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].id, 1);
}

#[tokio::test]
async fn retries_on_429_with_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "1"))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    let c = GithubClient::new(server.uri(), "tok".into()).unwrap();
    let out = c.list_pr_comments("o", "r", 1, None).await.unwrap();
    assert_eq!(out.len(), 0);
}

#[tokio::test]
async fn sends_auth_and_since() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(header("authorization", "Bearer tok"))
        .and(query_param("since", "2026-04-20T00:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&server)
        .await;
    let c = GithubClient::new(server.uri(), "tok".into()).unwrap();
    c.list_pr_comments("o", "r", 1, Some("2026-04-20T00:00:00Z"))
        .await
        .unwrap();
}
