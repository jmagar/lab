---
description: Scaffold a new Claude Code plugin directory structure
argument-hint: <plugin-name> [path]
allowed-tools: Bash, Write
---

Existing marketplace.json files: !`find . -name "marketplace.json" 2>/dev/null`
Existing plugin directories: !`find . -type d -name ".claude-plugin" 2>/dev/null | sed 's|/.claude-plugin||'`

Scaffold a new Claude Code plugin named `$1` at path `$2` (optional).

## Instructions

**If `$1` is empty**, stop immediately and tell the user:

> Usage: /scaffold-claude-plugin \<plugin-name\> [path]
> Example: /scaffold-claude-plugin my-plugin
> Example: /scaffold-claude-plugin my-plugin ./plugins/my-plugin

### 1. Determine target path

- If `$2` is provided, use it as the target directory.
- If `$2` is empty, examine the "Existing plugin directories" context above:
  - Find the common parent directory shared by the existing plugins (e.g. if plugins live under `./plugins/foo`, `./plugins/bar`, the parent is `./plugins`).
  - If no existing plugins are found, default to `./plugins/$1`.
  - Set TARGET to `<parent>/$1`.

Expand any leading `~` to `$HOME`.

### 2. Create directories

Use Bash to run mkdir -p for: `$TARGET/.claude-plugin`, `$TARGET/agents`, `$TARGET/bin`, `$TARGET/commands`, `$TARGET/hooks`, `$TARGET/monitors`, `$TARGET/output-styles`, `$TARGET/scripts`, `$TARGET/skills`.

### 3. Write .claude-plugin/plugin.json

Write the file with:
- `name`: `$1` (the plugin name argument)
- `version`: "0.1.0"
- `description`: "A Claude Code plugin"
- `author`: ""
- Empty arrays for `commands`, `agents`, `skills`, `hooks`, `monitors`

### 4. Write .mcp.json

Write a minimal file with an empty `mcpServers` object.

### 5. Write CHANGELOG.md

Write an initial changelog with a single `[0.1.0] - Unreleased` section listing "Initial plugin scaffold" under Added.

### 6. Write README.md

Write a README with:
- H1 title: `$1`
- One-line description placeholder
- An Installation section showing: `claude plugin install <path>`
- A Commands table with a "(none yet)" placeholder row
- Agents and Skills sections with "(none yet)" placeholders
- A Development section showing the full directory tree:
  - .claude-plugin/plugin.json — Plugin manifest
  - agents/ — Subagent definitions
  - bin/ — Executable scripts
  - commands/ — Slash command .md files
  - hooks/ — Pre/PostToolUse hooks
  - monitors/ — Background monitors
  - output-styles/ — Output formatting styles
  - scripts/ — Helper scripts
  - skills/ — Skill .md files
  - .mcp.json — MCP server configuration
  - CHANGELOG.md, README.md

### 7. Print summary

After all files and directories are created, print a concise summary:
- List every path created
- Remind the user to update `plugin.json` with the real name, description, and author
- Suggest next steps: add a skill in `skills/`, add a command in `commands/`, or add a hook in `hooks/`
