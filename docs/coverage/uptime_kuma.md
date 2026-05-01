# Uptime Kuma Coverage

Status: first-class read-oriented Socket.IO surface.

Actions: `contract.status`, `server.health`, `monitor.list`, `monitor.get`, `heartbeat.list`, `status.summary`, `notification.list`, plus built-in `help` and `schema`.

Implemented now: service metadata, env contract, registry/CLI/MCP/API/TUI wiring, web-root health probe, authenticated Socket.IO login with per-ack timeout, and read actions for monitor inventory, monitor detail, heartbeat history, status summary, and notification inventory.

Deferred: monitor mutation, status-page mutation, maintenance windows, notification tests, notification creation, and a fuller supervised actor with explicit disconnect-drain/reconnect task management.

Security: `UPTIME_KUMA_PASSWORD` is secret. The integration does not expose side-effecting notification or monitor mutations. Heartbeat history is clamped to a maximum 168 hour window.
