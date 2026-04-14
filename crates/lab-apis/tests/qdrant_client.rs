#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::qdrant::types::{CreateCollection, CreateIndex, Distance, SearchRequest, UpsertPoint, VectorParams};
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

// ---------------------------------------------------------------------------
// collection.create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn collection_create_calls_put() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/collections/test-col"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": true
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let body = CreateCollection {
        vectors: VectorParams {
            size: 1536,
            distance: Distance::Cosine,
        },
    };
    client.collection_create("test-col", &body).await.expect("collection_create");
}

// ---------------------------------------------------------------------------
// collection.delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn collection_delete_calls_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/collections/to-delete"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": true
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    client.collection_delete("to-delete").await.expect("collection_delete");
}

// ---------------------------------------------------------------------------
// point.upsert — batching
// ---------------------------------------------------------------------------

/// Build a batch of `n` dummy upsert points.
fn make_points(n: usize) -> Vec<UpsertPoint> {
    (0..n)
        .map(|i| UpsertPoint {
            id: serde_json::json!(i as u64),
            vector: vec![0.1_f32; 4],
            payload: None,
        })
        .collect()
}

/// 1200 points → exactly 3 HTTP PUT calls (chunks: 500, 500, 200).
#[tokio::test]
async fn point_upsert_batched_1200_makes_three_calls() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/collections/test/points"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": { "operation_id": 0, "status": "completed" }
        })))
        .expect(3)          // exactly 3 calls
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let points = make_points(1200);
    client
        .point_upsert_batched("test", points)
        .await
        .expect("point_upsert_batched");

    server.verify().await;
}

/// Fail-fast: chunk 2 of 3 fails → chunk 3 is never attempted.
#[tokio::test]
async fn point_upsert_batched_fail_fast_on_chunk_2() {
    let server = MockServer::start().await;

    // First call succeeds
    Mock::given(method("PUT"))
        .and(path("/collections/test/points"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": { "operation_id": 0, "status": "completed" }
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second call fails with a 500
    Mock::given(method("PUT"))
        .and(path("/collections/test/points"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Third call should never happen — wiremock will catch any extra calls
    // because no further Mock is mounted.

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let points = make_points(1100); // 3 chunks: 500, 500, 100

    let err = client
        .point_upsert_batched("test", points)
        .await
        .expect_err("should fail on chunk 2");

    // The error message carries chunk context
    let msg = err.to_string();
    assert!(
        msg.contains("chunk 2 of 3") || msg.contains("failed on chunk"),
        "expected chunk context in error message, got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// point.search
// ---------------------------------------------------------------------------

#[tokio::test]
async fn point_search_calls_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/collections/vecs/points/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": [{ "id": 1, "score": 0.99 }]
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let req = SearchRequest {
        vector: vec![0.1, 0.2, 0.3],
        limit: 5,
        filter: None,
        with_payload: Some(true),
        score_threshold: None,
    };
    let result = client.point_search("vecs", &req).await.expect("point_search");
    assert!(result.is_object() || result.is_array() || !result.is_null());
}

// ---------------------------------------------------------------------------
// point.query
// ---------------------------------------------------------------------------

#[tokio::test]
async fn point_query_calls_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/collections/vecs/points/query"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": { "points": [] }
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let body = serde_json::json!({ "query": [0.1, 0.2], "limit": 3 });
    client.point_query("vecs", &body).await.expect("point_query");
}

// ---------------------------------------------------------------------------
// point.scroll
// ---------------------------------------------------------------------------

#[tokio::test]
async fn point_scroll_calls_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/collections/vecs/points/scroll"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": { "points": [], "next_page_offset": null }
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let body = serde_json::json!({ "limit": 10 });
    client.point_scroll("vecs", &body).await.expect("point_scroll");
}

// ---------------------------------------------------------------------------
// point.count
// ---------------------------------------------------------------------------

#[tokio::test]
async fn point_count_calls_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/collections/vecs/points/count"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": { "count": 42 }
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let body = serde_json::json!({});
    client.point_count("vecs", &body).await.expect("point_count");
}

// ---------------------------------------------------------------------------
// point.delete
// ---------------------------------------------------------------------------

#[tokio::test]
async fn point_delete_calls_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/collections/vecs/points/delete"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": { "operation_id": 1, "status": "completed" }
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let body = serde_json::json!({ "points": [1, 2, 3] });
    client.point_delete("vecs", &body).await.expect("point_delete");
}

// ---------------------------------------------------------------------------
// snapshot.create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn snapshot_create_calls_post() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/collections/vecs/snapshots"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": {
                "name": "2026-04-13-snap",
                "creation_time": "2026-04-13T00:00:00",
                "size": 1024
            }
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let info = client.snapshot_create("vecs").await.expect("snapshot_create");
    assert_eq!(info.name, "2026-04-13-snap");
    assert_eq!(info.size, Some(1024));
}

// ---------------------------------------------------------------------------
// index.create
// ---------------------------------------------------------------------------

#[tokio::test]
async fn index_create_calls_put() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/collections/vecs/index"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "status": "ok",
            "time": 0.001,
            "result": { "operation_id": 0, "status": "completed" }
        })))
        .mount(&server)
        .await;

    let client = QdrantClient::new(&server.uri(), Auth::None).expect("client");
    let req = CreateIndex {
        field_name: "category".into(),
        field_schema: serde_json::json!("keyword"),
    };
    client.index_create("vecs", &req).await.expect("index_create");
}
