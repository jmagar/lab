# Beads Coverage

Status: local read-only CLI contract implemented.

Actions: `contract.status`, `health.status`, `version.get`, `context.get`, `status.summary`, `issue.list`, `issue.ready`, `issue.show`, `graph.show`, plus built-in `help` and `schema`.

Implemented now: first-class Lab service wiring for SDK, dispatch, CLI, MCP catalog, HTTP API, TUI metadata, and onboarding audit. The implementation shells out to `bd --json --readonly` and caps list/ready result limits at 500.

Deferred: issue/comment/dependency writes, close/reopen/update/create, raw SQL, Dolt push/pull/commit, branch operations, import/export, and any direct Dolt database access.

Security: v1 is read-only and uses `--readonly` on every `bd` invocation. It returns `bd` JSON output without exposing write operations through Lab.
