---
name: check-agent-parity
description: Authoritative checklist for auditing parity between Claude Code, Codex CLI, and Gemini CLI configurations across skills, subagents, hooks, and MCP servers. Encodes the cross-platform mapping rules including the hook-event capability gap. Preloaded by agent-parity.
user-invocable: false
---

The three platforms are intentionally similar but not identical. Your job is to find the **deltas that matter** — not every difference is a problem. A skill that's only useful in one platform (e.g. uses tools the others don't expose) doesn't belong everywhere. Don't flag it as a parity violation.

The canonical source of cross-platform configuration differences is `agent-config/references/platform-matrix.md`. When you need to disambiguate something — a tool name, a hook event semantic, a config key — read that file. The labels and rules below are derived from it.

Walk every section in order. Use the output format the section specifies. Skip platforms that aren't present in this project (e.g. `.gemini/` may not exist) — list them as `NOT_CONFIGURED` and continue.

---

## §0 — Detect which platforms are in play

```bash
test -d .claude  && echo "claude: present"  || echo "claude: NOT_CONFIGURED"
test -d .codex   && echo "codex: present"   || echo "codex: NOT_CONFIGURED"
test -d .gemini  && echo "gemini: present"  || echo "gemini: NOT_CONFIGURED"
test -d .agents  && echo ".agents/ shared dir: present (read by Codex + Gemini)" || true
```

A `.agents/` directory may exist independently of `.codex/` or `.gemini/` — it's the open-skill-standard location both consume. If `.agents/` is present, the project is implicitly configured for Codex and Gemini skill discovery, even if `.codex/` and `.gemini/` themselves aren't populated.

If only one platform is present, parity is trivially "no comparison possible" — report the missing platforms, ask the user whether they want to mirror the present platform's resources, and stop. Don't manufacture findings against absent platforms.

If two or more are present, run the full audit below for each pair (and for the three-way diff where all are present).

## Live state snapshot (injected at skill load)

The lists below are **pre-computed** at skill-load time — you do not need to re-run `ls` or `jq` to discover what exists. Use these directly to populate the per-section comparisons. If any list shows `(injection failed)` or is empty when you expect it not to be, fall back to running the equivalent command yourself.

### Platforms detected
```!
for p in .claude .codex .gemini .agents; do
  if [ -d "$p" ]; then echo "$p: present"; else echo "$p: NOT_CONFIGURED"; fi
done
```

### Skills present (names only)
- Claude (`.claude/skills/`):
  !`for d in .claude/skills/*/; do [ -f "${d}SKILL.md" ] && basename "$d"; done 2>/dev/null | sort | tr '\n' ' ' || echo "(injection failed)"`
- Shared (`.agents/skills/` — read by Codex AND Gemini):
  !`for d in .agents/skills/*/; do [ -f "${d}SKILL.md" ] && basename "$d"; done 2>/dev/null | sort | tr '\n' ' ' || echo "(injection failed)"`
- Gemini-only fallback (`.gemini/skills/`):
  !`for d in .gemini/skills/*/; do [ -f "${d}SKILL.md" ] && basename "$d"; done 2>/dev/null | sort | tr '\n' ' ' || echo "(injection failed)"`

### Subagents present (filename only)
- Claude (`.claude/agents/`):
  !`ls .claude/agents/*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | sort | tr '\n' ' ' || echo "(injection failed)"`
- Codex (`.codex/agents/`):
  !`ls .codex/agents/*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | sort | tr '\n' ' ' || echo "(injection failed)"`
- Gemini (`.gemini/agents/`):
  !`ls .gemini/agents/*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | sort | tr '\n' ' ' || echo "(injection failed)"`

### Hooks (event names only)
- Claude (`.claude/settings.json`):
  !`jq -r '.hooks | keys[]' .claude/settings.json 2>/dev/null | tr '\n' ' ' || echo "(no hooks or file missing)"`
- Codex enable flag (`.codex/config.toml`):
  !`grep -E "^\s*codex_hooks\s*=\s*true" .codex/config.toml 2>/dev/null && echo "ENABLED" || echo "DISABLED (or file missing)"`
- Codex JSON hooks (`.codex/hooks.json`):
  !`jq -r '.hooks | keys[]' .codex/hooks.json 2>/dev/null | tr '\n' ' ' || echo "(file missing)"`
