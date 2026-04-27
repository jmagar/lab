//! Beads MySQL client.
//!
//! Connects to a Dolt MySQL server, dispatched per-bead to the local lab
//! database. The schema is the bd tool's issues table — see the `types`
//! module for the columns we project.
//!
//! Connection pool, runtime queries, no compile-time macros (no
//! DATABASE_URL required). The pool is constructed once at startup and
//! shared via `Arc` from `AppState`.

use sqlx_core::pool::PoolOptions;
use sqlx_core::row::Row;
use sqlx_mysql::{MySql, MySqlPool};
use std::time::Duration;

type MySqlPoolOptions = PoolOptions<MySql>;

use super::error::BeadsError;
use super::types::{Issue, IssueListParams, IssueSummary};

/// Allowlisted ORDER BY columns for list queries.
///
/// User input is matched against this list before being interpolated into
/// the SQL string — never interpolate raw column names.
const ORDER_COLUMNS: &[&str] = &["created_at", "updated_at", "title", "priority", "status"];

/// Default LIMIT applied when callers do not specify one.
const DEFAULT_LIST_LIMIT: i64 = 50;
/// Hard upper bound on `limit`.
const MAX_LIST_LIMIT: i64 = 200;

/// MySQL connection pool wrapping the Dolt server.
#[derive(Clone)]
pub struct BeadsClient {
    pool: MySqlPool,
}

impl std::fmt::Debug for BeadsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BeadsClient").finish_non_exhaustive()
    }
}

