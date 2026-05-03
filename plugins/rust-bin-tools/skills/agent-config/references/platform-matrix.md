# Cross-Platform Configuration Matrix — Claude / Codex / Gemini

Authoritative reference for **project-scoped** configuration differences across the three agentic CLIs. Covers subagents, MCP, and hooks. Skills are similar enough across platforms that they live in the routing table in `agent-config/SKILL.md`.

User-scoped configs (`~/.claude/`, `~/.codex/`, `~/.gemini/`) are intentionally **out of scope** — this document is for what gets checked into a repo and shared with a team.

When you need deeper detail than this matrix, jump to the per-platform reference doc cited in each section.

---

## §1 Subagents

| | **Claude Code** | **Codex CLI** | **Gemini CLI** |
|---|---|---|---|
| Project location | `.claude/agents/<name>.md` | `.codex/agents/<name>.md` | `.gemini/agents/<name>.md` |
| File format | Markdown + YAML frontmatter | Markdown + YAML frontmatter | Markdown + YAML frontmatter |
| Required frontmatter | `name`, `description` | `name`, `description` | `name`, `description` |
| `tools` field | full Claude tool surface | restricted Codex tool surface (`Bash`, `apply_patch`, MCP tool names) | wildcards supported (`Read*`, `*`); see `core-subagents.md` |
| `model` field | `sonnet`/`opus`/`haiku`/full ID/`inherit` | model id strings | model id strings |
| Skill preloading | `skills:` list in frontmatter | `skills:` list (more limited) | not directly — link via `/skills link` |
| Per-agent memory | `memory: user/project/local` | different lifecycle — see `codex/memories.md` | not directly comparable; see `core-subagents.md` |
| Per-agent hooks | `hooks:` in frontmatter | not supported | not supported |
| Per-agent MCP | `mcpServers:` in frontmatter | not supported | tool isolation supported via `mcpServers` policy |
| Color/UI | `color:` in frontmatter | `color:` in frontmatter | not in frontmatter |
| Spawn syntax | `Agent` tool, `@-mention` | `@-mention`, automatic delegation | `@-mention`, automatic delegation |
| Subagent recursion | not supported | not supported | not supported (recursion protection built-in) |

Reference docs:
- Claude — `claude/sub-agents.md`
- Codex — `codex/subagents.md`, `codex/subagents-concepts.md`
- Gemini — `gemini/core-subagents.md`

**Portability rule of thumb:** if a subagent uses only `Read`/`Glob`/`Grep`/`Bash` tools and has no per-agent hooks/MCP, it's portable across all three platforms with only minor frontmatter normalization. Anything that depends on `tools: Edit, Write` plus Claude-specific tool names will need adaptation for Codex and Gemini.

---

## §2 MCP Servers

| | **Claude Code** | **Codex CLI** | **Gemini CLI** |
|---|---|---|---|
| Project config file | `.mcp.json` | `.codex/config.toml` (trusted projects only) | `.gemini/settings.json` |
| File format | JSON (top-level `mcpServers` key) | TOML (`[mcp_servers.<name>]` tables) | JSON (`mcpServers` key) |
| Server types | stdio, Streamable HTTP, SSE | stdio, Streamable HTTP | stdio, Streamable HTTP, SSE |
| Auth — bearer | header in `env` or config | header in `env` | header in `env` |
| Auth — OAuth | per-server config | `codex mcp login <server-name>` (CLI-driven) | automatic OAuth discovery + `gemini mcp login` |
| Per-server env vars | `env` object | `env` table | `env` object |
| Env var expansion | manual | manual | `${VAR}` syntax in values |
| Auto-redaction | manual `Secret<T>` patterns | manual | core project keys auto-redacted (`GEMINI_API_KEY`, `GOOGLE_API_KEY`, etc.) |
| Trusted-project gate | no | yes — `.codex/config.toml` only loads after explicit trust | yes — trusted folders mechanism |

Reference docs:
- Claude — `claude/mcp.md`
- Codex — `codex/mcp.md`, `codex/codex-mcp-server.md`
- Gemini — `gemini/tools-mcp-server.md`

**Portability rule of thumb:** MCP server definitions are 80% portable. The same stdio command + args + env will run on all three. Where they differ:

- File location and serialization (JSON vs TOML)
- OAuth bootstrap — Codex requires explicit `codex mcp login`; Gemini auto-discovers; Claude requires per-server config
- Trust model — Codex and Gemini won't load project-scoped MCP config until the project is marked trusted

To keep three platforms in sync, treat the Claude `.mcp.json` as the source of truth and translate to TOML/Gemini-JSON when porting. The fields are nearly 1:1 (`command`, `args`, `env`, `url`).

---

## §3 Hooks

This is the area with the **largest capability gap**. Each platform has a different event surface, different output schema conventions, and different ways to disable/enable.

### §3a Hook config locations

