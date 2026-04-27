//! Beads request/response types.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize, Serializer};

/// Serialize a `NaiveDateTime` as RFC 3339 with an explicit `Z` suffix.
///
/// Dolt/MySQL stores `created_at`/`updated_at` as naive timestamps but the
/// values are conceptually UTC. Without the `Z` suffix, JS `new Date(...)`
/// interprets the string in the local timezone, which silently shifts the
/// rendered values for any non-UTC client.
fn serialize_as_utc<S: Serializer>(dt: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_str(&format!("{}Z", dt.format("%Y-%m-%dT%H:%M:%S")))
}

fn serialize_as_utc_opt<S: Serializer>(
    dt: &Option<NaiveDateTime>,
    s: S,
) -> Result<S::Ok, S::Error> {
    match dt {
        Some(d) => s.serialize_str(&format!("{}Z", d.format("%Y-%m-%dT%H:%M:%S"))),
        None => s.serialize_none(),
    }
}

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
    #[serde(serialize_with = "serialize_as_utc")]
    pub created_at: NaiveDateTime,
    /// Last-update timestamp.
    #[serde(serialize_with = "serialize_as_utc")]
    pub updated_at: NaiveDateTime,
    /// Closed-at timestamp, if closed.
    #[serde(serialize_with = "serialize_as_utc_opt")]
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
    #[serde(serialize_with = "serialize_as_utc")]
    pub created_at: NaiveDateTime,
    /// Updated-at timestamp.
    #[serde(serialize_with = "serialize_as_utc")]
    pub updated_at: NaiveDateTime,
    /// Labels — **always empty** in list responses. The list query does not
    /// join the labels table; call `get_issue` for full label data.
    pub labels: Vec<String>,
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
