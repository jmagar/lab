---
name: agent-config
description: Specialized configuration help for Claude Code, Codex CLI, and Gemini CLI — skills, subagents, hooks, MCP servers, plugins, slash commands, permissions, settings, memory. Use whenever the user asks to add, modify, troubleshoot, or audit any agentic-CLI configuration. Trigger this skill aggressively for anything touching .claude/, .codex/, .gemini/, settings.json, config.toml, hooks.json, .mcp.json, AGENTS.md, GEMINI.md, or whenever the user mentions "Claude config", "Codex config", "Gemini config", "agent config", or any feature listed in the routing table — even if they don't explicitly ask to "configure" something.
---

You are configuring Claude Code, Codex CLI, Gemini CLI, or some combination. The references in this skill cover the full surface of all three platforms — your job is to route the user's request to the right reference, the right config file, and the right syntax for the platform(s) in scope.

## Why this skill exists

The three platforms have overlapping but non-identical feature sets. All three support skills, subagents, hooks, MCP, and slash commands — but file locations, schema shapes, and capability surfaces differ. Picking the wrong file or copying syntax between platforms silently breaks things. This skill keeps the differences in one place and points at the authoritative docs for everything deeper.

## Reference docs

```
references/
├── platform-matrix.md  ← cross-platform diffs for subagents, MCP, hooks (read first!)
├── claude/             ← Claude Code docs
├── codex/              ← Codex CLI docs
└── gemini/             ← Gemini CLI docs
```

**Always read `platform-matrix.md` first** when a request involves more than one platform — it is the single source of truth for cross-platform differences in subagents, MCP, and hooks. Per-platform reference files cover the deep surface (settings.md, skills.md, etc.). Don't re-derive what they already say — read the relevant file, then apply.

## Feature routing table (project-scoped only)

| Feature | Claude config | Codex config | Gemini config | Reference docs |
|---|---|---|---|---|
| **Skills** | `.claude/skills/<name>/SKILL.md` | `.agents/skills/<name>/SKILL.md` (Codex has no `.codex/skills/`) | `.agents/skills/<name>/SKILL.md` (preferred) or `.gemini/skills/<name>/SKILL.md` | `claude/skills.md`, `codex/skills.md`, `gemini/cli-skills.md` |
| **Subagents** | `.claude/agents/<name>.md` | `.codex/agents/<name>.md` | `.gemini/agents/<name>.md` | see `platform-matrix.md §1` |
| **Hooks** | `.claude/settings.json` (`hooks` key) | `.codex/hooks.json` OR `.codex/config.toml` (`[[hooks.<Event>]]`) — needs `[features].codex_hooks=true` | `.gemini/settings.json` (`hooks` key) | see `platform-matrix.md §3` |
| **MCP servers** | `.mcp.json` | `.codex/config.toml` (`[mcp_servers.*]`) | `.gemini/settings.json` (`mcpServers`) | see `platform-matrix.md §2` |
| **Slash commands** | `.claude/commands/<name>.md` | **not supported** — port to a skill in `.agents/skills/` | `.gemini/commands/<name>.md` | `claude/commands.md`, `codex/slash-commands.md`, `gemini/cli-custom-commands.md` |
| **Plugins** | `claude plugin add` | `codex plugin install` | `gemini extensions install` | `claude/create-plugins.md`, `codex/plugins.md`, `gemini/extensions.md` |
| **Permissions** | `.claude/settings.json` (`permissions.allow/deny`) | `.codex/config.toml` (sandbox/approval-policy) | `.gemini/settings.json` (policy engine) | `claude/permissions.md`, `codex/sandboxing.md`, `gemini/reference-policy-engine.md`, `gemini/cli-sandbox.md` |
| **Memory / instructions** | `CLAUDE.md` | `AGENTS.md` | `GEMINI.md` | `claude/memory.md`, `codex/agents-md.md`, `gemini/cli-gemini-md.md`, `gemini/cli-auto-memory.md` |
| **Trust model** | none — auto-loads | trusted-projects gate | trusted-folders gate | `codex/agent-approvals-security.md`, `gemini/cli-trusted-folders.md` |

User-scoped configs (`~/.claude/`, `~/.codex/`, `~/.gemini/`) are intentionally **out of scope** for this skill — focus is on what gets checked into a repo.

## Note on Codex custom commands

**Codex no longer supports user-defined custom commands.** The Codex `slash-commands.md` reference only documents built-in commands (`/permissions`, `/agent`, `/compact`, etc.) — there is no project-level `.codex/prompts/` or equivalent.

