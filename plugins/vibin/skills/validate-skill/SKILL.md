---
name: validate-skill
description: Validate a Claude Code skill's SKILL.md using skills-ref validate, then layer in Claude Code-specific checks that skills-ref doesn't cover (trigger phrases, argument-hint, disable-model-invocation, plugin.json registration). Use whenever the user says "validate this skill", "check my skill", "is my skill valid", "review skill structure", "does this skill follow conventions", or is about to publish or install a skill. Also use proactively after creating or editing any SKILL.md file.
allowed-tools: Read, Bash
argument-hint: [path/to/skill or path/to/SKILL.md]
---

## Context

- Skill path: $ARGUMENTS
- CWD: !`pwd`

# Validate Skill

## Step 1 — Resolve skill path

If `$ARGUMENTS` is empty, try `./SKILL.md`, then walk up one level at a time (up to 4 levels) until a `SKILL.md` is found. If nothing is found, stop: "Cannot locate SKILL.md — pass the skill directory or SKILL.md path as an argument."

Set `SKILL_DIR` to the resolved directory. Set `SKILL_NAME` to its last path component.

## Step 2 — Run skills-ref validate

```bash
skills-ref validate <SKILL_DIR>
```

Capture the output. `skills-ref` will report any schema violations.

**Known false positives from skills-ref** — suppress these specific messages when reporting, since they are valid Claude Code frontmatter fields:
- `Unexpected fields in frontmatter: argument-hint` — valid, skip
- `Unexpected fields in frontmatter: disable-model-invocation` — valid, skip

Report all other `skills-ref` output as-is.

## Step 3 — Claude Code-specific checks

Read `SKILL_DIR/SKILL.md` and check the following. Collect as WARN or FAIL:

| Check | Condition | Severity |
|-------|-----------|----------|
| `description` has trigger phrases | Contains at least one "Use when…", "Use whenever…", or a concrete trigger phrase (e.g. user says "…") | WARN |
| `allowed-tools` lists only known tools | Each entry is one of: `Bash`, `Read`, `Write`, `Edit`, `MultiEdit`, `Glob`, `Grep`, `LS`, `WebFetch`, `WebSearch`, `TodoWrite`, `TodoRead`, `NotebookRead`, `NotebookEdit`, `Task`, `Agent` | FAIL |
| Script references resolvable | Any `scripts/` path mentioned in the body exists on disk relative to SKILL_DIR | WARN |
| Reference paths resolvable | Any `references/` path mentioned in the body exists on disk relative to SKILL_DIR | WARN |
| `disable-model-invocation` type | If present, value must be `true` or `false` | FAIL |
| No raw secrets | Body contains no patterns like `sk-`, `ghp_`, `xoxb-`, `AIza`, `Bearer ` literals | FAIL |

## Step 4 — Plugin registration check

Walk up from `SKILL_DIR` looking for a `plugin.json` (max 5 levels). If found, check whether `SKILL_NAME` appears in the `skills` array. WARN if the skill is not listed.

## Step 5 — Report

```
Skill: <SKILL_NAME>
Path:  <absolute SKILL_DIR>

skills-ref
----------
<output from skills-ref, minus suppressed false positives, or "✓ No issues">

Claude Code checks
------------------
FAIL  <message>
WARN  <message>

Summary: N failed, N warned
```

If all checks pass: "Skill looks good — no issues found."
