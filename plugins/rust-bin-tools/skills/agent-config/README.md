# `agent-config` skill

**Last updated:** 2026-04-28

Specialized configuration help for **Claude Code, Codex CLI, and Gemini CLI** — skills, subagents, hooks, MCP servers, plugins, marketplaces, slash commands, permissions, settings. The skill itself routes the agent to the right config file and reference doc; this directory bundles the reference material and the validation/sync scripts that keep the configuration honest.

## Directory layout

```
agent-config/
├── SKILL.md                              ← the skill body (routing tables, key rules)
├── README.md (this file)
├── references/
│   ├── platform-matrix.md                ← cross-platform diff: subagents / MCP / hooks
│   ├── plugin-matrix.md                  ← cross-platform diff: plugins / marketplaces / extensions
│   ├── claude/      *.md (70+)           ← Claude Code documentation snapshot
│   ├── codex/       *.md (33+)           ← Codex CLI documentation snapshot
│   ├── gemini/      *.md (94+)           ← Gemini CLI documentation snapshot
│   ├── agent-skills/ *.md                ← Open agent-skills standard (agentskills.io)
│   └── schemas/                          ← JSON Schema files for validation (see below)
└── scripts/                              ← Utility scripts (see below)
```

## When to invoke this skill

The skill triggers whenever you're touching agent-CLI configuration — see `SKILL.md` for the full routing table. Typical entry points:

| User intent | First stop |
|---|---|
| "Add a hook for X" | `references/platform-matrix.md §3` (event support per platform) |
| "What goes in `.codex/config.toml`?" | `references/codex/config-reference.md` + `references/schemas/codex-config.schema.json` |
| "Build a marketplace that works on all three" | `references/plugin-matrix.md §6` (multi-platform packaging) |
| "Validate this `server.json` before I publish" | `mcp-registry-publish` skill (sibling skill) |
| "Why isn't my Codex hook firing?" | `references/codex/hooks.md` + check `[features].codex_hooks=true` |

## References — `references/`

### Cross-platform matrices (read these first when comparing platforms)

| File | Covers |
|---|---|
| `platform-matrix.md` | Loose-component differences across Claude/Codex/Gemini: subagent frontmatter, MCP server config locations, hook event mapping (with the explicit "don't half-ass" rule for events one platform lacks) |
| `plugin-matrix.md` | Packaged-distribution differences: plugin manifest schemas, marketplace catalog schemas, multi-platform repo layouts, the validation checklist before declaring a build done |

### Per-platform documentation snapshots

`claude/`, `codex/`, `gemini/`, `agent-skills/` each contain markdown copies of the upstream documentation. Refresh with `scripts/download-docs.sh` (reads URLs from `references/docs-urls.md`).

### JSON Schemas — `references/schemas/`

Eight schemas covering nine file types (Claude `settings.json` and `settings.local.json` share one):

| File | Schema | Source |
|---|---|---|
| `.claude-plugin/plugin.json` | `claude-plugin.schema.json` | upstream — `json.schemastore.org/claude-code-plugin-manifest.json` |
| `.claude-plugin/marketplace.json` | `claude-marketplace.schema.json` | upstream — `json.schemastore.org/claude-code-marketplace.json` |
| `.codex-plugin/plugin.json` | `codex-plugin.schema.json` | derived from upstream Rust types in `openai/codex` |
| `.agents/plugins/marketplace.json` | `codex-marketplace.schema.json` | derived from upstream Rust types in `openai/codex` |
| `gemini-extension.json` | `gemini-extension.schema.json` | derived from upstream TypeScript types in `google-gemini/gemini-cli` |
| `.claude/settings.json` + `settings.local.json` | `claude-settings.schema.json` | upstream — `json.schemastore.org/claude-code-settings.json` |
| `.codex/config.toml` | `codex-config.schema.json` | upstream — `developers.openai.com/codex/config-schema.json` |
| `.gemini/settings.json` | `gemini-settings.schema.json` | upstream — `github.com/google-gemini/gemini-cli/main/schemas/settings.schema.json` |

