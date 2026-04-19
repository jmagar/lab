---
name: gh-address-comments
description: Use when addressing GitHub pull request review comments systematically with mandatory resolution tracking. Triggers on "address PR comments", "fix review feedback", "handle PR review", "resolve PR threads", "respond to review", "work through comments", "mark threads resolved", or any mention of systematically handling GitHub PR review feedback. Fetches comments via gh CLI, presents a grouped summary, creates task checklists, links commits to review threads, and blocks completion if unresolved threads remain. Use proactively whenever the user is working through PR feedback — even if they don't explicitly ask for the full workflow.
---

# PR Comment Handler with Resolution Tracking

Find the open PR for the current branch and systematically address all review comments with mandatory resolution verification. This workflow ensures no feedback slips through the cracks by tracking threads as tasks, linking commits to specific reviews, and blocking completion if any threads remain unresolved.

**Prerequisites:** Verify `gh` is authenticated (`gh auth status`). If not, run `gh auth login --scopes repo,workflow`.

## Available CLI Tools

All scripts are in PATH — no need to call them via `python3 path/to/script`:

| Command | Purpose |
|---------|---------|
| `gh-fetch-comments` | Fetch all PR comments/threads via GraphQL |
| `gh-pr-summary` | Human-readable grouped digest of threads |
| `gh-mark-resolved` | Mark threads resolved (concurrent, supports `--all`) |
| `gh-verify-resolution` | Verify all threads are addressed (supports `--watch`) |

**Key flags at a glance:**
```
gh-fetch-comments --pr 2 -o pr.json          # fetch and cache
gh-fetch-comments --pr 2 --since pr_old.json # show only new/changed since snapshot
gh-pr-summary --input pr.json --open-only    # grouped view of open threads
gh-pr-summary --input pr.json --by reviewer  # group by who left feedback
gh-mark-resolved PRRT_kwDO... PRRT_kwDO...   # resolve specific threads
gh-mark-resolved --all --input pr.json       # resolve ALL unresolved at once
gh-mark-resolved --dry-run --all --input pr.json  # preview before resolving
gh-verify-resolution --input pr.json         # check from cached file
gh-verify-resolution --watch --pr 2          # poll live until all resolved
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

Always save to a file with `-o` — it avoids re-fetching during the session and enables `--since` diffs later.

### 2) Show a summary

Before creating tasks, give the user an overview:
```bash
gh-pr-summary --input /tmp/pr.json --open-only
```

This groups open threads by file (or use `--by reviewer` to see who left what). Ask the user which threads to tackle in this session — don't assume all must be addressed now.

### 3) Create tracking checklist

Parse the fetched comments and create a task using TaskCreate for each thread the user wants to address:

- **Subject:** `Address comment #N: [file]:[line]`
- **Description:** Comment author, body preview, what needs to be done
- **Metadata:** `{"thread_id": "PRRT_kwDOABC...", "file": "path/to/file.ts", "line": 42}`

### 4) Apply fixes with commit linking

For each selected thread:
1. Mark task `in_progress` via TaskUpdate
2. Apply code changes with Edit/Write
3. Commit referencing the thread:
   ```
   fix: address PR comment #1 - add email validation

   Resolves review thread PRRT_kwDOABCDEF1234567
   - Added Zod schema validation for email field
   - Added error handling for invalid formats

   Co-authored-by: @reviewer
   ```
4. Mark task `completed` via TaskUpdate

### 5) Mark threads resolved

After committing, resolve the corresponding threads. For a few specific threads:
```bash
gh-mark-resolved PRRT_kwDOABCDEF1234567 PRRT_kwDOABCDEF7654321
```

To resolve everything remaining at once (after you've addressed all of them):
```bash
gh-mark-resolved --all --input /tmp/pr.json
```

Use `--dry-run` first if unsure what `--all` will touch.

### 6) Verify complete resolution (MANDATORY)

Re-fetch and verify, or use `--watch` to poll until a reviewer resolves on their end:
```bash
# one-shot check from fresh fetch
gh-fetch-comments --pr 2 -o /tmp/pr.json && gh-verify-resolution --input /tmp/pr.json

# watch mode — polls every 30s until all threads resolved
gh-verify-resolution --watch --pr 2 --interval 30
```

**Exit behavior:**
- **Exit 0:** All threads resolved/outdated → safe to proceed
- **Exit 1:** Unresolved threads found → BLOCKED

If unresolved threads remain:
1. Ask the user which threads to defer
2. Create follow-up tasks for deferred threads
3. Only proceed after explicit user confirmation

### 7) Checking for new feedback (returning to a PR)

If coming back to a PR after time away, diff against the previous snapshot to see only what's changed:
```bash
cp /tmp/pr.json /tmp/pr_old.json
gh-fetch-comments --pr 2 -o /tmp/pr.json --since /tmp/pr_old.json
```

This outputs only new comments, newly opened threads, and threads whose resolution state changed — not the full history.

## Error Handling

**Not on the feature branch:** Pass `--pr NUMBER` explicitly to all fetch commands. Use `gh pr list` to find the right number.

**Authentication issues:** Re-authenticate with `gh auth login`, then retry.

**Thread resolution failures:** Check that the thread ID is correct (from fetch output), you have write permissions on the PR, and the thread hasn't been deleted by the reviewer.

**Verification blocked:** Either address the remaining threads or get explicit user approval to defer them. Document deferred threads as follow-up tasks.

## Notes

- Threads marked "outdated" (code changed since comment) count as addressed
- Conversation comments don't block completion but acknowledge them to the user
- Always link commits to thread IDs for traceability
- `gh-mark-resolved` runs mutations concurrently — safe for large thread counts

## Additional Resources

- `references/resolution-workflow.md` — Resolution workflow deep dive
- `references/quick-reference.md` — Command cheatsheet
- `references/api-endpoints.md` — GitHub API details
- `references/troubleshooting.md` — Common issues
