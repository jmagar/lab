---
name: sync-claude-mds
description: Mirror every CLAUDE.md in the repo to AGENTS.md and GEMINI.md siblings via symlinks, so Codex (which reads AGENTS.md) and Gemini CLI (which reads GEMINI.md) see the same agent instructions Claude does. Use this skill whenever the user mentions syncing agent-context files, mirroring CLAUDE.md, refreshing AGENTS/GEMINI symlinks, or says things like "make Codex see our CLAUDE.md", "wire up AGENTS.md", "sync the claude memory files", or after creating a new nested CLAUDE.md anywhere in the repo (e.g. crates/core/CLAUDE.md, apps/web/CLAUDE.md). Walks the entire repo, handles nested cases, and refuses to clobber real AGENTS.md or GEMINI.md files. Idempotent — safe to invoke even when all symlinks are already correct.
---

This skill ensures every `CLAUDE.md` in the repo has matching `AGENTS.md` and `GEMINI.md` symlinks beside it, pointing at the same file. The bundled script does the discovery and linking; the skill exists so Claude (or the user via `/sync-claude-mds`) can invoke it on demand.

## When to invoke

Trigger this skill when:

- A new nested `CLAUDE.md` was just created (e.g. `crates/core/CLAUDE.md`, `apps/web/CLAUDE.md`) and Codex/Gemini siblings need to be added
- The user explicitly says "sync the claude memory files", "wire up AGENTS.md and GEMINI.md", "make Codex see our CLAUDE.md", "mirror the agent context files"
- A fresh checkout might not have the siblings, or someone manually deleted them
- A `git status` shows orphaned `AGENTS.md` or `GEMINI.md` symlinks

Do NOT invoke when:

- The user is editing CLAUDE.md content (the PostToolUse hook auto-syncs on every CLAUDE.md edit — manual run is redundant in normal flow)
- The user wants different content per platform (this skill enforces a single source of truth; per-platform forks belong elsewhere)

## How it works

`CLAUDE.md` is the canonical agent-instruction file in this project. Codex reads `AGENTS.md`; Gemini reads `GEMINI.md`. Rather than maintain three copies of the same content, we keep `CLAUDE.md` as the only real file and create `AGENTS.md` → `CLAUDE.md` and `GEMINI.md` → `CLAUDE.md` symlinks beside it. Same content, three filenames.

The script walks the entire repo (skipping vendor/build dirs: `.git/`, `node_modules/`, `target/`, `.next/`, `dist/`, `build/`) so nested CLAUDE.md files are mirrored too. Each gets its own pair of sibling symlinks.

## Invocation

```bash
.claude/skills/sync-claude-mds/scripts/sync-claude-mds.sh
```

Or invoke the skill itself with `/sync-claude-mds` — Claude reads this file and runs the bundled script.

The script:

1. Walks the repo, finding every `CLAUDE.md` (excluding vendor/build dirs)
2. For each, creates `AGENTS.md` and `GEMINI.md` symlinks alongside it (target: just `CLAUDE.md`, since they live in the same directory)
3. Idempotent: existing correct symlinks are reported as `(already in sync)`
4. Replaces stale symlinks pointing somewhere wrong
5. Refuses to clobber `AGENTS.md` or `GEMINI.md` if either exists as a real file (data-loss guard) — surfaces every blocked site in one pass rather than bailing on the first
6. Prints a summary: `Summary: X created/refreshed · Y already in sync · Z blocked`

## Exit codes

- `0` — every CLAUDE.md has correct AGENTS.md + GEMINI.md siblings
- `1` — no `CLAUDE.md` found anywhere in the repo
- `2` — one or more siblings exist as real files; manual migration required

## Flags

- `--quiet` / `-q` — suppress success output
- `--dry-run` / `-n` — print what would be done without making changes
- `--help` / `-h` — print the script's docstring

## Why this is also a skill, not just a script

The script alone covers the mechanics. Wrapping it as a skill gives Claude a precise trigger surface — when the user says "make Codex see our agent instructions" without mentioning AGENTS.md or symlinks explicitly, the description lets Claude pattern-match the intent. Skill packaging makes the operation discoverable through natural language, not just through knowledge of the script's path.

## Related

- The `sync-skills` skill is the analogous mechanism for `.claude/skills/` → `.agents/skills/` directory-level mirroring.
- Both are wired into the `PostToolUse` hook in `.claude/settings.json` so authoring flows keep mirrors healthy without explicit user action — manual invocation of this skill is mostly for edge cases (fresh checkouts, deleted symlinks, dry-run inspection).