Five are upstream-published and refreshable via `scripts/refresh-schemas.sh`. Three are derived from the actual upstream type definitions (those vendors don't publish JSON Schema, but the Rust/TypeScript types are the source of truth) — update by hand when upstream types change.

## Scripts — `scripts/`

All scripts resolve their own paths from `BASH_SOURCE`, so they work from any CWD. All use `set -euo pipefail` and have `--help`.

| Script | Purpose |
|---|---|
| `download-docs.sh` | Downloads every URL listed in `references/docs-urls.md` to `references/{claude,codex,gemini,agent-skills}/<flattened-name>.md`. Idempotent — skips already-downloaded files unless `--force`. Parallelized via `xargs -P` (default 8). |
| `refresh-schemas.sh` | Re-fetches the 5 upstream-published JSON Schemas. Does NOT touch the 3 derived schemas. |
| `validate-manifest.sh` | Auto-routes by path to validate plugin/marketplace/extension manifests. Covers all 5 file types: Claude/Codex `plugin.json` and `marketplace.json`, plus Gemini `gemini-extension.json`. Falls through three validators: Python `jsonschema` → `check-jsonschema` → `npx ajv-cli`. |
| `validate-settings.sh` | Auto-routes by path to validate runtime settings/config files. Covers Claude `settings.json` + `settings.local.json` (draft-07), Codex `config.toml` (draft-07, parsed via Python `tomllib`), Gemini `settings.json` (draft 2020-12). |
| `skills-ref.sh` | Wrapper around `npx skills-ref` adding `--all` discovery for `.claude/skills/` and forgiveness for documented Claude-only skill frontmatter (`user-invocable`, `disable-model-invocation`). Subcommands: `validate`, `validate-all`, `read-properties`, `read-properties-all`, `to-prompt`. |

### Common usage

```bash
# Validate every skill
./scripts/skills-ref.sh validate-all

# Validate the Claude marketplace manifest
./scripts/validate-manifest.sh .claude-plugin/marketplace.json

# Validate the project's Claude settings
./scripts/validate-settings.sh .claude/settings.json

# Refresh upstream-published schemas after a doc bump upstream
./scripts/refresh-schemas.sh

# Resync the documentation snapshots from docs-urls.md
./scripts/download-docs.sh
```

## Validation is wired into `PostToolUse` hooks

The `.claude/settings.json` `PostToolUse` hook runs the relevant validator automatically when you edit any of these files (warning-only — never blocks):

- `.claude-plugin/{plugin,marketplace}.json` → `validate-manifest.sh` + `claude plugin validate <dir>` if available
- `.codex-plugin/plugin.json`, `.agents/plugins/marketplace.json`, `gemini-extension.json` → `validate-manifest.sh`
- `.claude/settings.json`, `settings.local.json`, `.codex/config.toml`, `.gemini/settings.json` → `validate-settings.sh`

## Maintenance protocols

**When upstream docs change:**
```bash
./scripts/download-docs.sh        # refresh per-platform doc snapshots
./scripts/refresh-schemas.sh      # refresh the 5 upstream-published schemas
```

**When upstream types change** (Codex Rust types, Gemini TypeScript interfaces):
- Re-read the upstream type definition (referenced in each derived schema's `description`)
- Update the matching schema by hand in `references/schemas/`
- Update the matching field table in `plugin-matrix.md` §3–§5

**When the local matrices drift from reality:**
- The `agent-parity` subagent audits loose-component parity (skills/agents/hooks/commands/MCP)
- The `plugin-builder` subagent audits packaging when shipping a marketplace/extension

## Sibling skills and agents

| Sibling | Relationship |
|---|---|
| `agent-parity` agent | Loads this skill's references to audit cross-platform component parity |
| `plugin-builder` agent | Loads this skill's references to scaffold plugins/marketplaces/extensions; relies on `plugin-matrix.md` schemas |
| `mcp-registry-publish` skill | Independent skill for publishing MCP servers to registry.modelcontextprotocol.io — its own `references/` and `scripts/` |
| `check-agent-parity` skill | Companion to `agent-parity` agent — preloaded into it, contains the audit procedure |

## Source-of-truth principles

- **Authoring**: every agent-config resource is authored under `.claude/` (see root `CLAUDE.md` § Agent-config authoring). Codex and Gemini read from `.agents/skills/` (a symlink to `.claude/skills/`) maintained by the `sync-skills` skill (auto-fired by `PostToolUse` hook on every edit under `.claude/skills/`).
- **References**: per-platform doc snapshots are downloaded artifacts; treat them as read-only between refreshes. Don't hand-edit `references/{claude,codex,gemini}/*.md`.
- **Schemas**: 5 upstream-published, 3 derived. Don't conflate the two — `refresh-schemas.sh` only touches the upstream-published ones.
- **Cross-platform matrices**: `platform-matrix.md` and `plugin-matrix.md` are hand-maintained authoritative summaries. Update them when the upstream truth changes; everything else (skills, agents, validators) reads them as canonical.