- Codex TOML hooks (`.codex/config.toml`):
  !`grep -oE "^\[\[hooks\.[A-Za-z]+" .codex/config.toml 2>/dev/null | sed 's/\[\[hooks\.//' | sort -u | tr '\n' ' ' || echo "(file missing)"`
- Gemini (`.gemini/settings.json`):
  !`jq -r '.hooks | keys[]' .gemini/settings.json 2>/dev/null | tr '\n' ' ' || echo "(no hooks or file missing)"`

### Custom slash commands (filename only)
- Claude (`.claude/commands/`):
  !`ls .claude/commands/*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | sort | tr '\n' ' ' || echo "(none)"`
- Gemini (`.gemini/commands/`):
  !`ls .gemini/commands/*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | sort | tr '\n' ' ' || echo "(none)"`

### MCP servers (names only)
- Claude (`.mcp.json`):
  !`jq -r '.mcpServers | keys[]' .mcp.json 2>/dev/null | tr '\n' ' ' || echo "(no .mcp.json or no mcpServers key)"`
- Codex (`.codex/config.toml`):
  !`grep -oE "^\[mcp_servers\.[A-Za-z0-9_-]+" .codex/config.toml 2>/dev/null | sed 's/\[mcp_servers\.//' | sort -u | tr '\n' ' ' || echo "(file missing)"`
- Gemini (`.gemini/settings.json`):
  !`jq -r '.mcpServers | keys[]' .gemini/settings.json 2>/dev/null | tr '\n' ' ' || echo "(no mcpServers key or file missing)"`

Use the snapshot above as your input. Drop into Bash only when you need *content* (frontmatter inspection, hook command bodies, MCP server URLs) that the names alone don't tell you.

---

## §1 — Skills parity

**Important location facts:**
- Claude reads ONLY `.claude/skills/`.
- Codex reads ONLY `.agents/skills/` (there is no `.codex/skills/`).
- Gemini reads `.agents/skills/` (preferred) AND `.gemini/skills/` (fallback). When both are present, `.agents/skills/` wins.

This means **`.agents/skills/`** is a single shared location consumed by both Codex and Gemini. Don't expect them to need separate skill directories — they don't.

### Procedure

```bash
list_skills() {
  local dir="$1"
  test -d "$dir" || return
  ls -1 "$dir" 2>/dev/null | while read d; do
    test -f "$dir/$d/SKILL.md" && echo "$d"
  done | sort
}

# Claude reads here only
list_skills .claude/skills

# Codex + Gemini both read here (the unified open-skill-standard location)
list_skills .agents/skills

# Gemini fallback only — flag any skill that lives only here as suspicious
# (Codex won't see it, and Gemini would see it from .agents/skills if it were there too)
list_skills .gemini/skills
```

Build the per-platform skill set:
- **Claude set** = `.claude/skills/`
- **Codex set** = `.agents/skills/`
- **Gemini set** = union of `.agents/skills/` and `.gemini/skills/` (with `.agents/` taking precedence on name collision)

For each skill present in only some platforms, decide:
- **Should it exist everywhere?** Workflow skills (release, promote-plan, template-init) belong in all configured platforms.
- **Is it intentionally limited?** A skill referencing platform-specific tools doesn't translate. Skip with reason.

For skills present in multiple:
- Compare `name` and `description` frontmatter — drift here is real (agents pick skills by description).
- Compare body length. A 500-line Claude SKILL with a 50-line `.agents/skills/` stub means the latter is incomplete.
- Compare any preloaded skill references (`skills:` in agent frontmatter) — reference targets must exist in each platform's skills dir.

**Special label for Gemini-only fallback location:**
- `GEMINI_FALLBACK_ONLY` — skill exists in `.gemini/skills/` but not `.agents/skills/`. Codex can't see it; if it's a portable workflow skill, suggest moving it to `.agents/skills/` so Codex picks it up too.

