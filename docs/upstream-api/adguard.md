# AdGuard Home Source Contract

Retrieved: 2026-05-01

Sources:
- https://github.com/AdguardTeam/AdGuardHome/tree/master/openapi
- https://github.com/AdguardTeam/AdGuardHome/wiki/Clients

AdGuard Home publishes an OpenAPI contract for the `/control/*` web API. Lab's v1 surface is read-only and avoids rule/config mutation until confirmation, dry-run, and rollback behavior are designed.

## Auth

Lab supports `ADGUARD_URL` plus one of:

- `ADGUARD_SESSION_COOKIE`
- `ADGUARD_USERNAME` and `ADGUARD_PASSWORD`

Username/password mode logs in through `/control/login` and uses the returned cookie for subsequent requests. Cookie and password values are secret.

## V1 Actions

| Action | Endpoint | Bounds | Hosted posture |
|---|---|---|---|
| `server.status` | `GET /control/status` | no body | safe |
| `server.version` | `GET /control/version.json` | no body | safe |
| `stats.summary` | `GET /control/stats` | no body | safe |
| `querylog.search` | `GET /control/querylog` | required `limit`, optional search/cursor | sensitive, redacted |
| `filtering.status` | `GET /control/filtering/status` | no body | safe |
| `filtering.check-host` | `GET /control/filtering/check_host` | one host | safe |

## Security

Query-log payloads can expose client IPs, upstream DNS servers, protocol details, and browsing history. Lab redacts common client/upstream identity fields and requires an explicit `limit`.
