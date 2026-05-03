---
name: sync-skills
description: Wire `.agents/skills` as a symlink to `.claude/skills` so Codex and Gemini CLI can read skills from the same canonical location Claude does. Use this skill whenever the user mentions syncing skills across platforms, the `.agents/skills` directory is missing or broken, after cloning a fresh checkout, or when the user says things like "set up the skills mirror", "wire up the skills symlink", "make Codex see our skills", or "sync skills". Idempotent — safe to invoke even if the link is already in place. Also fires the underlying script automatically via PostToolUse hook on every edit under `.claude/skills/` so manual invocation is rarely needed in normal authoring flow.
---

This skill ensures the `.agents/skills` symlink points at `.claude/skills` so all three agentic CLIs (Claude, Codex, Gemini) discover the same set of skills from one canonical location. The bundled script does the work; the skill exists so Claude (or the user via `/sync-skills`) can invoke it on demand.

## When to invoke

Trigger this skill when:

- A fresh checkout doesn't yet have `.agents/skills` (Codex and Gemini will see no skills until it's created)
- The user explicitly says "sync skills", "wire the symlink", "set up the agents skills mirror", "make Codex see our skills"
- `.agents/skills` exists but points somewhere wrong (pulled from an unrelated branch, manually edited, etc.)
- A diagnostic step suggests Codex or Gemini isn't seeing skills the user expects

Do NOT invoke when:

- The user is editing skill content (the PostToolUse hook handles incremental syncs automatically — running this manually adds no value)
- The user wants to copy skills somewhere else (the script only manages the canonical `.claude/skills` ↔ `.agents/skills` link)

## How it works

`.claude/` is the single source of truth for agent resources in this project — see the root `CLAUDE.md` § "Agent-config authoring". Codex and Gemini both read skills from `.agents/skills/` (the open-skill-standard location). Rather than maintain two parallel directory trees, we keep `.claude/skills/` as the real directory and symlink `.agents/skills` → `../.claude/skills`. One source, three readers.

The symlink uses a relative target (`../.claude/skills`) so it survives a fresh clone or move — Git stores the link itself as a tree object, no path normalization needed.

## Invocation

```bash
.claude/skills/sync-skills/scripts/sync-skills.sh
```

Or invoke the skill itself with `/sync-skills` — Claude reads this file and runs the bundled script.

The script:

1. Verifies `.claude/skills/` exists (errors if not — nothing to sync)
2. Refuses to overwrite `.agents/skills` if it exists as a real directory (data-loss guard — tells the user to migrate contents into `.claude/skills/` first)
3. Creates `.agents/` if missing
4. Drops any stale symlink and creates a fresh `../.claude/skills` relative link
5. Reports `✓ .agents/skills -> .claude/skills (Codex + Gemini read from here)`

Idempotent: re-running when the link is already correct is a no-op (`readlink` check, prints "(already in sync)", exits 0).

## Exit codes

- `0` — symlink in place
- `1` — `.claude/skills/` missing
- `2` — `.agents/skills` is a real directory; manual migration required

## Flags

- `--quiet` / `-q` — suppress success output (useful from CI / hooks)
- `--help` / `-h` — print the script's docstring

## Why this is also a skill, not just a script

The script alone covers the *mechanical* operation. Wrapping it as a skill gives Claude a precise trigger surface — when the user says "set up the agents/skills mirror" without mentioning the exact path or script name, the description above lets Claude pattern-match the intent and invoke. Pure script-only tooling depends on the user knowing the script exists; skill packaging makes it discoverable through natural language.

## Related

- The `agent-config` skill at `.claude/skills/agent-config/` is the broader cross-platform-config reference; this skill handles the specific symlink-maintenance task it depends on.
- The `sync-claude-mds` skill is the analogous mechanism for `CLAUDE.md` → `AGENTS.md`/`GEMINI.md` sibling symlinks.
- Both are wired into the `PostToolUse` hook in `.claude/settings.json` so authoring flows keep the mirror healthy without explicit user action.
