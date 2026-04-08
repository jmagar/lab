# bin/ — Operator scripts

Shell helpers for CI/CD and local operations. All scripts in this directory are on `PATH` when running via Claude Code.

## Scripts

| Script | Purpose |
|--------|---------|
| `health-check` | Smoke-test all enabled services; used in CI. Exits non-zero if any service is unreachable. |

## Rules

- Scripts here are **not** part of the `lab` binary. They're shell helpers for operators.
- Keep them POSIX-compatible (`#!/usr/bin/env bash` with `set -euo pipefail`).
- No service credentials in scripts — read from `~/.lab/.env` via `source` or `dotenvy`.
