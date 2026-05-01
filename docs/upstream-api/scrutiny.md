# Scrutiny Source Contract

Retrieved: 2026-05-01

Sources:
- https://github.com/AnalogJ/scrutiny
- https://github.com/AnalogJ/scrutiny/releases/tag/v0.9.2
- https://github.com/AnalogJ/scrutiny/blob/master/docs/API_NOTIFY_HEALTHCHECKS.md

Scrutiny public material is stronger on deployment than stable API reference. Lab v1 is source-contract-first and exposes only health/dashboard/device summary probes with redaction.

## Auth

Lab uses `SCRUTINY_URL` and optional `SCRUTINY_TOKEN` or `SCRUTINY_API_KEY` as bearer auth for deployments protected by a reverse proxy or future upstream auth.

## V1 Actions

| Action | Endpoint | Bounds | Hosted posture |
|---|---|---|---|
| `server.health` | `GET /api/health` | no body | safe |
| `dashboard.summary` | `GET /api/summary` | redacted summary only | disk health sensitive |
| `device.list` | `GET /api/devices` | redacted list only | disk identity sensitive |

## Security

Serial numbers, WWNs, `/dev/*` paths, hostnames, raw SMART attributes, collector output, and notification payloads are sensitive. Notify/test, collector, rescan, repair, prune, settings, and raw SMART endpoints are deferred.