| | **Claude Code** | **Codex CLI** | **Gemini CLI** |
|---|---|---|---|
| Project file(s) | `.claude/settings.json` (`hooks` key) | `.codex/hooks.json` OR `.codex/config.toml` (`[[hooks.<Event>]]`) | `.gemini/settings.json` (`hooks` key) |
| Enable flag | none — always on | **`[features].codex_hooks = true` in `config.toml` REQUIRED** — without it hooks silently no-op | none — always on |
| Per-subagent hooks | yes (in agent frontmatter) | no | no |
| Concurrent execution | matching hooks fire concurrently | matching hooks fire concurrently | matching hooks fire concurrently |
| Output protocol | JSON on stdout, exit codes 0/2 | JSON on stdout, exit codes 0/2 | **strict JSON only** ("the Golden Rule" — non-JSON output is rejected); exit codes 0/2 |
| Project-hook fingerprinting | no | no | yes — Gemini fingerprints project hooks; changes require re-trust |
| Env vars exposed | `CLAUDE_PROJECT_DIR`, etc. | similar | `GEMINI_PROJECT_DIR`, `GEMINI_PLANS_DIR`, `GEMINI_CWD` |

### §3b Event mapping

| Concept | Claude | Codex | Gemini |
|---|---|---|---|
| Tool about to run | `PreToolUse` | `PreToolUse` | `BeforeTool` |
| Tool just ran | `PostToolUse` | `PostToolUse` | `AfterTool` |
| User submitted prompt | `UserPromptSubmit` | `UserPromptSubmit` | (no direct equivalent — closest is `BeforeAgent` on the main agent) |
| Agent finished a turn | `Stop` | `Stop` | `AfterAgent` (on main agent) |
| Subagent started | `SubagentStart` | — | `BeforeAgent` (on subagent) |
| Subagent stopped | `SubagentStop` | — | `AfterAgent` (on subagent) |
| Session started | `SessionStart` | `SessionStart` | `SessionStart` |
| Session ending | `SessionEnd` | — | `SessionEnd` |
| Compaction about to run | `PreCompact` | — | `PreCompress` |
| User notification surface | `Notification` | — | `Notification` |
| Approval prompt about to fire | — | `PermissionRequest` | — |
| Model about to be called | — | — | `BeforeModel` |
| Model finished | — | — | `AfterModel` |
| Right before tool selection | — | — | `BeforeToolSelection` |

### §3c Capability gaps — what NOT to do

When a hook needs an event one platform lacks, **don't fake it.** A wrong-event substitute fires at the wrong times and gives false confidence the hook is working. Specifically:

- A Claude `PreCompact` hook has no Codex equivalent. Don't try `Stop` or `UserPromptSubmit` — they fire at unrelated times.
- A Claude `SubagentStop` hook has a Gemini equivalent (`AfterAgent` matched on subagent name) but **no Codex equivalent**. Codex-side, accept the gap.
- A Claude `Notification` hook has a Gemini equivalent (`Notification`) but **no Codex equivalent**.
- Codex's `PermissionRequest` is unique to Codex (the others have no separate approval-prompt hook). Don't try to emulate elsewhere.
- Gemini's `BeforeModel`/`AfterModel`/`BeforeToolSelection` are model-level hooks unique to Gemini. The closest cross-platform equivalents are `PreToolUse`/`PostToolUse` but they fire at a different layer.

### §3d Matcher semantics

| Event class | Claude | Codex | Gemini |
|---|---|---|---|
| Tool events | `Edit\|Write\|Bash` regex etc. | exact tool name (`Bash`, `apply_patch`, MCP tool name) | regex against tool name |
| `UserPromptSubmit` | regex against prompt | `matcher` ignored | n/a |
| `Stop` / `AfterAgent` | `matcher` rules | `matcher` ignored | matcher matches agent name |
| `SessionStart` | source matcher | source matcher (`startup`, `resume`, `clear`) | source matcher |

Reference docs:
- Claude — `claude/hooks.md`, `claude/hooks-guide.md`
- Codex — `codex/hooks.md`
- Gemini — `gemini/hooks.md`, `gemini/hooks-reference.md`, `gemini/hooks-best-practices.md`, `gemini/hooks-writing-hooks.md`

**Portability rule of thumb:** events in the top half of §3b (Tool, UserPrompt, Stop, SessionStart) port across all three with minor renaming. Everything below `SessionStart` in that table is platform-asymmetric — accept the gap rather than half-ass it.

---

## Cross-cutting trust model

Both Codex and Gemini gate project-scoped config behind a trust mechanism. Claude does not.

| Action | Claude | Codex | Gemini |
|---|---|---|---|
| Loads `.<dir>/settings.{json,toml}` automatically | yes | only if project is in trusted list | only if folder is trusted (`gemini trust` flow) |
| Loads project hooks automatically | yes | only with `codex_hooks = true` AND trust | only after hook fingerprint check |
| Loads project MCP servers automatically | yes | only after trust | only after trust |

When porting Claude config to Codex or Gemini, the user (or onboarding doc) needs to explicitly mark the project as trusted before any of it activates. Forgetting this step is a common cause of "I added the hook but it never fires" reports.
