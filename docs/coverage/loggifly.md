# LoggiFly Coverage

Status: local heartbeat/config contract implemented.

Actions: `contract.status`, `health.status`, `config.summary`, plus built-in `help` and `schema`.

Implemented now: documented heartbeat-file health inspection, redacted config.yaml summary under `LOGGIFLY_CONFIG_ROOT`, CLI/MCP/API/TUI/catalog wiring, and SDK tests.

Deferred: full config validation, Docker socket access, live logs, Docker labels, remote hosts, OliveTin actions, container actions, notification tests, and notification sends.

Security: no Docker, raw log, raw config, or notification surface is exposed in v1. `config.summary` reports metadata and section presence only.
