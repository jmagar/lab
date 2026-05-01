# Uptime Kuma Coverage

Status: first-class contract surface with live Socket.IO reads deferred.

Actions: `contract.status`, `server.health`, `monitor.list`, `monitor.get`, `heartbeat.list`, `status.summary`, `notification.list`, plus built-in `help` and `schema`.

Implemented now: service metadata, env contract, registry/CLI/MCP/API/TUI wiring, web-root health probe, and a structured `contract.status` response that makes the Socket.IO gap explicit.

Deferred: authenticated Socket.IO actor, monitor/heartbeat/notification reads, monitor mutation, status-page mutation, maintenance windows, and notification tests.

Security: `UPTIME_KUMA_PASSWORD` is secret. Uptime Kuma does not publish a stable REST API for monitor data, so live monitor reads must go through a bounded Socket.IO actor before they can safely cross Lab surfaces.
