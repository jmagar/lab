use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventError {
    #[error("malformed webhook JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("required field missing: {0}")]
    Missing(&'static str),
}

#[derive(Debug, Clone)]
pub enum Event {
    PrComment {
        owner: String,
        repo: String,
        pr: u64,
        branch: String,
    },
    PrLifecycle {
        owner: String,
        repo: String,
        pr: u64,
        branch: String,
        action: String,
    },
    CiFailed {
        owner: String,
        repo: String,
        branch: String,
        url: String,
        name: String,
    },
    Ignored,
}

#[derive(Deserialize)]
struct Repo {
    owner: RepoOwner,
    name: String,
}
#[derive(Deserialize)]
struct RepoOwner {
    login: String,
}

#[derive(Deserialize)]
struct PrRef {
    number: u64,
    head: BranchRef,
}
#[derive(Deserialize)]
struct BranchRef {
    #[serde(rename = "ref")]
    r#ref: String,
}

#[derive(Deserialize)]
struct PrCommentPayload {
    repository: Repo,
    pull_request: PrRef,
}

/// `issue_comment` carries the PR ref only when the comment is on a PR.
/// Plain-issue comments have no `pull_request` field — treat as Ignored rather than error.
#[derive(Deserialize)]
struct IssueCommentPayload {
    repository: Repo,
    #[serde(default)]
    issue: Option<IssueBody>,
}
#[derive(Deserialize)]
struct IssueBody {
    number: u64,
    #[serde(default)]
    pull_request: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct PrPayload {
    action: String,
    repository: Repo,
    pull_request: PrRef,
}

#[derive(Deserialize)]
struct WorkflowRun {
    conclusion: Option<String>,
    head_branch: Option<String>,
    html_url: String,
    name: String,
}
#[derive(Deserialize)]
struct WorkflowPayload {
    repository: Repo,
    workflow_run: WorkflowRun,
}

pub fn parse_event(event_header: &str, body: &[u8]) -> Result<Event, EventError> {
    match event_header {
        "pull_request_review_comment" | "pull_request_review" => {
            let p: PrCommentPayload = serde_json::from_slice(body)?;
            Ok(Event::PrComment {
                owner: p.repository.owner.login,
                repo: p.repository.name,
                pr: p.pull_request.number,
                branch: p.pull_request.head.r#ref,
            })
        }
        "issue_comment" => {
            // Plain-issue comments have no `pull_request` field on `issue`. Ignore them
            // so a chatty issue thread does not spam the webhook dashboard with 400s.
            let p: IssueCommentPayload = serde_json::from_slice(body)?;
            let Some(issue) = p.issue else {
                return Ok(Event::Ignored);
            };
            if issue.pull_request.is_none() {
                return Ok(Event::Ignored);
            }
            // Branch name is not on the issue payload; use a sentinel. The real branch is
            // not needed for dedup or flushing (flusher looks up via API).
            Ok(Event::PrComment {
                owner: p.repository.owner.login,
                repo: p.repository.name,
                pr: issue.number,
                branch: String::new(),
            })
        }
        "pull_request" => {
            let p: PrPayload = serde_json::from_slice(body)?;
            Ok(Event::PrLifecycle {
                owner: p.repository.owner.login,
                repo: p.repository.name,
                pr: p.pull_request.number,
                branch: p.pull_request.head.r#ref,
                action: p.action,
            })
        }
        "workflow_run" => {
            let p: WorkflowPayload = serde_json::from_slice(body)?;
            let Some(conclusion) = p.workflow_run.conclusion else {
                return Ok(Event::Ignored);
            };
            if conclusion != "failure" {
                return Ok(Event::Ignored);
            }
            let branch = p
                .workflow_run
                .head_branch
                .ok_or(EventError::Missing("head_branch"))?;
            Ok(Event::CiFailed {
                owner: p.repository.owner.login,
                repo: p.repository.name,
                branch,
                url: p.workflow_run.html_url,
                name: p.workflow_run.name,
            })
        }
        _ => Ok(Event::Ignored),
    }
}
