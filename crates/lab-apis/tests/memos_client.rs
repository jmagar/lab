#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::memos::MemosClient;
use lab_apis::memos::types::{
    Attachment, Comment, CreateCommentRequest, CreateWebhookRequest, MemoUser, ShareLink,
    UserStats, Webhook,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn make_client(base_url: &str) -> MemosClient {
    MemosClient::new(
        base_url,
        Auth::Bearer {
            token: "test-token".into(),
        },
    )
    .expect("client construction")
}

// ── users_list ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn users_list_returns_vec() {
    let server = MockServer::start().await;
    let users = vec![MemoUser {
        name: "users/1".into(),
        username: Some("alice".into()),
        display_name: Some("Alice".into()),
        email: Some("alice@example.com".into()),
        role: Some("HOST".into()),
    }];
    Mock::given(method("GET"))
        .and(path("/api/v1/users"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&users))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.users_list().await.expect("users_list");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "users/1");
}

#[tokio::test]
async fn users_list_propagates_403() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/v1/users"))
        .respond_with(ResponseTemplate::new(403))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let err = client.users_list().await.unwrap_err();
    let kind = format!("{err:?}");
    // 403 maps to ApiError::Auth
    assert!(
        kind.contains("Api(Auth)"),
        "expected Auth error, got: {kind}"
    );
}

// ── user_stats ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn user_stats_uses_colon_path() {
    let server = MockServer::start().await;
    let stats = UserStats {
        memo_count: Some(42),
        tags: std::collections::HashMap::new(),
    };
    Mock::given(method("GET"))
        .and(path("/api/v1/users/me:getStats"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&stats))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.user_stats("users/me").await.expect("user_stats");
    assert_eq!(result.memo_count, Some(42));
}

#[tokio::test]
async fn user_me_uses_auth_endpoint() {
    let server = MockServer::start().await;
    let payload = serde_json::json!({
        "user": {
            "name": "users/1",
            "username": "alice"
        }
    });
    Mock::given(method("GET"))
        .and(path("/api/v1/auth/me"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&payload))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.user_me().await.expect("user_me");
    assert_eq!(result["user"]["name"], "users/1");
}

// ── webhooks_list ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn webhooks_list_returns_vec() {
    let server = MockServer::start().await;
    let webhooks = vec![Webhook {
        name: "users/1/webhooks/7".into(),
        display_name: Some("My hook".into()),
        url: "https://example.com/hook".into(),
        create_time: "2026-01-01T00:00:00Z".into(),
    }];
    Mock::given(method("GET"))
        .and(path("/api/v1/users/1/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&webhooks))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .webhooks_list("users/1")
        .await
        .expect("webhooks_list");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].url, "https://example.com/hook");
}

// ── webhook_create ────────────────────────────────────────────────────────────

#[tokio::test]
async fn webhook_create_posts_and_returns_webhook() {
    let server = MockServer::start().await;
    let webhook = Webhook {
        name: "users/1/webhooks/8".into(),
        display_name: Some("New hook".into()),
        url: "https://example.com/new".into(),
        create_time: "2026-01-01T00:00:00Z".into(),
    };
    Mock::given(method("POST"))
        .and(path("/api/v1/users/1/webhooks"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&webhook))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let req = CreateWebhookRequest {
        url: "https://example.com/new".into(),
        display_name: "New hook".into(),
    };
    let result = client
        .webhook_create("users/1", &req)
        .await
        .expect("webhook_create");
    assert_eq!(result.name, "users/1/webhooks/8");
}

// ── attachment_upload ─────────────────────────────────────────────────────────

#[tokio::test]
async fn attachment_upload_posts_multipart_and_returns_attachment() {
    let server = MockServer::start().await;
    let attachment = Attachment {
        name: "attachments/99".into(),
        filename: "test.txt".into(),
        content_type: "text/plain".into(),
        size: Some(12),
        external_link: None,
    };
    Mock::given(method("POST"))
        .and(path("/api/v1/attachments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&attachment))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .attachment_upload("test.txt", b"hello world!".to_vec(), "text/plain")
        .await
        .expect("attachment_upload");
    assert_eq!(result.name, "attachments/99");
    assert_eq!(result.filename, "test.txt");
}

// ── attachment_delete ─────────────────────────────────────────────────────────

#[tokio::test]
async fn attachment_delete_sends_delete_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/v1/attachments/99"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .attachment_delete("attachments/99")
        .await
        .expect("attachment_delete");
}

// ── memo_comments_list ────────────────────────────────────────────────────────

#[tokio::test]
async fn memo_comments_list_returns_vec() {
    let server = MockServer::start().await;
    let comments = vec![Comment {
        name: "memos/123/comments/1".into(),
        content: "Nice memo!".into(),
        create_time: "2026-01-01T00:00:00Z".into(),
        update_time: "2026-01-01T00:00:00Z".into(),
    }];
    Mock::given(method("GET"))
        .and(path("/api/v1/memos/123/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&comments))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .memo_comments_list("memos/123")
        .await
        .expect("memo_comments_list");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].content, "Nice memo!");
}

// ── memo_comment_create ───────────────────────────────────────────────────────

#[tokio::test]
async fn memo_comment_create_posts_and_returns_comment() {
    let server = MockServer::start().await;
    let comment = Comment {
        name: "memos/123/comments/2".into(),
        content: "My comment".into(),
        create_time: "2026-01-01T00:00:00Z".into(),
        update_time: "2026-01-01T00:00:00Z".into(),
    };
    Mock::given(method("POST"))
        .and(path("/api/v1/memos/123/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&comment))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let req = CreateCommentRequest {
        content: "My comment".into(),
    };
    let result = client
        .memo_comment_create("memos/123", &req)
        .await
        .expect("memo_comment_create");
    assert_eq!(result.name, "memos/123/comments/2");
}

// ── memo_shares_list ──────────────────────────────────────────────────────────

#[tokio::test]
async fn memo_shares_list_returns_vec() {
    let server = MockServer::start().await;
    let shares = vec![ShareLink {
        name: "memos/123/shares/abc".into(),
        create_time: "2026-01-01T00:00:00Z".into(),
    }];
    Mock::given(method("GET"))
        .and(path("/api/v1/memos/123/shares"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&shares))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .memo_shares_list("memos/123")
        .await
        .expect("memo_shares_list");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "memos/123/shares/abc");
}

// ── memo_share_create ─────────────────────────────────────────────────────────

#[tokio::test]
async fn memo_share_create_posts_and_returns_share() {
    let server = MockServer::start().await;
    let share = ShareLink {
        name: "memos/123/shares/xyz".into(),
        create_time: "2026-01-01T00:00:00Z".into(),
    };
    Mock::given(method("POST"))
        .and(path("/api/v1/memos/123/shares"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&share))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .memo_share_create("memos/123")
        .await
        .expect("memo_share_create");
    assert_eq!(result.name, "memos/123/shares/xyz");
}
