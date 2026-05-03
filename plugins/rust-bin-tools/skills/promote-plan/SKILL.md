---
name: promote-plan
description: REQUIRED whenever an execution plan's real-world state diverges from its directory location. MUST run when (a) starting work that has a proposed plan, (b) finishing work that has an active plan, (c) abandoning a proposed plan, or (d) the drift-detector flags LIKELY_ACTIVE / LIKELY_DONE / STALLED. Performs `git mv` between proposed/ → active/ → completed/, updates the Status line, and fixes cross-references. Skipping this leaves the docs lying about project state.
disable-model-invocation: false
---

## When this skill MUST trigger

This is non-negotiable — the plan's directory is supposed to reflect reality. Run this skill any time the directory and reality diverge.

### proposed/ → active/ — REQUIRED triggers

- User says "let's start on X" / "begin implementing X" / "I'm working on X" and a plan exists in `proposed/` matching X
- About to make the **first** code change implementing a `proposed/` plan — run BEFORE the first edit, not after
- A new branch is created whose name matches a `proposed/` plan slug
- A bead linked to a `proposed/` plan transitions to `in_progress`
- `drift-detector` reports `LIKELY_ACTIVE`

### active/ → completed/ — REQUIRED triggers

- User says "we're done with X" / "ship it" / "X is complete" / "close out X"
- All target files/features in an `active/` plan are implemented and merged to main
- `just lint && just test` passes for an `active/` plan's deliverable
- A bead linked to an `active/` plan is closed
- `drift-detector` reports `LIKELY_DONE`

### Abandon (stays in proposed/, status updated) — REQUIRED triggers

- User says "drop X" / "we're not doing X" / "abandon X"
- `drift-detector` reports `STALLED` and the user confirms abandonment instead of activation
- A `proposed/` plan is superseded by a different plan or ADR — abandon the old one, do not delete

### Backwards (active/ → proposed/) — RARE, requires explicit user confirmation

Only when work was un-started (not undone), e.g. someone moved a plan to `active/` prematurely. Ask the user to confirm before performing.

## Anti-trigger

Do **not** run this skill for:
- Reading or referencing a plan (no state change)
- Editing a plan's content (no location change)
- Creating a brand-new plan — that's a different skill (use `/exec-plan` if it exists, or write directly into `proposed/`)

## Usage

`/promote-plan` — interactively pick a plan and target state.

`/promote-plan <slug>` — move the named plan one stage forward.

`/promote-plan <slug> abandon` — mark a proposed plan as abandoned (keeps it in `proposed/` per `docs/CLAUDE.md` — never delete).

## Lifecycle

```
proposed/  →  active/  →  completed/
   |
   └─→ (annotated abandoned, stays in proposed/)
```

Backwards moves (active → proposed) are allowed but require explicit confirmation — usually means the work was un-started, not undone.

## Steps

### 1. Locate the plan

```bash
find docs/exec-plans/{proposed,active,completed} -name "*{slug}*.md" 2>/dev/null
```

If multiple matches, ask the user which one. If zero, list available plans:
```bash
ls docs/exec-plans/proposed/ docs/exec-plans/active/ 2>/dev/null
```

### 2. Determine target

| Current location | Default target | Notes |
|---|---|---|
| `proposed/` | `active/` | Confirm work is actually starting |
| `active/` | `completed/` | Confirm work shipped (tests pass, in main) |
| `completed/` | (no-op) | Already terminal |
| any | `proposed/` w/ `Status: abandoned` | Only when user passes `abandon` |

### 3. Pre-move checks

- **proposed → active:** verify there's evidence work is starting (recent commits, branch, or explicit user confirmation)
- **active → completed:** verify the work shipped:
  ```bash
  # Check the plan's referenced files exist with non-stub content
  # Check git log for closing keywords
  git log --oneline --grep "{slug}" | head
  ```
  If checks don't pass cleanly, surface the gaps and ask the user to confirm before moving.

### 4. Update the plan's metadata

Read the plan file. Look for a `Status:` line near the top. Update it:

| Move | New Status line |
|---|---|
| → `active/` | `**Status:** Active (started YYYY-MM-DD)` |
| → `completed/` | `**Status:** Completed (shipped YYYY-MM-DD)` |
| → abandon | `**Status:** Abandoned (YYYY-MM-DD) — <one-line reason from user>` |

If no `Status:` line exists, add one near the top of the file (after the title, before the first `##` heading) following the same format as ADRs.

### 5. Move the file (preserves git history)

```bash
git mv docs/exec-plans/{old}/{slug}.md docs/exec-plans/{new}/{slug}.md
```

For abandonment, no move — only the Status line update, then commit.

### 6. Find and fix cross-references

```bash
grep -rn "exec-plans/{old}/{slug}\|exec-plans/{old}/{any-prefix-of-slug}" \
  CLAUDE.md docs/ spec.md 2>/dev/null
```

Show every match. Update each to point at the new path. Common locations:

- `docs/exec-plans/tech-debt-tracker.md`
- `spec.md` (rare, but possible if a plan was canonicalized)
- ADRs in `docs/design-docs/` that reference the plan
- `CLAUDE.md` (entry-point map)
- Bead comments (`bd comments` if relevant)

### 7. Commit

```bash
git add -A
git commit -m "chore(plan): promote {slug} → {new-stage}"
```

For abandonment:
```bash
git commit -m "chore(plan): abandon {slug} — <reason>"
```

### 8. Confirm

List the plan's new location and any cross-reference fixes applied. Note if any references couldn't be auto-fixed.

## Notes

- This skill respects the `docs/CLAUDE.md` rule that abandoned plans stay in `proposed/` rather than being deleted (archaeology beats churn).
- For plans without a `Status:` line, this skill adds one — bringing older plans into the standard format.
- Pair with `drift-detector` to find which plans need promoting; this skill performs the move once you decide.
