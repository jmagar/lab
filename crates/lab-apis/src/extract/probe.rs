use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

use reqwest::{Client, StatusCode, header::LOCATION};
use tracing::{debug, warn};
use url::Url;

use super::error::ExtractError;

static PROBE_CLIENT: LazyLock<Result<Client, String>> = LazyLock::new(|| {
    Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|error| error.to_string())
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedEndpoint {
    pub url: String,
    pub status: u16,
}

#[must_use]
pub fn choose_probe_host(
    tailscale_ip: Option<&str>,
    ssh_hostname: Option<&str>,
    ssh_hostname_resolves: bool,
    alias: &str,
    alias_resolves: bool,
) -> Option<String> {
    tailscale_ip
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| {
            ssh_hostname
                .filter(|value| !value.is_empty() && ssh_hostname_resolves)
                .map(ToOwned::to_owned)
        })
        .or_else(|| alias_resolves.then(|| alias.to_owned()))
}

#[must_use]
pub fn refine_base_path(url: &str, base_path: Option<&str>) -> Option<String> {
    let base_path = base_path?.trim();
    if base_path.is_empty() {
        return Some(url.to_owned());
    }

    let mut parsed = Url::parse(url).ok()?;
    parsed.set_path(base_path);
    Some(parsed.to_string().trim_end_matches('/').to_owned())
}

pub async fn probe_endpoint(url: &str) -> Result<Option<VerifiedEndpoint>, ExtractError> {
    let started = Instant::now();
    let client = probe_client()?;

    let mut current = Url::parse(url).map_err(probe_error)?;

    for _ in 0..5 {
        let response = match client.get(current.clone()).send().await {
            Ok(response) => response,
            Err(err) if err.is_connect() || err.is_timeout() || err.is_request() => {
                debug!(
                    host = current.host_str().unwrap_or("-"),
                    elapsed_ms = started.elapsed().as_millis(),
                    "extract probe unreachable"
                );
                return Ok(None);
            }
            Err(err) => return Err(probe_error(err)),
        };

        let status = response.status();
        if is_reachable_status(status) {
            if is_followable_redirect(status) {
                if let Some(next) = response
                    .headers()
                    .get(LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| current.join(value).ok())
                {
                    current = next;
                    continue;
                }
            }

            debug!(
                host = current.host_str().unwrap_or("-"),
                status = status.as_u16(),
                elapsed_ms = started.elapsed().as_millis(),
                "extract probe reachable"
            );
            return Ok(Some(VerifiedEndpoint {
                url: current.to_string().trim_end_matches('/').to_owned(),
                status: status.as_u16(),
            }));
        }

        debug!(
            host = current.host_str().unwrap_or("-"),
            status = status.as_u16(),
            elapsed_ms = started.elapsed().as_millis(),
            "extract probe rejected"
        );
        return Ok(None);
    }

    Ok(None)
}

fn probe_client() -> Result<&'static Client, ExtractError> {
    PROBE_CLIENT.as_ref().map_err(|error| {
        warn!(
            kind = "internal_error",
            message = %error,
            "extract probe client init failed"
        );
        probe_error(error)
    })
}

fn is_reachable_status(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::OK
            | StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
            | StatusCode::UNAUTHORIZED
            | StatusCode::FORBIDDEN
    )
}

fn is_followable_redirect(status: StatusCode) -> bool {
    matches!(
        status,
        StatusCode::MOVED_PERMANENTLY
            | StatusCode::FOUND
            | StatusCode::SEE_OTHER
            | StatusCode::TEMPORARY_REDIRECT
            | StatusCode::PERMANENT_REDIRECT
    )
}

