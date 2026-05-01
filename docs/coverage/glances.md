# Glances Coverage

Status: first-class v1 read-only system telemetry surface.

Actions: `server.health`, `system.info`, `quicklook.summary`, `cpu.summary`, `memory.summary`, `load.summary`, `network.summary`, `diskio.summary`, `fs.summary`, `process.top`, plus built-in `help` and `schema`.

Deferred: plugin mutation, process kill, exporter config, alert config, and unbounded process inventory.

Security: `GLANCES_TOKEN` is secret. `process.top` requires an explicit bounded `limit` and strips process command lines and usernames.
