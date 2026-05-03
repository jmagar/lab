---
name: security-baseline
description: Authoritative security checklist sourced from spec.md § Security baseline (L4302-4508). Covers CSP, OAuth callback, PKCE, JWKS, /api/v1/log CSRF, /mcp auth parity, stdio MCP, redact_url, and backtrace policy. Preloaded by security-reviewer.
user-invocable: false
---

Source of truth: `spec.md § Plan Integrations → Security baseline`. When in doubt, that section wins.

Run all nine checks against the changed files. Anything marked HIGH in the spec is a release blocker — flag it accordingly.

---

## 1. CSP + security headers (HIGH)

The static-export SPA route group must emit (via `tower_http::set_header::SetResponseHeaderLayer`):

- `Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self'; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'`
- `X-Frame-Options: DENY`
- `X-Content-Type-Options: nosniff`
- `Referrer-Policy: strict-origin-when-cross-origin`
- `Permissions-Policy: camera=(), microphone=(), geolocation=()`
- `Strict-Transport-Security: max-age=63072000; includeSubDomains` — **only** when `scheme=https` or `X-Forwarded-Proto: https`

`/api/v1/*` and `/mcp` get only the smaller baseline: `X-Content-Type-Options: nosniff` + `X-Frame-Options: DENY`. CSP on JSON responses is meaningless.

**Violations to flag:**
- CSP applied to `/api/v1/*` or `/mcp` (wrong route group)
- `'unsafe-eval'` in `script-src` (banned)
- `connect-src` widened beyond `'self'` without explicit documentation
- HSTS unconditionally set (must be HTTPS-gated)

## 2. OAuth callback flow without URL token (HIGH)

The callback flow must NOT pass `?token=<jwt>` in any URL.

Locked steps:
1. `GET /api/v1/auth/callback?code=...&state=...`
2. axum exchanges code → token; validates `state` against `auth-pkce` cookie; sets `auth-token` httpOnly+SameSite=Strict+Secure
3. axum responds 302 to `/auth/callback` (NO query string)
4. Web `/auth/callback` page reads NOTHING from URL; calls `GET /api/v1/auth/me` (cookie auto-sent), then redirects to `return_to` from the cleared `auth-pkce` cookie
5. Agents needing raw token: explicit `GET /api/v1/auth/me` returning `{ user, token }`
6. Web `/auth/callback` mounts → `history.replaceState({}, '', '/auth/callback')` (defense-in-depth scrub)

**Violations to flag:**
- Any redirect URL containing `token=`, `jwt=`, or similar
- `localStorage.setItem('token', ...)` anywhere
- Web page reading `searchParams.get('token')` or `?` parsing on `/auth/callback`

## 3. PKCE + state storage: signed cookie (HIGH)

`code_verifier` + `state` stored in a signed cookie:

| Attribute | Value |
|---|---|
| Name | `auth-pkce` |
| Path | `/api/v1/auth` |
| HttpOnly | yes |
| SameSite | **Lax** (must round-trip via OAuth provider) |
| Secure | yes |
| Max-Age | 300 (5 minutes) |
| Encoding | `base64(JSON) + HMAC-SHA-256` |
| HMAC key | `APP_SESSION_SIGNING_KEY` (env, rotated independently) |

Payload: `{ verifier, state, return_to, expires_at }`.

**Violations to flag:**
- In-memory `HashMap<state, verifier>` (DoS surface, banned)
- `SameSite=Strict` on `auth-pkce` (would block the redirect roundtrip)
- Path scope wider than `/api/v1/auth`
- HMAC key reused with bearer/JWT signing keys

## 4. JWKS: single-flight + stale-while-revalidate (HIGH)

`moka::future::Cache` with `try_get_with` for single-flight:

- TTL: `max(provider Cache-Control max-age, 300s)` — 5-minute floor; provider `no-cache`/`max-age=0` does NOT force per-request refresh
- Expiry: serve stale + spawn one background refresh; on refresh failure, prior value valid +300s before hard-fail
- `InvalidKid` error: invalidate entry, fetch fresh once, retry validation. **Single retry only** — no loops.

**Violations to flag:**
- Cache with no single-flight (multiple concurrent fetches on miss)
- Per-request JWKS fetch with no cache
- Unbounded retry on `InvalidKid`
- TTL pulled directly from provider header without 300s floor

## 5. `/api/v1/log` triple CSRF defense

