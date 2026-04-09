# CI/CD

Last updated: 2026-04-09

This document is the authoritative contract for CI, release, and artifact delivery in `lab`. All pipeline implementations must conform to this spec.

## CI Checks

Every push and pull request must pass all of the following:

| Check | Command |
|-------|---------|
| Compile | `cargo check --workspace --all-features` |
| Format | `cargo fmt --all -- --check` |
| Lint | `cargo clippy --workspace --all-features -- -D warnings` |
| Deny | `cargo deny check` |
| Tests | `cargo nextest run --workspace --all-features` |
| Docs | `cargo doc --no-deps --all-features` (must be warning-free) |

Clippy runs with `-D warnings` — zero warnings are permitted. This is enforced at the workspace lint layer.

## CI Platform

- **Provider:** GitHub Actions
- **Job split:**
  - Fast checks (check, fmt, clippy, deny, test) on every push and PR
  - Release builds on `vX.Y.Z` tags only
  - Publishing after successful release builds

## Build Matrix

| Platform | Target |
|----------|--------|
| Linux x86_64 | `x86_64-unknown-linux-gnu` |
| Linux aarch64 | `aarch64-unknown-linux-gnu` |
| Windows x86_64 | `x86_64-pc-windows-msvc` |

All three platforms are required for release builds. Fast checks run on Linux x86_64 only.

## Integration Tests

Live service integration tests are **excluded from CI**. They require real service instances and are run locally only.

```bash
# Local only — never runs in CI
just test-integration
```

Integration tests must be marked `#[ignore]` so `cargo nextest run` skips them without explicit opt-in.

## Release Process

1. Bump version with `cargo-release` (single workspace version)
2. `cargo-release` tags the commit `vX.Y.Z` and pushes
3. The `vX.Y.Z` tag triggers the release CI job
4. Release job builds artifacts for all three platforms
5. `git-cliff` generates the changelog entry
6. Artifacts published to GitHub Releases

**Tag format:** `vX.Y.Z` — no other formats are accepted.

**Version policy:** single version across the entire workspace. `lab` and `lab-apis` always share the same version number.

## Artifact Distribution

- **Surface:** GitHub Releases
- **Artifacts per release:** one binary per platform (Linux x86_64, Linux aarch64, Windows x86_64)
- **No package registry publishing** (crates.io, npm, etc.) unless explicitly decided

## Size Policy

Binary size is tracked but not hard-gated in CI unless repo tooling enforces a monolith size limit. If a size gate is added, it runs in the fast check job.

## Self-Update

`lab self-update` is explicit-only:

- no startup polling
- no background update checks
- verify release integrity before replacement

## Non-Goals

- no telemetry pipeline
- no background analytics
- no phone-home behavior in any CI or release step
