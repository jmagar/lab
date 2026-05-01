# AdGuard Home Coverage

Status: first-class v1 read-only DNS/security surface.

Actions: `server.status`, `server.version`, `stats.summary`, `querylog.search`, `filtering.status`, `filtering.check-host`, plus built-in `help` and `schema`.

Deferred: filtering rule mutation, filter refresh, stats reset, rewrite/client/DHCP/DNS config mutation, and allow/deny list edits.

Security: `ADGUARD_SESSION_COOKIE` and `ADGUARD_PASSWORD` are secret. Query-log responses redact common client/upstream identity fields before crossing CLI, MCP, or API boundaries.
