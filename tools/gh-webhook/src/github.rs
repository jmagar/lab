use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode, header};
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Deserialize)]
pub struct Comment {
    pub id: u64,
    pub user: User,
    pub body: String,
    pub created_at: String,
    /// GitHub's `since=` filter is based on `updated_at`, not `created_at`.
    /// We use this for the watermark so edits don't re-deliver, and so we
    /// don't miss an edit to an older comment.
    #[serde(default)]
    pub updated_at: String,
    pub html_url: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub line: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct User {
    pub login: String,
}

pub struct GithubClient {
    base: String,
    base_host: String,
    token: String,
    http: Client,
}

const MAX_PAGES: usize = 20; // 20 pages × 100 per-page = 2000 comments per endpoint

impl GithubClient {
    pub fn new(base: String, token: String) -> Result<Self> {
        let base_host = url::Url::parse(&base)
            .context("base url")?
            .host_str()
            .context("base url must have a host")?
            .to_string();
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("gh-webhook/0.1")
            .build()?;
        Ok(Self { base, base_host, token, http })
    }

    pub async fn list_pr_comments(
        &self,
        owner: &str,
        repo: &str,
        pr: u64,
        since: Option<&str>,
    ) -> Result<Vec<Comment>> {
        // PR review comments. issue_comment events land here too via /issues/{n}/comments;
        // for the MVP we aggregate both endpoints and dedup by comment id.
        let mut out: Vec<Comment> = Vec::new();
        let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for suffix in [format!("pulls/{pr}/comments"), format!("issues/{pr}/comments")] {
            let mut url = format!("{}/repos/{owner}/{repo}/{suffix}?per_page=100", self.base);
            if let Some(s) = since {
                url.push_str(&format!("&since={s}"));
            }
            let mut pages = 0usize;
            loop {
                let resp = self.get_with_retry(&url).await?;
                let next = parse_next_link(resp.headers().get(header::LINK));
                let page: Vec<Comment> = resp.json().await.context("decode comments json")?;
                for c in page {
                    if seen.insert(c.id) {
                        out.push(c);
                    }
                }
                pages += 1;
                match next {
                    Some(u) if pages < MAX_PAGES => {
                        // SSRF guard: require next-page URL to live on the same host as base.
                        let next_host = url::Url::parse(&u)
                            .ok()
                            .and_then(|v| v.host_str().map(str::to_owned));
                        if next_host.as_deref() != Some(&self.base_host) {
                            warn!(target: "gh_webhook::github",
                                next_host = ?next_host, expected = %self.base_host,
                                "refusing cross-host pagination (possible SSRF)");
                            break;
                        }
                        url = u;
                    }
                    Some(_) => {
                        warn!(target: "gh_webhook::github", pages, "pagination cap hit, truncating");
                        break;
                    }
                    None => break,
                }
            }
        }
        Ok(out)
    }

    async fn get_with_retry(&self, url: &str) -> Result<reqwest::Response> {
        for attempt in 0..3 {
            let resp = self
                .http
                .get(url)
                .bearer_auth(&self.token)
                .header(header::ACCEPT, "application/vnd.github+json")
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await
                .context("github GET")?;
            let status = resp.status();
            if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::FORBIDDEN {
                // GitHub rate-limit contract:
                //   1. `retry-after` is integer seconds (not HTTP-date). Honor if present.
                //   2. On primary limit, `x-ratelimit-remaining=0` + `x-ratelimit-reset=<epoch>`.
                //   3. Otherwise fall back to 60s.
                let headers = resp.headers();
                let retry_after = headers
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok());
                let wait = retry_after.unwrap_or_else(|| {
                    let remaining = headers
                        .get("x-ratelimit-remaining")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok());
                    let reset = headers
                        .get("x-ratelimit-reset")
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok());
                    if remaining == Some(0) {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_secs())
                            .unwrap_or(0);
                        reset.and_then(|r| r.checked_sub(now)).unwrap_or(60)
                    } else {
                        60
                    }
                });
                warn!(target: "gh_webhook::github", attempt, wait_s = wait, "rate limited, retrying");
                tokio::time::sleep(Duration::from_secs(wait.min(300))).await;
                continue;
            }
            return resp.error_for_status().context("github response");
        }
        anyhow::bail!("github: rate-limited after 3 retries")
    }
}

fn parse_next_link(link: Option<&header::HeaderValue>) -> Option<String> {
    let v = link?.to_str().ok()?;
    for part in v.split(',') {
        let part = part.trim();
        if part.ends_with("rel=\"next\"") {
            let lt = part.find('<')?;
            let gt = part.find('>')?;
            return Some(part[lt + 1..gt].to_string());
        }
    }
    None
}
