# Environment Variables

This document lists the `lab` environment variables that matter for transport
and auth setup, plus deployment-ready examples for selected homelab services.
It is not the complete service env inventory. The complete per-service source
of truth is each service `PluginMeta`, surfaced through
[SERVICES.md](./SERVICES.md), [coverage docs](./coverage/README.md), and
generated help.

## HTTP Auth

Bearer mode:

```env
LAB_AUTH_MODE=bearer
LAB_MCP_HTTP_TOKEN=replace-me
```

OAuth mode:

```env
LAB_AUTH_MODE=oauth
LAB_PUBLIC_URL=https://lab.example.com
LAB_GOOGLE_CLIENT_ID=google-client-id
LAB_GOOGLE_CLIENT_SECRET=google-client-secret
LAB_AUTH_ADMIN_EMAIL=admin@example.com
```

Optional auth overrides:

```env
LAB_AUTH_SQLITE_PATH=/var/lib/lab/auth.db
LAB_AUTH_KEY_PATH=/var/lib/lab/auth-jwt.pem
LAB_AUTH_ALLOWED_REDIRECT_URIS=https://callback.tootie.tv/callback/*
LAB_GOOGLE_CALLBACK_PATH=/auth/google/callback
LAB_GOOGLE_SCOPES=openid,email,profile
LAB_AUTH_ACCESS_TOKEN_TTL_SECS=3600
LAB_AUTH_REFRESH_TOKEN_TTL_SECS=2592000
LAB_AUTH_CODE_TTL_SECS=300
```

These non-secret overrides can also live in `config.toml` under `[auth]`.

Rules:

- `LAB_AUTH_MODE` defaults to `bearer`
- bearer mode keeps using `LAB_MCP_HTTP_TOKEN`
- oauth mode requires `LAB_PUBLIC_URL`, `LAB_GOOGLE_CLIENT_ID`, `LAB_GOOGLE_CLIENT_SECRET`, and `LAB_AUTH_ADMIN_EMAIL`
- `LAB_AUTH_ADMIN_EMAIL` is the bootstrap admin Google email; startup fails closed if unset under oauth mode so no Google account can authenticate without explicit permission. Future SQLite-backed allowlist (web-UI managed) will grant access to additional users.
- the old external issuer variables (`LAB_OAUTH_ISSUER`, `LAB_OAUTH_AUDIENCE`, `LAB_OAUTH_CLIENT_ID`) are no longer used
- `LAB_PUBLIC_URL` also feeds RFC 9728 metadata, JWT issuer/audience, and HTTP allowed-host derivation

## Selected Homelab Service Examples

Immich:

```env
IMMICH_URL=http://localhost:2283
IMMICH_API_KEY=replace-me
```

Navidrome:

```env
NAVIDROME_URL=http://localhost:4533
NAVIDROME_USERNAME=admin
NAVIDROME_TOKEN=precomputed-subsonic-token
NAVIDROME_SALT=randomsalt
```

FreshRSS:

```env
FRESHRSS_URL=https://rss.example.com/api/greader.php
FRESHRSS_USERNAME=admin
FRESHRSS_API_PASSWORD=replace-me
```

Scrutiny:

```env
SCRUTINY_URL=http://localhost:8080
# Optional if protected by a reverse proxy or future upstream auth:
SCRUTINY_TOKEN=replace-me
```

AdGuard Home:

```env
ADGUARD_URL=http://localhost:3000
# Either provide an existing session cookie:
ADGUARD_SESSION_COOKIE=agh_session=replace-me
# Or let Lab log in with web credentials:
ADGUARD_USERNAME=admin
ADGUARD_PASSWORD=replace-me
```

Pi-hole:

```env
PIHOLE_URL=http://localhost
PIHOLE_PASSWORD=replace-me
# Optional when Pi-hole 2FA is enabled:
PIHOLE_TOTP=123456
```

Glances:

```env
GLANCES_URL=http://localhost:61208
# Optional for protected deployments:
GLANCES_TOKEN=replace-me
```

Neo4j:

```env
NEO4J_URL=neo4j://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=replace-me
# Optional:
NEO4J_DB=neo4j
NEO4J_POOL_SIZE=16
NEO4J_CA_CERT_PATH=/etc/ssl/neo4j-ca.pem
```

Uptime Kuma:

```env
UPTIME_KUMA_URL=http://localhost:3001
UPTIME_KUMA_USERNAME=admin
UPTIME_KUMA_PASSWORD=replace-me
```

Beads:

```env
# Optional; defaults to bd on PATH.
BEADS_BIN=bd
```

Jellyfin:

```env
JELLYFIN_URL=http://localhost:8096
JELLYFIN_API_KEY=replace-me
```

OpenACP:

```env
OPENACP_URL=http://127.0.0.1:21420
OPENACP_TOKEN=replace-me
```

Lab does not auto-read OpenACP `<instance-root>/api-secret` files. Use an
explicit bearer token or scoped JWT. The upstream `openacp` service is separate
from Lab's internal `acp` service.

Named Jellyfin instances use the same suffix pattern as other multi-instance services:

```env
JELLYFIN_NODE2_URL=http://node2.local:8096
JELLYFIN_NODE2_API_KEY=replace-me
```

Named OpenACP instances use `TOKEN` rather than `API_KEY`:

```env
OPENACP_NODE2_URL=http://node2.local:21420
OPENACP_NODE2_TOKEN=replace-me
```

LoggiFly exposes a local heartbeat/config contract. `LOGGIFLY_CONFIG_ROOT`
is optional and enables redacted `config.summary`; `LOGGIFLY_HEARTBEAT_PATH`
and `LOGGIFLY_HEARTBEAT_INTERVAL_SECS` tune `health.status` for deployments
using LoggiFly's documented `ENABLE_HEALTHCHECK=true` heartbeat file.

```env
LOGGIFLY_DOCS_URL=https://clemcer.github.io/LoggiFly/
LOGGIFLY_CONFIG_ROOT=/etc/loggifly
LOGGIFLY_HEARTBEAT_PATH=/dev/shm/loggifly-heartbeat
LOGGIFLY_HEARTBEAT_INTERVAL_SECS=60
```
