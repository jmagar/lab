#![allow(clippy::expect_used, clippy::unwrap_used)]

use lab_apis::core::Auth;
use lab_apis::gotify::GotifyClient;
use lab_apis::gotify::types::{
    Application, ApplicationId, ApplicationParams, Client, ClientId, ClientParams, Plugin,
    PluginId, ServerVersion, UserCreate, UserId,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

fn make_client(base_url: &str) -> GotifyClient {
    GotifyClient::new(
        base_url,
        Auth::ApiKey {
            header: "X-Gotify-Key".into(),
            key: "test-token".into(),
        },
    )
    .expect("client construction")
}

// ── application.update ────────────────────────────────────────────────────────

#[tokio::test]
async fn app_update_puts_and_returns_application() {
    let server = MockServer::start().await;
    let app = Application {
        id: ApplicationId(3),
        name: "Renamed".into(),
        description: "Updated desc".into(),
        token: "tok".into(),
        internal: false,
        image: None,
    };
    Mock::given(method("PUT"))
        .and(path("/application/3"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&app))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let params = ApplicationParams {
        name: "Renamed".into(),
        description: Some("Updated desc".into()),
    };
    let result = client
        .app_update(ApplicationId(3), &params)
        .await
        .expect("app_update");
    assert_eq!(result.name, "Renamed");
}

// ── application.messages ──────────────────────────────────────────────────────

#[tokio::test]
async fn app_messages_list_returns_paged_messages() {
    let server = MockServer::start().await;
    let body = serde_json::json!({
        "paging": {"size": 0, "page": 1, "total_page": 1, "since": null, "limit": 100},
        "messages": []
    });
    Mock::given(method("GET"))
        .and(path("/application/5/message"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .app_messages_list(ApplicationId(5))
        .await
        .expect("app_messages_list");
    assert_eq!(result.messages.len(), 0);
}

// ── application.messages-delete ───────────────────────────────────────────────

#[tokio::test]
async fn app_messages_delete_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/application/7/message"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .app_messages_delete(ApplicationId(7))
        .await
        .expect("app_messages_delete");
}

// ── client.update ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn client_update_puts_and_returns_client() {
    let server = MockServer::start().await;
    let c = Client {
        id: ClientId(2),
        name: "NewName".into(),
        token: "ctok".into(),
    };
    Mock::given(method("PUT"))
        .and(path("/client/2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&c))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let params = ClientParams {
        name: "NewName".into(),
    };
    let result = client
        .client_update(ClientId(2), &params)
        .await
        .expect("client_update");
    assert_eq!(result.name, "NewName");
}

// ── plugin.list ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn plugins_list_returns_vec() {
    let server = MockServer::start().await;
    let plugins = vec![Plugin {
        id: PluginId(1),
        name: "myplugin".into(),
        module_path: None,
        enabled: true,
        capabilities: None,
        license: None,
        description: None,
        website: None,
    }];
    Mock::given(method("GET"))
        .and(path("/plugin"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&plugins))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.plugins_list().await.expect("plugins_list");
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "myplugin");
}

// ── plugin.enable ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn plugin_enable_posts_to_enable_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/plugin/1/enable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .plugin_enable(PluginId(1))
        .await
        .expect("plugin_enable");
}

// ── plugin.disable ────────────────────────────────────────────────────────────

#[tokio::test]
async fn plugin_disable_posts_to_disable_endpoint() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/plugin/1/disable"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .plugin_disable(PluginId(1))
        .await
        .expect("plugin_disable");
}

// ── plugin.config-get ─────────────────────────────────────────────────────────

#[tokio::test]
async fn plugin_config_get_returns_yaml_string() {
    let server = MockServer::start().await;
    let yaml = "key: value\nother: 42\n";
    Mock::given(method("GET"))
        .and(path("/plugin/1/config"))
        .respond_with(ResponseTemplate::new(200).set_body_string(yaml))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client
        .plugin_config_get(PluginId(1))
        .await
        .expect("plugin_config_get");
    assert_eq!(result, yaml);
}

// ── plugin.config-set ─────────────────────────────────────────────────────────

#[tokio::test]
async fn plugin_config_set_posts_text() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/plugin/1/config"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client
        .plugin_config_set(PluginId(1), "key: value\n")
        .await
        .expect("plugin_config_set");
}

// ── user.list ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn users_list_returns_value() {
    let server = MockServer::start().await;
    let users = serde_json::json!([{"id": 1, "name": "admin", "admin": true}]);
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&users))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.users_list().await.expect("users_list");
    assert_eq!(result[0]["name"], "admin");
}

// ── user.create ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn user_create_posts_and_returns_user() {
    let server = MockServer::start().await;
    let response = serde_json::json!({"id": 2, "name": "newuser", "admin": false});
    Mock::given(method("POST"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let params = UserCreate {
        name: "newuser".into(),
        pass: "secret".into(),
        admin: false,
    };
    let result = client.user_create(&params).await.expect("user_create");
    assert_eq!(result["name"], "newuser");
}

// ── user.delete ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn user_delete_sends_delete() {
    let server = MockServer::start().await;
    Mock::given(method("DELETE"))
        .and(path("/user/2"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    client.user_delete(UserId(2)).await.expect("user_delete");
}

// ── server.version ────────────────────────────────────────────────────────────

#[tokio::test]
async fn server_version_returns_version_info() {
    let server = MockServer::start().await;
    let version = ServerVersion {
        version: "2.0.2".into(),
        commit: "abc123".into(),
        build_date: "2024-01-01T00:00:00Z".into(),
    };
    Mock::given(method("GET"))
        .and(path("/version"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&version))
        .mount(&server)
        .await;

    let client = make_client(&server.uri());
    let result = client.server_version().await.expect("server_version");
    assert_eq!(result.version, "2.0.2");
    assert_eq!(result.commit, "abc123");
}
