#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration test — `HttpClient::get_json` must inject the Auth header
//! and decode a JSON body into a user-provided type.

use serde::Deserialize;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

use lab_apis::core::{Auth, HttpClient};

#[derive(Debug, serde::Serialize, Deserialize, PartialEq)]
struct Pong {
    message: String,
}

#[tokio::test]
async fn get_json_injects_api_key_header_and_decodes_body() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/ping"))
        .and(header("X-Api-Key", "secret"))
        .respond_with(ResponseTemplate::new(200).set_body_json(Pong {
            message: "pong".into(),
        }))
        .mount(&server)
        .await;

    let client = HttpClient::new(
        server.uri(),
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "secret".into(),
        },
    )
    .expect("HttpClient::new");

    let pong: Pong = client.get_json("/ping").await.expect("get_json");
    assert_eq!(
        pong,
        Pong {
            message: "pong".into()
        }
    );
}

#[tokio::test]
async fn get_json_returns_not_found_on_404() {
    use lab_apis::core::ApiError;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/missing"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = HttpClient::new(
        server.uri(),
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "secret".into(),
        },
    )
    .expect("HttpClient::new");

    let err: ApiError = client
        .get_json::<Pong>("/missing")
        .await
        .expect_err("should fail on 404");
    assert!(matches!(err, ApiError::NotFound), "expected NotFound, got {err:?}");
}

#[tokio::test]
async fn get_json_returns_auth_failed_on_401() {
    use lab_apis::core::ApiError;

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/secure"))
        .respond_with(ResponseTemplate::new(401))
        .mount(&server)
        .await;

    let client = HttpClient::new(
        server.uri(),
        Auth::ApiKey {
            header: "X-Api-Key".into(),
            key: "wrong".into(),
        },
    )
    .expect("HttpClient::new");

    let err: ApiError = client
        .get_json::<Pong>("/secure")
        .await
        .expect_err("should fail on 401");
    assert!(matches!(err, ApiError::Auth), "expected Auth, got {err:?}");
}
