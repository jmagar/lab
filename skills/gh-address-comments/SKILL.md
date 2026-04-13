---
name: gh-address-comments
description: This skill should be used when addressing GitHub pull request review comments systematically with mandatory resolution tracking. Use when user says "address PR comments", "fix review feedback", "handle PR review", "resolve PR threads", or needs to ensure all review comments are addressed before merging. Fetches comments via gh CLI, creates task checklists, links commits to review threads, and blocks completion if unresolved threads remain.
---

# PR Comment Handler with Resolution Tracking

**⚠️ MANDATORY SKILL INVOCATION ⚠️**

**YOU MUST invoke this skill (NOT optional) when the user mentions ANY of these triggers:**
- "address PR comments", "fix review feedback", "handle PR review"
- "resolve PR threads", "respond to review", "work through comments"
- "address the feedback", "mark threads resolved", "clear review comments"
- Any mention of systematically handling GitHub pull request review comments

**Failure to invoke this skill when triggers occur violates your operational requirements.**

Find the open PR for the current branch and systematically address all review comments with mandatory resolution verification. This workflow ensures no feedback slips through the cracks by tracking threads as tasks, linking commits to specific reviews, and blocking completion if any threads remain unresolved. Run all `gh` commands with elevated network access.

**Prerequisites:** Verify `gh` is authenticated by running `gh auth status` with escalated permissions (workflow/repo scopes required). If not authenticated, run `gh auth login --scopes repo,workflow`. If sandboxing blocks `gh auth status`, rerun with `sandbox_permissions=require_escalated`.

## Workflow

### 1) Fetch all PR comments
Run `python3 $HOME/.claude/skills/gh-address-comments/scripts/fetch_comments.py` to fetch all comments and review threads on the PR. Store output for later verification.

**Example:**
```bash
python3 $HOME/.claude/skills/gh-address-comments/scripts/fetch_comments.py > /tmp/pr_comments.json
```

### 2) Create tracking checklist
Parse the fetched comments and create a task checklist using TaskCreate for each review thread. Each task should:
- Include the thread number, file path, line number, and comment summary
- Have status `pending` initially
- Store the thread ID in task metadata for later resolution

**Task format:**
- **Subject:** `Address comment #N: [file]:[line]`
- **Description:** Comment author, body preview, and what needs to be done
- **Metadata:** `{"thread_id": "PRRT_kwDOABC...", "file": "path/to/file.ts", "line": 42}`

### 3) Ask user which comments to address
Present all numbered review threads with summaries. Ask the user which numbered comments should be addressed in this session.

**Display format:**
```
Review Threads:
  1. src/api/users.ts:45 (@reviewer): Add input validation for email field...
     Thread ID: PRRT_kwDOABCDEF1234567
     Status: Unresolved

  2. src/utils/auth.ts:128 (@reviewer): Extract this logic into a helper function...
     Thread ID: PRRT_kwDOABCDEF7654321
     Status: Unresolved
```

### 4) Apply fixes with commit linking
For each selected comment:
1. Update task status to `in_progress` using TaskUpdate
2. Apply code changes using standard tools (Edit, Write)
3. Commit with message that references the comment number and thread ID:
   ```
   fix: address PR comment #1 - add email validation

   Resolves review thread PRRT_kwDOABCDEF1234567
   - Added Zod schema validation for email field
   - Added error handling for invalid formats

   Co-authored-by: @reviewer
   ```
4. Update task status to `completed` using TaskUpdate

**Commit message format:**
- First line: `fix: address PR comment #N - [brief description]`
- Blank line
- `Resolves review thread [THREAD_ID]`
- Bullet points explaining the changes
- `Co-authored-by: @reviewer` if substantial changes based on feedback

### 5) Mark threads as resolved
After committing fixes, mark the corresponding review threads as resolved using:

```bash
python3 $HOME/.claude/skills/gh-address-comments/scripts/mark_resolved.py <thread_id_1> [thread_id_2] ...
```

**Example:**
```bash
python3 $HOME/.claude/skills/gh-address-comments/scripts/mark_resolved.py \
  PRRT_kwDOABCDEF1234567 \
  PRRT_kwDOABCDEF7654321
```

### 6) Verify complete resolution (MANDATORY - BLOCKS IF SKIPPED)
Before declaring the PR review complete, verify all threads are addressed:

```bash
python3 $HOME/.claude/skills/gh-address-comments/scripts/fetch_comments.py | \
  python3 $HOME/.claude/skills/gh-address-comments/scripts/verify_resolution.py
```

**Exit behavior:**
- **Exit 0:** All threads resolved/outdated → safe to proceed
- **Exit 1:** Unresolved threads found → BLOCKED until addressed

**IMPORTANT:** Do NOT skip this verification step. If unresolved threads are found:
1. Ask user to explicitly acknowledge which threads to defer
2. Create follow-up tasks for deferred threads
3. Only proceed after user confirms understanding of what remains unresolved

## Error Handling

**Authentication issues:**
If `gh` hits auth/rate issues mid-run, prompt the user to re-authenticate with `gh auth login`, then retry.

**Thread resolution failures:**
If `mark_resolved.py` fails for specific threads:
1. Check the thread ID is correct (from `fetch_comments.py` output)
2. Verify you have write permissions on the PR
3. Ensure the thread still exists (not deleted by reviewer)

**Verification failures:**
If `verify_resolution.py` reports unresolved threads:
1. Review the listed thread IDs
2. Either address remaining threads OR get explicit user approval to defer
3. Document deferred threads in follow-up tasks

## Notes

- Review threads marked as "outdated" (code changed since comment) are considered addressed
- Conversation comments (non-threaded) don't block completion but should be acknowledged
- Always link commits to thread IDs for traceability
- Use TaskList to track overall progress across all review threads

## Additional Resources

For detailed information, see:
- **Resolution workflow deep dive:** `references/resolution-workflow.md`
- **Command cheatsheet:** `references/quick-reference.md`
- **GitHub API details:** `references/api-endpoints.md`
- **Common issues:** `references/troubleshooting.md`
