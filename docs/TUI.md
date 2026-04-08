# TUI

The TUI is a Claude Code marketplace browser and plugin manager.

## Scope

The TUI exists to help users:

- browse Claude Code plugin marketplaces
- inspect available plugins with metadata and descriptions
- install and remove plugins
- update the `lab` binary itself
- understand required env setup for installed plugins

It is not intended to replicate every CLI or MCP operation.

## Implementation

- `ratatui`
- `crossterm`
- always compiled

## Primary Screen

The core experience is a plugin list grouped by category with install state and concise metadata.

Each row should reflect:

- plugin name
- short description
- install state (installed / available / update available)
- marketplace source
- env readiness hints

An example mental model is:

- installed vs available vs updatable
- category grouping
- marketplace source label
- env readiness indicators

## Data Source

The TUI should consume the same `PluginMeta` records used elsewhere.

That keeps:

- categories consistent
- env requirements consistent
- display names consistent

Marketplace data is fetched from configured registry endpoints and merged with local install state.

## Interaction Model

Expected interaction is simple:

- navigate
- install / remove
- update binary
- quit

Complex modal workflows should be the exception, not the baseline.

The TUI should stay optimized for quick plugin setup tasks rather than deep service management.

## Plugin Install / Remove

The TUI drives plugin installation and removal using the same atomic semantics as the CLI install/uninstall path:

- fetch plugin manifest from marketplace
- validate env requirements
- write plugin files atomically
- verify after write
- update `.mcp.json` if applicable (using CLI install/uninstall semantics — no custom patching logic)

## Binary Updates

The TUI exposes a binary update flow:

- check latest release from the configured update source
- display current vs available version
- prompt user to confirm before downloading
- download and replace binary atomically
- verify the new binary reports the expected version

## State Policy

TUI state is ephemeral.

Rules:

- no persisted window state
- no persisted selection state
- no config file for TUI preferences

This is a hard anti-creep rule.

## Secrets and Prompts

If the TUI prompts for env values:

- secret values must be masked
- secret values must never be echoed back in the UI
- writes should go only to the designated env file flow

## Relationship to CLI

The TUI is a convenience layer for marketplace browsing and plugin lifecycle. The CLI remains the general-purpose operator surface.
