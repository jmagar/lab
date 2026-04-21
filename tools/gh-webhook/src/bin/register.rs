//! `gh-webhook-register` — register a GitHub webhook for a repository.
//!
//! Reads the GitHub token and webhook secret from environment variables
//! (configurable via `--token-env` / `--secret-env`), constructs the hook
//! payload, and `POST`s it to the GitHub REST API. The `--dry-run` flag
//! prints the payload without contacting the API.

use anyhow::{Context, Result, bail};
use clap::Parser;
use serde_json::json;

const DEFAULT_EVENTS: &[&str] = &[
    "pull_request",
    "pull_request_review",
    "pull_request_review_comment",
    "issue_comment",
    "workflow_run",
];

fn default_events_csv() -> String {
    DEFAULT_EVENTS.join(",")
}

/// Register a GitHub webhook for a repository.
#[derive(Parser, Debug)]
#[command(
    name = "gh-webhook-register",
    about = "Register a GitHub webhook for the gh-webhook server."
)]
struct Cli {
    /// `owner/repo` slug, e.g. `jmagar/lab`.
    #[arg(long)]
    repo: String,

    /// Public webhook URL (e.g. `https://host.ts.net/gh-webhook/webhook`).
    #[arg(long)]
    url: String,

    /// Comma-separated list of events to subscribe to.
    #[arg(long, default_value_t = default_events_csv())]
    events: String,

    /// Environment variable holding the GitHub token (Bearer).
    #[arg(long, default_value = "GH_WEBHOOK_GITHUB_TOKEN")]
    token_env: String,

    /// Environment variable holding the webhook shared secret.
    #[arg(long, default_value = "GH_WEBHOOK_SECRET")]
    secret_env: String,

    /// Print the payload and target URL without calling the GitHub API.
    #[arg(long)]
    dry_run: bool,
}

fn parse_events(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let (owner, repo) = cli
        .repo
        .split_once('/')
        .context("--repo must be in the form owner/repo")?;
    if owner.is_empty() || repo.is_empty() {
        bail!("--repo must be in the form owner/repo");
    }

    let events = parse_events(&cli.events);
    if events.is_empty() {
        bail!("--events must contain at least one event");
    }

    let secret = std::env::var(&cli.secret_env)
        .with_context(|| format!("environment variable {} is required", cli.secret_env))?;

    let body = json!({
        "name": "web",
        "active": true,
        "events": events,
        "config": {
            "url": cli.url,
            "content_type": "json",
            "secret": secret,
            "insecure_ssl": "0",
        },
    });

    let api_url = format!("https://api.github.com/repos/{owner}/{repo}/hooks");

    if cli.dry_run {
        // Redact the secret in dry-run output so it is safe to paste.
        let mut redacted = body.clone();
        if let Some(cfg) = redacted.get_mut("config").and_then(|v| v.as_object_mut()) {
            cfg.insert("secret".to_string(), json!("***redacted***"));
        }
        println!("POST {api_url}");
        println!("{}", serde_json::to_string_pretty(&redacted)?);
        return Ok(());
    }

    let token = std::env::var(&cli.token_env)
        .with_context(|| format!("environment variable {} is required", cli.token_env))?;

    let resp = reqwest::blocking::Client::new()
        .post(&api_url)
        .bearer_auth(&token)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "gh-webhook-register")
        .json(&body)
        .send()
        .context("sending request to GitHub")?;

    let status = resp.status();
    let text = resp.text().unwrap_or_default();
    if !status.is_success() {
        bail!("GitHub API returned {status}: {text}");
    }
    println!("registered webhook for {owner}/{repo}: {status}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_events_splits_csv_and_trims() {
        let events = parse_events("pull_request, issue_comment ,workflow_run");
        assert_eq!(events, vec!["pull_request", "issue_comment", "workflow_run"]);
    }

    #[test]
    fn parse_events_ignores_empties() {
        let events = parse_events(",,pull_request,,");
        assert_eq!(events, vec!["pull_request"]);
    }

    #[test]
    fn default_events_csv_has_all_defaults() {
        let csv = default_events_csv();
        let parsed = parse_events(&csv);
        assert_eq!(parsed.len(), DEFAULT_EVENTS.len());
        for e in DEFAULT_EVENTS {
            assert!(parsed.iter().any(|p| p == e), "missing {e}");
        }
    }
}
