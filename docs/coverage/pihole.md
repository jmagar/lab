# Pi-hole Coverage

Status: first-class v1 Pi-hole v6 API surface.

Actions: `server.summary`, `server.settings`, `blocking.status`, `blocking.set`, `querylog.search`, `adlist.list`, `adlist.add`, `adlist.remove`, `domain.list`, `domain.add`, plus built-in `help` and `schema`.

Deferred: group management, client management, local DNS host records, config writes beyond the listed actions, teleporter/export, and long-window historical drains.

Security: `PIHOLE_PASSWORD` and `PIHOLE_TOTP` are secret. The client uses v6 session auth through `POST /api/auth`, sends `X-FTL-SID` and `X-FTL-CSRF`, retries once after 401, and rejects legacy `?auth=` token URLs.
