# Scrutiny Coverage

Status: cautious v1 read-only health/device summary surface.

Actions: `server.health`, `dashboard.summary`, `device.list`, plus built-in `help` and `schema`.

Deferred: notification tests/sends, collector triggers, rescan/repair/prune, settings, raw SMART dumps, raw metrics and raw device detail.

Security: optional tokens are secret. Serial numbers, WWNs, device paths, hostnames, and raw SMART fields are removed recursively from v1 payloads.

