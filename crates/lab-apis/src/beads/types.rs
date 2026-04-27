//! Beads request/response types.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Full issue with all body fields and labels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue id, e.g. `lab-5t4b`.
    pub id: String,
    /// Title.
    pub title: String,
    /// Markdown description body.
    pub description: String,
    /// Markdown design body.
    pub design: String,
    /// Markdown acceptance criteria body.
    pub acceptance_criteria: String,
    /// Status — `open`, `closed`, `in_progress`, etc.
    pub status: String,
    /// Priority — 0 (P0/highest) … 4 (P4/backlog).
    pub priority: i32,
    /// Type — `task`, `epic`, `bug`, `feature`, `chore`.
    pub issue_type: String,
    /// Owner.
    pub owner: Option<String>,
    /// Assignee.
    pub assignee: Option<String>,
    /// Created-by author.
    pub created_by: Option<String>,
    /// Creation timestamp.
    pub created_at: NaiveDateTime,
    /// Last-update timestamp.
    pub updated_at: NaiveDateTime,
    /// Closed-at timestamp, if closed.
    pub closed_at: Option<NaiveDateTime>,
    /// Linked spec id.
    pub spec_id: Option<String>,
    /// Labels joined from the labels table.
    pub labels: Vec<String>,
}

/// Lightweight summary used for list views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueSummary {
    /// Issue id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Status.
    pub status: String,
    /// Priority 0..=4.
    pub priority: i32,
    /// Type.
    pub issue_type: String,
    /// Owner.
    pub owner: Option<String>,
    /// Created-at timestamp.
    pub created_at: NaiveDateTime,
    /// Updated-at timestamp.
    pub updated_at: NaiveDateTime,
    /// Labels — empty in list responses (populated only on `get`).
    pub labels: Vec<String>,
}

/// One comment on an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment id.
    pub id: String,
    /// Parent issue id.
    pub issue_id: String,
    /// Comment author.
    pub author: String,
    /// Body text.
    pub text: String,
    /// Creation timestamp.
    pub created_at: NaiveDateTime,
}

/// List filters.
#[derive(Debug, Clone, Default)]
pub struct IssueListParams {
    /// Status filter (`open`, `closed`, `in_progress`).
    pub status: Option<String>,
    /// Type filter.
    pub issue_type: Option<String>,
    /// Owner filter.
    pub owner: Option<String>,
    /// Label filter (a single label).
    pub label: Option<String>,
    /// Result limit; defaults to 50 when `None`.
    pub limit: Option<i64>,
    /// Pagination offset; defaults to 0 when `None`.
    pub offset: Option<i64>,
}
