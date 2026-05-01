# Pi-hole Source Contract

Retrieved: 2026-05-01

Sources:
- https://docs.pi-hole.net/api/
- https://docs.pi-hole.net/api/auth/

Pi-hole v6 uses a REST API under `/api` and removed the older v5 token-in-query flow. Lab's v1 surface uses session authentication and bounded reads.

## Auth

Lab uses:

```env
PIHOLE_URL=http://localhost
PIHOLE_PASSWORD=replace-me
PIHOLE_TOTP=123456
```

`PIHOLE_TOTP` is optional. `POST /api/auth` returns a session id and CSRF token. Lab sends them as `X-FTL-SID` and `X-FTL-CSRF`, treats the session as stale 30 seconds before upstream expiry, and retries one request after a 401 by refreshing the session.

## V1 Actions

| Action | Endpoint | Bounds | Hosted posture |
|---|---|---|---|
| `server.summary` | `GET /api/summaries` | no body | safe |
| `server.settings` | `GET /api/settings` | no body | sensitive config read |
| `blocking.status` | `GET /api/dns/blocking` | no body | safe |
| `blocking.set` | `POST /api/dns/blocking` | boolean plus optional timer | destructive |
| `querylog.search` | `GET /api/logs` | `offset`, `limit` 1-500 | sensitive, redacted |
| `adlist.list` | `GET /api/adlists` | no body | safe |
| `adlist.add` | `POST /api/adlists` | one address | destructive |
| `adlist.remove` | `DELETE /api/adlists/{id}` | one id | destructive |
| `domain.list` | `GET /api/domains` | no body | safe |
| `domain.add` | `POST /api/domains` | one domain and rule type | destructive |

## Security

Query logs can expose client identity and browsing history. Lab bounds `querylog.search`, redacts common client/upstream identity fields, and does not implement the removed v5 `?auth=` credential style.
