use lab_apis::core::ApiError;
use lab_apis::pihole::{PiholeClient, PiholeError};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use wiremock::matchers::{body_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

fn client(server: &MockServer) -> PiholeClient {
    PiholeClient::new(&server.uri(), "secret-password".into(), None).unwrap()
}

fn auth_response(sid: &str) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "session": {
            "sid": sid,
            "csrf": "csrf-token",
            "validity": 300
        }
    }))
}

#[derive(Clone)]
struct RotatingAuth {
    count: Arc<AtomicUsize>,
}

impl Respond for RotatingAuth {
    fn respond(&self, _: &Request) -> ResponseTemplate {
        let attempt = self.count.fetch_add(1, Ordering::SeqCst);
        auth_response(if attempt == 0 { "sid-one" } else { "sid-two" })
    }
}

#[tokio::test]
async fn summary_logs_in_and_sends_session_headers() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth"))
        .and(body_json(
            serde_json::json!({"password": "secret-password"}),
        ))
        .respond_with(auth_response("sid-one"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/summaries"))
        .and(header("X-FTL-SID", "sid-one"))
        .and(header("X-FTL-CSRF", "csrf-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "queries": { "total": 10 }
        })))
        .mount(&server)
        .await;

    let response = client(&server).summary().await.unwrap();

    assert_eq!(response.value["queries"]["total"], 10);
}

#[tokio::test]
async fn cached_session_is_reused_for_consecutive_calls() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth"))
        .respond_with(auth_response("sid-one"))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/dns/blocking"))
        .and(header("X-FTL-SID", "sid-one"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "blocking": true
        })))
        .mount(&server)
        .await;

    let client = client(&server);
    client.blocking_status().await.unwrap();
    client.blocking_status().await.unwrap();
}

#[tokio::test]
async fn unauthorized_response_refreshes_session_once() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth"))
        .respond_with(RotatingAuth {
            count: Arc::new(AtomicUsize::new(0)),
        })
        .expect(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/settings"))
        .and(header("X-FTL-SID", "sid-one"))
        .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
            "error": { "message": "expired", "hint": "login again" }
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/settings"))
        .and(header("X-FTL-SID", "sid-two"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "dns": { "blocking": true }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let response = client(&server).settings().await.unwrap();

    assert_eq!(response.value["dns"]["blocking"], true);
}

#[tokio::test]
async fn querylog_sends_offset_limit_and_redacts_client_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth"))
        .respond_with(auth_response("sid-one"))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/api/logs"))
        .and(query_param("offset", "10"))
        .and(query_param("limit", "25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "queries": [{
                "domain": "example.com",
                "client": "192.0.2.10",
                "upstream": "resolver"
            }]
        })))
        .mount(&server)
        .await;

    let response = client(&server).querylog_search(10, 25).await.unwrap();

    assert_eq!(response.offset, 10);
    assert_eq!(response.limit, 25);
    assert_eq!(response.value["queries"][0]["domain"], "example.com");
    assert!(response.value["queries"][0].get("client").is_none());
    assert!(response.value["queries"][0].get("upstream").is_none());
}

#[tokio::test]
async fn login_error_maps_hint_into_validation_message() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/auth"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "error": { "message": "bad request", "hint": "missing password" }
        })))
        .mount(&server)
        .await;

    let err = client(&server).summary().await.unwrap_err();

    assert!(matches!(err, PiholeError::Api(ApiError::Validation { .. })));
    let message = match err {
        PiholeError::Api(ApiError::Validation { message, .. }) => message,
        _ => String::new(),
    };
    assert!(message.contains("missing password"));
}

#[tokio::test]
async fn querylog_rejects_unbounded_limit() {
    let server = MockServer::start().await;
    let err = client(&server).querylog_search(0, 501).await.unwrap_err();

    assert!(matches!(err, PiholeError::InvalidParam(_)));
}
