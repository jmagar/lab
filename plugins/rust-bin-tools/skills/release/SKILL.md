---
name: release
description: Walk through the release-plz + cargo-dist + cocogitto release workflow. REQUIRED before tagging a release. Verifies clean state, runs all CI gates, validates the release-plz plan, walks the version bump and changelog, then drives the tag/push.
disable-model-invocation: false
---

## When this skill MUST trigger

- User says "release", "ship a release", "cut a release", "tag a release", "publish a new version"
- User says "let's prepare for release" or asks about the release plan
- A merged PR closes work that completes a milestone or epic and the user signals it's release-worthy

## Anti-trigger

Do NOT run for:
- Building binaries locally (`just build`) — that's a different operation
- Running CI gates (`just ci`) — already covered by lefthook + release flow runs them
- Pushing a branch — that's a normal `git push`

## Release model (per spec.md § Build & Release)

This project uses three tools in concert:

- **release-plz** — opens a PR with version bumps + CHANGELOG entries derived from Conventional Commits (`feat:` → minor, `fix:` → patch, `feat!:` / `BREAKING CHANGE:` → major)
- **cargo-dist** — builds and publishes binaries on tag push; owns the GitHub Release
- **cocogitto** — verifies commit messages are Conventional Commits (commit-msg hook)

`git_release_enable = false` on release-plz so cargo-dist owns the GitHub Release. Don't enable both.

## Workflow

### 1. Verify clean working tree

```bash
git status                # must show clean working tree
git branch --show-current # confirm on main (or release branch)
```

If not clean: surface the dirty files, ask user to commit/stash before continuing. Do not proceed.

### 2. Sync with remote

```bash
git pull --rebase origin main
```

If the rebase conflicts: stop and surface. Don't bulldoze.

### 3. Run full CI gates locally

```bash
just ci
```

This runs `fmt-check`, `lint` (incl. all `cargo xtask check-*`), `test`, and `audit`. If anything fails:
- Fix or roll back before continuing
- Do NOT proceed past failures with `--no-verify` or any bypass — the no-verify hook will block it anyway

### 4. Verify CI parity

```bash
cargo xtask verify-ci-parity
```

Confirms `just lint` runs the same set of checks GitHub Actions does. Drift here means CI could pass while a local check is silently absent.

### 5. Inspect the release-plz plan

```bash
release-plz update --dry-run
```

Surface to the user:
- Which crate(s) get bumped
- The version bump per crate (patch/minor/major)
- The CHANGELOG entries that will be added
- Any commits that release-plz couldn't categorize (these are non-conventional and should be amended or noted)

If the proposed bump doesn't match user intent, fix the conventional-commit footers before re-running.

### 6. Open or merge the release-plz PR

Two paths depending on team workflow:

**Path A (recommended): let release-plz CI handle PR creation.** Push to main; the release-plz GitHub Action opens/updates the release PR. Wait for it. Then merge.

**Path B (manual): run release-plz locally and push the branch.** Use only if the GitHub Action is broken or the user explicitly wants the local path.

### 7. After the release PR merges → tag is pushed → cargo-dist takes over

cargo-dist's GitHub Action:
- Builds binaries for x86_64-linux, aarch64-linux, x86_64-darwin, aarch64-darwin
- Creates the GitHub Release with the tag
- Attaches binaries + checksums

Verify by watching the Actions tab. If a build fails, fix and tag a patch version — never delete and re-push the same tag.

### 8. Post-release verification

```bash
gh release view <new-tag>          # confirm release exists with binaries
gh release view <new-tag> --json assets --jq '.assets | length'  # asset count matches matrix
```

Then surface to the user:
- Release URL
- Asset count vs expected matrix size
- Any cargo-dist warnings worth noting

### 9. Promote any active exec-plans → completed

If this release shipped work tracked by plans in `docs/exec-plans/active/`, invoke the `promote-plan` skill for each. Don't leave the docs lying about state.

## Common pitfalls

- **Non-conventional commits** in the release range silently get omitted from the CHANGELOG. Run `cog verify` (cocogitto) on the range first if unsure.
- **`feat!:` vs `feat:` matters** — the `!` is the major-bump signal. A breaking change without `!` will release as a minor and break consumers.
- **Don't tag manually** — release-plz creates the tag from its merged PR. Manual tags bypass the changelog generation.
- **Don't enable `git_release_enable`** on release-plz; cargo-dist owns the release. Both fighting for the release leads to duplicate or empty releases.

## Output

After successful release, summarize:
- New version(s) released
- Crate(s) bumped
- CHANGELOG snippet
- Release URL
- Plans promoted to `completed/`
