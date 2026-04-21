use gh_webhook::debounce::PrKey;
use gh_webhook::flush::Flusher;
use gh_webhook::github::GithubClient;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn key(owner: &str, repo: &str, pr: u64) -> PrKey {
    PrKey {
        owner: owner.into(),
        repo: repo.into(),
        pr,
    }
}

fn comment_json(id: u64, updated_at: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "user": { "login": "alice" },
        "body": "hello world",
        "created_at": "2026-04-20T00:00:00Z",
        "updated_at": updated_at,
        "html_url": "https://github.com/o/r/pull/1#issuecomment-1",
    })
}

#[tokio::test]
async fn happy_path_writes_digest_and_jsonl() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/repos/o/r/pulls/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            comment_json(1, "2026-04-20T01:00:00Z"),
        ])))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            comment_json(2, "2026-04-20T02:00:00Z"),
        ])))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().to_path_buf();
    let jsonl_path = data_dir.join("notifications.jsonl");
    let gh = GithubClient::new(server.uri(), "tok".into()).unwrap();
    let f = Flusher::new(gh, data_dir.clone(), jsonl_path.clone());

    f.flush_pr(key("o", "r", 1), 2).await;

    let md_path = data_dir.join("o-r-1.md");
    assert!(md_path.exists(), "digest file should exist");
    let md = std::fs::read_to_string(&md_path).unwrap();
    assert!(md.contains("o/r#1"), "digest should carry header");
    assert!(md.contains("hello world"));

    let jsonl = std::fs::read_to_string(&jsonl_path).unwrap();
    assert!(jsonl.contains("\"kind\":\"pr_comments\""));
    assert!(jsonl.contains("\"digest_path\""));
    assert!(jsonl.contains("o-r-1.md"));

    // Watermark should be set to max updated_at
    let wm = f.watermark(&key("o", "r", 1));
    assert_eq!(wm.as_deref(), Some("2026-04-20T02:00:00Z"));
}

#[tokio::test]
async fn fetch_failure_emits_degraded_line() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().to_path_buf();
    let jsonl_path = data_dir.join("notifications.jsonl");
    let gh = GithubClient::new(server.uri(), "tok".into()).unwrap();
    let f = Flusher::new(gh, data_dir.clone(), jsonl_path.clone());

    f.flush_pr(key("o", "r", 1), 1).await;

    let md_path = data_dir.join("o-r-1.md");
    assert!(!md_path.exists(), "no digest should be written on failure");
    let jsonl = std::fs::read_to_string(&jsonl_path).unwrap();
    assert!(jsonl.contains("\"kind\":\"flush_error\""));
}

#[tokio::test]
async fn watermark_filters_subsequent_fetch() {
    let server = MockServer::start().await;
    // First call (no `since`) returns one comment with updated_at=T1.
    Mock::given(method("GET"))
        .and(path("/repos/o/r/pulls/1/comments"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            comment_json(1, "2026-04-20T05:00:00Z"),
        ])))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1/comments"))
        .and(query_param("per_page", "100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    let dir = tempfile::tempdir().unwrap();
    let data_dir = dir.path().to_path_buf();
    let jsonl_path = data_dir.join("notifications.jsonl");
    let gh = GithubClient::new(server.uri(), "tok".into()).unwrap();
    let f = Flusher::new(gh, data_dir.clone(), jsonl_path);

    f.flush_pr(key("o", "r", 1), 1).await;
    assert_eq!(
        f.watermark(&key("o", "r", 1)).as_deref(),
        Some("2026-04-20T05:00:00Z")
    );

    // Second call MUST pass since=T1; mock requires that query param to match.
    let since_mock = Mock::given(method("GET"))
        .and(path("/repos/o/r/pulls/1/comments"))
        .and(query_param("since", "2026-04-20T05:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .named("since-filtered pulls");
    server.register(since_mock).await;
    let since_mock2 = Mock::given(method("GET"))
        .and(path("/repos/o/r/issues/1/comments"))
        .and(query_param("since", "2026-04-20T05:00:00Z"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .named("since-filtered issues");
    server.register(since_mock2).await;

    f.flush_pr(key("o", "r", 1), 1).await;
    // Wiremock verifies `.expect(1)` on drop — if `since` wasn't passed, this test fails.
}