Labels:
- `CLAUDE_ONLY`, `CODEX_ONLY`, `GEMINI_ONLY` — present on only one platform
- `MISSING_FROM_<PLATFORM>` — present on most platforms but missing from one
- `DRIFTED` — present in two or more, body or description differs significantly
- `INTENTIONALLY_<PLATFORM>_ONLY` — uses platform-specific tools, not portable
- `GEMINI_FALLBACK_ONLY` — in `.gemini/skills/` but not in `.agents/skills/` (Codex can't see it)
- `OK` — in sync across all configured platforms

## §2 — Subagent parity

Compare `.claude/agents/`, `.codex/agents/`, `.gemini/agents/`.

For each agent, inspect the YAML frontmatter and flag:

1. **Tool list compatibility** — each platform exposes a different tool set:
   - Claude: full surface (Read, Edit, Write, Glob, Grep, Bash, all MCP tools)
   - Codex: narrower; primarily `Bash`, `apply_patch`, MCP tool names
   - Gemini: regex/wildcard supported (`Read*`, `*`); see `gemini/core-subagents.md`
   Flag any tool in an agent's frontmatter that isn't valid for that platform.

2. **Preloaded skill references** — if a Claude agent has `skills: [foo-checklist]`, the corresponding skill must exist in each other platform's skills directory. Flag missing companion skills.

3. **Per-agent hooks / MCP** — Claude only. If the Codex or Gemini version has these, they're being silently ignored. Flag.

4. **Memory scope** — `memory: user/project/local` is Claude-specific. If the Codex or Gemini version has this frontmatter, flag as `MEMORY_SCHEMA_MISMATCH`.

5. **Color** — Claude and Codex support `color:` frontmatter; Gemini doesn't.

Labels:
- `CLAUDE_ONLY`, `CODEX_ONLY`, `GEMINI_ONLY`, `MISSING_FROM_<PLATFORM>`
- `DRIFTED` — body or frontmatter differs significantly
- `INVALID_TOOL` — frontmatter references a tool that platform doesn't expose
- `MISSING_PRELOADED_SKILL` — agent loads a skill that doesn't exist on this platform
- `MEMORY_SCHEMA_MISMATCH` — memory frontmatter incompatible
- `IGNORED_FRONTMATTER` — fields like per-agent hooks/MCP that this platform silently ignores
- `OK` — in sync

## §3 — Hook parity (the hard one)

Each platform has a different event surface. The canonical mapping is in `platform-matrix.md §3b`. Reproduced here for convenience:

| Concept | Claude | Codex | Gemini |
|---|---|---|---|
| Tool about to run | `PreToolUse` | `PreToolUse` | `BeforeTool` |
| Tool just ran | `PostToolUse` | `PostToolUse` | `AfterTool` |
| User submitted prompt | `UserPromptSubmit` | `UserPromptSubmit` | (no direct equivalent) |
| Agent finished a turn | `Stop` | `Stop` | `AfterAgent` (main) |
| Subagent started | `SubagentStart` | — | `BeforeAgent` (sub) |
| Subagent stopped | `SubagentStop` | — | `AfterAgent` (sub) |
| Session started | `SessionStart` | `SessionStart` | `SessionStart` |
| Session ending | `SessionEnd` | — | `SessionEnd` |
| Compaction about to run | `PreCompact` | — | `PreCompress` |
| Notification surface | `Notification` | — | `Notification` |
| Approval prompt about to fire | — | `PermissionRequest` | — |
| Model about to be called | — | — | `BeforeModel` |
| Model finished | — | — | `AfterModel` |
| Tool selection | — | — | `BeforeToolSelection` |

### Procedure

Read all hook config locations:
- `.claude/settings.json` (under `hooks`)
- `.codex/hooks.json` AND `.codex/config.toml` (`[features].codex_hooks` flag + `[[hooks.<Event>]]`)
- `.gemini/settings.json` (under `hooks`)

For each hook, walk the table above and identify portable equivalents.

### Pre-check: enable flags

Before comparing hooks, verify enable conditions:

- **Codex**: `.codex/config.toml` must contain `[features] codex_hooks = true`, and the project must be in the trusted-projects list. Without these, ALL Codex hooks silently no-op.
- **Gemini**: project hooks are fingerprinted. If hook commands changed since last trust, re-trust may be required. Note in output if you see hooks that have likely changed.
- **Claude**: no enable flag needed.

Labels for pre-check failures:
- `CODEX_HOOKS_DISABLED` — `codex_hooks = true` not set; hooks present but inactive
- `GEMINI_HOOKS_FINGERPRINT_RISK` — hooks present in `.gemini/settings.json` but command paths look likely-changed (heuristic)

### Per-hook procedure

For each hook on each platform, check:

1. **Is the event portable to the other platforms in play?** Use the mapping table.
2. **For portable events: does an equivalent hook exist on the others?** Match by event + matcher + behavior, not by exact command (since command paths may differ).
3. **For non-portable events** (one platform only): note the gap but **don't flag as a violation**. Specifically:
   - Claude `PreCompact` / `Notification` / `SessionEnd` → portable to Gemini (`PreCompress` / `Notification` / `SessionEnd`), NOT to Codex
   - Claude `SubagentStart` / `SubagentStop` → portable to Gemini (`BeforeAgent` / `AfterAgent` matched on subagent name), NOT to Codex
   - Claude `UserPromptSubmit` / `Stop` → portable to Codex, NOT directly to Gemini (closest is `BeforeAgent`/`AfterAgent` on main agent — note as approximate)
   - Codex `PermissionRequest` → no equivalent on Claude or Gemini; intentionally Codex-only
   - Gemini `BeforeModel` / `AfterModel` / `BeforeToolSelection` → no equivalent on Claude or Codex; intentionally Gemini-only

### Critical rule: don't recommend half-assed hooks

The user's stated policy: "if it's not possible, don't create a half-assed hook." Apply this in your output:

- Don't recommend "use `UserPromptSubmit` to fake `PreCompact`" — wrong event, wrong timing.
- Don't recommend "use `Stop` to fake `SubagentStop`" — fires for the main agent too.
- Gemini's `AfterAgent` matched against the main-agent name is a *legitimate* port of Claude's `Stop`, not a substitute. Distinguish "legitimate equivalent" from "wrong-event substitute."

Labels:
- `CODEX_HOOKS_DISABLED`, `GEMINI_HOOKS_FINGERPRINT_RISK` — pre-check failures
- `MISSING_FROM_<PLATFORM>` — portable event, hook present elsewhere, should be ported
- `INTENTIONALLY_<PLATFORM>_ONLY` — uses event that other platforms lack; not a violation
- `DRIFTED` — present on multiple platforms but matcher or behavior differs meaningfully
- `OK` — in sync

## §4 — Custom slash-command parity

Locations:
- Claude: `.claude/commands/<name>.md`
- Codex: **no longer supported** — Codex dropped user-defined custom commands. Only built-in commands exist.
- Gemini: `.gemini/commands/<name>.md`

### Procedure

```bash
list_commands() {
  local dir="$1"
  test -d "$dir" || return
  ls -1 "$dir"/*.md 2>/dev/null | xargs -n1 basename 2>/dev/null | sort
}
list_commands .claude/commands
list_commands .gemini/commands
```

If `.claude/commands/` contains any files, **flag every one** — even if `.gemini/commands/` has a matching entry. The Codex side has no place to put this workflow, so the question for each command is "should this become a skill instead?"

### Recommendation

For every Claude custom command found, the auditor should output:

```
NEEDS_SKILL_PORT  .claude/commands/<name>.md
  → Codex doesn't support custom commands. Convert to a skill at
    .agents/skills/<name>/SKILL.md so both Codex and Gemini can use it.
    Recommend invoking the `skill-creator` skill (or `plugin-dev:skill-development`)
    to perform the conversion. The original .claude/commands/<name>.md can stay —
    Claude users keep the slash-command UX while Codex/Gemini get the skill.
```

If a Gemini custom command exists but no Claude equivalent, that's also flag-worthy:

```
GEMINI_COMMAND_NO_CLAUDE  .gemini/commands/<name>.md
  → Add a corresponding .claude/commands/<name>.md, or convert both to a single
    skill at .agents/skills/<name>/SKILL.md (works for Codex and Gemini; Claude
    keeps the .claude/commands/ wrapper for the slash UX).
```

Labels:
- `NEEDS_SKILL_PORT` — Claude has a custom command; Codex can't host one; recommend skill conversion
- `GEMINI_COMMAND_NO_CLAUDE` — Gemini has a command Claude doesn't
- `DRIFTED` — same name on Claude and Gemini, body differs significantly
- `OK` — Claude command + Gemini command in sync (Codex gap is implicit and not flagged separately if the Claude→skill port has been done — verify the skill exists in `.agents/skills/`)

## §5 — MCP server parity

Compare:
- `.mcp.json` (Claude project)
- `.codex/config.toml` `[mcp_servers.*]` tables (Codex project)
- `.gemini/settings.json` `mcpServers` object (Gemini project)

For each server, compare:

1. **Server name** — matches across configured platforms?
2. **Transport type** — `stdio` vs `http` (Streamable HTTP) vs `sse` — Codex doesn't support SSE
3. **Command + args** for stdio servers — should match modulo path differences
4. **URL** for HTTP/SSE servers — should match
5. **Environment variables** — same vars present? Sensitive values redacted?
6. **Auth bootstrap**:
   - Claude: per-server config
   - Codex: `codex mcp login <server-name>` (CLI bootstrap)
   - Gemini: automatic OAuth discovery + `gemini mcp login`
7. **Trust gating**: Codex and Gemini won't load project MCP without explicit trust. Note if MCP is configured but project isn't trusted.

Labels:
- `CLAUDE_ONLY`, `CODEX_ONLY`, `GEMINI_ONLY`, `MISSING_FROM_<PLATFORM>`
- `DRIFTED` — same name, different command/url/env across platforms
- `AUTH_MISMATCH` — different auth method on the same server (likely intentional, but note it)
- `UNSUPPORTED_TRANSPORT` — e.g. SSE server defined for Codex (not supported)
- `TRUST_GATE_NOT_SET` — server defined but project isn't in trusted list (Codex/Gemini)
- `OK` — in sync

## Output format

Group findings by section. List each finding with the platform pair(s) it concerns:

```
=== §0: Platforms detected ===
claude:  present
codex:   NOT_CONFIGURED
gemini:  present

=== §1: Skills (claude vs gemini) ===
MISSING_FROM_GEMINI    promote-plan         (workflow skill, should port)
INTENTIONALLY_CLAUDE_ONLY  chrome-helper    (uses Claude-only chrome-devtools tool)
DRIFTED                spec-check           (Gemini version is 12 lines, Claude is 38 lines)
OK                     add-domain
OK                     template-init

=== §2: Subagents (claude vs gemini) ===
MISSING_FROM_GEMINI    drift-detector
INVALID_TOOL           aurora-reviewer      (.gemini/agents/aurora-reviewer.md uses tool `Edit` — not in Gemini's exposed surface; verify)
MISSING_PRELOADED_SKILL  rust-spec-reviewer (loads `spec-check`, missing at .gemini/skills/spec-check/)
IGNORED_FRONTMATTER    security-reviewer    (.gemini/agents/security-reviewer.md has per-agent hooks — Gemini ignores)
OK                     mcp-tool-reviewer

=== §3: Hooks (claude vs gemini) ===
MISSING_FROM_GEMINI    PreToolUse "block --no-verify" → port to BeforeTool
INTENTIONALLY_CLAUDE_ONLY  PreCompact log-trim  (PreCompact has Gemini equivalent PreCompress — port? or intentional?)
INTENTIONALLY_GEMINI_ONLY  BeforeModel log-truncate  (Gemini-only event; not portable)
OK                     SessionStart surface-active-work

=== §4: Custom slash-commands ===
NEEDS_SKILL_PORT       .claude/commands/release-checks.md  (Codex can't host; convert to .agents/skills/release-checks/SKILL.md)
GEMINI_COMMAND_NO_CLAUDE  .gemini/commands/quick-deploy.md   (mirror to Claude or convert both to a shared skill)

=== §5: MCP (claude vs gemini) ===
MISSING_FROM_GEMINI    context7              (uses npx; portable)
DRIFTED                lab                   (Claude command path /usr/local/bin/lab, Gemini /opt/lab/bin/lab — verify)
OK                     github

=== Summary ===
{n} skills out of sync, {n} agents out of sync, {n} hook gaps ({n} portable, {n} platform-specific), {n} commands needing skill-port, {n} MCP servers out of sync
```

End with a one-line verdict and an action list. Verdict:
- **PASS** — everything OK or `INTENTIONALLY_<PLATFORM>_ONLY`
- **WARNINGS** — portable gaps that should be ported (e.g. `MISSING_FROM_<PLATFORM>` for portable resources)
- **FAIL** — silent breakage: `CODEX_HOOKS_DISABLED`, `INVALID_TOOL`, `MISSING_PRELOADED_SKILL`, `IGNORED_FRONTMATTER`, `UNSUPPORTED_TRANSPORT`. `NEEDS_SKILL_PORT` is a WARNING, not a FAIL — Claude users keep working; only the Codex/Gemini side is missing the workflow.
