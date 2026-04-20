---
name: gh-address-comments
description: Use when addressing GitHub pull request review comments systematically with mandatory resolution tracking. Triggers on "address PR comments", "fix review feedback", "handle PR review", "resolve PR threads", "respond to review", "work through comments", "mark threads resolved", or any mention of systematically handling GitHub PR review feedback. Fetches comments via gh CLI, creates beads for each thread, presents a grouped summary, links commits to review threads, closes beads on resolution, and blocks completion if unresolved threads remain. Use proactively whenever the user is working through PR feedback — even if they don't explicitly ask for the full workflow.
---

# PR Comment Handler with Resolution Tracking

Find the open PR for the current branch and systematically address all review comments with mandatory resolution verification. Threads are tracked as beads so work is visible in `bd ready` alongside other project work.

**Prerequisites:** Verify `gh` is authenticated (`gh auth status`). If not, run `gh auth login --scopes repo,workflow`.

## Available CLI Tools

All scripts are in PATH:

| Command | Purpose |
|---------|---------|
| `gh-fetch-comments` | Fetch all PR comments/threads via GraphQL |
| `gh-pr-summary` | Human-readable grouped digest of threads |
| `gh-create-beads` | Create a bead for each open thread |
| `gh-mark-resolved` | Mark threads resolved; auto-closes beads |
| `gh-close-beads` | Close beads for threads that are now resolved |
| `gh-verify-resolution` | Verify all threads are addressed |
| `gh-post-reply` | Post a reply to a thread (e.g. "Fixed in abc1234") |
| `gh-ai-triage` | AI-powered priority/effort analysis of open threads |
| `gh-thread-context` | Show file code context for a thread |
| `gh-pr-status` | Quick merge-readiness dashboard |
| `gh-pr-checklist` | Full pre-merge gate with actionable fix commands |
| `gh-pr-changelog` | Generate changelog from resolved-thread commits |

**Key flags at a glance:**
```
gh-fetch-comments --pr 2 -o pr.json
gh-fetch-comments --pr 2 --since pr_old.json   # diff — only new/changed
gh-pr-summary --input pr.json --open-only
gh-pr-summary --input pr.json --by priority
gh-pr-summary --input pr.json --format markdown
gh-create-beads --input pr.json                # create beads for all open threads
gh-create-beads --input pr.json --dry-run
gh-mark-resolved --all --input pr.json         # resolve + auto-close beads
gh-close-beads --input pr.json --refresh       # close beads for newly resolved threads
gh-verify-resolution --input pr.json
gh-verify-resolution --watch --pr 2
gh-post-reply PRRT_kwDO... --commit            # reply "Fixed in <HEAD>"
gh-ai-triage --input pr.json
gh-thread-context PRRT_kwDO... --input pr.json
gh-pr-status --pr 2 --input pr.json
gh-pr-checklist --pr 2 --input pr.json
gh-pr-changelog --pr 2 --input pr.json --format markdown
```

## Workflow

### 1) Fetch and cache PR comments

If on the PR's feature branch, auto-detect:
```bash
gh-fetch-comments -o /tmp/pr.json
```

If on `main` or a different branch, specify the PR number:
```bash
gh-fetch-comments --pr 2 -o /tmp/pr.json
```

Always save with `-o` — it automatically creates beads for open threads, enables diffs, and avoids re-fetching. Pass `--no-beads` to skip bead creation.

Beads for all open threads are created automatically after the fetch. No separate step needed.

### 2) Show a summary and triage

Give the user an overview:
```bash
gh-pr-summary --input /tmp/pr.json --open-only
```

For large PRs, run AI triage first to prioritise:
```bash
gh-ai-triage --input /tmp/pr.json
```

Ask which threads to tackle in this session — don't assume all must be addressed now.
To inspect what a specific thread is actually commenting on:
```bash
gh-thread-context PRRT_kwDO... --input /tmp/pr.json
```

### 4) Apply fixes with commit linking

