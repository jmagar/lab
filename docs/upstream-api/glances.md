# Glances Source Contract

Retrieved: 2026-05-01

Sources:
- https://glances.readthedocs.io/en/latest/api.html
- https://github.com/nicolargo/glances

Glances exposes a RESTful API, with v4 endpoints under `/api/4/*`. Lab's v1 surface is read-only and limits the plugin set to stable host telemetry summaries.

## Auth

Lab uses `GLANCES_URL`. `GLANCES_TOKEN` is optional and is sent as a bearer token for protected deployments.

## V1 Actions

| Action | Endpoint | Bounds | Hosted posture |
|---|---|---|---|
| `server.health` | `GET /api/4/status` | no body | safe |
| `system.info` | `GET /api/4/system` | no body | safe |
| `quicklook.summary` | `GET /api/4/quicklook` | no body | safe |
| `cpu.summary` | `GET /api/4/cpu` | no body | safe |
| `memory.summary` | `GET /api/4/mem` | no body | safe |
| `load.summary` | `GET /api/4/load` | no body | safe |
| `network.summary` | `GET /api/4/network` | no body | potentially sensitive |
| `diskio.summary` | `GET /api/4/diskio` | no body | safe |
| `fs.summary` | `GET /api/4/fs` | no body | path-sensitive |
| `process.top` | `GET /api/4/processlist/top/{limit}` | `limit` 1-50 | sensitive, redacted |

## Security

Process listings can expose command lines and local usernames. Lab strips those fields from `process.top` responses and rejects unbounded limits.
