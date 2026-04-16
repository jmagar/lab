# Operations

This document covers operator-facing workflows, verification surfaces, CI, and release behavior.

## Repo-Level Helpers

The repo includes helper tooling outside the shipped binary.

### `bin/health-check`

Purpose:

- smoke-test configured services from the repo env file
- validate reachability quickly
- provide operator-friendly shell output

It is distinct from the product-level `lab health` surface.

It is intended as a repo-local smoke test, not as the canonical SDK-level health API.

### `just mcp-token`

Purpose:

- generate or rotate `LAB_MCP_HTTP_TOKEN`
- update the env file safely

## OAuth Auth State

When `LAB_AUTH_MODE=oauth`, `lab` persists local auth state on disk:

- SQLite database: `~/.lab/auth.db` by default
- JWT signing key: `~/.lab/auth-jwt.pem` by default

Rules:

- both files must use restrictive permissions; on Unix, `lab` requires they are not group- or world-readable
- new files are created with `0600` permissions on Unix
- the SQLite store is opened in WAL mode with a non-zero busy timeout
- Google tokens stay server-side only; clients always receive `lab` access tokens and receive `lab` refresh tokens only when Google granted an upstream refresh token

Recovery guidance:

- deleting `auth-jwt.pem` invalidates every previously issued `lab` access token and refresh token exchange path tied to those access tokens
- deleting `auth.db` removes registered clients, pending authorization requests, authorization codes, and refresh tokens
- if you back up either file, back up both together to preserve a coherent auth state snapshot

## Browser-Local OAuth Callback Forwarding

Some MCP clients can pin the OAuth callback port but still redirect the browser to
`http://127.0.0.1:<port>/...`. When the real callback listener lives on another machine, run
`lab oauth relay-local` on the browser machine to accept that loopback redirect and forward it to
the actual listener.

Named-machine workflow:

```bash
lab oauth relay-local --machine dookie --port 38935
```

Ad hoc workflow:

```bash
lab oauth relay-local \
  --forward-base http://100.88.16.79:38935/callback/dookie \
  --port 38935
```

Operational rules:

- the remote callback listener must already be running
- the helper is transport-only; it does not exchange codes or mint tokens
- the listener is loopback-only and normally run on demand for the active login flow
- startup output shows the resolved forwarding target before the first callback arrives
- failures map to HTTP responses on the local callback port: unreachable target -> `502`, timeout -> `504`

Recommended setup checklist:

1. Configure the browser-side machine target in `~/.lab/config.toml`:

```toml
[oauth.machines.dookie]
target_url = "http://100.88.16.79:38935/callback/dookie"
description = "dookie Codex callback listener"
default_port = 38935
```

2. Start the real OAuth client listener on the remote machine.
3. Start `lab oauth relay-local` on the browser machine.
4. Complete the OAuth login flow in the browser before either listener exits.

If you need public redirect URIs for a relay or browser-facing callback domain, remember to
allowlist them in `lab-auth` with `LAB_AUTH_ALLOWED_REDIRECT_URIS` or
`[auth].allowed_client_redirect_uris`.

## Product-Level Health Tooling

### `lab doctor`

`lab doctor` is the main read-only validation command.

It should audit:

- required env vars
- URL validity
- connectivity
- auth
- version visibility

It should support:

- all services
- single-service runs
- JSON output
- quick mode

Typical checks include:

- required env presence
- optional env visibility
- DNS/URL validity
- TCP reachability
- health endpoint success
- auth acceptance
- version reporting

### `lab health`

`lab health` should expose normalized health status using shared service contracts.

## Device Runtime Operations

In the current Linux `x86_64` v1 target, every supported fleet member runs `lab serve` as a device runtime.

Operationally:

- one device is the `master`
- non-master devices report to the master over `/v1/device/*`
- fleet inventory and fleet logs are queried from the master

Useful commands:

```bash
lab device list
lab device get dookie
lab logs search dookie oauth
```

Useful HTTP checks:

```bash
curl http://<device>:8765/health
curl -H "Authorization: Bearer $LAB_MCP_HTTP_TOKEN" http://<master>:8765/v1/device/devices
```

Current operational limits:

- fleet state is in-memory on the master
- non-master background uploads reuse the shared static bearer token when bearer auth is enabled
- non-master devices intentionally do not expose Web UI, gateway management, or MCP

## Install and Patch Workflows

Install and uninstall operations should:

- validate env requirements
- prompt for missing values when appropriate
- patch `.mcp.json` atomically
- back up before write
- support dry-run behavior

## CI

CI should verify:

- workspace builds
- formatting
- linting
- deny checks
- CI-safe tests
- docs when rustdoc verification is enabled

Expected job split:

- fast correctness and style checks on pushes and PRs
- release builds on tags
- publishing after successful release builds

Live service integration tests are intentionally excluded from normal CI.

## Release Process

Locked release expectations:

- single workspace version
- tagged releases
- release artifacts per supported platform
- GitHub Releases as the artifact distribution surface
- `cargo-release` for version bumps and tagging
- `git-cliff` for changelog generation

Tag format should stay `vX.Y.Z`.

## Privacy Rule

Operator workflows must respect the project-wide privacy rule:

- no telemetry
- no analytics
- no phone-home traffic except explicit service calls or explicit update operations
