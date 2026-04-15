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
