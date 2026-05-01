use lab_apis::core::Auth;
use lab_apis::immich::ImmichClient;
use lab_apis::immich::types::AssetSearchRequest;
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> ImmichClient {
    ImmichClient::new(
        &server.uri(),
        Auth::ApiKey {
            header: "x-api-key".into(),
            key: "immich-key".into(),
        },
    )
    .unwrap()
}

#[tokio::test]
async fn health_uses_server_ping_with_api_key() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/server/ping"))
        .and(header("x-api-key", "immich-key"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    client(&server).health().await.unwrap();
}

#[tokio::test]
async fn asset_search_clamps_limit() {
    let server = MockServer::start().await;
    let request = AssetSearchRequest {
        query: None,
        limit: 51,
        page: None,
    };
    let err = client(&server).asset_search(request).await.unwrap_err();

    assert!(err.to_string().contains("limit must be between 1 and 50"));
}

#[tokio::test]
async fn asset_search_redacts_sensitive_asset_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/search/metadata"))
        .and(body_string_contains("\"size\":1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "assets": {
                "items": [{
                    "id": "asset-1",
                    "originalPath": "/photos/private.jpg",
                    "ownerId": "owner-1",
                    "exifInfo": {"gps": "secret"}
                }],
                "page": 1
            }
        })))
        .mount(&server)
        .await;

    let response = client(&server)
        .asset_search(AssetSearchRequest {
            query: None,
            limit: 1,
            page: None,
        })
        .await
        .unwrap();
    let item = &response.items[0];

    assert_eq!(item["id"], "asset-1");
    assert!(item.get("originalPath").is_none());
    assert!(item.get("ownerId").is_none());
    assert!(item.get("exifInfo").is_none());
}