fn probe_error(error: impl std::fmt::Display) -> ExtractError {
    drop(error);
    ExtractError::Parse {
        service: "extract".to_owned(),
        path: std::path::PathBuf::new(),
        message: "probe request failed".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn tailscale_ip_is_preferred_over_ssh_hostname_and_alias() {
        let host = choose_probe_host(
            Some("100.64.0.12"),
            Some("media.example.ts.net"),
            true,
            "media",
            true,
        );

        assert_eq!(host.as_deref(), Some("100.64.0.12"));
    }

    #[test]
    fn ssh_hostname_is_preferred_over_alias() {
        let host = choose_probe_host(None, Some("media.example.ts.net"), true, "media", true);

        assert_eq!(host.as_deref(), Some("media.example.ts.net"));
    }

    #[test]
    fn ssh_hostname_is_skipped_when_not_locally_resolvable() {
        let host = choose_probe_host(None, Some("media.example.ts.net"), false, "media", true);

        assert_eq!(host.as_deref(), Some("media"));
    }

    #[test]
    fn alias_is_used_only_when_locally_resolvable() {
        assert_eq!(
            choose_probe_host(None, None, false, "media", true).as_deref(),
            Some("media")
        );
        assert_eq!(choose_probe_host(None, None, false, "media", false), None);
    }

    #[tokio::test]
    async fn probe_accepts_documented_success_statuses() {
        let server = MockServer::start().await;
        for (path_name, status) in [
            ("/ok", 200),
            ("/moved", 301),
            ("/found", 302),
            ("/see-other", 303),
            ("/temporary-redirect", 307),
            ("/permanent-redirect", 308),
            ("/unauthorized", 401),
            ("/forbidden", 403),
        ] {
            Mock::given(method("GET"))
                .and(path(path_name))
                .respond_with(ResponseTemplate::new(status))
                .mount(&server)
                .await;
        }

        for path_name in [
            "/ok",
            "/moved",
            "/found",
            "/see-other",
            "/temporary-redirect",
            "/permanent-redirect",
            "/unauthorized",
            "/forbidden",
        ] {
            let endpoint = probe_endpoint(&format!("{}{}", server.uri(), path_name))
                .await
                .expect("probe request")
                .expect("reachable endpoint");
            assert!(endpoint.status >= 200);
        }
    }

    #[tokio::test]
    async fn probe_follows_redirects_and_refines_base_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(302)
                    .insert_header("Location", format!("{}/radarr", server.uri())),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/radarr"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let endpoint = probe_endpoint(&server.uri())
            .await
            .expect("probe request")
            .expect("reachable endpoint");

        assert_eq!(endpoint.url, format!("{}/radarr", server.uri()));
        assert_eq!(
            refine_base_path(&server.uri(), Some("/radarr")).as_deref(),
            Some(format!("{}/radarr", server.uri()).as_str())
        );
    }

    #[tokio::test]
    async fn probe_follows_temporary_redirects_to_login() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(307)
                    .insert_header("Location", format!("{}/login", server.uri())),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let endpoint = probe_endpoint(&server.uri())
            .await
            .expect("probe request")
            .expect("reachable endpoint");

        assert_eq!(endpoint.url, format!("{}/login", server.uri()));
        assert_eq!(endpoint.status, 200);
    }

    #[tokio::test]
    async fn probe_follows_see_other_redirects() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(303)
                    .insert_header("Location", format!("{}/login", server.uri())),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let endpoint = probe_endpoint(&server.uri())
            .await
            .expect("probe request")
            .expect("reachable endpoint");

        assert_eq!(endpoint.url, format!("{}/login", server.uri()));
        assert_eq!(endpoint.status, 200);
    }

    #[tokio::test]
    async fn probe_returns_none_after_redirect_hop_limit() {
        let server = MockServer::start().await;
        for index in 0..6 {
            let current = if index == 0 {
                "/".to_owned()
            } else {
                format!("/hop-{index}")
            };
            let next = format!("{}/hop-{}", server.uri(), index + 1);
            Mock::given(method("GET"))
                .and(path(current))
                .respond_with(ResponseTemplate::new(302).insert_header("Location", next))
                .mount(&server)
                .await;
        }

        let endpoint = probe_endpoint(&server.uri()).await.expect("probe request");

        assert_eq!(endpoint, None);
    }

    #[test]
    fn probe_errors_do_not_echo_credentials() {
        let error = probe_error("builder failure for http://user:secret@example.com");
        let rendered = error.to_string();

        assert!(rendered.contains("probe request failed"));
        assert!(!rendered.contains("user:secret"));
    }
}
