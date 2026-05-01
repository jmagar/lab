use lab_apis::core::ApiError;
use lab_apis::jellyfin::JellyfinClient;
use lab_apis::jellyfin::types::{ItemsQuery, SystemInfo};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(base_url: &str) -> JellyfinClient {
    JellyfinClient::new(base_url, JellyfinClient::auth_from_api_key("secret-token")).unwrap()
}

#[tokio::test]
async fn system_info_deserializes_pascal_case_fields() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/System/Info"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Id": "server-1",
            "ServerName": "media",
            "Version": "10.11.8",
            "ProductName": "Jellyfin Server"
        })))
        .mount(&server)
        .await;

    let info = client(&server.uri()).system_info().await.unwrap();

    assert_eq!(info.server_name.as_deref(), Some("media"));
    assert_eq!(info.version.as_deref(), Some("10.11.8"));
}

#[tokio::test]
async fn authenticated_users_request_uses_mediabrowser_authorization_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/Users"))
        .and(header(
            "Authorization",
            "MediaBrowser Token=\"secret-token\"",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            { "Id": "user-1", "Name": "Alice" }
        ])))
        .mount(&server)
        .await;

    let users = client(&server.uri()).users().await.unwrap();

    assert_eq!(users.len(), 1);
    assert_eq!(users[0].name.as_deref(), Some("Alice"));
}

#[tokio::test]
async fn items_query_maps_bounded_params_to_upstream_names() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/Items"))
        .and(query_param("searchTerm", "matrix"))
        .and(query_param("includeItemTypes", "Movie,Series"))
        .and(query_param("recursive", "true"))
        .and(query_param("limit", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Items": [{ "Id": "item-1", "Name": "The Matrix", "Type": "Movie" }],
            "TotalRecordCount": 1,
            "StartIndex": 0
        })))
        .mount(&server)
        .await;

    let result = client(&server.uri())
        .items(&ItemsQuery {
            search_term: Some("matrix".into()),
            include_item_types: Some(vec!["Movie".into(), "Series".into()]),
            recursive: Some(true),
            limit: Some(25),
            ..ItemsQuery::default()
        })
        .await
        .unwrap();

    assert_eq!(result.total_record_count, Some(1));
    assert_eq!(result.items[0].item_type.as_deref(), Some("Movie"));
}

#[tokio::test]
async fn non_success_status_maps_to_api_error_kind() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/Users"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let err = client(&server.uri()).users().await.unwrap_err();

    assert!(matches!(
        err,
        lab_apis::jellyfin::JellyfinError::Api(ApiError::Auth)
    ));
}

#[test]
fn system_info_serialization_uses_upstream_field_names() {
    let value = serde_json::to_value(SystemInfo {
        id: Some("server-1".into()),
        server_name: Some("media".into()),
        version: Some("10.11.8".into()),
        product_name: None,
        operating_system: None,
        operating_system_display_name: None,
        startup_wizard_completed: Some(true),
        extra: serde_json::json!({}),
    })
    .unwrap();

    assert_eq!(value["ServerName"], "media");
    assert_eq!(value["StartupWizardCompleted"], true);
}
