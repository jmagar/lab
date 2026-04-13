#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::tei::{TeiClient, types::EmbedRequest};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, method, path},
};

#[tokio::test]
async fn model_info_decodes_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "model_id": "bge-small-en-v1.5",
            "model_sha": "abc123",
            "model_dtype": "float16",
            "model_type": "embedding",
            "max_concurrent_requests": 32,
            "max_input_length": 512,
            "max_batch_tokens": 4096,
            "version": "1.9.3"
        })))
        .mount(&server)
        .await;

    let client = TeiClient::new(&server.uri(), Auth::None).expect("client");
    let info = client.model_info().await.expect("model_info");
    assert_eq!(info.model_id, "bge-small-en-v1.5");
    assert_eq!(info.version.as_deref(), Some("1.9.3"));
}

#[tokio::test]
async fn embed_posts_json_and_decodes_vectors() {
    let server = MockServer::start().await;
    let request = EmbedRequest {
        inputs: vec!["hello".into(), "world".into()],
        normalize: Some(true),
        truncate: Some(false),
    };

    Mock::given(method("POST"))
        .and(path("/embed"))
        .and(body_json(&request))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!([[0.1, 0.2], [0.3, 0.4]])),
        )
        .mount(&server)
        .await;

    let client = TeiClient::new(&server.uri(), Auth::None).expect("client");
    let vectors = client.embed(&request).await.expect("embed");
    assert_eq!(vectors.len(), 2);
    assert_eq!(vectors[0], vec![0.1, 0.2]);
}
