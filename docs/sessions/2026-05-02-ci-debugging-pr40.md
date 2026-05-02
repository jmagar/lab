# CI Debugging and PR 40 Handoff

Saved: 2026-05-02 02:38:15 EDT

## Repository Context

- Repo: `/home/jmagar/workspace/lab`
- Branch: `bd-work/mcp-gateway-review-remediation`
- Remote branch: `origin/bd-work/mcp-gateway-review-remediation`
- PR: https://github.com/jmagar/lab/pull/40
- PR title: `Integrate service wave and CI updates`
- Base: `main`
- Head SHA: `9689190cad7150c06d7ee4d3ffda42c1bc8c1308`
- PR state: open
- Mergeable state from GitHub: `CONFLICTING`

## User Request Thread

The active work was to systematically debug CI failures on PR 40, especially the
Windows release build and security scan failures, then keep going until the CI
was actually green.

The user explicitly confirmed that Windows should build on GitHub's native
`windows-latest` runner, not by Linux-to-Windows cross-compilation.

## Root Causes Found

1. Neo4j unit test expectation drift
   - Test expected a sanitized Bolt URI with a trailing slash.
   - Current URL serialization no longer emits the trailing slash.

2. Windows build pulled a Unix-only dependency
   - `sd-notify` was available to all targets through the `systemd` feature.
   - On `windows-latest`, compiling all features attempted to compile the
     Unix-only dependency.

3. GitGuardian flagged secret-shaped fixtures
   - Current and historical PR commits contained strings shaped like OpenAI keys
     and JWT fragments.
   - GitGuardian scans the PR range, so current-file fixes alone were not enough.

4. Windows `-D warnings` failures
   - The native Windows runner found unused variable/function warnings that did
     not appear in the Linux all-features check.

5. Windows release smoke cache post-job failure
   - The Windows release binary build and `lab.exe --help` verification passed.
   - The job still failed afterward in `Post Run Swatinem/rust-cache@v2` while
     saving the cache.

## Pushed Fixes

Recent relevant commits on the branch:

- `9689190c Avoid Windows CI cache save failure`
- `6f8b8189 Fix Windows release warnings`
- `68a35a37 Trigger CI after history rewrite`
- `11199215 Fix CI failures`
- `60568674 Set up CI release smoke and generated docs`

Key changes:

- Updated the Neo4j sanitized URI test expectation.
- Moved `sd-notify` to a Unix target-specific dependency and guarded systemd
  notify code with Unix cfg.
- Replaced secret-shaped examples with placeholder-safe fixture strings.
- Rewrote PR branch history to remove GitGuardian findings from historical PR
  commits.
- Fixed Windows-only warnings in stash export, sysmetrics, and gateway process
  helper code.
- Hardened `.github/workflows/ci.yml` so the Windows release-smoke job restores
  Rust cache but does not save a new cache in post-job cleanup:
  `save-if: ${{ runner.os != 'Windows' }}`.

## Verification Evidence

Local verification performed during the run:

- `cargo test -p lab-apis --features neo4j --lib neo4j::client::tests::sanitized_uri_redacts_credentials -- --nocapture`
- `cargo test -p lab@0.12.1 --lib docs::artifacts::tests::secret_lint_rejects_token_like_values -- --nocapture`
- `cargo fmt --all -- --check`
- `cargo check -p lab@0.12.1 --all-features`
- `cargo tree -p lab --target x86_64-pc-windows-msvc --all-features -i sd-notify`
- `go run github.com/rhysd/actionlint/cmd/actionlint@latest .github/workflows/ci.yml`

Fresh GitHub Actions verification:

- Run: https://github.com/jmagar/lab/actions/runs/25240559658
- Head: `9689190cad7150c06d7ee4d3ffda42c1bc8c1308`
- Conclusion: success

Passing jobs in that run:

- Format
- Frontend assets
- Cargo Deny
- Actionlint
- Container build
- Clippy
- Test
- Check
- Release smoke (`windows-latest`)
- Release smoke (`ubuntu-latest`)

PR checks after the fix:

- CodeRabbit: pass
- GitGuardian Security Checks: pass
- cubic AI code reviewer: pass

## Important Current Worktree State

At save time the branch was pushed, but the local worktree was still dirty with
changes not included in the pushed CI fix commit:

```text
 M .github/workflows/ci.yml
 M config/Dockerfile.fast
 M crates/lab-apis/src/extract/client.rs
 M crates/lab-apis/src/extract/runtime.rs
 M crates/lab/src/acp/runtime.rs
 M docker-compose.yml
?? config/acp-providers.docker.json
```

The extra local `.github/workflows/ci.yml` diff adds a `docs-check` job after
`check`. That local edit was not part of commit `9689190c` and was not part of
the verified green CI run.

Do not assume the dirty files are disposable; they may be user or parallel-agent
work.

## Open Questions

- PR 40 is still reported by GitHub as `CONFLICTING`; the merge conflict was not
  resolved in this session.
- GitHub Actions PR checks did not consistently attach after history rewrites,
  so the authoritative full CI proof is the manual workflow run listed above.
- The local dirty `docs-check` workflow edit may need its own verification and
  commit, but it is separate from the green CI fix.

