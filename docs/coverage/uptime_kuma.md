# Uptime Kuma Coverage

Status: first-class Socket.IO surface.

Actions: `contract.status`, `server.health`, `monitor.list`, `monitor.get`, `monitor.create`, `monitor.update`, `monitor.delete`, `monitor.pause`, `monitor.resume`, `heartbeat.list`, `status.summary`, `notification.list`, `notification.test`, `notification.add`, plus built-in `help` and `schema`.

Implemented now: service metadata, env contract, registry/CLI/MCP/API/TUI wiring, web-root health probe, authenticated Socket.IO login with per-ack timeout, read actions for monitor inventory, monitor detail, heartbeat history, status summary, and notification inventory, plus destructive Socket.IO actions for monitor create/update/delete/pause/resume, notification test sends, and notification channel creation.

Deferred: status-page mutation, maintenance windows, and a fuller supervised actor with explicit disconnect-drain/reconnect task management.

Security: `UPTIME_KUMA_PASSWORD` is secret. Side-effecting monitor and notification actions are marked destructive so CLI/MCP confirmation policy applies. `notification.add` rejects template markers and local/private webhook targets before emit. Heartbeat history is clamped to a maximum 168 hour window.