impl BeadsClient {
    /// Construct a client from a fully-formed MySQL DSN.
    ///
    /// # Errors
    ///
    /// Returns `BeadsError::Database` when pool construction fails.
    pub async fn connect(dsn: &str) -> Result<Self, BeadsError> {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Some(Duration::from_secs(30)))
            .connect(dsn)
            .await?;
        Ok(Self { pool })
    }

    /// Construct a client wrapping an existing pool — useful in tests.
    #[must_use]
    pub fn from_pool(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Construct a client with a lazily-connected pool.
    ///
    /// The pool does not open a TCP connection until the first query.
    /// Suitable for startup paths that must not block on a Dolt round-trip.
    ///
    /// # Errors
    ///
    /// Returns `BeadsError::Database` when the DSN is malformed.
    pub fn lazy(dsn: &str) -> Result<Self, BeadsError> {
        let pool = MySqlPoolOptions::new()
            .max_connections(5)
            .min_connections(0)
            .acquire_timeout(Duration::from_secs(5))
            .idle_timeout(Some(Duration::from_secs(30)))
            .connect_lazy(dsn)?;
        Ok(Self { pool })
    }

    /// Borrow the underlying pool.
    #[must_use]
    pub fn pool(&self) -> &MySqlPool {
        &self.pool
    }

    /// List issues with optional filters and pagination.
    ///
    /// Labels are NOT joined — every returned `IssueSummary` has
    /// `labels = vec![]`. Use `get_issue` for full label data.
    ///
    /// # Errors
    ///
    /// Returns `BeadsError::Database` on connection or query failure.
    pub async fn list_issues(
        &self,
        params: &IssueListParams,
    ) -> Result<Vec<IssueSummary>, BeadsError> {
        let limit = params
            .limit
            .unwrap_or(DEFAULT_LIST_LIMIT)
            .clamp(1, MAX_LIST_LIMIT);
        let offset = params.offset.unwrap_or(0).max(0);

        // Default ordering — priority ASC then updated_at DESC. Always validated
        // against the static allowlist; we never interpolate user-supplied
        // column names.
        debug_assert!(ORDER_COLUMNS.contains(&"priority"));
        debug_assert!(ORDER_COLUMNS.contains(&"updated_at"));

        // Construct the WHERE clause dynamically. We use sqlx_core::query::query and
        // .bind() for every parameter — no string interpolation of user input.
        let mut sql = String::from(
            "SELECT i.id, i.title, i.status, i.priority, i.issue_type, i.owner, \
             i.created_at, i.updated_at \
             FROM issues i WHERE 1=1",
        );

        if params.status.is_some() {
            sql.push_str(" AND i.status = ?");
        }
        if params.issue_type.is_some() {
            sql.push_str(" AND i.issue_type = ?");
        }
        if params.owner.is_some() {
            sql.push_str(" AND i.owner = ?");
        }
        if params.label.is_some() {
            // Use IN subquery to avoid duplicating rows on labels join.
            sql.push_str(" AND i.id IN (SELECT issue_id FROM labels WHERE label = ?)");
        }
        sql.push_str(" ORDER BY i.priority ASC, i.updated_at DESC LIMIT ? OFFSET ?");

        let mut q = sqlx_core::query::query(&sql);
        if let Some(s) = &params.status {
            q = q.bind(s);
        }
        if let Some(t) = &params.issue_type {
            q = q.bind(t);
        }
        if let Some(o) = &params.owner {
            q = q.bind(o);
        }
        if let Some(l) = &params.label {
            q = q.bind(l);
        }
        q = q.bind(limit).bind(offset);

        let rows = q.fetch_all(&self.pool).await?;
        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            out.push(IssueSummary {
                id: row.try_get("id")?,
                title: row.try_get("title")?,
                status: row.try_get("status")?,
                priority: row.try_get("priority")?,
                issue_type: row.try_get("issue_type")?,
                owner: row.try_get::<Option<String>, _>("owner")?,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                labels: Vec::new(),
            });
        }
        Ok(out)
    }

    /// Fetch one issue by id, joining its labels via `GROUP_CONCAT`.
    ///
    /// # Errors
    ///
    /// Returns `BeadsError::NotFound` when the id does not match a row,
    /// `BeadsError::Database` on connection or query failure.
    pub async fn get_issue(&self, id: &str) -> Result<Issue, BeadsError> {
        // Pin both the SET SESSION and the SELECT to the same pool connection.
        // sqlx pools may otherwise hand out a different connection for the
        // SELECT, in which case the per-session group_concat_max_len bump has
        // no effect and labels silently truncate at 1024 bytes.
        let mut conn = self.pool.acquire().await?;

        // group_concat truncates by default at 1024 bytes — extend the per-
        // session limit so issues with many labels do not silently truncate.
        sqlx_core::query::query("SET SESSION group_concat_max_len = 65536")
            .execute(&mut *conn)
            .await?;

        let row = sqlx_core::query::query(
            "SELECT i.id, i.title, i.description, i.design, i.acceptance_criteria, \
             i.status, i.priority, i.issue_type, i.owner, i.assignee, i.created_by, \
             i.created_at, i.updated_at, i.closed_at, i.spec_id, \
             COALESCE(GROUP_CONCAT(l.label ORDER BY l.label SEPARATOR '\u{1f}'), '') AS labels \
             FROM issues i LEFT JOIN labels l ON l.issue_id = i.id \
             WHERE i.id = ? GROUP BY i.id LIMIT 1",
        )
        .bind(id)
        .fetch_optional(&mut *conn)
        .await?;

        let row = row.ok_or_else(|| BeadsError::NotFound { id: id.to_string() })?;
        let labels_raw: String = row.try_get("labels")?;
        let labels = if labels_raw.is_empty() {
            Vec::new()
        } else {
            labels_raw.split('\u{1f}').map(str::to_string).collect()
        };
        Ok(Issue {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            design: row.try_get("design")?,
            acceptance_criteria: row.try_get("acceptance_criteria")?,
            status: row.try_get("status")?,
            priority: row.try_get("priority")?,
            issue_type: row.try_get("issue_type")?,
            owner: row.try_get::<Option<String>, _>("owner")?,
            assignee: row.try_get::<Option<String>, _>("assignee")?,
            created_by: row.try_get::<Option<String>, _>("created_by")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            closed_at: row.try_get::<Option<_>, _>("closed_at")?,
            spec_id: row.try_get::<Option<String>, _>("spec_id")?,
            labels,
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    /// DSN used by integration tests. Override via `BEADS_TEST_DSN` if your
    /// local Dolt server is on a different port.
    fn test_dsn() -> String {
        std::env::var("BEADS_TEST_DSN")
            .unwrap_or_else(|_| "mysql://root@127.0.0.1:3306/lab".to_string())
    }

    /// Basic smoke test for `list_issues()`: connecting to a live Dolt server
    /// and listing issues with no filters should succeed and return a vec
    /// (possibly empty).
    ///
    /// Run with: `cargo test -p lab-apis -- --ignored beads`
    #[tokio::test]
    #[ignore = "requires a running Dolt server (bd start)"]
    async fn list_issues_basic() {
        let client = BeadsClient::connect(&test_dsn())
            .await
            .expect("failed to connect to Dolt; start bd server and re-run");
        let params = IssueListParams::default();
        let issues = client
            .list_issues(&params)
            .await
            .expect("list_issues failed");
        // Returned slice may be empty on a fresh DB — just verify the call succeeds.
        let _ = issues;
    }

    /// Verify that `get_issue()` returns `BeadsError::NotFound` for a
    /// syntactically valid but non-existent id.
    ///
    /// Run with: `cargo test -p lab-apis -- --ignored beads`
    #[tokio::test]
    #[ignore = "requires a running Dolt server (bd start)"]
    async fn get_issue_not_found() {
        let client = BeadsClient::connect(&test_dsn())
            .await
            .expect("failed to connect to Dolt; start bd server and re-run");
        let result = client.get_issue("nonexistent-0000").await;
        assert!(
            matches!(result, Err(BeadsError::NotFound { .. })),
            "expected NotFound, got: {result:?}"
        );
    }
}
