#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::qdrant::QdrantClient;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

#[tokio::test]
async fn collections_list_decodes_names() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/collections"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": {
                "collections": [
                    { "name": "movies" },
                    { "name": "shows" }
                ]
            }
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let collections = client.collections_list().await.expect("collections_list");
    assert_eq!(collections.len(), 2);
    assert_eq!(collections[0].name.0, "movies");
}

#[tokio::test]
async fn collection_get_decodes_result_payload() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/collections/movies"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": {
                "status": "green",
                "vectors_count": 42
            }
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let collection = client
        .collection_get("movies")
        .await
        .expect("collection_get");
    assert_eq!(collection.status, "green");
    assert_eq!(collection.vectors_count, Some(42));
}
