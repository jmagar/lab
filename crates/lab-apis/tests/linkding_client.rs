#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::linkding::LinkdingClient;
use lab_apis::linkding::types::{Bundle, BundleCreate, BookmarkAsset};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn make_client(base_url: &str) -> LinkdingClient {
    // Linkding uses `Authorization: Token <api_token>`
    let auth = Auth::ApiKey {
        header: "Authorization".into(),
        key: "Token test-token".into(),
    };
    LinkdingClient::new(base_url, auth).expect("client construction")
}

// ── Bundles ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn bundle_list_returns_value() {
    let server = MockServer::start().await;
    let body = serde_json::json!([
        {"id": 1, "name": "Rust reads", "search_query": "#rust", "description": null}
    ]);
    Mock::given(method("GET"))
        .and(path("/api/bundles/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.bundles_list().await.expect("bundles_list");
    assert_eq!(result[0]["name"], "Rust reads");
}

#[tokio::test]
async fn bundle_create_posts_and_returns_bundle() {
    let server = MockServer::start().await;
    let response_bundle = Bundle {
        id: 42,
        name: "My bundle".into(),
        search_query: "#rust".into(),
        description: Some("desc".into()),
    };
    Mock::given(method("POST"))
        .and(path("/api/bundles/"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&response_bundle))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let create = BundleCreate {
        name: "My bundle".into(),
        search_query: "#rust".into(),
        description: Some("desc".into()),
    };
    let result = client.bundle_create(&create).await.expect("bundle_create");
    assert_eq!(result.id, 42);
    assert_eq!(result.name, "My bundle");
}

#[tokio::test]
async fn bundle_update_patches_and_returns_bundle() {
    let server = MockServer::start().await;
    let response_bundle = Bundle {
        id: 5,
        name: "Updated".into(),
        search_query: "#updated".into(),
        description: None,
    };
    Mock::given(method("PATCH"))
        .and(path("/api/bundles/5/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_bundle))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let patch = serde_json::json!({"name": "Updated", "search_query": "#updated"});
    let result = client.bundle_update(5, &patch).await.expect("bundle_update");
    assert_eq!(result.name, "Updated");
}

#[tokio::test]
async fn bundle_delete_sends_delete_request() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/api/bundles/7/"))
        .respond_with(ResponseTemplate::new(204))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.bundle_delete(7).await.expect("bundle_delete");
}

// ── Bookmark Assets ───────────────────────────────────────────────────────────

#[tokio::test]
async fn bookmark_assets_returns_vec() {
    let server = MockServer::start().await;
    let assets = vec![BookmarkAsset {
        id: 10,
        asset_type: "snapshot".into(),
        status: "complete".into(),
        file_size: Some(1024),
        date_created: "2026-04-13T00:00:00Z".into(),
    }];
    Mock::given(method("GET"))
        .and(path("/api/bookmarks/99/assets/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&assets))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.bookmark_assets(99).await.expect("bookmark_assets");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].asset_type, "snapshot");
    assert_eq!(result[0].status, "complete");
}

#[tokio::test]
async fn bookmark_assets_upload_posts_multipart() {
    let server = MockServer::start().await;
    let returned_asset = BookmarkAsset {
        id: 55,
        asset_type: "pdf".into(),
        status: "pending".into(),
        file_size: None,
        date_created: "2026-04-13T00:00:00Z".into(),
    };
    Mock::given(method("POST"))
        .and(path("/api/bookmarks/12/assets/upload/"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&returned_asset))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .bookmark_assets_upload(12, "test.pdf".into(), b"PDF content".to_vec())
        .await
        .expect("bookmark_assets_upload");
    assert_eq!(result.id, 55);
    assert_eq!(result.asset_type, "pdf");
}