Practical implication: a workflow that lives as `.claude/commands/<name>.md` in Claude has no direct port to Codex. The right port is to **convert the command into a skill** under `.agents/skills/<name>/SKILL.md` — that location is consumed by both Codex and Gemini, and skills can do everything custom commands can do plus progressive-disclosure references.

When you encounter a `.claude/commands/<name>.md` that should be portable, recommend invoking the `skill-creator` skill (or `plugin-dev:skill-development`) to convert it. Don't hand-port it to a "Codex prompt" — that mechanism doesn't exist anymore.

## Note on `.agents/skills/` — the open-skill-standard location

Both **Codex** and **Gemini** support the open-skill-standard `.agents/skills/<name>/SKILL.md` directory. Codex reads only this location (it has no `.codex/skills/` equivalent). Gemini reads both, with `.agents/skills/` taking precedence over `.gemini/skills/`.

Practical implication: skills you want to share between Codex and Gemini live in **one** directory — `.agents/skills/`. Don't duplicate them. Only Claude needs its own `.claude/skills/` because it doesn't read `.agents/skills/`.

## Quick reference: which platform stores hooks where

The biggest source of confusion when porting between platforms:

- **Claude**: hooks live in `.claude/settings.json` under the `hooks` key. Always loaded.
- **Codex**: hooks live in either `.codex/hooks.json` (Claude-style JSON) OR `.codex/config.toml` ([[hooks.<Event>]] tables). **Both require `[features].codex_hooks = true` in `config.toml`**, and the project must be trusted. Without those, hooks silently no-op.
- **Gemini**: hooks live in `.gemini/settings.json` under the `hooks` key — same file as MCP servers. Strict-JSON output protocol ("Golden Rule"). Fingerprinted — changes require re-trust.

## Quick reference: hook event mapping

For the canonical event-mapping table across Claude / Codex / Gemini, see `platform-matrix.md §3b`. Highlights:

- **Portable across all three** (with renames): `PreToolUse`/`PostToolUse` ↔ `BeforeTool`/`AfterTool`, `SessionStart`
- **Two-of-three**: `Notification` (Claude+Gemini), `PreCompact`/`PreCompress` (Claude+Gemini), `SessionEnd` (Claude+Gemini)
- **One platform only**: `PermissionRequest` (Codex), `BeforeModel`/`AfterModel`/`BeforeToolSelection` (Gemini)

**The hard rule when porting:** if a hook depends on an event another platform lacks, accept the gap. Don't substitute a wrong event — a half-working hook is worse than a missing one because it gives false confidence.

## Subagent shape (cross-platform notes)

All three platforms use Markdown + YAML frontmatter under their respective `agents/` directory. Schema overlap is high but not 1:1 — see `platform-matrix.md §1` for the full diff.

The most common porting issues:
- **Tool list compatibility**: Claude's tool list is the broadest. Codex and Gemini support narrower surfaces; verify against per-platform reference before copying.
- **Per-agent hooks/MCP**: only Claude. Strip these from Codex/Gemini ports.
- **Skill preloading**: Claude has explicit `skills:` frontmatter. Codex similar. Gemini links via `/skills link` rather than per-agent declaration.

## MCP shape (cross-platform notes)

Server definitions are 80% portable across all three. The fields are nearly 1:1:
- `command` + `args` for stdio servers
- `url` for HTTP servers
- `env` for environment variables
- type/transport (stdio, http, sse) — all three support stdio + http; Claude and Gemini also support sse

Where they differ — see `platform-matrix.md §2`:
- File format: JSON vs TOML
- OAuth bootstrap mechanism
- Trust gating (Claude=auto, Codex=trusted-project, Gemini=trusted-folder)

## When making changes

1. Read `platform-matrix.md` first if cross-platform; otherwise read the specific per-platform reference doc.
2. Edit the right file (routing table above).
3. If the change needs to exist on multiple platforms, do all edits in the same change so they don't drift.
4. After hook or settings changes, restart the agent session — most config is loaded at session start.
5. For Codex, check that `[features].codex_hooks = true` is set in `config.toml` and the project is trusted.
6. For Gemini, project hooks are fingerprinted — if hook commands change, re-trust may be required.

## When the user asks about parity

Don't manually walk all three directory trees in this skill — that's what the `agent-parity` subagent is for. Recommend invoking it: "I can run the `agent-parity` subagent to check whether your Claude/Codex/Gemini configs are in sync." That agent has its own preloaded checklist (`check-agent-parity` skill) covering all four resource types across all three platforms.