For each selected thread:
1. Apply code changes with Edit/Write
2. Commit referencing the thread:
   ```
   fix: address PR comment #1 - add email validation

   Resolves review thread PRRT_kwDOABCDEF1234567
   - Added Zod schema validation for email field
   - Added error handling for invalid formats

   Co-authored-by: @reviewer
   ```
   The `Resolves review thread PRRT_...` footer is what `gh-pr-changelog` uses to
   build the changelog, so always include it.

### 5) Reply, resolve, and close beads

After committing, post a reply so the reviewer sees acknowledgment, then resolve.
Beads are closed automatically:
```bash
# Reply to the thread
gh-post-reply PRRT_kwDOABCDEF1234567 --commit

# Resolve the thread (bead closes automatically)
gh-mark-resolved PRRT_kwDOABCDEF1234567 --input /tmp/pr.json
```

To do this for all remaining threads at once:
```bash
gh-mark-resolved --all --input /tmp/pr.json
```

If a reviewer resolves threads on their end (outside this workflow):
```bash
gh-close-beads --input /tmp/pr.json --refresh
```

Pass `--no-beads` to either command to skip bead operations entirely.

### 6) Verify complete resolution (MANDATORY)

```bash
gh-fetch-comments --pr 2 -o /tmp/pr.json && gh-verify-resolution --input /tmp/pr.json
```

Or watch mode — polls every 30s until all threads are resolved:
```bash
gh-verify-resolution --watch --pr 2 --interval 30
```

**Exit behavior:**
- **Exit 0:** All threads resolved/outdated → safe to proceed
- **Exit 1:** Unresolved threads found → BLOCKED

If threads remain: ask the user which to defer, create follow-up beads/tasks for
them, and only proceed after explicit confirmation.

### 7) Pre-merge check

Before calling the PR done:
```bash
gh-pr-checklist --pr 2 --input /tmp/pr.json
```

This verifies CI, approvals, thread resolution, and merge conflicts in one pass
with actionable fix commands for each failure.

### 8) Checking for new feedback (returning to a PR)

Diff against a previous snapshot to see only what changed:
```bash
cp /tmp/pr.json /tmp/pr_old.json
gh-fetch-comments --pr 2 -o /tmp/pr.json --since /tmp/pr_old.json
```

Beads for any newly opened threads are created automatically after the fetch.

## Bead Lifecycle

```
gh-fetch-comments -o pr.json   →  beads auto-created  →  appear in `bd ready`
                                                                  ↓
gh-mark-resolved --input pr.json          →  beads auto-closed in bd
        or
gh-close-beads --input pr.json --refresh  →  beads closed when reviewer resolves
```

Both steps are fully automatic. Pass `--no-beads` to opt out.

The mapping file `<input>.beads.json` links thread IDs to bead IDs. If it's
missing, run `gh-create-beads --input pr.json` to recreate it.

## Error Handling

**Not on the feature branch:** Pass `--pr NUMBER` explicitly. Use `gh pr list` to find the number.

**Authentication issues:** Re-authenticate with `gh auth login`, then retry.

**`bd` not found:** The beads CLI must be installed. Bead steps can be skipped if unavailable — the rest of the workflow functions without it.

**Thread resolution failures:** Check the thread ID is correct, you have write permissions, and the thread hasn't been deleted.

**Verification blocked:** Defer threads explicitly with user approval. Document deferred threads as beads with `--defer`.

## Notes

- Threads marked "outdated" (code changed since comment) count as addressed
- Conversation comments don't block completion but acknowledge them to the user
- Always include `Resolves review thread PRRT_...` in commit footers — `gh-pr-changelog` depends on it
- `gh-mark-resolved` runs mutations concurrently — safe for large thread counts

## Additional Resources

- `references/resolution-workflow.md` — Resolution workflow deep dive
- `references/quick-reference.md` — Command cheatsheet
- `references/api-endpoints.md` — GitHub API details
- `references/troubleshooting.md` — Common issues
