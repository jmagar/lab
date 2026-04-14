//! Wiremock-based unit tests for the 10 new `PaperlessClient` actions.
#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::paperless::PaperlessClient;
use lab_apis::paperless::types::{
    CustomFieldCreateRequest, DocumentBulkEditRequest, TagUpdateRequest,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn make_client(base_url: &str) -> PaperlessClient {
    let auth = Auth::ApiKey {
        header: "Authorization".into(),
        key: "Token test-token".into(),
    };
    PaperlessClient::new(base_url, auth).expect("client construction")
}

// ── document.upload ────────────────────────────────────────────────────────────

#[tokio::test]
async fn document_upload_posts_multipart() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"task_id": "abc123"});
    Mock::given(method("POST"))
        .and(path("/api/documents/post_document/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .document_upload(
            b"PDF content".to_vec(),
            "invoice.pdf".to_string(),
            Some("Test Invoice".to_string()),
            None,
            None,
            None,
        )
        .await
        .expect("document_upload");
    assert_eq!(result["task_id"], "abc123");
}

// ── document.bulk-edit ─────────────────────────────────────────────────────────

#[tokio::test]
async fn document_bulk_edit_posts_json() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"result": "OK"});
    Mock::given(method("POST"))
        .and(path("/api/documents/bulk_edit/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let req = DocumentBulkEditRequest {
        documents: vec![1, 2, 3],
        method: "set_correspondent".to_string(),
        parameters: serde_json::json!({"correspondent": 5}),
    };
    let result = client.document_bulk_edit(&req).await.expect("document_bulk_edit");
    assert_eq!(result["result"], "OK");
}

// ── document.download ──────────────────────────────────────────────────────────

#[tokio::test]
async fn document_download_returns_base64_content() {
    let server = MockServer::start().await;
    let file_content = b"PDF-BINARY-CONTENT";
    Mock::given(method("GET"))
        .and(path("/api/documents/42/download/"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(file_content.as_ref()))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.document_download(42, false).await.expect("document_download");
    assert_eq!(result.size, file_content.len());
    // Verify the content can be decoded back
    use base64::Engine as _;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&result.content_base64)
        .expect("valid base64");
    assert_eq!(decoded, file_content);
}

// ── tag.update ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn tag_update_patches_tag() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": 7, "name": "invoices", "color": "#00ff00"});
    Mock::given(method("PATCH"))
        .and(path("/api/tags/7/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let req = TagUpdateRequest {
        name: Some("invoices".to_string()),
        colour: Some("#00ff00".to_string()),
        match_expr: None,
    };
    let result = client.tag_update(7, &req).await.expect("tag_update");
    assert_eq!(result["name"], "invoices");
}

// ── saved-view.list ────────────────────────────────────────────────────────────

#[tokio::test]
async fn saved_views_list_returns_value() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"count": 1, "results": [{"id": 1, "name": "Inbox"}]});
    Mock::given(method("GET"))
        .and(path("/api/saved_views/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.saved_views_list().await.expect("saved_views_list");
    assert_eq!(result["count"], 1);
}

// ── saved-view.create ──────────────────────────────────────────────────────────

#[tokio::test]
async fn saved_view_create_posts_and_returns() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": 10, "name": "My View"});
    Mock::given(method("POST"))
        .and(path("/api/saved_views/"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let payload = serde_json::json!({"name": "My View", "filter_rules": []});
    let result = client.saved_view_create(&payload).await.expect("saved_view_create");
    assert_eq!(result["id"], 10);
}

// ── custom-field.list ──────────────────────────────────────────────────────────

#[tokio::test]
async fn custom_fields_list_returns_value() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"count": 2, "results": [
        {"id": 1, "name": "Invoice Amount", "data_type": "monetary"},
        {"id": 2, "name": "Due Date", "data_type": "date"}
    ]});
    Mock::given(method("GET"))
        .and(path("/api/custom_fields/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.custom_fields_list().await.expect("custom_fields_list");
    assert_eq!(result["count"], 2);
}

// ── custom-field.create ────────────────────────────────────────────────────────

#[tokio::test]
async fn custom_field_create_posts_and_returns() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": 3, "name": "Reference", "data_type": "string"});
    Mock::given(method("POST"))
        .and(path("/api/custom_fields/"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let req = CustomFieldCreateRequest {
        name: "Reference".to_string(),
        data_type: "string".to_string(),
    };
    let result = client.custom_field_create(&req).await.expect("custom_field_create");
    assert_eq!(result["id"], 3);
}

// ── storage-path.list ──────────────────────────────────────────────────────────

#[tokio::test]
async fn storage_paths_list_returns_value() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"count": 1, "results": [{"id": 1, "name": "Default", "path": "/docs/{{created}}/{{title}}"}]});
    Mock::given(method("GET"))
        .and(path("/api/storage_paths/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.storage_paths_list().await.expect("storage_paths_list");
    assert_eq!(result["count"], 1);
}

// ── storage-path.create ────────────────────────────────────────────────────────

#[tokio::test]
async fn storage_path_create_posts_and_returns() {
    let server = MockServer::start().await;
    let body = serde_json::json!({"id": 5, "name": "Invoices", "path": "/invoices/{{title}}"});
    Mock::given(method("POST"))
        .and(path("/api/storage_paths/"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let payload = serde_json::json!({"name": "Invoices", "path": "/invoices/{{title}}"});
    let result = client.storage_path_create(&payload).await.expect("storage_path_create");
    assert_eq!(result["id"], 5);
}