The browser-error log endpoint requires ALL THREE checks before processing the body. No body inspection until all three pass.

1. `Origin` header present and matches `config.api.cors_origins`
2. `Sec-Fetch-Site: same-origin`
3. `Content-Type: application/json`

Reject with 403 + `kind=auth_failed` envelope on any failure.

Rate-limit session ID resolution order:
1. Valid `auth-token` cookie → JWT `jti` (or `sub`+UTC-day if `jti` absent)
2. Valid `Authorization: Bearer` → labeled bearer name
3. Else → SHA-256 of `(peer_ip, User-Agent)` truncated to 16 bytes

Default 100/min per session via `config.toml [log] web_rate_limit_per_min`.

**Violations to flag:**
- Missing any of the three checks
- Body inspection before all three pass
- No rate limiting on the endpoint
- Unauthenticated requests bucketed by IP only (no UA discriminator)

## 6. `/mcp` accepts all auth methods (parity with `/api/v1/*`)

`/mcp` over Streamable HTTP MUST accept all three auth methods (per ADR `2026-04-28-mcp-auth-parity.md`):
1. Static `Authorization: Bearer <token>` (constant-time compare)
2. OAuth Bearer JWT (validated against JWKS)
3. Session cookie (`auth-token`)

**Violations to flag:**
- Path-specific override that skips OAuth or session methods on `/mcp`
- Bearer-only assertion in test or middleware
- `/mcp` with completely separate auth implementation (must reuse the shared layer)

stdio transport is governed by §7 — separate model.

## 7. stdio MCP `REQUIRES_AUTH` carve-out

stdio MCP relies on the OS process trust boundary.

```rust
pub const REQUIRES_AUTH: bool = true; // default per domain
```

Dispatcher behavior matrix:

| `REQUIRES_AUTH` | `STDIO_TRUST_LOCAL_USER=1` | Behavior |
|---|---|---|
| `true` | unset | 401 — `auth_failed` kind, `auth.method=stdio` |
| `true` | set | granted; `auth.method=stdio`, `auth.subject=<local user>` |
| `false` | either | granted unconditionally |

`STDIO_TRUST_LOCAL_USER` is in the env-var allowlist, **Operator** category.

**Violations to flag:**
- New domain without `REQUIRES_AUTH` const declared (default missing)
- Environment-based gate other than `STDIO_TRUST_LOCAL_USER`
- stdio dispatch granting auth without the const + env check

## 8. `redact_url()` everywhere base URLs are logged

`core::http::redact::redact_url(url: &Url) -> String` MUST be used by:
- `client.ready` log event (`client.base_url` field)
- Per-request HTTP logger
- `doctor` JSON output

Behavior:
- Replace `userinfo` (`user:pass@`) with `redacted@`
- Replace query values where key matches `(?i)(token|key|password|secret|authorization)` with `[REDACTED]`

**Violations to flag:**
- `tracing::*!(client.base_url = %url)` without `redact_url()`
- Any log event emitting a raw `Url` containing credentials
- `Auth::Bearer(Secret::new(...))` or `Auth::Session(...)` appearing as cleartext in any log span

## 9. Backtraces never reach response body

`HttpError`/`ToolError` capture backtraces via `thiserror`'s `#[backtrace]`, but rendering is restricted:

- `panic` log event always renders backtrace
- Error log events render backtrace only when build profile is `debug` OR `APP_ERROR_BACKTRACE=1`
- `IntoResponse for ToolError` produces only the sanitized `ErrorEnvelope` — backtrace logged separately by `CatchPanicLayer`, never in the wire response

**Violations to flag:**
- Backtrace string included in `ErrorEnvelope`
- Response body containing source paths (`/path/to/source.rs:123`) or dependency-version strings
- `CatchPanicLayer` propagating panic strings to response body

---

## Output format

Group violations by section number. Use `file:line` refs. Mark HIGH-priority violations explicitly:

```
=== §1: CSP + security headers (HIGH) ===
HIGH  apps/web/.../middleware.rs:42  CSP applied to /api/v1/* (wrong route group)

=== §2: OAuth callback (HIGH) ===
OK

=== §3: PKCE cookie (HIGH) ===
HIGH  crates/app/src/api/auth/oauth.rs:88  in-memory HashMap<state,verifier> — DoS surface, must be signed cookie

...
```

End with: `{n} HIGH violations, {n} other violations, {n} sections OK`. Any HIGH count > 0 means **FAIL**.
